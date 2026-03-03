use std::ops::Deref;
use glam::{Vec3};
use crate::common::world::pos::chunkpos::{ChunkPos, CHUNK_SIZE};
use crate::common::world::pos::iblockpos::IBlockPos;

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

impl Deref for BlockPos {
    type Target = glam::Vec3;
    fn deref(&self) -> &Self::Target {
        &self.pos
    }
}