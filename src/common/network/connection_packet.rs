use crate::common::network::packet::{Packet, PacketType};

pub struct ConnectionPacket {

}

impl ConnectionPacket {
    pub fn new() -> Self {
        Self {}
    }
}

impl Packet for ConnectionPacket {
    fn packet_type(&self) -> PacketType{
        PacketType::Connect
    }

    fn serialize(&self) -> String {
        "CONNECT".to_string()
    }
}