use crate::common::error::err_type::GameError;
use crate::common::network::bit_cursor::BitCursor;
use bitvec::order::Lsb0;
use bitvec::prelude::BitVec;
use std::fmt::Display;

pub trait L5PacketTrait: Display + Sized {
    fn serialize(&self, vec: &mut BitVec<u8, Lsb0>) -> Result<(), GameError>;
    fn deserialize(cursor: &mut BitCursor) -> Result<Self, GameError>;
}

pub trait UdpPacketTrait: Display + Sized {
    fn serialize(&self, vec: &mut BitVec<u8, Lsb0>) -> Result<(), GameError>;
    fn deserialize(cursor: &mut BitCursor) -> Result<Self, GameError>;
}
