use crate::server::network::client_connection::ClientConnection;
use crate::server::world_data::data::ServerWorldData;
use crate::server::world_data::event::events::ServerEvent;
use crossbeam::channel as cb;
use shared::common::error::err_type::{CustomErrorKind, GameError};
use shared::common::network::l5_packet::L5Packet;
use shared::common::network::packet_type::{L5PacketType, UdpPacketType};
use shared::common::network::socket::common_socket::CommonSocket;
use shared::{custom_error, print_base};
use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::net::{SocketAddr, SocketAddrV6};
use std::sync::Arc;
use tokio::sync::mpsc as tchannel;
use tokio::sync::mpsc::{Receiver, Sender, channel};

pub struct Socket {
    receiver: Receiver<(L5Packet, SocketAddr, UdpPacketType)>,
    sender: Sender<(L5Packet, SocketAddr, UdpPacketType)>,
    socket: CommonSocket,
    event_sx: cb::Sender<ServerEvent>,
    map: HashMap<SocketAddr, Sender<L5Packet>>,
    server_world_data: Arc<ServerWorldData>,
}

impl Socket {
    /// Creates a socket that listens on the IPv6 address
    pub fn listen(server_world_data: Arc<ServerWorldData>, socket_addr_v6: SocketAddr, event_sx: cb::Sender<ServerEvent>) -> Result<(), GameError> {
        print_base!("Creating thread");
        // creating a thread that will host a tokio task
        std::thread::Builder::new()
            .name("network_thread".to_string())
            .spawn(move || {
                // creating tokio runtime
                let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
                // running the runtime with a task
                rt.block_on(async {
                    print_base!("Starting network_thread at {}", chrono::offset::Local::now());
                    // creating socket
                    if let Ok(mut socket) = Self::new(server_world_data, socket_addr_v6, event_sx).await {
                        // socket can now wait for data to read or to write
                        match socket.poll().await {
                            Ok(()) => {}
                            Err(e) => {
                                print_base!("Got error {}", e);
                            }
                        }
                    }
                });
            })
            .map_err(|e| custom_error!(CustomErrorKind::Io, e.to_string()))
            .map(|_| ())
    }

    pub async fn new(server_world_data: Arc<ServerWorldData>, socket_addr_v6: SocketAddr, event_sx: cb::Sender<ServerEvent>) -> Result<Socket, GameError> {
        let (sender, receiver) = channel(1000);
        let udp_socket = CommonSocket::new(socket_addr_v6).await?;
        Ok(Self {
            receiver,
            sender,
            event_sx,
            server_world_data,
            map: HashMap::new(),
            socket: udp_socket,
        })
    }

    /// Method that updates the state of the socket
    ///
    /// Two states are possible:
    /// - Either receiving data from network
    /// - Or sending data through network
    pub async fn poll(&mut self) -> Result<(), GameError> {
        loop {
            tokio::select! {
                // receiving packet
                frame = self.socket.recv_from() => {
                    self.handle_frame(frame?).await?;
                }
                // packet to send
                p = self.receiver.recv() => {
                    if let Some((packet, addr, udp_packet_type)) = p {
                        self.socket.send_to(packet,&addr, udp_packet_type).await?;
                    }
                }
            }
        }
    }

    /// Handling Layer5 packet received
    ///
    /// If its type is Login, creating a new connection.
    /// Otherwise, redirecting it to the correct tokio task through channel
    async fn handle_packet(&mut self, l5_packet: L5Packet, socket_addr: SocketAddr) -> Result<(), GameError> {
        // if type is login
        if l5_packet.get_packet_type() == L5PacketType::Login {
            // creating channel
            let (sender, receiver) = tokio::sync::mpsc::channel(1000);
            let event_sx_clone = self.event_sx.clone();
            // inserting the channel in a map with the address as key
            self.map.insert(socket_addr, sender);
            // cloning world_data ARC
            let world_ref = self.server_world_data.clone();
            //
            let clone: ServerConnection = self.get_child(socket_addr, receiver);
            let ack = self.socket.init_con(socket_addr);
            print_base!("Starting connection for {} with ack {}", socket_addr, ack);
            tokio::spawn(async move {
                if let Err(e) = ClientConnection::start(l5_packet.clone(), clone, world_ref, event_sx_clone).await {
                    print_base!("Connection {} finished with error {}", socket_addr, e);
                }
            });
            Ok(())
        } else {
            // if not a login packet, sending the packet to the right task through channel
            self.map
                .get(&socket_addr)
                .ok_or_else(|| Error::new(ErrorKind::NotFound, "No matching connection to process for this packet"))?
                .send(l5_packet.clone())
                .await
                .map_err(|e| custom_error!(CustomErrorKind::SendError, e.to_string()))
        }
    }

    /// Extracting l5 packet from Result struct
    async fn handle_frame(&mut self, frame: (L5Packet, SocketAddr)) -> Result<(), GameError> {
        let (packet, addr): (L5Packet, SocketAddr) = frame;
        // handling it if extraction was successful
        self.handle_packet(packet, addr).await
        // print_base!("Got packet {}", packet);
    }

    /// Slicing the socket to get the child part associated to the address
    fn get_child(&self, socket_addr: SocketAddr, packet_receiver: Receiver<L5Packet>) -> ServerConnection {
        ServerConnection::new(self.sender.clone(), socket_addr, packet_receiver)
    }
}

pub struct ServerConnection {
    sender: tchannel::Sender<(L5Packet, SocketAddr, UdpPacketType)>,
    socket_addr: SocketAddr,
    packet_receiver: tchannel::Receiver<L5Packet>,
}

impl ServerConnection {
    pub fn new(sender: tchannel::Sender<(L5Packet, SocketAddr, UdpPacketType)>, socket_addr: SocketAddr, packet_receiver: tchannel::Receiver<L5Packet>) -> ServerConnection {
        Self { sender, socket_addr, packet_receiver }
    }

    pub async fn send(&self, l5packet: L5Packet, udp_packet_type: UdpPacketType) -> Result<(), tokio::sync::mpsc::error::SendError<(L5Packet, SocketAddr, UdpPacketType)>> {
        self.sender.send((l5packet, self.socket_addr, udp_packet_type)).await
    }

    pub fn get_addr(&self) -> SocketAddr {
        self.socket_addr
    }

    pub async fn recv(&mut self) -> Option<L5Packet> {
        self.packet_receiver.recv().await
    }
}
