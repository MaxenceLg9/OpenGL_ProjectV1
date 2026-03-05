use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::net::SocketAddrV6;
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
use shared::common::network::server::mesh_packet::MeshPacket;
use shared::common::network::server::packet::ServerPacket;
use shared::common::world::pos::chunkpos::ChunkPos;
use shared::print_base;
use crate::client::display::renderer::mesh::chunk_mesh::ChunkMesh;
use crate::client::network::decoder::PacketCodec;

pub struct ClientSocket {
    sender : tokio::sync::mpsc::Sender<Vec<u8>>
}

impl ClientSocket {
    pub fn new(socket_addr_v6: SocketAddrV6, meshes : Arc<RwLock<HashMap<ChunkPos,ChunkMesh>>>) -> Self {
        let (sender, rx) = tokio::sync::mpsc::channel(20);
        Self::listen(socket_addr_v6, rx, meshes);
        Self {
            sender
        }
    }

    pub fn send(&self) {
        self.sender.try_send(ClientPacket::Connect(ConnectionPacket::new(2000, "maxence\0".to_string())).serialize().into_vec());
    }

    pub fn listen(socket_addr_v6: SocketAddrV6, rx : tokio::sync::mpsc::Receiver<Vec<u8>>, meshes : Arc<RwLock<HashMap<ChunkPos,ChunkMesh>>>) {
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

    async fn handle_server(socket_addr_v6: SocketAddrV6, mut rx : tokio::sync::mpsc::Receiver<Vec<u8>>, meshes : Arc<RwLock<HashMap<ChunkPos,ChunkMesh>>>) {
        let stream = tokio::net::TcpStream::connect(socket_addr_v6).await.unwrap();
        print_base!("ClientSocket connecting to {:?}", socket_addr_v6);

        let addr = stream.peer_addr().unwrap();
        let (reader, mut writer) = stream.into_split();
        let mut framed_reader = FramedRead::new(reader, PacketCodec);
        let _cstate = ConnectionState::Stream;
        let mut temp : HashMap<ChunkPos,HashMap<u8,MeshPacket>> = HashMap::new();
        loop {
            tokio::select! {
                    frame = framed_reader.next() => {
                        match frame {
                            Some(Ok(packet)) => {
                                if let ServerPacket::Mesh(m) = packet  {
                                    match temp.entry(m.get_chunk_pos()) {
                                        Entry::Occupied(mut e) => {
                                            let total = m.get_total();
                                            e.get_mut().insert(m.get_indice(),m);
                                            if e.get().len() as u8 == total {
                                                let c = ChunkMesh::from_packets(e.get());
                                                print_base!("Mesh {}", c.is_linked());
                                                meshes.write().unwrap().insert(e.key().clone(),c);
                                            }
                                        },
                                        Entry::Vacant(e) => {
                                            let mut submap = HashMap::new();
                                            submap.insert(m.get_indice(),m);
                                            e.insert(submap);
                                        }
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
                                break; // Connection closed or error
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
            ServerPacket::Mesh(_) => ConnectionState::Stream,
            _ => ConnectionState::Stream,
        }
    }

    fn get_header_size(t : ServerPacketType) -> u16 {
        0
    }

    fn get_packet_type(buff : &[u8; 1]) -> ServerPacketType {
        ServerPacketType::from_repr(buff[0]).unwrap()
    }

    fn parse_request(buff: &[u8; 1], size: usize) -> ServerPacket {
        let bits = buff.split_at(size).0.view_bits();
        let p = ServerPacket::from_bits(ServerPacketType::Mesh, bits);
        print_base!("{}", p);
        p
    }
}

impl Drop for ClientSocket {
    fn drop(&mut self) {
    }
}