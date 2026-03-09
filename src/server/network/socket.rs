use std::io::Error;
use crate::server::world_data::data::ServerWorldData;
use bitvec::view::BitView;
use shared::common::account::puid::PUID;
use shared::common::network::network_traits::ClientPacketTrait;
use shared::common::network::packet_type::{ConnectionState, ClientPacketType};
use shared::print_base;
use std::net::SocketAddrV6;
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};
use bitvec::order::Lsb0;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::net::tcp::OwnedReadHalf;
use tokio::time::timeout;
use tokio_stream::StreamExt;
use tokio_util::codec::FramedRead;
use zstd::zstd_safe::WriteBuf;
use shared::common::network::client::packet;
use shared::common::network::client::packet::ClientPacket;
use shared::common::network::server::chunk_packet::ChunkPacket;
use shared::common::network::server::packet::ServerPacket;
use shared::common::world::chunk::chunkmap::ChunkMap;
use shared::common::world::pos::chunkpos::ChunkPos;
use crate::server::network::decoder::PacketCodec;

pub struct ServerSocket {
}

pub struct CState {
    cstate: ConnectionState,
    time: Instant
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
            let (reader, mut writer) = stream.into_split();
            let mut framed_reader = FramedRead::new(reader, PacketCodec);

            let (puid, mut rx) : (PUID, tokio::sync::mpsc::Receiver<Vec<u8>>) = match Self::try_to_connect(server_world_data.clone(),framed_reader.next().await).await {
                Ok((puid,rx)) => (puid,rx),
                Err(e) => {
                    print_base!("Error on {}",e);
                    return;
                }
            };
            let mut cstate = CState {
                cstate: ConnectionState::Stream,
                time: Instant::now()
            };
            let mut heartbeat_interval = tokio::time::interval(std::time::Duration::from_millis(100));
            loop {
                tokio::select! {
                    // waiting till the socket receives data or timeout limit is reached
                    frame = timeout(Duration::from_secs(5), framed_reader.next()) => {
                        if Self::handle_frame(frame, server_world_data.clone()) == ConnectionState::Quit {
                            break;
                        }
                    }

                    // if a packet is scheduled to be sent
                    Some(packet_data) = rx.recv() => {
                        // ChunkPacket::from_bits(packet_data[1..].view_bits::<Lsb0>().to_bitvec());
                        writer.write_all(&packet_data).await.unwrap();
                    }

                    // querying the player information at a constant interval
                    _ = heartbeat_interval.tick() => {
                        if cstate.cstate == ConnectionState::Stream {
                            cstate.time = Instant::now()
                            // ask for the information of the player
                        }
                    }
                }
            }
            server_world_data.disconnect_player(&puid);
            print_base!("Closed connection from {}, with PUID {}",addr,puid);
        });
    }

    
    async fn try_to_connect(server_world_data: Arc<ServerWorldData>, frame : Option<Result<ClientPacket,Error>>) -> Result<(PUID, tokio::sync::mpsc::Receiver<Vec<u8>>),String> {
        match frame {
            Some(Ok(packet)) => {
                if packet.get_packet_type() != ClientPacketType::Connect {
                    return Err("Packet is invalid".to_string())
                }

                match server_world_data.connect_player(packet.get_puid().clone()) {
                    Err(e) => {
                        Err(e)
                    }
                    Ok(rx) => Ok((packet.get_puid().clone(), rx)),
                }
            },
            _ => Err("Error when trying to connect the client".to_string())
        }


    }

    fn handle_frame(frame : Result<Option<Result<ClientPacket,Error>>,tokio::time::error::Elapsed>, server_world_data: Arc<ServerWorldData>) -> ConnectionState {
        match frame {
            Ok(Some(Ok(packet))) => {
                Self::handle_packet(&packet, server_world_data)
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

    fn handle_packet(p: &ClientPacket, server_world_data: Arc<ServerWorldData>) -> ConnectionState {
        match p {
            ClientPacket::Quit(_) => ConnectionState::Quit,
            _ => ConnectionState::Stream,
        }
    }
}