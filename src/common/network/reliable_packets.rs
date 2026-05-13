use std::fmt::{Display, Formatter};
use bitvec::order::Lsb0;
use bitvec::prelude::BitVec;
use crate::common::network::bit_cursor::BitCursor;
use crate::common::network::network_traits::NetPacket;

#[derive(Clone)]
pub struct ReliablePacket {

}

impl Display for ReliablePacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"Reliable packet")
    }
}

impl NetPacket for ReliablePacket {
    fn serialize(&self) -> BitVec<u8, Lsb0> {
        todo!()
    }

    fn deserialize(cursor: &mut BitCursor) -> Self {
        todo!()
    }
}