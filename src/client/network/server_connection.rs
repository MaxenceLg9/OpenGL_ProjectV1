use std::io::{Error, ErrorKind};
use std::net::{Ipv6Addr, SocketAddr, SocketAddrV6};
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpStream, UdpSocket};
use tokio::sync::mpsc::Sender;
use tokio::time::timeout;
use shared::common::network::client::login_packet::LoginPacket;
use shared::common::network::client::packet::ClientPacket;
use shared::common::network::client::player_packet::UpdatePlayerPacket;
use shared::common::network::packet_type::{ClientPacketType, ConnectionState, ServerPacketType};
use shared::common::network::server::packet::ServerPacket;
use shared::common::world::chunk::chunkmap::ChunkMap;
use shared::print_base;
use crate::client::world_data::client_data::ClientWorldData;

pub struct ServerConnection {
    socket: UdpSocket,
    client_world_data: Arc<ClientWorldData>,
    rx : tokio::sync::mpsc::Receiver<ClientPacket>,
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
            rx,
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
        loop {
            tokio::select! {
                    frame = timeout(Duration::from_secs(5), self.socket.recv(&mut buff)) => {
                        if let Err(e) = self.receive(frame, self.client_world_data.get_chunks().clone(), buff) {
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


    fn receive(&mut self, frame : Result<Result<usize, Error>, tokio::time::error::Elapsed>, chunk_map: Arc<RwLock<ChunkMap>>, buff : [u8; 1024]) -> Result<(), Error> {
        match frame {
            Ok(Ok(size)) => {
                // trying to parse the raw bytes into a struct
                let packet = match ServerPacket::decode(&buff[0..size].to_vec()){
                    Some(p) => p,
                    None => return Err(Error::new(ErrorKind::StaleNetworkFileHandle,format!("Cannot parse the packet, invalid bytes {:?}", buff[0..size].to_vec()))),
                };
                // handling the packet as it is correct
                self.handle_packet(&packet, chunk_map)
            }
            Ok(Err(e)) => {
                Err(e)
            },
            Err(e) => {
                Err(Error::new(ErrorKind::TimedOut,e.to_string()))
            }
        }
    }

    fn handle_packet(&mut self, p: &ServerPacket, chunk_map: Arc<RwLock<ChunkMap>>) -> Result<(), Error> {
        match p {
            ServerPacket::Chunk(chunk_packet) => {
                print_base!("Receiving packets {}",p.get_packet_type());
                chunk_map.write().unwrap().add_temp(chunk_packet.clone());
                Ok(())
            },
            ServerPacket::GetPlayer(player_packet) => {
                let tick_packet = ClientPacket::UpdatePlayer(UpdatePlayerPacket::new(self.client_world_data.get_player().read().unwrap().get_block_pos()));
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