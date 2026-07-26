use std::any::Any;
use std::fmt::{Display, Formatter};
use bitvec::order::Lsb0;
use bitvec::prelude::BitVec;
use bitvec::view::BitView;
use strum::{Display, FromRepr};
use crate::common::network::bit_cursor::BitCursor;
use crate::common::network::network_traits::L5PacketTrait;
use crate::common::world::block::block::BlockType;
use crate::common::world::pos::iblockpos::IBlockPos;
use crate::common::world::pos::pos_trait::PosTrait;

#[derive(Clone)]
pub struct BlockPacket {
    interaction_type : BlockInteraction,
    pos : IBlockPos,
    block_type : BlockType,
}
#[derive(Clone, Copy, Debug, PartialEq, FromRepr, Display)]
#[repr(u8)]
pub enum BlockInteraction {
    LEFT,
    RIGHT
}

impl BlockPacket {
    pub fn new(interaction_type : BlockInteraction, pos : IBlockPos, block_type: BlockType) -> Self {
        Self {
            interaction_type,
            pos,
            block_type
        }
    }

    pub fn get_interaction_type(&self) -> BlockInteraction {
        self.interaction_type
    }

    pub fn get_pos(&self) -> IBlockPos {
        self.pos
    }

    pub fn get_block_type(&self) -> BlockType {
        self.block_type
    }
}

impl Display for BlockPacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl L5PacketTrait for BlockPacket {
    fn serialize(&self, vec: &mut BitVec<u8>) {
        vec.extend_from_bitslice((self.interaction_type as u8).view_bits::<Lsb0>());
        vec.extend(self.pos.serialize());
        vec.extend(self.block_type.get_value().view_bits::<Lsb0>());
    }
    fn deserialize(cursor: &mut BitCursor) -> Self {
        Self {
            interaction_type : BlockInteraction::from_repr(cursor.read_bits::<u8>(8)).unwrap(),
            pos : IBlockPos::deserialize(cursor.read_bytes(12)),
            block_type : BlockType::from_repr(cursor.read_bits::<u16>(16)).unwrap()
        }
    }

}

