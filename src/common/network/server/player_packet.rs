use std::fmt::{Display, Formatter};
use bitvec::field::BitField;
use bitvec::order::Lsb0;
use bitvec::prelude::BitVec;
use crate::common::network::client::sample_packet::SamplePacket;
use crate::common::network::network_traits::{Message, ServerMessage};
use crate::common::network::packet_type::ServerPacketType;

pub struct AskPlayerPacket {

}

impl AskPlayerPacket {
    pub(crate) fn from_bits(p0: BitVec<u8>) -> AskPlayerPacket {
        Self {

        }
    }

    pub fn get_header_size() -> usize {
        0
    }

    pub fn get_body_size(header : &BitVec<u8>) -> usize {
        0
    }
}

impl AskPlayerPacket {
    
}

impl Message for AskPlayerPacket {
    fn serialize(&self, type_val: u8) -> BitVec<u8, Lsb0> {
        todo!()
    }
}

impl Display for AskPlayerPacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("")
    }
}

impl ServerMessage for AskPlayerPacket {
    fn get_packet_type(&self) -> ServerPacketType {
        ServerPacketType::PlayerPos
    }
}