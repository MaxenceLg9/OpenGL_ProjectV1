use std::fmt::{Display, Formatter};
use bitvec::order::Lsb0;
use bitvec::prelude::BitVec;
use crate::common::network::bit_cursor::BitCursor;
use crate::common::network::network_traits::ClientNetPacket;
use crate::common::network::packet_type::ClientPacketType;
#[derive(Clone)]
pub struct DefaultPacket;

impl Display for DefaultPacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"Default Packet")
    }
}

impl ClientNetPacket for DefaultPacket {
    const P_TYPE: ClientPacketType = ClientPacketType::Quit;

    fn serialize(&self) -> BitVec<u8, Lsb0> {
        BitVec::new()
    }

    fn deserialize(cursor: &mut BitCursor) -> Self {
        Self
    }

    fn get_packet_type() -> ClientPacketType {
        Self::P_TYPE
    }
}