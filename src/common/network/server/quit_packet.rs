use std::fmt::{Display, Formatter};
use bitvec::order::Lsb0;
use bitvec::prelude::BitVec;
use crate::common::account::puid::PUID;
use crate::common::network::bit_cursor::BitCursor;
use crate::common::network::network_traits::{ServerNetPacket};
use crate::common::network::packet_type::{ServerPacketType};
#[derive(Clone)]
pub struct QuitPacket {
}

impl QuitPacket {
    pub fn new() -> Self {
        Self {
        }
    }

    pub(crate) fn from_bits(bits: BitVec<u8>) -> Self {
        Self::new()
    }

}

impl Display for QuitPacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("")
    }
}

impl ServerNetPacket for QuitPacket {
    const P_TYPE: ServerPacketType = ServerPacketType::Quit;

    fn serialize(&self) -> BitVec<u8, Lsb0> {
        BitVec::new()
    }

    fn deserialize(cursor: &mut BitCursor) -> Self {
        QuitPacket::new()
    }

    fn get_packet_type(&self) -> ServerPacketType {
        Self::P_TYPE
    }
}