use crate::server::network::server_socket::ChildSocket;
use crate::server::world_data::chunk::chunk_map::ServerChunkMap;
use crate::server::world_data::data::ServerWorldData;
use crate::server::world_data::event::events::{PlayerEvent, ServerEvent};
use crate::server::world_data::player::player::ServerPlayer;
use crossbeam::channel as cb;
use shared::common::account::puid::PUID;
use shared::common::error::err_type::{CustomErrorKind, GameError};
use shared::common::network::client::player_packet::UpdatePlayerPacket;
use shared::common::network::l5_packet::L5Packet;
use shared::common::network::packet_type::UdpPacketType;
use shared::common::network::server::connection_packet::ConnectionPacket;
use shared::common::network::server::quit_packet::QuitPacket;
use shared::common::network::server::tick_packet::GetPlayerPacket;
use shared::common::network::udp_packet::TIMEOUT_DURATION;
use shared::common::world::block::block::BlockId;
use shared::common::world::pos::chunkpos::ChunkPos;
use shared::{custom_error, print_base};
use std::collections::HashSet;
use std::io::{Error, ErrorKind};
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::mpsc as tchannel;

pub const UPDATE_INTERVAL: Duration = std::time::Duration::from_millis(50);
pub struct ClientConnection {
    server_world_data: Arc<ServerWorldData>,
    player: Arc<RwLock<ServerPlayer>>,
    player_position: ChunkPos,
    socket_receiver: ChildSocket,
    player_rx: tchannel::Receiver<(L5Packet, UdpPacketType)>,
    view_distance: u8,
    event_sx: cb::Sender<ServerEvent>,
    puid: PUID,
    ping: Instant,
    ping_id: Option<u16>,
}

impl ClientConnection {
    pub async fn start(packet: L5Packet, socket: ChildSocket, server_world_data: Arc<ServerWorldData>, event_sx: cb::Sender<ServerEvent>) -> Result<(), GameError> {
        let (player_sx, player_rx) = tchannel::channel::<(L5Packet, UdpPacketType)>(10000);
        let L5Packet::Login(con_packet) = packet else {
            print_base!("Connection {}: Wrong Packet, returning", socket.get_addr());
            return Err(custom_error!(CustomErrorKind::InvalidData, "Got wrong packet from network"));
        };

        let (pos, player) = server_world_data.connect_player(con_packet.get_puid().clone(), player_sx)?;
        let mut block_table: Vec<(String, BlockId)> = Vec::new();
        for (name, id) in server_world_data.get_registries().get("blocks").unwrap().iter() {
            block_table.push((name.clone(), *id));
        }
        socket
            .send(L5Packet::Connect(ConnectionPacket::new(pos, block_table)), UdpPacketType::Simple)
            .await
            .unwrap();

        print_base!("Initiating connection with {} and creating player in the world with PUID:{} at {}", socket.get_addr(), con_packet.get_puid().id(), pos.deref());

        let mut client_connection = Self {
            player_position: pos.get_chunk_pos(),
            socket_receiver: socket,
            server_world_data,
            player,
            event_sx,
            player_rx,
            ping: Instant::now(),
            ping_id: Some(0),
            view_distance: 8,
            puid: con_packet.get_puid().clone(),
        };
        client_connection.handle_client().await
    }

