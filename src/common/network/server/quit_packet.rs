use crate::common::network::network_traits::L5PacketTrait;
use crate::common::{error::err_type::GameError, network::bit_cursor::BitCursor};
use bitvec::order::Lsb0;
use bitvec::prelude::BitVec;
use std::fmt::{Display, Formatter};
#[derive(Clone)]
pub struct QuitPacket {}

impl QuitPacket {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for QuitPacket {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for QuitPacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("")
    }
}

impl L5PacketTrait for QuitPacket {
    fn serialize(&self, _vec: &mut BitVec<u8, Lsb0>) -> Result<(), GameError> {
        Ok(())
    }
    fn deserialize(_cursor: &mut BitCursor) -> Result<Self, GameError> {
        Ok(QuitPacket::new())
    }
}
