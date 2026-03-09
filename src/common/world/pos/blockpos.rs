use std::any::Any;
use std::ops::Deref;
use bitvec::vec::BitVec;
use glam::{Vec3};
use crate::common::world::pos::chunkpos::{ChunkPos, CHUNK_SIZE};
use crate::common::world::pos::iblockpos::IBlockPos;
use crate::common::world::pos::pos_trait::PosTrait;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct BlockPos {
    pos : glam::Vec3,
}

impl BlockPos {
    pub fn new(pos : Vec3) -> Self {
        Self { pos }
    }

    pub fn from_floats(pos : [f32; 3]) -> Self {
        Self::new(Vec3::from(pos))
    }

    pub fn get_chunk_pos(&self) -> ChunkPos {
        ChunkPos::from_block_pos(self)
    }

    pub fn get_iblock_pos(&self) -> IBlockPos {
        IBlockPos::new(self.pos.as_ivec3().rem_euclid(glam::IVec3::new(CHUNK_SIZE as i32, CHUNK_SIZE as i32, CHUNK_SIZE as i32)))
    }
}
impl PosTrait for BlockPos {
    fn serialize(&self) -> BitVec<u8> {
        let mut bits = BitVec::new();
        bits.extend_from_raw_slice(bytemuck::cast_slice(&self.to_array()));
        bits
    }

    fn deserialize(pos_bits : BitVec<u8>) -> Box<dyn PosTrait> {
        let raw_bytes = pos_bits[0..96].to_bitvec().into_vec();
        let coords: &[f32] = bytemuck::cast_slice(&raw_bytes);
        Box::new(BlockPos::new(glam::Vec3::from_slice(coords)))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Deref for BlockPos {
    type Target = glam::Vec3;
    fn deref(&self) -> &Self::Target {
        &self.pos
    }
}