    /// The Main Loop handling the connection to the client
    ///
    /// Switching between receiving a packet from the main connection thread through the channel,
    /// or between receiving a packet to send or the interval ticking to ask for player update
    pub async fn handle_client(&mut self) -> Result<(), GameError> {
        // defining the interval
        let mut heartbeat_interval = tokio::time::interval(UPDATE_INTERVAL);
        let mut timeout = Instant::now() + TIMEOUT_DURATION;
        let mut id = 0;
        self.generate_chunks()?;
        self.load_chunks(self.player.clone(), self.player_position, self.player_position, 0, 10)?;

        loop {
            tokio::select! {
                // waiting till the socket receives data or timeout limit is reached
                packet = self.socket_receiver.recv() => {
                    if let Err(e) = self.receive(&packet).await {
                        print_base!("Exiting {} due to error {}", self.socket_receiver.get_addr(), e);
                        break;
                    } else {
                        timeout = Instant::now() + TIMEOUT_DURATION;
                    }
                }

                // if a packet is scheduled to be sent
                Some((packet, udp_type)) = self.player_rx.recv() => {
                    // ChunkPacket::from_bits(packet_data[1..].view_bits::<Lsb0>().to_bitvec());
                    self.socket_receiver.send(packet, udp_type).await.map_err(Error::other)?;
                }

                // asking the player his information at a constant rate
                _ = heartbeat_interval.tick() => {
                    // checking if the timeout of 5 isn't elapsed
                    if Instant::now() > timeout {
                        print_base!("Exiting {} at {} due to time elapsed", self.socket_receiver.get_addr(), chrono::offset::Local::now());
                        break;
                    }
                    if self.ping_id.is_none() {
                        self.ping_id = Some(id);
                        self.ping = Instant::now();
                    }
                    // sending the packet to query the information of the player
                    let packet = L5Packet::GetPlayer(GetPlayerPacket::new(id));
                    id += 1;
                    self.socket_receiver.send(packet, UdpPacketType::Simple).await.unwrap();
                }
            }
        }
        self.socket_receiver
            .send(L5Packet::Quit(QuitPacket::new()), UdpPacketType::Simple)
            .await
            .expect("Cannot send quit packet");
        self.event_sx
            .send(ServerEvent::PlayerEvent {
                player_sender: self.player.read().unwrap().get_sender().clone(),
                e_type: PlayerEvent::DisconnectPlayer(self.puid),
            })
            .map_err(|e| custom_error!(CustomErrorKind::SendError, e.to_string()))
    }

    fn generate_chunks(&self) -> Result<(), Error> {
        self.event_sx
            .send(ServerEvent::PlayerEvent {
                player_sender: self.player.read().unwrap().get_sender().clone(),
                e_type: PlayerEvent::GenerateChunk(self.player_position),
            })
            .map_err(Error::other)
    }

    async fn receive(&mut self, frame: &Option<L5Packet>) -> Result<(), Error> {
        match frame {
            Some(packet) => self.handle_packet(packet).await,
            None => Err(Error::new(ErrorKind::BrokenPipe, "Remote client closed the connection")),
        }
    }

    /// Asks the main thread to send missing chunks to the player
    fn load_chunks(&self, server_player: Arc<RwLock<ServerPlayer>>, last_pos: ChunkPos, new_pos: ChunkPos, old_view_distance: i32, new_view_distance: i32) -> Result<(), Error> {
        let (to_load, to_unload) = ServerChunkMap::compute_chunk_diff(last_pos, new_pos, old_view_distance, new_view_distance);
        self.event_sx
            .send(ServerEvent::PlayerEvent {
                player_sender: server_player.read().unwrap().get_sender().clone(),
                e_type: PlayerEvent::AskChunk(to_load, server_player.read().unwrap().get_puid()),
            })
            .map_err(Error::other)?;

        for pos in to_unload {}
        Ok(())
    }

    async fn handle_packet(&mut self, packet: &L5Packet) -> Result<(), Error> {
        match packet {
            // ClientPacket::Quit(_) => Ok(()),
            L5Packet::UpdatePlayer(p) => {
                self.update_player(p)?;
                if Some(p.get_id()) == self.ping_id {
                    // print_base!("Ping of : {}ms",Instant::now().duration_since(self.ping).as_millis());
                    self.ping_id = None;
                }
                Ok(())
            }
            L5Packet::Block(p) => {
                self.event_sx
                    .send(ServerEvent::PlayerEvent {
                        player_sender: self.player.read().unwrap().get_sender().clone(),
                        e_type: PlayerEvent::BlockInteraction(p.clone()),
                    })
                    .map_err(Error::other)?;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// update the client connection and the world data with the information from the player
    /// schedule to load chunks if needed
    fn update_player(&mut self, packet: &UpdatePlayerPacket) -> Result<(), Error> {
        self.player.write().unwrap().set_pos(packet.get_pos());

        let moved = packet.get_pos().get_chunk_pos() != self.player_position;
        let viewed = self.view_distance != packet.get_view_distance();

        let last_pos = self.player_position;

        if viewed {
            self.view_distance = packet.get_view_distance();
        }

        if moved {
            self.player_position = packet.get_pos().get_chunk_pos();
            self.generate_chunks()?;
            // print_base!("New ChunkPos : {}", self.chunks_pos.deref());
        };

        if moved || viewed {
            self.load_chunks(self.player.clone(), last_pos, self.player_position, self.view_distance as i32, self.view_distance as i32)?;
        }
        Ok(())
    }
}
