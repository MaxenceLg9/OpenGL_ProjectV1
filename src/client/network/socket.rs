use std::net::{Ipv6Addr, SocketAddrV6};
use std::sync::{Arc, RwLock};
use shared::common::network::client::login_packet::LoginPacket;
use shared::common::network::client::packet::ClientPacket;
use crate::client::network::server_connection::ServerConnection;
use crate::client::world_data::client_data::ClientWorldData;

pub struct ClientSocket {
    sender : tokio::sync::mpsc::Sender<ClientPacket>
}

impl ClientSocket {
    pub fn new(ipv6_address : Ipv6Addr, client_world_data : Arc<ClientWorldData>) -> Self {
        let (sender, receiver) = tokio::sync::mpsc::channel(100000);
        ServerConnection::start(ipv6_address, receiver, client_world_data);

        Self {
            sender
        }
    }

    pub fn send(&self, packet : ClientPacket) {
        self.sender.try_send(packet);
    }


}

impl Drop for ClientSocket {
    fn drop(&mut self) {
    }
}