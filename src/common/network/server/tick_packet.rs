use std::fmt::Display;
use bitvec::order::Lsb0;
use bitvec::prelude::BitVec;
use bitvec::view::BitView;
use crate::common::network::bit_cursor::BitCursor;
use crate::common::network::l5_packet::L5Packet;
use crate::common::network::network_traits::{L5PacketTrait};
use crate::common::network::packet_type::{L5PacketType, ServerPacketType};
#[derive(Clone)]
pub struct GetPlayerPacket {
    id : u16
}

impl GetPlayerPacket {

    pub fn new(id : u16) -> GetPlayerPacket {
        Self {
            id
        }
    }

    pub fn get_id(&self) -> u16 {
        self.id
    }
}

impl Display for GetPlayerPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::new())
    }
}

impl L5PacketTrait for GetPlayerPacket {
    fn serialize(&self) -> BitVec<u8, Lsb0> {
        let mut bits =BitVec::new();
        bits.extend_from_bitslice(self.id.view_bits::<Lsb0>());
        bits
    }

    fn deserialize(cursor: &mut BitCursor) -> Self {
        GetPlayerPacket::new(cursor.read_bits::<u16>(16))
    }

}