use std::collections::HashSet;
use std::io::{Error, ErrorKind};
use std::net::SocketAddr;
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use shared::common::network::client::packet::ClientPacket;
use shared::common::network::server::tick_player::GetPlayerPacket;
use shared::common::world::pos::chunkpos::ChunkPos;
use shared::{print_base};
use shared::common::account::puid::PUID;
use shared::common::network::client::player_packet::UpdatePlayerPacket;
use shared::common::network::server::connection_packet::ConnectionPacket;
use shared::common::network::server::packet::ServerPacket;
use shared::common::network::server::quit_packet::QuitPacket;
use crate::server::world_data::data::ServerWorldData;
use crate::server::world_data::player::player::ServerPlayer;

pub struct ClientConnection {
    server_world_data: Arc<ServerWorldData>,
    addr : SocketAddr,
    chunks_loaded : HashSet<ChunkPos>,
    player : Arc<RwLock<ServerPlayer>>,
    packet_receiver : mpsc::Receiver<ClientPacket>,
    chunks_pos: ChunkPos,
    socket : Arc<tokio::net::UdpSocket>,
    prx : tokio::sync::mpsc::Receiver<ServerPacket>,
    view_distance : u8,
    chunks_to_load: HashSet<ChunkPos>,
    puid : PUID,
    ping : Instant,
    ping_id : Option<u16>
}

impl ClientConnection {
    pub async fn start(addr : SocketAddr, packet : ClientPacket, packet_receiver: mpsc::Receiver<ClientPacket>, socket : Arc<tokio::net::UdpSocket>, server_world_data: Arc<ServerWorldData>) {
        let (psx, prx) = mpsc::channel::<ServerPacket>(10000);
        let ClientPacket::Login(con_packet) = packet else {
            print_base!("Connection {}: Wrong Packet, returning", addr);
            return;
        };

        let Ok((pos, player)) = server_world_data.connect_player(con_packet.get_puid().clone(), psx).inspect_err(|e| print_base!("Connection {}: Got Error {}, returning", addr, e)) else {
            return;
        };
        socket.send_to(ServerPacket::Connect(ConnectionPacket::new(pos)).encode().as_raw_slice(),addr).await.unwrap();



        let mut client_connection = Self {
            chunks_loaded: HashSet::new(),
            chunks_pos: pos.get_chunk_pos(),
            packet_receiver,
            socket,
            server_world_data,
            chunks_to_load : HashSet::new(),
            player,
            prx,
            addr,
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
        self.get_chunks_to_load();
        self.send_chunks();

        loop {
            tokio::select! {
                // waiting till the socket receives data or timeout limit is reached
                packet = self.packet_receiver.recv() => {
                    if let Err(e) = self.receive(&packet) {
                        print_base!("Exiting {} due to error {}", self.addr, e);
                        break;
                    } else {
                        instant = Instant::now();
                    }
                }

                // if a packet is scheduled to be sent
                Some(packet_data) = self.prx.recv() => {
                    // ChunkPacket::from_bits(packet_data[1..].view_bits::<Lsb0>().to_bitvec());
                    self.socket.send_to(packet_data.encode().as_raw_slice(), self.addr).await.unwrap();
                    if let ServerPacket::Chunk(p) = packet_data {
                        self.chunks_to_load.remove(&p.get_chunk_pos());
                    };
                }

                // querying the player information at a constant interval
                _ = heartbeat_interval.tick() => {
                    // checking if the timeout of 5 isn't elapsed
                    if instant.elapsed().gt(&Duration::from_secs(5)) {
                        print_base!("Exiting {} due to time elapsed", self.addr);
                        break;
                    }
                    if self.ping_id.is_none() {
                        self.ping_id = Some(id);
                        self.ping = Instant::now();
                    }
                    // sending the packet to query the information of the player
                    let packet = ServerPacket::GetPlayer(GetPlayerPacket::new(id));
                    id += 1;
                    self.socket.send_to(packet.encode().as_raw_slice(),self.addr).await.unwrap();
                }
            }
        }
        self.socket.send_to(ServerPacket::Quit(QuitPacket::new()).encode().as_raw_slice(), self.addr).await.expect("");
        self.server_world_data.disconnect_player(&self.puid);
    }

    fn generate_chunks(&self) {
        let mut array: [ChunkPos; 20*20*20] = [ChunkPos::from_i32(0, 0, 0);20*20*20];
        let mut i = 0;
        for x in -10..10 {
            for y in -10..10 {
                for z in -10..10 {
                    array[i] = self.chunks_pos + ChunkPos::from_i32(x, y, z);
                    i += 1;
                }
            }
        }
        self.server_world_data.get_generator().write().unwrap().schedule_chunks(array);
    }

    fn send_chunks(&mut self) {
        let start = Instant::now();
        let arc_chunk_map = self.server_world_data.get_chunk_map().clone();
        let mut chunk_map = arc_chunk_map.write().unwrap();
        for chunk_pos in self.chunks_to_load.drain() {
            self.chunks_loaded.insert(chunk_pos);
            chunk_map.ask_for_chunks(chunk_pos,self.player.clone());
        }
        // print_base!("Sent chunks in {}ms", Instant::now().duration_since(start).as_millis());
    }

    fn receive(&mut self, frame: &Option<ClientPacket>) -> Result<(), Error> {
        match frame {
            Some(packet) => {
                self.handle_packet(&packet)
            }
            None => {
                Err(Error::new(ErrorKind::BrokenPipe,"The packet received is None"))
            },
        }
    }

    fn get_chunks_to_load(&mut self) {
        for i in 0..(self.view_distance as i32 * 2).pow(3) {
            let pos = ChunkPos::from_single_value(i, self.view_distance as i32 * 2) + self.chunks_pos;
            if !self.chunks_loaded.contains(&pos) {
            self.chunks_to_load.insert(pos);
            }
        }
        print_base!("len of chunks to load : {}",self.chunks_to_load.len());
    }

    fn handle_packet(&mut self, packet : &ClientPacket) -> Result<(), Error> {
        match packet {
            // ClientPacket::Quit(_) => Ok(()),
            ClientPacket::UpdatePlayer(p) => {
                self.update_player(p);
                if Some(p.get_id()) == self.ping_id {
                    print_base!("Ping of : {}ms",Instant::now().duration_since(self.ping).as_millis());
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
        if packet.get_pos().get_chunk_pos() != self.chunks_pos {
            self.chunks_pos = packet.get_pos().get_chunk_pos();
            self.generate_chunks();
            self.get_chunks_to_load();
            self.send_chunks();
            print_base!("New ChunkPos : {}", self.chunks_pos.deref());
        };
        if self.view_distance != packet.get_view_distance() {
            self.view_distance = packet.get_view_distance();
            self.get_chunks_to_load();
        }
    }
}