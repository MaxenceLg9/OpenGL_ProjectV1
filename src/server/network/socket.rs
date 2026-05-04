use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use crate::server::world_data::data::ServerWorldData;
use shared::print_base;
use std::net::{SocketAddr, SocketAddrV6};
use std::sync::{Arc, RwLock};
use tokio::net::UdpSocket;
use shared::common::network::client::packet::ClientPacket;
use shared::common::network::packet_type::{ClientPacketType, ServerPacketType};
use shared::common::network::packet_type::ConnectionState::Quit;
use shared::common::network::server::packet::ServerPacket;
use shared::common::world::chunk::chunkmap::ChunkMap;
use crate::server::network::client_connection::ClientConnection;

pub struct ServerSocket {
}

impl ServerSocket {

    pub fn new() -> Self {
        Self {
        }
    }

    pub fn listen(&self, server_world_data: Arc<ServerWorldData>, socket_addr_v6: SocketAddrV6, ) {
        std::thread::Builder::new()
            .name("network_thread".to_string())
            .spawn(move || {
                // Create a single-threaded executor for this specific thread
                Self::thread_connections(server_world_data, socket_addr_v6);
            })
            .unwrap();
    }

    fn thread_connections(server_world_data: Arc<ServerWorldData>, socket_addr_v6: SocketAddrV6, ) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            let socket = Arc::new(UdpSocket::bind(socket_addr_v6).await.unwrap());
            let mut map = HashMap::new();
            let mut buff: [u8; 1024]  = [0;1024];
            print_base!("ServerSocket binding to {:?}", socket_addr_v6);
            loop {
                // waiting till the socket receives data or timeout limit is reached
                let result = socket.recv_from(&mut buff).await;
                let (packet, addr) = match Self::receive(result,buff) {
                    Err(e) => {
                        print_base!("Got error when receiving packet {}",e.to_string());
                        continue;
                    }
                    Ok(p) => p,
                };
                // print_base!("Got packet {}", packet);
                if packet.get_packet_type() == ClientPacketType::Login {
                    print_base!("Got Connection from {}", addr);
                    let (sender, receiver) = tokio::sync::mpsc::channel(1000);
                    map.insert(addr,sender);
                    let world_ref = server_world_data.clone();
                    let clone = socket.clone();
                    tokio::spawn(async move {
                        ClientConnection::start(addr, packet, receiver, clone, world_ref).await;
                        print_base!("Connection {} finished", addr);
                    });
                } else {
                    map.get(&addr).unwrap().send(packet).await.unwrap();
                }

            }
        });
    }

    fn receive(frame : Result<(usize, SocketAddr), Error>, buff : [u8; 1024]) -> Result<(ClientPacket, SocketAddr), Error> {
        match frame {
            Ok(result) => {
                /// trying to parse the raw bytes into a struct
                match ClientPacket::decode(&buff[0..result.0].to_vec()){
                    Some(p) => Ok((p,result.1)),
                    None => return Err(Error::new(ErrorKind::StaleNetworkFileHandle,"Cannot parse the packet, invalid bytes")),
                }
            }
            Err(e) => {
                Err(e)
            }
        }
    }
}