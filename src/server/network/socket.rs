use std::net::SocketAddrV6;
use tokio::net::{TcpListener, TcpStream};
use std::sync::{Arc, RwLock};
use tokio::io::AsyncReadExt;
use shared::common::account::puid::PUID;
use shared::common::network::connection_packet::ConnectionPacket;
use shared::print_base;
use shared::common::network::packet::{Packet, PacketType};
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
    pub fn send(packet: &dyn Packet) {
        packet.serialize();
    }

    pub async fn listen(&self, server_world_data: Arc<RwLock<ServerWorldData>>, socket_addr_v6: SocketAddrV6) {
        // We take ownership of the listener to move it into the thread
        // Note: TcpListener doesn't have try_clone, so we move it or use Arc
        let socket = TcpListener::bind(socket_addr_v6).await.unwrap();
        print_base!("ServerSocket binding to {:?}", socket_addr_v6);

        std::thread::Builder::new()
            .name("network_thread".to_string())
            .spawn(move || {
                // Create a single-threaded executor for this specific thread
                Self::thread_connections(server_world_data, socket);
            })
            .unwrap();
    }

    fn thread_connections(server_world_data: Arc<RwLock<ServerWorldData>>, socket: TcpListener) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
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
            while let Ok(n) = stream.read(&mut buffer).await {
                if n == 0 { break; }
                // implement TLS handshake
                let p = Self::parse_request(buffer);
                Self::handle_packet(p, server_world_data.clone());
            }
        });
    }

    fn handle_packet( p : Arc<dyn Packet>, server_world_data: Arc<RwLock<ServerWorldData>>) {
        match p.packet_type() {
            PacketType::Connect => {
                server_world_data.write().as_mut().unwrap().connect_player(PUID::new(1));
            },
            PacketType::Quit => {
                return;
            },
            PacketType::Update => {}
            PacketType::Correction => {}
        }
    }

    fn parse_request(buff : [u8; 1024]) -> Arc<dyn Packet> {
        let p = ConnectionPacket::new();
        Arc::new(p)
    }
}