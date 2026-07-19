use std::fmt::{Display, Formatter};
use bitvec::order::Lsb0;
use bitvec::prelude::BitVec;
use crate::common::account::puid::PUID;
use crate::common::network::bit_cursor::BitCursor;
use crate::common::network::network_traits::{L5PacketTrait};
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

impl L5PacketTrait for SamplePacket {
    fn serialize(&self, vec: &mut BitVec<u8, Lsb0>) {
        
    }
    fn deserialize(cursor: &mut BitCursor) -> Self {
        SamplePacket::new()
    }
}