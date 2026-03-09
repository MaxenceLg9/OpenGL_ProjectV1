use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::io::Error;
use std::net::SocketAddrV6;
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use bitvec::order::BitOrder;
use tokio_stream::StreamExt;
use bitvec::view::BitView;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::time::timeout;
use tokio_util::codec::FramedRead;
use shared::common::network::client::connection_packet::ConnectionPacket;
use shared::common::network::client::packet::ClientPacket;
use shared::common::network::network_traits::{PacketTrait, ServerPacketTrait};
use shared::common::network::packet_type::{ConnectionState, ServerPacketType};
use shared::common::network::server::chunk_packet::ChunkPacket;
use shared::common::network::server::packet::ServerPacket;
use shared::common::world::chunk::chunkmap::ChunkMap;
use shared::common::world::pos::chunkpos::ChunkPos;
use shared::print_base;
use crate::client::display::renderer::mesh::chunk_mesh::Mesh;
use crate::client::network::decoder::PacketCodec;
use crate::client::world_data::mesh_map::MeshMap;

pub struct ClientSocket {
    sender : tokio::sync::mpsc::Sender<Vec<u8>>
}

impl ClientSocket {
    pub fn new(socket_addr_v6: SocketAddrV6, meshes : Arc<RwLock<ChunkMap>>) -> Self {
        let (sender, rx) = tokio::sync::mpsc::channel(20);
        Self::listen(socket_addr_v6, rx, meshes);
        Self {
            sender
        }
    }

    pub fn send(&self) {
        self.sender.try_send(ClientPacket::Connect(ConnectionPacket::new(2000, "maxence\0".to_string())).serialize().into_vec());
    }

    pub fn listen(socket_addr_v6: SocketAddrV6, rx : tokio::sync::mpsc::Receiver<Vec<u8>>, meshes : Arc<RwLock<ChunkMap>>) {
        print_base!("ClientSocket binding to {:?}", socket_addr_v6);

        std::thread::Builder::new()
            .name("network_thread".to_string())
            .spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();

                rt.block_on(async {
                    Self::handle_server(socket_addr_v6, rx, meshes).await;
                });
            })
            .unwrap();
    }

    async fn handle_server(socket_addr_v6: SocketAddrV6, mut rx : tokio::sync::mpsc::Receiver<Vec<u8>>, chunk_map: Arc<RwLock<ChunkMap>>) {
        let stream = tokio::net::TcpStream::connect(socket_addr_v6).await.unwrap();
        print_base!("ClientSocket connecting to {:?}", socket_addr_v6);

        let addr = stream.peer_addr().unwrap();
        let (reader, mut writer) = stream.into_split();
        let mut framed_reader = FramedRead::new(reader, PacketCodec);
        let _cstate = ConnectionState::Stream;

        loop {
            tokio::select! {
                    frame = timeout(Duration::from_secs(5), framed_reader.next()) => {
                        if Self::handle_frame(frame, chunk_map.clone()) == ConnectionState::Quit {
                            break;
                        }
                    }
                    Some(packet_data) = rx.recv() => {
                        writer.write_all(&packet_data).await.unwrap();
                    }
                }
        }
        print_base!("Closed connection from {}",addr);
    }

    fn handle_packet(p: &ServerPacket, chunk_map: Arc<RwLock<ChunkMap>>) -> ConnectionState {
        match p {
            ServerPacket::Chunk(chunk_packet) => {
                chunk_map.write().unwrap().add_temp(chunk_packet.clone());
                ConnectionState::Stream
            },
            _ => ConnectionState::Stream,
        }
    }

    fn handle_frame(frame : Result<Option<Result<ServerPacket,Error>>,tokio::time::error::Elapsed>, chunk_map: Arc<RwLock<ChunkMap>>) -> ConnectionState {
        match frame {
            Ok(Some(Ok(packet))) => {
                Self::handle_packet(&packet, chunk_map)
            },
            Ok(Some(Err(e))) => {
                print_base!("Error on socket {}", e);
                ConnectionState::Quit
            },
            Ok(None) => {
                print_base!("Exit cause of EOF");
                ConnectionState::Quit
            },
            Err(e) => {
                print_base!("Error due to timeout");
                ConnectionState::Quit
            }
        }
    }
}

impl Drop for ClientSocket {
    fn drop(&mut self) {
    }
}