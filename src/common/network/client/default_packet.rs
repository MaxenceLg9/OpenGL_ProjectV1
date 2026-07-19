use std::fmt::{Display, Formatter};
use bitvec::order::Lsb0;
use bitvec::prelude::BitVec;
use crate::common::network::bit_cursor::BitCursor;
use crate::common::network::network_traits::{L5PacketTrait};
#[derive(Clone)]
pub struct DefaultPacket;

impl Display for DefaultPacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"Default Packet")
    }
}

impl L5PacketTrait for DefaultPacket {
    fn serialize(&self, _: &mut BitVec<u8>) { todo!() }

    fn deserialize(cursor: &mut BitCursor) -> Self {
        Self
    }
}