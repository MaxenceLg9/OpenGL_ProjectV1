use std::collections::HashSet;
use std::io::{Error, ErrorKind};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::mpsc as tchannel;
use shared::common::network::server::tick_packet::GetPlayerPacket;
use shared::common::world::pos::chunkpos::ChunkPos;
use shared::{print_base};
use shared::common::account::puid::PUID;
use shared::common::network::client::player_packet::UpdatePlayerPacket;
use shared::common::network::server::connection_packet::ConnectionPacket;
use shared::common::network::l5_packet::L5Packet;
use shared::common::network::packet_type::UdpPacketType;
use shared::common::network::server::quit_packet::QuitPacket;
use crate::server::network::server_socket::ServerConnection;
use crate::server::world_data::chunk::chunk_map::ServerChunkMap;
use crate::server::world_data::data::ServerWorldData;
use crate::server::world_data::player::player::ServerPlayer;

pub struct ClientConnection {
    server_world_data: Arc<ServerWorldData>,
    chunks_loaded : HashSet<ChunkPos>,
    player : Arc<RwLock<ServerPlayer>>,
    chunks_pos: ChunkPos,
    socket_receiver: ServerConnection,
    prx : tchannel::Receiver<(L5Packet, UdpPacketType)>,
    view_distance : u8,
    puid : PUID,
    ping : Instant,
    ping_id : Option<u16>
}

impl ClientConnection {
    pub async fn start(packet : L5Packet, socket : ServerConnection, server_world_data: Arc<ServerWorldData>) {
        let (psx, prx) = tchannel::channel::<(L5Packet, UdpPacketType)>(100000);
        let L5Packet::Login(con_packet) = packet else {
            print_base!("Connection {}: Wrong Packet, returning", socket.get_addr());
            return;
        };

        let Ok((pos, player)) = server_world_data
            .connect_player(con_packet.get_puid().clone(), psx)
            .inspect_err(|e| print_base!("Connection {}: Got Error {}, returning", socket.get_addr(), e)) else {
            return;
        };
        socket.send(L5Packet::Connect(ConnectionPacket::new(pos)),UdpPacketType::Simple).await.unwrap();



        let mut client_connection = Self {
            chunks_loaded: HashSet::new(),
            chunks_pos: pos.get_chunk_pos(),
            socket_receiver: socket,
            server_world_data,
            player,
            prx,
            ping: Instant::now(),
            ping_id: Some(0),
            view_distance: 8,
            puid: con_packet.get_puid().clone()
        };
        client_connection.handle_client().await;
    }

    /// The Main Loop handling the connection to the client
    /// Switching between receiving a packet from the main connection thread through the channel,
    /// or between receiving a packet to send or the interval ticking to ask for player update
    pub async fn handle_client(&mut self) {
        let mut heartbeat_interval = tokio::time::interval(std::time::Duration::from_millis(20));
        let mut instant = Instant::now();
        let mut id = 0;
        self.generate_chunks();
        Self::load_chunks(self.server_world_data.get_chunk_map().clone(), self.player.clone(), self.chunks_pos,self.chunks_pos,0,10);

        loop {
            tokio::select! {
                // waiting till the socket receives data or timeout limit is reached
                packet = self.socket_receiver.recv() => {
                    if let Err(e) = self.receive(&packet) {
                        print_base!("Exiting {} due to error {}", self.socket_receiver.get_addr(), e);
                        break;
                    } else {
                        instant = Instant::now();
                    }
                }

                // if a packet is scheduled to be sent
                Some((packet, udp_type)) = self.prx.recv() => {
                    // ChunkPacket::from_bits(packet_data[1..].view_bits::<Lsb0>().to_bitvec());
                    self.socket_receiver.send(packet, udp_type).await;
                }


                // querying the player information at a constant interval
                _ = heartbeat_interval.tick() => {
                    // checking if the timeout of 5 isn't elapsed
                    if instant.elapsed().gt(&Duration::from_secs(5)) {
                        print_base!("Exiting {} due to time elapsed", self.socket_receiver.get_addr());
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
        self.socket_receiver.send(L5Packet::Quit(QuitPacket::new()), UdpPacketType::Simple).await.expect("");
        self.server_world_data.disconnect_player(&self.puid);
    }

    fn generate_chunks(&self) {
        let mut array: [ChunkPos; 20*13*20] = [ChunkPos::from_i32(0, 0, 0);20*13*20];
        let mut i = 0;
        for x in -10..10 {
            for y in -2..10 {
                for z in -10..10 {
                    array[i] = self.chunks_pos.flattened() + ChunkPos::from_i32(x, y, z);
                    i += 1;
                }
            }
        }
        self.server_world_data.get_generator().write().unwrap().schedule_chunks(array);
    }

    fn receive(&mut self, frame: &Option<L5Packet>) -> Result<(), Error> {
        match frame {
            Some(packet) => {
                self.handle_packet(packet)
            }
            None => {
                Err(Error::new(ErrorKind::BrokenPipe,"The packet received is None"))
            },
        }
    }

    fn load_chunks(arc_chunk_map :  Arc<RwLock<ServerChunkMap>>, server_player: Arc<RwLock<ServerPlayer>>, last_pos : ChunkPos, new_pos : ChunkPos, old_view_distance : i32, new_view_distance : i32) {
        let mut chunk_map = arc_chunk_map.write().unwrap();
        let (to_load, to_unload) = ServerChunkMap::compute_chunk_diff(last_pos, new_pos, old_view_distance, new_view_distance);
        // print_base!("Chunks to load {}", to_load.len());
        for pos in to_load {
            chunk_map.ask_for_chunks(pos, server_player.clone());
        }
        for pos in to_unload {

        }
    }

    fn handle_packet(&mut self, packet : &L5Packet) -> Result<(), Error> {
        match packet {
            // ClientPacket::Quit(_) => Ok(()),
            L5Packet::UpdatePlayer(p) => {
                self.update_player(p);
                if Some(p.get_id()) == self.ping_id {
                    // print_base!("Ping of : {}ms",Instant::now().duration_since(self.ping).as_millis());
                    self.ping_id = None;
                }
                Ok(())
            },
            _ => Ok(()),
        }
    }

    /// update the client connection and the world data with the information from the player
    /// schedule to load chunks if needed
    fn update_player(&mut self, packet : &UpdatePlayerPacket) {
        self.player.write().unwrap().set_pos(packet.get_pos());

        let moved = packet.get_pos().get_chunk_pos() != self.chunks_pos;
        let viewed = self.view_distance != packet.get_view_distance();

        let last_pos = self.chunks_pos;

        if viewed {
            self.view_distance = packet.get_view_distance();
        }

        if moved {
            self.chunks_pos = packet.get_pos().get_chunk_pos();
            self.generate_chunks();
            // print_base!("New ChunkPos : {}", self.chunks_pos.deref());
        };

        if moved || viewed {
            Self::load_chunks(self.server_world_data.get_chunk_map().clone(), self.player.clone(),last_pos, self.chunks_pos, self.view_distance as i32, self.view_distance as i32);
        }
    }
}