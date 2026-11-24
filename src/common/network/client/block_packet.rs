use crate::common::error::err_type::GameError;
use crate::common::network::bit_cursor::BitCursor;
use crate::common::network::network_traits::L5PacketTrait;
use crate::common::world::pos::iblockpos::IBlockPos;
use crate::common::world::pos::pos_trait::PosTrait;
use bitvec::order::Lsb0;
use bitvec::prelude::BitVec;
use bitvec::view::BitView;
use std::fmt::{Display, Formatter};
use strum::{Display, FromRepr};

#[derive(Clone)]
pub struct BlockPacket {
    interaction_type: BlockInteraction,
    pos: IBlockPos,
    blockstate: u16,
}
#[derive(Clone, Copy, Debug, PartialEq, FromRepr, Display)]
#[repr(u8)]
pub enum BlockInteraction {
    LEFT,
    RIGHT,
}

impl BlockPacket {
    pub fn new(interaction_type: BlockInteraction, pos: IBlockPos, blockstate: u16) -> Self {
        Self { interaction_type, pos, blockstate }
    }

    pub fn get_interaction_type(&self) -> BlockInteraction {
        self.interaction_type
    }

    pub fn get_pos(&self) -> IBlockPos {
        self.pos
    }

    pub fn get_block_type(&self) -> u16 {
        self.blockstate
    }
}

impl Display for BlockPacket {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl L5PacketTrait for BlockPacket {
    fn serialize(&self, vec: &mut BitVec<u8>) -> Result<(), GameError> {
        vec.extend_from_bitslice((self.interaction_type as u8).view_bits::<Lsb0>());
        vec.extend(self.pos.serialize());
        vec.extend(self.blockstate.view_bits::<Lsb0>());
        Ok(())
    }
    fn deserialize(cursor: &mut BitCursor) -> Result<Self, GameError> {
        Ok(Self {
            interaction_type: BlockInteraction::from_repr(cursor.read_bits::<u8>(8)).unwrap(),
            pos: IBlockPos::deserialize(cursor.read_bytes(12)),
            blockstate: cursor.read_bits::<u16>(16),
        })
    }
}
