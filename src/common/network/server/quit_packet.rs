use std::fmt::{Display, Formatter};
use bitvec::order::Lsb0;
use bitvec::prelude::BitVec;
use crate::common::network::bit_cursor::BitCursor;
use crate::common::network::network_traits::{L5PacketTrait};
#[derive(Clone)]
pub struct QuitPacket {
}

impl QuitPacket {
    pub fn new() -> Self {
        Self {
        }
    }
}

impl Display for QuitPacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("")
    }
}

impl L5PacketTrait for QuitPacket {
    fn serialize(&self, vec: &mut BitVec<u8, Lsb0>) {
    }
    fn deserialize(cursor: &mut BitCursor) -> Self {
        QuitPacket::new()
    }

}
