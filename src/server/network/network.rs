use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use crate::server::world_data::data::ServerWorldData;
use shared::print_base;
use std::net::{SocketAddr, SocketAddrV6};
use std::sync::{Arc};
use tokio::net::UdpSocket;
use shared::common::network::default_packet::ClientPacket;
use shared::common::network::packet_type::{ClientPacketType};
use crate::server::network::client_connection::ClientConnection;
use crate::server::network::socket::{ServerConnection, ServerSocket};

pub struct Network;

impl Network {

    pub fn listen(server_world_data: Arc<ServerWorldData>, socket_addr_v6: SocketAddrV6, ) {
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
            let mut socket = ServerSocket::new(socket_addr_v6).await.unwrap();
            let mut map = HashMap::new();
            print_base!("ServerSocket binding to {:?}", socket_addr_v6);
            loop {
                // waiting till the socket receives data or timeout limit is reached
                let (packet, addr) = match socket.receive().await {
                    Err(e) => {
                        print_base!("Got error when receiving packet {}",e.to_string());
                        continue;
                    }
                    Ok(p) => p,
                };
                // print_base!("Got packet {}", packet);
                if packet.get_packet_type() == ClientPacketType::Login {
                    print_base!("Got Connection from {}", addr);
                    let (sender, receiver) = tokio::sync::mpsc::channel(10000);
                    map.insert(addr,sender);
                    let world_ref = server_world_data.clone();
                    let clone : ServerConnection = socket.get_child(addr, receiver);
                    tokio::spawn(async move {
                        ClientConnection::start(packet, clone, world_ref).await;
                        print_base!("Connection {} finished", addr);
                    });
                } else {
                    if let Err(e) = map.get(&addr).unwrap().send(packet).await {
                        print_base!("Error when sending packet : {}",e);
                    }
                }

            }
        });
    }
}