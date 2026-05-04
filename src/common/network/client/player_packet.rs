use std::fmt::{Display, Formatter};
use bitvec::order::Lsb0;
use bitvec::prelude::BitVec;
use crate::common::network::bit_cursor::BitCursor;
use crate::common::network::network_traits::ClientNetPacket;
use crate::common::network::packet_type::ClientPacketType;
use crate::common::world::pos::blockpos::BlockPos;
use crate::common::world::pos::pos_trait::PosTrait;

pub struct UpdatePlayerPacket {
    block_pos: BlockPos
}

impl UpdatePlayerPacket {

    pub fn new(block_pos: BlockPos) -> UpdatePlayerPacket {
        Self {
            block_pos
        }
    }

    pub fn get_header_size() -> usize {
        12
    }

    pub fn get_pos(&self) -> BlockPos {
        self.block_pos
    }
}

impl Display for UpdatePlayerPacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"")
    }
}

impl ClientNetPacket for UpdatePlayerPacket {
    const P_TYPE: ClientPacketType = ClientPacketType::UpdatePlayer;

    fn serialize(&self) -> BitVec<u8, Lsb0> {
        let mut bits = BitVec::new();
        bits.extend(self.block_pos.serialize());
        bits
    }

    fn deserialize(cursor: &mut BitCursor) -> Self {
        Self::new(BlockPos::deserialize(cursor.read_bytes(12)))
    }

    fn get_packet_type() -> ClientPacketType {
        Self::P_TYPE
    }
}