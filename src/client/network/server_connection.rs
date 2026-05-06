use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::net::{Ipv6Addr, SocketAddrV6};
use std::ops::Deref;
use std::sync::{Arc};
use std::time::{Duration, Instant, UNIX_EPOCH};
use tokio::io::AsyncWriteExt;
use tokio::net::{UdpSocket};
use tokio::time::timeout;
use shared::common::network::client::login_packet::LoginPacket;
use shared::common::network::client::packet::ClientPacket;
use shared::common::network::client::player_packet::UpdatePlayerPacket;
use shared::common::network::server::chunk_packet::ChunkPacket;
use shared::common::network::server::packet::ServerPacket;
use shared::common::world::pos::chunkpos::ChunkPos;
use shared::print_base;
use crate::client::world_data::client_data::ClientWorldData;

pub struct ServerConnection {
    socket: UdpSocket,
    client_world_data: Arc<ClientWorldData>,
    rx : tokio::sync::mpsc::Receiver<ClientPacket>,
    temp_chunks : HashMap<ChunkPos,HashMap<u8, ChunkPacket>>,
    start_time : Instant
}

impl ServerConnection {
    pub fn start(ipv6addr: Ipv6Addr, rx : tokio::sync::mpsc::Receiver<ClientPacket>, client_world_data: Arc<ClientWorldData>) {
        std::thread::Builder::new()
            .name("network_thread".to_string())
            .spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();

                rt.block_on(async {
                    ServerConnection::new(ipv6addr, rx, client_world_data).await;
                });
            })
            .unwrap();
    }
    pub async fn new(ipv6addr: Ipv6Addr, rx : tokio::sync::mpsc::Receiver<ClientPacket>, client_world_data: Arc<ClientWorldData>) -> Result<(),Error> {
        let socket = tokio::net::UdpSocket::bind(SocketAddrV6::new(ipv6addr, 50000, 0, 0)).await?;
        socket.connect(SocketAddrV6::new(ipv6addr, 25000, 0, 0)).await?;
        let addr = socket.peer_addr()?;
        // let (psx, prx) = tokio::sync::mpsc::channel(10000);
        print_base!("ClientSocket connecting to {:?}", ipv6addr);


        let mut con = Self {
            socket,
            temp_chunks: HashMap::new(),
            rx,
            start_time : Instant::now(),
            client_world_data
        };
        con.connect().await?;
        con.handle_server().await;
        print_base!("Closed connection from {}", addr);
        Ok(())
    }

    async fn connect(&self) -> Result<(),Error> {
        self.socket.send(ClientPacket::Login(LoginPacket::new(2000, "maxence".to_string())).encode().as_raw_slice()).await.expect("Panic when sending connection packet");
        let mut buff: [u8; 1024] = [0; 1024];

        let bytes = self.socket.recv(&mut buff).await?;
        let Some(ServerPacket::Connect(packet)) = ServerPacket::decode(&buff[0..bytes].to_vec()) else {
            return Err(Error::new(ErrorKind::InvalidData,"Expected Connect packet, got something else"));
        };
        self.client_world_data.get_player().write().unwrap().set_pos(packet.get_pos().clone());
        Ok(())

    }

    async fn handle_server(&mut self) {
        let mut buff : [u8; 1024] = [0; 1024];
        self.start_time = Instant::now();
        loop {
            tokio::select! {
                    frame = timeout(Duration::from_secs(5), self.socket.recv(&mut buff)) => {
                        if let Err(e) = self.receive(frame, buff) {
                            print_base!("Breaking due to error: {}", e);
                            break;
                        }
                    }
                    Some(packet) = self.rx.recv() => {
                        // print_base!("Sending packet {}",packet.get_packet_type());
                        self.socket.send(packet.encode().as_raw_slice()).await.unwrap();
                    }
                }
        }
    }


    fn receive(&mut self, frame : Result<Result<usize, Error>, tokio::time::error::Elapsed>, buff : [u8; 1024]) -> Result<(), Error> {
        match frame {
            Ok(Ok(size)) => {
                // trying to parse the raw bytes into a struct
                let packet = match ServerPacket::decode(&buff[0..size].to_vec()){
                    Some(p) => p,
                    None => return Err(Error::new(ErrorKind::StaleNetworkFileHandle,format!("Cannot parse the packet, invalid bytes {:?}", buff[0..size].to_vec()))),
                };
                // handling the packet as it is correct
                self.handle_packet(&packet)
            }
            Ok(Err(e)) => {
                Err(e)
            },
            Err(e) => {
                Err(Error::new(ErrorKind::TimedOut,e.to_string()))
            }
        }
    }

    fn handle_packet(&mut self, p: &ServerPacket) -> Result<(), Error> {
        match p {
            ServerPacket::Chunk(chunk_packet) => {
                // print_base!("Receiving packets {}",p.get_packet_type());
                self.client_world_data.get_chunks().write().unwrap().add_temp(chunk_packet.clone());
                // let total = chunk_packet.get_total();
                // let chunk_pos = chunk_packet.get_chunk_pos();
                // match self.temp_chunks.entry(chunk_packet.get_chunk_pos()) {
                //     Entry::Occupied(mut e) => {
                //         e.get_mut().insert(chunk_packet.get_indice(),chunk_packet.clone());
                //     },
                //     Entry::Vacant(e) => {
                //         let mut submap = HashMap::new();
                //         submap.insert(chunk_packet.get_indice(),chunk_packet.clone());
                //         e.insert(submap);
                //     }
                // }
                // if self.temp_chunks.get(&chunk_pos).unwrap().len() as u8 == total {
                //     let c = ChunkPacket::from_packets_to_chunk(self.temp_chunks.get(&chunk_pos).expect("Error when getting"), chunk_pos);
                //     self.client_world_data.get_chunks().write().unwrap().add_chunk(c);
                //     // self.add_chunk(c);
                // }
                print_base!("Received packets in {}ms",Instant::now().duration_since(self.start_time).as_millis());
                Ok(())
            },
            ServerPacket::GetPlayer(player_packet) => {
                let tick_packet = ClientPacket::UpdatePlayer(UpdatePlayerPacket::new(self.client_world_data.get_player().read().unwrap().get_block_pos(),10));
                self.socket.try_send(&tick_packet.encode().into_vec())?;
                Ok(())
            },
            ServerPacket::Connect(packet) => {
                print_base!("Spawning on the world with coords : {}", packet.get_pos().deref());
                Ok(())
            },
            _ => Ok(()),
        }
    }
}