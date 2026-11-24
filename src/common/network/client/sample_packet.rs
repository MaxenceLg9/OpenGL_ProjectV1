use crate::common::network::bit_cursor::BitCursor;
use crate::common::network::network_traits::L5PacketTrait;
use crate::common::{account::puid::PUID, error::err_type::GameError};
use bitvec::order::Lsb0;
use bitvec::prelude::BitVec;
use std::fmt::{Display, Formatter};
#[derive(Clone)]
pub struct SamplePacket {
    puid: PUID,
}

impl SamplePacket {
    pub fn new() -> Self {
        Self { puid: PUID::new(0) }
    }

    pub(crate) fn from_bits(_bits: BitVec<u8>) -> Self {
        Self::new()
    }
}

impl Default for SamplePacket {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for SamplePacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("")
    }
}

impl L5PacketTrait for SamplePacket {
    fn serialize(&self, _vec: &mut BitVec<u8, Lsb0>) -> Result<(), GameError> {
        Ok(())
    }
    fn deserialize(_cursor: &mut BitCursor) -> Result<Self, GameError> {
        Ok(SamplePacket::new())
    }
}
