use std::fmt::Display;
use bitvec::order::Lsb0;
use bitvec::prelude::BitVec;
use crate::common::network::bit_cursor::BitCursor;
use crate::common::network::network_traits::{ServerNetPacket};
use crate::common::network::packet_type::ServerPacketType;

pub struct GetPlayerPacket {

}

impl GetPlayerPacket {

    pub fn new() -> GetPlayerPacket {
        Self {

        }
    }
}

impl Display for GetPlayerPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::new())
    }
}

impl ServerNetPacket for GetPlayerPacket {
    const P_TYPE: ServerPacketType = ServerPacketType::GetPlayer;

    fn serialize(&self) -> BitVec<u8, Lsb0> {
        BitVec::new()
    }

    fn deserialize(cursor: &mut BitCursor) -> Self {
        GetPlayerPacket::new()
    }

    fn get_packet_type(&self) -> ServerPacketType {
        Self::P_TYPE
    }
}