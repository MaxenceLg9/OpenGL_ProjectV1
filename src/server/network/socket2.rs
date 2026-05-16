use std::collections::{HashMap, VecDeque};
use std::collections::hash_map::Entry;
use std::io::{Error, ErrorKind};
use std::net::{SocketAddr, SocketAddrV6};
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration};
use shared::print_base;
use tokio::net::UdpSocket;
use chrono::Timelike;
use noise::{NoiseFn, Perlin};
use shared::common::network::udp_packet::{UdpPacket};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::time::Instant;
use shared::common::network::l5_packet::L5Packet;
use shared::common::network::packet_type::{ClientPacketType, L5PacketType, UdpPacketType};
use shared::common::network::packet_type::UdpPacketType::Reliable;
use shared::common::network::reliable_packets::{AckPacket, ReliablePacket, SimplePacket};
use crate::server::network::client_connection::ClientConnection;
use crate::server::world_data::data::ServerWorldData;
use tokio::sync::mpsc as tchannel;
use shared::common::network::socket::common_socket::CommonSocket;

pub struct AckState {
    ack : u32,
    acks : HashMap<u32, usize>
}
pub struct Socket {
    receiver: Receiver<(L5Packet, SocketAddr, UdpPacketType)>,
    sender: Sender<(L5Packet, SocketAddr, UdpPacketType)>,
    socket: CommonSocket,
    packet_queue : VecDeque<(UdpPacket, Instant, SocketAddr)>,
    ack_map: HashMap<SocketAddr, AckState>,
    map: HashMap<SocketAddr, Sender<L5Packet>>,
    server_world_data: Arc<ServerWorldData>
}

impl AckState {

    pub fn new() -> AckState {
        let time = chrono::offset::Local::now().second();
        let ack = Perlin::new(time % 2_u32.pow(20)).get([time as f64 / 3600.0, time as f64 / 24.0 ]) * 2.0_f64.powi(18);
        Self {
            ack: ack as u32,
            acks: HashMap::new(),
        }
    }
    pub fn add_ack(&mut self, index : usize) -> u32 {
        self.ack += 1;
        self.acks.insert(self.ack, index);
        self.ack
    }
}

impl Socket {

    pub fn listen(server_world_data: Arc<ServerWorldData>, socket_addr_v6: SocketAddrV6, ) {
        std::thread::Builder::new()
            .name("network_thread".to_string())
            .spawn(move || {
                // Create a single-threaded executor for this specific thread
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();

                rt.block_on(async {
                    print_base!("Starting network_thread at {}", chrono::offset::Local::now());
                    if let Ok(mut socket) = Self::new(server_world_data,socket_addr_v6).await {
                        socket.poll().await;
                    }
                });
            })
            .unwrap();
    }

    pub async fn new(server_world_data: Arc<ServerWorldData>, socket_addr_v6: SocketAddrV6) -> Result<Socket, Error> {
        let (sender, receiver) = channel(1000);
        let udp_socket = CommonSocket::new(socket_addr_v6).await?;
        Ok(Self {
            receiver,
            sender,
            server_world_data,
            map : HashMap::new(),
            packet_queue: VecDeque::new(),
            ack_map : HashMap::new(),
            socket: udp_socket
        })
    }

    pub async fn poll(&mut self) {
        loop {
            tokio::select! {
                frame = self.socket.recv_from() => {
                    self.handle_frame(frame).await;
                }
                p = self.receiver.recv() => {
                    if let Some((packet, addr, udp_packet_type)) = p {
                        self.socket.send_to(packet,addr, udp_packet_type).await;
                    }
                }
            }
        }
    }

    async fn handle_packet(&mut self, l5_packet: L5Packet, socket_addr: SocketAddr) {

        if l5_packet.get_packet_type() == L5PacketType::Login {
            print_base!("Got Connection from {}", socket_addr);
            let (sender, receiver) = tokio::sync::mpsc::channel(10000);
            self.map.insert(socket_addr,sender);
            let world_ref = self.server_world_data.clone();
            let clone : ServerConnection = self.get_child(socket_addr, receiver);
            tokio::spawn(async move {
                ClientConnection::start(l5_packet.clone(), clone, world_ref).await;
                print_base!("Connection {} finished", socket_addr);
            });
        } else {
            if let Err(e) = self.map.get(&socket_addr).unwrap().send(l5_packet.clone()).await {
                print_base!("Error when sending packet : {}",e);
            }
        }
    }

    async fn handle_frame(&mut self, frame :  Result<(L5Packet, SocketAddr), Error>) {
        let (packet, addr) : (L5Packet, SocketAddr) = match frame {
            Err(e) => {
                print_base!("Got error when receiving packet {}",e.to_string());
                return;
            }
            Ok(p) => p,
        };
        self.handle_packet(packet, addr).await;
        // print_base!("Got packet {}", packet);

    }

    fn get_child(&self, socket_addr: SocketAddr, packet_receiver : Receiver<L5Packet>) -> ServerConnection {
        ServerConnection::new(self.sender.clone(),socket_addr, packet_receiver)
    }
}

pub struct ServerConnection {
    sender: tchannel::Sender<(L5Packet,SocketAddr,UdpPacketType)>,
    socket_addr : SocketAddr,
    packet_receiver: tchannel::Receiver<L5Packet>,
}

impl ServerConnection {
    pub fn new(sender : tchannel::Sender<(L5Packet,SocketAddr,UdpPacketType)>, socket_addr: SocketAddr, packet_receiver: tchannel::Receiver<L5Packet>) -> ServerConnection {
        Self {
            sender,
            socket_addr,
            packet_receiver
        }
    }

    pub async fn send(&self, l5packet: L5Packet, udp_packet_type: UdpPacketType) -> Result<(),tokio::sync::mpsc::error::SendError<(L5Packet,SocketAddr, UdpPacketType)>> {
        self.sender.send((l5packet,self.socket_addr, udp_packet_type)).await
    }

    pub fn get_addr(&self) -> SocketAddr {
        self.socket_addr
    }

    pub async fn recv(&mut self) -> Option<L5Packet> {
        self.packet_receiver.recv().await
    }
}