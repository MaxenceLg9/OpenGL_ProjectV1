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
use shared::common::network::network_traits::ServerNetPacket;
use shared::common::network::server::chunk_packet::ChunkPacket;
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
}

impl ClientConnection {
    pub async fn start(addr : SocketAddr, packet : ClientPacket, packet_receiver: mpsc::Receiver<ClientPacket>, socket : Arc<tokio::net::UdpSocket>, server_world_data: Arc<ServerWorldData>) {
        let (psx, prx) = mpsc::channel::<ServerPacket>(1000);
        let ClientPacket::Login(con_packet) = packet else {
            print_base!("Connection {}: Wrong Packet, returning", addr);
            return;
        };

        // let (pos, player) = match server_world_data.connect_player(con_packet.get_puid().clone(), psx) {
        //     Err(e) => {
        //         print_base!("Connection {}: Got Error {}, returning", addr, e);
        //         return;
        //     }
        //     Ok(res) => {
        //         socket.send_to(ServerPacket::Connect(ConnectionPacket::new()).encode().as_raw_slice(),addr).await.unwrap();
        //         res
        //     }
        // };

        let Ok((pos, player, chunks_to_load)) = server_world_data.connect_player(con_packet.get_puid().clone(), psx).inspect_err(|e| print_base!("Connection {}: Got Error {}, returning", addr, e)) else {
            return;
        };
        socket.send_to(ServerPacket::Connect(ConnectionPacket::new(pos)).encode().as_raw_slice(),addr).await.unwrap();


        let mut client_connection = Self {
            chunks_loaded: HashSet::new(),
            chunks_pos: pos.get_chunk_pos(),
            packet_receiver,
            socket,
            server_world_data,
            chunks_to_load,
            player,
            prx,
            addr,
            view_distance: 0,
            puid: con_packet.get_puid().clone()
        };
        client_connection.generate_chunks();
        client_connection.handle_client().await;
    }

    pub async fn handle_client(&mut self) {
        let mut heartbeat_interval = tokio::time::interval(std::time::Duration::from_millis(20));
        let mut instant = Instant::now();
        self.send_chunks();
        loop {
            tokio::select! {
                // waiting till the socket receives data or timeout limit is reached
                bytes = self.packet_receiver.recv() => {
                    if let Err(e) = self.receive(&bytes) {
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
                }

                // querying the player information at a constant interval
                _ = heartbeat_interval.tick() => {
                    // ask for the information of the player
                    if instant.elapsed().gt(&Duration::from_secs(5)) {
                        print_base!("Exiting {} due to time elapsed", self.addr);
                        break;
                    }
                    let packet = ServerPacket::GetPlayer(GetPlayerPacket::new());
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
        for chunk_pos in self.chunks_to_load.clone() {
            if let Some(chunk) = self.server_world_data.get_chunk_map().read().unwrap().get(&chunk_pos) {
                let packets = ChunkPacket::from_chunk_to_packets(chunk);
                for (_, packet) in packets {
                    self.socket.try_send_to(ServerPacket::Chunk(packet).encode().as_raw_slice(),self.addr);
                    self.chunks_loaded.insert(chunk_pos);
                }
            }
        }
        print_base!("Sent chunks in {}ms", Instant::now().duration_since(start).as_millis());
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

    fn handle_packet(&mut self, packet : &ClientPacket) -> Result<(), Error> {
        match packet {
            ClientPacket::Quit(_) => Ok(()),
            ClientPacket::AskChunk(p) => {
                // if the chunk isn't already loaded, send it
                // print_debug!("Received request for chunk {}", p.get_chunk_pos().deref());
                if !self.chunks_loaded.contains(&p.get_chunk_pos()) {
                    let chunk_map = self.server_world_data.get_chunk_map();
                    if let Some(chunk) = chunk_map.read().unwrap().get(&p.get_chunk_pos()) {
                        for (_,packet) in ChunkPacket::from_chunk_to_packets(&chunk) {
                            self.socket.try_send_to(ServerPacket::Chunk(packet).encode().as_raw_slice(), self.addr).expect("");
                        }
                        self.chunks_loaded.insert(p.get_chunk_pos().clone());
                    }
                }
                Ok(())
            },
            ClientPacket::UpdatePlayer(p) => {
                self.player.write().unwrap().set_pos(p.get_pos());
                if p.get_pos().get_chunk_pos() != self.chunks_pos {
                    self.chunks_pos = p.get_pos().get_chunk_pos();
                    self.generate_chunks();
                    print_base!("New ChunkPos : {}", self.chunks_pos.deref());
                }
                Ok(())
            },
            _ => Ok(()),
        }
    }
}