use crate::common::error::err_type::{CustomErrorKind, GameError};
use crate::common::network::l5_packet::L5Packet;
use crate::common::network::packet_type::L5PacketType::Quit;
use crate::common::network::packet_type::UdpPacketType;
use crate::common::network::reliable_packets::{AckPacket, ReliablePacket, SimplePacket};
use crate::common::network::udp_packet::UdpPacket;
use crate::{custom_error, print_base};
use rand::RngExt;
use rand::rngs::ChaCha20Rng;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, VecDeque};
use std::io::Error;
use std::net::SocketAddr;
use std::ops::Deref;
use std::pin::Pin;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time::{Instant, Sleep};

pub struct GameSocket {
    udp_socket: UdpSocket,
}

pub struct CommonSocket {
    // shared buffer to read the packet
    buffer: [u8; 1300],
    // the raw socket from tokio
    game_socket: GameSocket,
    // map to store the packet, its timeout, and the address it has to be sent
    resend_queue: VecDeque<(UdpPacket, Instant, SocketAddr, Ack)>,
    // map to store the packet that has to be acknowledged
    acks: ReliabilityManager,
    // timeout
    timeout: Pin<Box<Sleep>>,
}

impl GameSocket {
    pub async fn send_raw(&self, udp_packet: UdpPacket, socket_addr: SocketAddr) -> Result<usize, GameError> {
        self.udp_socket
            .send_to(udp_packet.encode()?.as_raw_slice(), socket_addr)
            .await
            .map_err(|e| custom_error!(CustomErrorKind::NetworkSocket, e.to_string()))
    }

    pub async fn new(socket_addr: SocketAddr) -> Result<Self, GameError> {
        let udp_socket = UdpSocket::bind(socket_addr).await?;
        Ok(Self { udp_socket })
    }
}

impl Deref for GameSocket {
    type Target = UdpSocket;

    fn deref(&self) -> &Self::Target {
        &self.udp_socket
    }
}

pub type Ack = u64;

pub struct ReliabilityManager {
    resend_queue: VecDeque<(UdpPacket, Instant, SocketAddr, Ack)>,
    acks: HashMap<SocketAddr, Ack>,
    timeout: Pin<Box<Sleep>>,
}

impl ReliabilityManager {
    pub fn new() -> Self {
        Self {
            acks: HashMap::new(),
            resend_queue: VecDeque::new(),
            timeout: Box::pin(tokio::time::sleep_until(Instant::now() + Duration::from_hours(1))),
        }
    }

    pub fn create_or_get_ack(&mut self, addr: SocketAddr) -> Ack {
        match self.acks.entry(addr) {
            Entry::Occupied(mut entry) => {
                *entry.get_mut() += 1;
                *entry.get()
            }

            Entry::Vacant(entry) => {
                let mut prng: ChaCha20Rng = rand::make_rng();
                let ack = prng.random::<Ack>();
                *entry.insert(ack)
            }
        }
    }

    pub fn contains(&self, addr: &SocketAddr) -> bool {
        self.acks.contains_key(addr)
    }

    pub fn remove(&mut self, addr: &SocketAddr) -> bool {
        self.acks.remove(addr).is_some()
    }

    pub fn prepare_packet(&mut self, addr: SocketAddr, packet: L5Packet) -> UdpPacket {
        let ack = self.create_or_get_ack(addr);
        let udp_packet = UdpPacket::Reliable(ReliablePacket::new(ack, packet));
        // print_base!("Send packet with ack {}", ack);
        let timeout = Instant::now() + Duration::from_millis(500);
        if self.resend_queue.is_empty() {
            self.timeout.as_mut().reset(timeout);
        };
        self.resend_queue.push_back((udp_packet.clone(), timeout, addr, ack));
        udp_packet
    }

    async fn retry_send(&mut self, game_socket: &GameSocket) -> Result<(), GameError> {
        match self.resend_queue.pop_front() {
            Some((packet, _instant, addr, ack)) => {
                // re-sending the packet
                game_socket
                    .send_raw(packet.clone(), addr)
                    .await
                    .map_err(|_| custom_error!(CustomErrorKind::NetworkSocket, "Cannot re-send network packet"))?;

                let instant = Instant::now();
                // replacing it into the queue
                self.resend_queue.push_back((packet.clone(), instant + Duration::from_millis(500), addr, ack));
                // resetting the timeout
                match self.resend_queue.front() {
                    None => self.timeout.as_mut().reset(instant + Duration::from_hours(1)),
                    Some((_packet, instant, _addr, _ack)) => {
                        self.timeout.as_mut().reset(instant.clone());
                    }
                }
            }
            None => {
                // print_base!("Long timeout reached");
                self.timeout.as_mut().reset(Instant::now() + Duration::from_hours(1));
            }
        }
        Ok(())
    }
}

