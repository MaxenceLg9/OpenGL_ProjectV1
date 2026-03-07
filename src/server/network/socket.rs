use crate::server::world_data::data::ServerWorldData;
use bitvec::view::BitView;
use shared::common::account::puid::PUID;
use shared::common::network::network_traits::ClientPacketTrait;
use shared::common::network::packet_type::{ConnectionState, ClientPacketType};
use shared::print_base;
use std::net::SocketAddrV6;
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use bitvec::order::Lsb0;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::net::tcp::OwnedReadHalf;
use zstd::zstd_safe::WriteBuf;
use shared::common::network::client::packet;
use shared::common::network::client::packet::ClientPacket;
use shared::common::network::server::chunk_packet::ChunkPacket;
use shared::common::world::pos::chunkpos::ChunkPos;

pub struct ServerSocket {
}

impl ServerSocket {

    pub fn new() -> Self {
        Self {
        }
    }

    pub fn listen(&self, server_world_data: Arc<ServerWorldData>, socket_addr_v6: SocketAddrV6, ) {
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

    fn thread_connections(server_world_data: Arc<ServerWorldData>, socket_addr_v6: SocketAddrV6, ) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            let socket = TcpListener::bind(socket_addr_v6).await.unwrap();
            loop {
                // In async, .accept() is non-blocking!
                let (stream, addr) = match socket.accept().await {
                    Ok((stream,addr)) => (stream,addr),
                    Err(e) => {
                        print_base!("Got error {}",e); return;
                    }
                };

                print_base!("Connection from {}", addr);

                let world_ref = server_world_data.clone();
                Self::handle_client(stream, world_ref);
            }
        });
    }

    fn handle_client(stream: TcpStream, server_world_data: Arc<ServerWorldData>) {
        tokio::spawn(async move {
            let addr = stream.peer_addr().unwrap();
            let (mut reader, mut writer) = stream.into_split();

            let (puid, mut rx) = match Self::try_to_connect(server_world_data.clone(),&mut reader).await {
                Ok((puid,rx)) => (puid,rx),
                Err(e) => {
                    print_base!("Error on {}",e);
                    return;
                }
            };
            let _cstate = ConnectionState::Stream;
            let mut buffer = [0; 1024];
            loop {
                tokio::select! {
                    // CASE 1: Data coming FROM the player (Inbound)
                    result = reader.read(&mut buffer) => {
                        let n = result.unwrap_or(0);
                        if n == 0 {
                            break;
                        }
                        // implement TLS handshake
                        let p = Self::parse_request(buffer, n);

                        if Self::handle_packet(p,server_world_data.clone()) == ConnectionState::Quit {
                            break
                        }
                    }
                    //
                    Some(packet_data) = rx.recv() => {
                        // ChunkPacket::from_bits(packet_data[1..].view_bits::<Lsb0>().to_bitvec());
                        writer.write_all(&packet_data).await.unwrap();
                    }
                }
            }
            server_world_data.disconnect_player(&puid);
            print_base!("Closed connection from {}, with PUID {}",addr,puid);
        });
    }

    async fn try_to_connect(server_world_data: Arc<ServerWorldData>, reader : &mut OwnedReadHalf) -> Result<(PUID, tokio::sync::mpsc::Receiver<Vec<u8>>),String> {
        let mut buffer = [0; 1024];
        let n = reader.read(&mut buffer).await.unwrap();
        let packet = Self::parse_request(buffer,n);

        if packet.get_packet_type() != ClientPacketType::Connect {
            return Err("Packet is invalid".to_string())
        }

        match server_world_data.connect_player(packet.get_puid().clone()) {
            Err(e) => {
                Err(e)
            }
            Ok(rx) => Ok((packet.get_puid().clone(), rx)),
        }
    }

    fn handle_packet(p: ClientPacket, server_world_data: Arc<ServerWorldData>) -> ConnectionState {
        match p {
            ClientPacket::Quit(_) => ConnectionState::Quit,
            _ => ConnectionState::Stream,
        }
    }

    fn parse_request(buff: [u8; 1024], size: usize) -> ClientPacket {
        let bits = buff.split_at(size).0.view_bits();
        let p = ClientPacket::from_type(bits);
        print_base!("{}", p);
        p
    }
}