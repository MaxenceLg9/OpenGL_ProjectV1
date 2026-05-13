use std::io::{Error, ErrorKind};
use std::net::{SocketAddr, SocketAddrV6};
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use shared::common::network::default_packet::{ClientPacket, ServerPacket};

pub struct ServerSocket {
    udp_socket: Arc<UdpSocket>,
    buffer : [u8; 1024],
}

pub struct ServerConnection {
    udp_socket: Arc<UdpSocket>,
    socket_addr : SocketAddr,
    packet_receiver: mpsc::Receiver<ClientPacket>,
}

impl ServerSocket {

    pub async fn new(socket_addr_v6: SocketAddrV6) -> Result<Self,Error> {
        let socket = UdpSocket::bind(socket_addr_v6).await?;

        Ok(Self {
            udp_socket : Arc::new(socket),
            buffer : [0;1024]
        })
    }
    pub async fn receive(&mut self) -> Result<(ClientPacket, SocketAddr), Error> {
        let frame = self.udp_socket.recv_from(&mut self.buffer).await;
        match frame {
            Ok(result) => {
                /// trying to parse the raw bytes into a struct
                match ClientPacket::decode(&self.buffer[0..result.0].to_vec()){
                    Some(p) => Ok((p,result.1)),
                    None => Err(Error::new(ErrorKind::StaleNetworkFileHandle,"Cannot parse the packet, invalid bytes")),
                }
            }
            Err(e) => {
                Err(e)
            }
        }
    }

    pub fn get_child(&self, socket_addr : SocketAddr, packet_receiver: mpsc::Receiver<ClientPacket>) -> ServerConnection {
        ServerConnection::new(self.udp_socket.clone(), socket_addr, packet_receiver)
    }
}

impl ServerConnection {
    pub fn new(udp_socket: Arc<UdpSocket>, socket_addr: SocketAddr, packet_receiver: mpsc::Receiver<ClientPacket>) -> ServerConnection {
        Self {
            udp_socket,
            socket_addr,
            packet_receiver
        }
    }

    pub fn get_addr(&self) -> SocketAddr {
        self.socket_addr
    }

    pub async fn send(&self, server_packet: ServerPacket) -> std::io::Result<usize> {
        self.udp_socket.send_to(server_packet.encode().as_raw_slice(),self.socket_addr).await
    }
    
    pub async fn recv(&mut self) -> Option<ClientPacket> {
        self.packet_receiver.recv().await
    }
}