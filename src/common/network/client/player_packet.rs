use crate::common::network::network_traits::L5PacketTrait;
use crate::common::world::pos::blockpos::BlockPos;
use crate::common::world::pos::pos_trait::PosTrait;
use crate::common::{error::err_type::GameError, network::bit_cursor::BitCursor};
use bitvec::order::Lsb0;
use bitvec::prelude::BitVec;
use bitvec::view::BitView;
use std::fmt::{Display, Formatter};
#[derive(Clone)]
pub struct UpdatePlayerPacket {
    block_pos: BlockPos,
    view_distance: u8,
    id: u16,
}

impl UpdatePlayerPacket {
    pub fn new(block_pos: BlockPos, view_distance: u8, id: u16) -> UpdatePlayerPacket {
        Self { block_pos, view_distance, id }
    }

    pub fn get_id(&self) -> u16 {
        self.id
    }

    pub fn get_pos(&self) -> BlockPos {
        self.block_pos
    }

    pub fn get_view_distance(&self) -> u8 {
        self.view_distance
    }
}

impl Display for UpdatePlayerPacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

impl L5PacketTrait for UpdatePlayerPacket {
    fn serialize(&self, vec: &mut BitVec<u8>) -> Result<(), GameError> {
        vec.extend(self.block_pos.serialize());
        vec.extend(self.view_distance.view_bits::<Lsb0>());
        vec.extend(self.id.view_bits::<Lsb0>());
        Ok(())
    }

    fn deserialize(cursor: &mut BitCursor) -> Result<Self, GameError> {
        Ok(Self::new(BlockPos::deserialize(cursor.read_bytes(12)), cursor.read_bits::<u8>(8), cursor.read_bits::<u16>(16)))
    }
}
