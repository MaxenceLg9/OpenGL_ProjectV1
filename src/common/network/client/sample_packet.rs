use std::fmt::{Display, Formatter};
use bitvec::order::Lsb0;
use bitvec::prelude::BitVec;
use crate::common::account::puid::PUID;
use crate::common::network::network_traits::{ClientMessage, Message, ServerMessage};
use crate::common::network::packet_type::{ClientPacketType, ServerPacketType};
use crate::common::network::server::chunk_packet::ChunkPacket;

pub struct  SamplePacket {
    puid: PUID,
}

impl SamplePacket {
    pub fn new() -> Self {
        Self {
            puid: PUID::new(0)
        }
    }

    pub(crate) fn from_bits(bits: BitVec<u8>) -> Self {
        Self::new()
    }

}

impl Message for SamplePacket {
    fn serialize(&self, type_val : u8) -> BitVec<u8> {
        let bits = BitVec::new();
        bits
    }
}

impl ClientMessage for SamplePacket {
    fn get_puid(&self) -> PUID {
        self.puid
    }

    fn get_packet_type(&self) -> ClientPacketType {
        ClientPacketType::Quit
    }
}

impl Display for SamplePacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl ServerMessage for SamplePacket {
    fn get_packet_type(&self) -> ServerPacketType {
        ServerPacketType::Chunk
    }
}