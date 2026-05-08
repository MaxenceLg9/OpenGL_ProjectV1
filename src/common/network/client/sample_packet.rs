use std::fmt::{Display, Formatter};
use bitvec::order::Lsb0;
use bitvec::prelude::BitVec;
use crate::common::account::puid::PUID;
use crate::common::network::bit_cursor::BitCursor;
use crate::common::network::network_traits::{ServerNetPacket};
use crate::common::network::packet_type::{ServerPacketType};
#[derive(Clone)]
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

impl Display for SamplePacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("")
    }
}

impl ServerNetPacket for SamplePacket {
    const P_TYPE: ServerPacketType = ServerPacketType::Correction;

    fn serialize(&self) -> BitVec<u8, Lsb0> {
        BitVec::new()
    }

    fn deserialize(cursor: &mut BitCursor) -> Self {
        SamplePacket::new()
    }

    fn get_packet_type(&self) -> ServerPacketType {
        Self::P_TYPE
    }
}