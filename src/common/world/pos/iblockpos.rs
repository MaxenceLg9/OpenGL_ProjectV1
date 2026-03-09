use std::any::Any;
use std::ops::{Deref, Div};
use bitvec::prelude::BitVec;
use glam::{IVec3};
use crate::common::world::pos::blockpos::BlockPos;
use crate::common::world::pos::chunkpos::{ChunkPos, CHUNK_SIZE};
use crate::common::world::pos::pos_trait::PosTrait;
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct IBlockPos {
    pos : glam::IVec3,
}


impl IBlockPos {
    pub fn new(pos : IVec3) -> Self {
        Self { pos }
    }

    pub fn from_ints(i1 : i32, i2 : i32, i3 : i32) -> Self {
        Self::new(IVec3::new(i1, i2, i3))
    }

    pub fn from_array(pos : [i32; 3]) -> Self {
        Self::new(IVec3::from(pos))
    }

    pub fn as_split_pos(&self) -> (IBlockPos, ChunkPos) {
        (self.get_block_pos(),self.get_chunk_pos())
    }

    fn get_chunk_pos(&self) -> ChunkPos {
        ChunkPos::new(self.pos.div_euclid(glam::IVec3::new(CHUNK_SIZE as i32, CHUNK_SIZE as i32, CHUNK_SIZE as i32)))
    }

    fn get_block_pos(&self) -> IBlockPos {
        IBlockPos::new(self.pos.rem_euclid(glam::IVec3::new(CHUNK_SIZE as i32, CHUNK_SIZE as i32, CHUNK_SIZE as i32)))
    }
}
impl PosTrait for IBlockPos {
    fn serialize(&self) -> BitVec<u8> {
        let mut bits = BitVec::new();
        bits.extend_from_raw_slice(bytemuck::cast_slice(&self.to_array()));
        bits
    }

    fn deserialize(pos_bits : BitVec<u8>) -> Box<dyn PosTrait> {
        let raw_bytes = pos_bits[0..96].to_bitvec().into_vec();
        let coords: &[i32] = bytemuck::cast_slice(&raw_bytes);
        Box::new(ChunkPos::new(glam::IVec3::from_slice(coords)))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
impl Div<i32> for &IBlockPos {
    type Output = IBlockPos;

    fn div(self, rhs: i32) -> IBlockPos {
        IBlockPos::new(self.pos / rhs)
    }
}

impl Deref for IBlockPos {
    type Target = glam::IVec3;
    fn deref(&self) -> &Self::Target {
        &self.pos
    }
}