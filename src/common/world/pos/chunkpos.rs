use std::ops::{Add, Deref, Mul};
use glam::{IVec3};
use crate::common::world::pos::blockpos::BlockPos;
use crate::common::world::pos::iblockpos::IBlockPos;

pub const CHUNK_SIZE : usize = 64;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct ChunkPos {
    pos : glam::IVec3,
}

impl ChunkPos {
    pub fn new(pos : IVec3) -> Self {
        Self { pos }
    }

    pub fn from_block_pos(block_pos : &BlockPos) -> Self {
        Self::new(block_pos.as_ivec3().div_euclid(glam::IVec3::new(CHUNK_SIZE as i32, CHUNK_SIZE as i32, CHUNK_SIZE as i32)))
    }

    pub fn get_vec3(&self) -> IVec3 {
        self.pos
    }
}

impl Deref for ChunkPos {
    type Target = glam::IVec3;
    fn deref(&self) -> &Self::Target {
        &self.pos
    }
}

impl Mul<i32> for ChunkPos {

    type Output = ChunkPos;

    fn mul(self, other : i32) -> ChunkPos {
        ChunkPos::new(self.pos * other)
    }
}

impl Mul<usize> for ChunkPos {

    type Output = ChunkPos;

    fn mul(self, other : usize) -> ChunkPos {
        ChunkPos::new(self.pos * other as i32)
    }
}

impl Add<IBlockPos> for ChunkPos {
    type Output = IBlockPos;

    fn add(self, rhs: IBlockPos) -> IBlockPos {
        IBlockPos::new((self * CHUNK_SIZE).pos + rhs.deref())
    }
}