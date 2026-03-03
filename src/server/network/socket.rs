use std::net::SocketAddrV6;
use tokio::net::{TcpListener, TcpStream};
use std::sync::{Arc, RwLock};
use bitvec::prelude::BitVec;
use bitvec::view::BitView;
use tokio::io::AsyncReadExt;
use shared::common::account::puid::PUID;
use shared::print_base;
use shared::common::network::packet::{PacketTrait, Packet};
use crate::server::world_data::data::ServerWorldData;

pub struct ServerSocket {
    // receiver : channel::Receiver<Arc<dyn Packet>>,
}

impl ServerSocket {
    // pub fn new(receiver: channel::Receiver<Arc<dyn Packet>>) -> Self {
    //     Self {
    //         receiver
    //     }
    // }

    pub fn new() -> Self {
        Self {
        }
    }
    pub fn send(packet: Packet) {
        packet.serialize();
    }

    pub fn listen(&self, server_world_data: Arc<RwLock<ServerWorldData>>, socket_addr_v6: SocketAddrV6) {
        // We take ownership of the listener to move it into the thread
        // Note: TcpListener doesn't have try_clone, so we move it or use Arc
        print_base!("ServerSocket binding to {:?}", socket_addr_v6);

        std::thread::Builder::new()
            .name("network_thread".to_string())
            .spawn(move || {
                // Create a single-threaded executor for this specific thread
                Self::thread_connections(server_world_data, socket_addr_v6);
            })
            .unwrap();
    }

    fn thread_connections(server_world_data: Arc<RwLock<ServerWorldData>>, socket_addr_v6: SocketAddrV6) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();


        rt.block_on(async {
            let socket = TcpListener::bind(socket_addr_v6).await.unwrap();
            loop {
                // In async, .accept() is non-blocking!
                let (stream, addr) = socket.accept().await.unwrap();
                print_base!("Connection from {}", addr);

                let world_ref = server_world_data.clone();
                Self::handle_client(stream, world_ref);
            }
        });
    }

    fn handle_client(mut stream: TcpStream, server_world_data: Arc<RwLock<ServerWorldData>>) {
        tokio::spawn(async move {
            let mut buffer = [0; 1024];

            let n = match stream.read(&mut buffer).await {
                Ok(0) => return,
                Ok(n) => n,
                Err(e) => {
                    eprintln!("Failed to read : ");
                    return;
                }
            };

            let p = Self::parse_request(buffer, n);
            let puid = p.get_header().puid();
            server_world_data.write().as_mut().unwrap().connect_player(puid.clone());

            while let Ok(n) = stream.read(&mut buffer).await {
                if n == 0 {
                    break;
                }
                // implement TLS handshake
                let p = Self::parse_request(buffer, n);
                Self::handle_packet(p, server_world_data.clone());
            }
            server_world_data.write().as_mut().unwrap().disconnect_player(puid);
            print_base!("Closed connection from {}, with PUID {}", stream.peer_addr().unwrap(), puid);
        });
    }

    fn handle_packet(p : Packet, server_world_data: Arc<RwLock<ServerWorldData>>) {
        match p {
            Packet::Connect(_) => {
                server_world_data.write().as_mut().unwrap().connect_player(PUID::new(1));
            },
            Packet::Quit(_) => {
                return;
            },
            Packet::Update(_) => {}
            Packet::Correction(_) => {},
            _ => {}
        }
    }

    fn parse_request(buff : [u8; 1024], size : usize) -> Packet {
        let bits = buff.split_at(size).0.view_bits();
        let p = Packet::from_type(bits);
        print_base!("{}",p);
        p
    }
}