use crate::common::network::network_traits::L5PacketTrait;
use crate::common::{error::err_type::GameError, network::bit_cursor::BitCursor};
use bitvec::prelude::BitVec;
use std::fmt::{Display, Formatter};

#[derive(Clone)]
pub struct DefaultPacket;

impl Display for DefaultPacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Default Packet")
    }
}

impl L5PacketTrait for DefaultPacket {
    fn serialize(&self, _: &mut BitVec<u8>) -> Result<(), GameError> {
        Ok(())
    }
    fn deserialize(_cursor: &mut BitCursor) -> Result<Self, GameError> {
        Ok(Self)
    }
}
