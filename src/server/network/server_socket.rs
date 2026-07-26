use std::collections::{HashMap};
use std::io::{Error};
use std::net::{SocketAddr, SocketAddrV6};
use std::sync::Arc;
use shared::print_base;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use shared::common::network::l5_packet::L5Packet;
use shared::common::network::packet_type::{L5PacketType, UdpPacketType};
use crate::server::network::client_connection::ClientConnection;
use crate::server::world_data::data::ServerWorldData;
use tokio::sync::mpsc as tchannel;
use shared::common::network::socket::common_socket::CommonSocket;
use crate::server::world_data::event::events::{ServerEvent};
use crossbeam::channel as cb;

pub struct Socket {
    receiver: Receiver<(L5Packet, SocketAddr, UdpPacketType)>,
    sender: Sender<(L5Packet, SocketAddr, UdpPacketType)>,
    socket: CommonSocket,
    event_sx : cb::Sender<ServerEvent>,
    map: HashMap<SocketAddr, Sender<L5Packet>>,
    server_world_data: Arc<ServerWorldData>
}

impl Socket {

    /// Creates a socket that listens on the IPv6 address
    pub fn listen(server_world_data: Arc<ServerWorldData>, socket_addr_v6: SocketAddrV6, event_sx : cb::Sender<ServerEvent>) {
        // creating a thread that will host a tokio task
        std::thread::Builder::new()
            .name("network_thread".to_string())
            .spawn(move || {
                // creating tokio runtime
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();
                // running the runtime with a task
                rt.block_on(async {
                    print_base!("Starting network_thread at {}", chrono::offset::Local::now());
                    // creating socket
                    if let Ok(mut socket) = Self::new(server_world_data, socket_addr_v6, event_sx).await {
                        // socket can now wait for data to read or to write
                        socket.poll().await;
                    }
                });
            })
            .unwrap();
    }

    pub async fn new(server_world_data: Arc<ServerWorldData>, socket_addr_v6: SocketAddrV6, event_sx : cb::Sender<ServerEvent>) -> Result<Socket, Error> {
        let (sender, receiver) = channel(1000);
        let udp_socket = CommonSocket::new(socket_addr_v6).await?;
        Ok(Self {
            receiver,
            sender,
            event_sx,
            server_world_data,
            map : HashMap::new(),
            socket: udp_socket
        })
    }

    /// Method that updates the state of the socket
    ///
    /// Two states are possible:
    /// - Either receiving data from network
    /// - Or sending data through network
    pub async fn poll(&mut self) {
        loop {
            tokio::select! {
                // receiving packet
                frame = self.socket.recv_from() => {
                    self.handle_frame(frame).await;
                }
                // packet to send
                p = self.receiver.recv() => {
                    if let Some((packet, addr, udp_packet_type)) = p {
                        self.socket.send_to(packet,addr, udp_packet_type).await;
                    }
                }
            }
        }
    }

    /// Handling Layer5 packet received
    ///
    /// If its type is Login, creating a new connection.
    /// Otherwise, redirecting it to the correct tokio task through channel
    async fn handle_packet(&mut self, l5_packet: L5Packet, socket_addr: SocketAddr) {
        // if type is login
        if l5_packet.get_packet_type() == L5PacketType::Login {
            print_base!("Got Connection from {}", socket_addr);
            // creating channel
            let (sender, receiver) = tokio::sync::mpsc::channel(1000);
            let event_sx_clone = self.event_sx.clone();
            // inserting the channel in a map with the address as key
            self.map.insert(socket_addr,sender);
            // cloning world_data ARC
            let world_ref = self.server_world_data.clone();
            //
            let clone : ServerConnection = self.get_child(socket_addr, receiver);
            tokio::spawn(async move {
                ClientConnection::start(l5_packet.clone(), clone, world_ref, event_sx_clone).await;
                print_base!("Connection {} finished", socket_addr);
            });
        } else {
            // if not a login packet, sending the packet to the right task through channel
            if let Err(e) = self.map.get(&socket_addr).unwrap().send(l5_packet.clone()).await {
                print_base!("Error when sending packet : {}",e);
            }
        }
    }

    /// Extracting l5 packet from Result struct
    async fn handle_frame(&mut self, frame :  Result<(L5Packet, SocketAddr), Error>) {
        let (packet, addr) : (L5Packet, SocketAddr) = match frame {
            Err(e) => {
                print_base!("Got error when receiving packet {}",e.to_string());
                return;
            }
            Ok(p) => p,
        };
        // handling it if extraction was successful
        self.handle_packet(packet, addr).await;
        // print_base!("Got packet {}", packet);

    }

    /// Slicing the socket to get the child part associated to the address
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