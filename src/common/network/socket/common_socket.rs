use std::collections::hash_map::Entry;
use std::collections::{HashMap, VecDeque};
use std::io::{Error, ErrorKind};
use std::net::{SocketAddr, SocketAddrV6};
use std::sync::Arc;
use std::time::Duration;
use chrono::Timelike;
use noise::{NoiseFn, Perlin};
use tokio::net::UdpSocket;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::Instant;
use crate::common::network::l5_packet::L5Packet;
use crate::common::network::packet_type::UdpPacketType;
use crate::common::network::reliable_packets::{AckPacket, ReliablePacket, SimplePacket};
use crate::common::network::udp_packet::UdpPacket;
use crate::print_base;

pub struct CommonSocket {
    // shared buffer to read the packet
    buffer: [u8; 1024],
    // the raw socket from tokio
    udp_socket: UdpSocket,
    // map to store the packet, its timeout, and the address it has to be sent
    packet_queue : VecDeque<(UdpPacket, Instant, SocketAddr)>,
    // map to store the packet that has to be acknowledged
    ack_map: HashMap<SocketAddr, AckState>,
    // map to send the received packet to the associated tokio task
    map: HashMap<SocketAddr, Sender<L5Packet>>,
}

pub struct AckState {
    ack : u32,
    acks : HashMap<u32, usize>
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

impl CommonSocket {

    pub async fn new(socket_addr_v6: SocketAddrV6) -> Result<CommonSocket, Error> {
        let udp_socket = UdpSocket::bind(socket_addr_v6).await?;
        Ok(Self {
            udp_socket,
            buffer : [0; 1024],
            packet_queue : VecDeque::new(),
            ack_map : HashMap::new(),
            map : HashMap::new()
        })
    }

    pub async fn connect(&self, socket_addr_v6: SocketAddrV6) -> Result<(), Error> {
        self.udp_socket.connect(socket_addr_v6).await
    }

    pub fn peer_addr(&self) -> Result<SocketAddr, Error> {
        self.udp_socket.peer_addr()
    }

    async fn parse(&self, frame : Result<(usize, SocketAddr), Error>) -> Result<(UdpPacket, SocketAddr), Error> {
        match frame {
            Ok((size, addr)) => {
                // trying to parse the raw bytes into a struct
                match UdpPacket::decode(&self.buffer[0..size].to_vec()){
                    Some(p) => {
                        Ok((p,addr))
                    },
                    None => Err(Error::new(ErrorKind::StaleNetworkFileHandle,"Cannot parse the packet, invalid bytes")),
                }
            }
            Err(e) => {
                Err(e)
            }
        }
    }

    pub async fn recv_from(&mut self) -> Result<(L5Packet, SocketAddr), Error> {
        let mut timeout = Box::pin(tokio::time::sleep_until(Instant::now() + Duration::from_hours(1)));
        loop {
            tokio::select! {
                frame = self.udp_socket.recv_from(&mut self.buffer) => {
                    let (packet, addr) = self.parse(frame).await?;
                    return match packet {
                        UdpPacket::Reliable(packet) => {
                            self.send_raw(UdpPacket::Ack(AckPacket::new(packet.get_ack())), addr).await?;
                            Ok((packet.get_l5_packet(), addr))
                        }
                        UdpPacket::Simple(packet) => {
                            Ok((packet.get_l5_packet(), addr))
                        }
                        UdpPacket::Ack(packet) => {
                            self.add_ack(addr,packet);
                            continue;
                        }
                    };
                }
                _ = &mut timeout => {
                    match self.packet_queue.pop_front() {
                        Some((packet,instant, addr)) => {
                            self.send_raw(packet.clone(),addr).await.expect("Cannot re send packet");
                            self.packet_queue.push_back((packet.clone(),Instant::now() + Duration::from_millis(500), addr));
                            match self.packet_queue.front() {
                                None => {
                                    timeout.as_mut().reset(Instant::now() + Duration::from_hours(1))
                                }
                                Some((packet,instant, addr)) => {
                                    timeout.as_mut().reset(instant.clone());
                                }
                            }
                        }
                        None => {
                            print_base!("Long timeout reached");
                            timeout.as_mut().reset(Instant::now() + Duration::from_hours(1));
                        }
                    }
                }
            }

        }
    }

    fn add_ack(&mut self, socket_addr: SocketAddr, packet : AckPacket) {
        match self.ack_map.entry(socket_addr) {
            Entry::Occupied(mut e) => {
                if let Some(index) = e.get_mut().acks.remove(&packet.get_ack()) {
                    self.packet_queue.remove(index);
                }
            }
            Entry::Vacant(e) => {
                print_base!("Invalid ack packet from {}", socket_addr)
            }
        };
    }

    /// public function to send a packet L5Packet (Layer 5) to an Address, with the associated packaging provided by the UdpPacketType
    pub async fn send_to(&mut self, packet : L5Packet, addr : SocketAddr, udp_packet_type: UdpPacketType) -> Result<usize, Error>{
        match udp_packet_type {
            UdpPacketType::Reliable => {
                let ack = match self.ack_map.entry(addr) {
                    Entry::Occupied(mut e) => {
                        e.get_mut().add_ack(self.packet_queue.len())
                    },
                    Entry::Vacant(e) => {
                        let mut ack_state = AckState::new();
                        let r = ack_state.add_ack(self.packet_queue.len());
                        e.insert(ack_state);
                        r
                    }
                };
                let udp_packet = UdpPacket::Reliable(ReliablePacket::new(ack, packet));
                self.packet_queue.push_back((udp_packet.clone(),Instant::now(), addr));
                self.send_raw(udp_packet, addr).await
            }
            UdpPacketType::Simple => {
                let udp_packet = UdpPacket::Simple(SimplePacket::new(packet));
                self.packet_queue.push_back((udp_packet.clone(),Instant::now(), addr));
                self.send_raw(udp_packet, addr).await
            }
            UdpPacketType::Ack => {
                Err(Error::new(ErrorKind::InvalidInput, "Type Ack is not valid to be sent"))
            }
        }
    }

    async fn send_raw(&self, udp_packet: UdpPacket, socket_addr: SocketAddr) -> Result<usize,Error>{
        self.udp_socket.send_to(udp_packet.encode().as_raw_slice(),socket_addr).await
    }
}