use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::net::SocketAddrV6;
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use bitvec::order::BitOrder;
use tokio_stream::StreamExt;
use bitvec::view::BitView;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
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
        let mut temp : HashMap<ChunkPos,HashMap<u8, ChunkPacket>> = HashMap::new();
        let mut n = 0;
        loop {
            tokio::select! {
                    frame = framed_reader.next() => {
                        match frame {
                            Some(Ok(packet)) => {
                                if let ServerPacket::Chunk(m) = packet  {
                                    let total = m.get_total();
                                    let chunk_pos = m.get_chunk_pos();
                                    match temp.entry(m.get_chunk_pos()) {
                                        Entry::Occupied(mut e) => {
                                            e.get_mut().insert(m.get_indice(),m);
                                        },
                                        Entry::Vacant(e) => {
                                            let mut submap = HashMap::new();
                                            submap.insert(m.get_indice(),m);
                                            e.insert(submap);
                                        }
                                    }
                                    if temp.get(&chunk_pos).unwrap().len() as u8 == total {
                                        n += 1;
                                        let c = ChunkPacket::from_packets_to_chunk(temp.get(&chunk_pos).expect("Error when getting"), chunk_pos);
                                        // print_base!("Mesh at {} {},{}, {} chunks received", chunk_pos.deref(), c.ilen(), c.vlen(), n);
                                        chunk_map.write().unwrap().add_chunk(c);
                                    }
                                    continue;
                                }

                                if Self::handle_packet(packet) == ConnectionState::Quit {
                                    break;
                                }
                            },
                            Some(Err(e)) => print_base!("Error on socket {}", e),
                            None => {
                                print_base!("Exit cause of EOF");
                                break; // Connection closed or errorcreate_mesh
                            }
                        }
                    }
                    Some(packet_data) = rx.recv() => {
                        writer.write_all(&packet_data).await.unwrap();
                    }
                }
        }
        print_base!("Closed connection from {}",addr);
    }

    fn handle_packet(p: ServerPacket) -> ConnectionState {
        match p {
            ServerPacket::Chunk(_) => ConnectionState::Stream,
            _ => ConnectionState::Stream,
        }
    }

    fn get_packet_type(buff : &[u8; 1]) -> ServerPacketType {
        ServerPacketType::from_repr(buff[0]).unwrap()
    }
}

impl Drop for ClientSocket {
    fn drop(&mut self) {
    }
}