use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::net::{Ipv6Addr, SocketAddr, SocketAddrV6};
use std::ops::Deref;
use std::sync::{Arc};
use std::time::{Duration, Instant};
use tokio::io::AsyncWriteExt;
use tokio::time::timeout;
use shared::common::network::client::login_packet::LoginPacket;
use shared::common::network::client::player_packet::UpdatePlayerPacket;
use shared::common::network::server::chunk_packet::ChunkPacket;
use shared::common::world::pos::chunkpos::ChunkPos;
use shared::print_base;
use crate::client::world_data::client_data::ClientWorldData;
use crossbeam::channel as cb;
use shared::common::network::l5_packet::L5Packet;
use shared::common::network::packet_type::UdpPacketType;
use shared::common::network::socket::common_socket::CommonSocket;
use shared::common::world::chunk::chunk::Chunk;

pub struct ServerConnection {
    socket: CommonSocket,
    client_world_data: Arc<ClientWorldData>,
    rx : tokio::sync::mpsc::Receiver<(L5Packet, UdpPacketType)>,
    temp_chunks : HashMap<ChunkPos,HashMap<u8, ChunkPacket>>,
    start_time : Instant,
    chunk_sender : cb::Sender<Chunk>,
    addr : SocketAddr
}

impl ServerConnection {
    pub fn start(ipv6addr: Ipv6Addr, rx : tokio::sync::mpsc::Receiver<(L5Packet, UdpPacketType)>, client_world_data: Arc<ClientWorldData>, chunk_sender : cb::Sender<Chunk>) {
        std::thread::Builder::new()
            .name("network_thread".to_string())
            .spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();

                rt.block_on(async {
                    ServerConnection::new(ipv6addr, rx, client_world_data, chunk_sender).await.unwrap();
                });
            })
            .unwrap();
    }
    pub async fn new(ipv6addr: Ipv6Addr, rx : tokio::sync::mpsc::Receiver<(L5Packet, UdpPacketType)>, client_world_data: Arc<ClientWorldData>, chunk_sender : cb::Sender<Chunk>) -> Result<(),Error> {
        let socket = CommonSocket::new(SocketAddrV6::new(ipv6addr, 50000, 0, 0)).await?;
        socket.connect(SocketAddrV6::new(ipv6addr, 25000, 0, 0)).await?;
        let addr = socket.peer_addr()?;
        // let (psx, prx) = tokio::sync::mpsc::channel(10000);
        print_base!("ClientSocket connecting to {:?}", ipv6addr);


        let mut con = Self {
            socket,
            temp_chunks: HashMap::new(),
            rx,
            start_time : Instant::now(),
            client_world_data,
            chunk_sender,
            addr,
        };
        con.connect().await?;
        con.handle_server().await;
        print_base!("Closed connection from {}", addr);
        Ok(())
    }

    async fn connect(&mut self) -> Result<(),Error> {
        self.socket.send_to(L5Packet::Login(LoginPacket::new(2000, "maxence")),self.addr, UdpPacketType::Reliable).await.expect("Panic when sending connection packet");

        let (packet, addr) = self.socket.recv_from().await?;
        let L5Packet::Connect(packet) = packet else {
            return Err(Error::new(ErrorKind::InvalidData,"Expected Connect packet, got something else"));
        };
        self.client_world_data.get_player().write().unwrap().set_pos(packet.get_pos().clone());
        Ok(())

    }

    async fn handle_server(&mut self) {
        self.start_time = Instant::now();
        loop {
            tokio::select! {
                    frame = timeout(Duration::from_secs(5), self.socket.recv_from()) => {
                        if let Err(e) = self.receive(frame).await {
                            print_base!("Breaking due to error: {}", e);
                            break;
                        }
                    }
                    Some((packet, udp_type)) = self.rx.recv() => {
                        // print_base!("Sending packet {}",packet.get_packet_type());
                        self.socket.send_to(packet, self.addr, udp_type).await.unwrap();
                    }
                }
        }
    }


    async fn receive(&mut self, frame : Result<Result<(L5Packet, SocketAddr), Error>, tokio::time::error::Elapsed>) -> Result<(), Error> {
        match frame {
            Ok(Ok((packet, addr))) => {
                self.handle_packet(&packet).await
            }
            Ok(Err(e)) => {
                Err(e)
            },
            Err(e) => {
                Err(Error::new(ErrorKind::TimedOut,e.to_string()))
            }
        }
    }

    async fn handle_packet(&mut self, p: &L5Packet) -> Result<(), Error> {
        match p {
            L5Packet::Chunk(chunk_packet) => {
                // print_base!("Receiving packets {}",p.get_packet_type());
                self.push_chunk_packet(chunk_packet);
                Ok(())
            },
            L5Packet::GetPlayer(player_packet) => {
                let tick_packet = L5Packet::UpdatePlayer(UpdatePlayerPacket::new(self.client_world_data.get_player().read().unwrap().get_coords(),8, player_packet.get_id()));
                self.socket.send_to(tick_packet, self.addr, UdpPacketType::Simple).await?;
                Ok(())
            },
            L5Packet::Connect(packet) => {
                print_base!("Spawning on the world with coords : {}", packet.get_pos().deref());
                Ok(())
            },
            _ => Ok(()),
        }
    }

    fn push_chunk_packet(&mut self, chunk_packet: &ChunkPacket) {
        let total = chunk_packet.get_total();
        let chunk_pos = chunk_packet.get_chunk_pos();
        match self.temp_chunks.entry(chunk_packet.get_chunk_pos()) {
            Entry::Occupied(mut e) => {
                e.get_mut().insert(chunk_packet.get_indice(),chunk_packet.clone());
            },
            Entry::Vacant(e) => {
                let mut submap = HashMap::new();
                submap.insert(chunk_packet.get_indice(),chunk_packet.clone());
                e.insert(submap);
            }
        }
        if self.temp_chunks.get(&chunk_pos).unwrap().len() as u8 == total {
            let c = ChunkPacket::from_packets_to_chunk(self.temp_chunks.get(&chunk_pos).expect("Error when getting"), chunk_pos);
            // self.client_world_data.get_chunks().write().unwrap().add_chunk(c.clone());
            self.chunk_sender.send(c).expect("Cannot send the chunk to the map");
        }
        // print_base!("Received packets in {}ms",Instant::now().duration_since(self.start_time).as_millis());
    }
}