impl CommonSocket {
    pub async fn new(socket_addr: SocketAddr) -> Result<CommonSocket, GameError> {
        Ok(Self {
            game_socket: GameSocket::new(socket_addr).await?,
            buffer: [0; 1300],
            resend_queue: VecDeque::new(),
            acks: ReliabilityManager::new(),
            timeout: Box::pin(tokio::time::sleep_until(Instant::now() + Duration::from_hours(1))),
        })
    }

    pub async fn connect(&self, sockaddr: SocketAddr) -> Result<(), Error> {
        self.game_socket.connect(sockaddr).await
    }

    pub fn peer_addr(&self) -> Result<SocketAddr, Error> {
        self.game_socket.peer_addr()
    }

    pub fn init_con(&mut self, sockaddr: SocketAddr) -> Ack {
        self.acks.create_or_get_ack(sockaddr)
    }

    async fn parse(&self, frame: Result<(usize, SocketAddr), Error>) -> Result<(UdpPacket, SocketAddr), GameError> {
        match frame {
            Ok((size, addr)) => {
                // trying to parse the raw bytes into a struct
                match UdpPacket::decode(&self.buffer[0..size].to_vec()) {
                    Ok(p) => Ok((p, addr)),
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(custom_error!(CustomErrorKind::InvalidData, e.to_string())),
        }
    }

    pub fn flush(&mut self) {
        loop {
            match self.game_socket.try_recv(&mut self.buffer) {
                Ok(a) => {}
                Err(e) => {
                    print_base!("Error {} returning", e);
                    return;
                }
            }
        }
    }

    pub async fn recv_from(&mut self) -> Result<(L5Packet, SocketAddr), GameError> {
        loop {
            tokio::select! {
                frame = self.game_socket.recv_from(&mut self.buffer) => {
                    let (packet, addr) = self.parse(frame).await?;
                    return match packet {
                        UdpPacket::Reliable(packet) => {
                            self.game_socket.send_raw(UdpPacket::Ack(AckPacket::new(packet.get_ack())), addr).await.map_err(|e| custom_error!(CustomErrorKind::NetworkSocket, e.to_string()))?;
                            Ok((packet.get_l5_packet(), addr))
                        }
                        UdpPacket::Simple(packet) => {
                            Ok((packet.get_l5_packet(), addr))
                        }
                        UdpPacket::Ack(packet) => {
                            self.ack_packet(addr,packet);
                            continue;
                        }
                    };
                }

                // timeout expires, if a packet sent hasn't been acked
                _ = &mut self.timeout => {
                    self.acks.retry_send(&self.game_socket).await?;
                    // checking if there's a packet to re-send

                }
            }
        }
    }

    /// FUNCTION TO ACKNOWLEDGE PACKETS, AND REMOVE THEM FROM RE-SENDER QUEUE
    ///
    /// HUGE BUG : to ack a packet, the function checks only the ack but it has to check also the SockAddr!!!!
    ///
    fn ack_packet(&mut self, socket_addr: SocketAddr, packet: AckPacket) {
        if self.acks.contains(&socket_addr) {
            // self.;
            self.resend_queue.retain(|e| (*e).3 != packet.get_ack());
        } else {
            print_base!("Invalid ack packet from {}", socket_addr)
        }
    }

    /// public function to send a packet L5Packet (Layer 5) to an Address, with the associated packaging provided by the UdpPacketType
    pub async fn send_to(&mut self, packet: L5Packet, addr: SocketAddr, udp_packet_type: UdpPacketType) -> Result<usize, GameError> {
        if packet.get_packet_type() == Quit {
            self.close_connection(addr);
        }
        match udp_packet_type {
            UdpPacketType::Reliable => self.game_socket.send_raw(self.acks.prepare_packet(addr, packet), addr).await,
            UdpPacketType::Simple => {
                let udp_packet = UdpPacket::Simple(SimplePacket::new(packet));
                self.game_socket.send_raw(udp_packet, addr).await
            }
            UdpPacketType::Ack => Err(custom_error!(CustomErrorKind::InvalidData, "Type Ack is not valid to be sent")),
        }
    }

    pub fn close_connection(&mut self, socket_addr: SocketAddr) {
        self.resend_queue.retain(|e| (*e).2 != socket_addr);
        self.acks.remove(&socket_addr);
    }
}
