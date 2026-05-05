use std::any::Any;
use std::ops::{Add, AddAssign, Deref};
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

    pub fn deserialize(pos_bits : Vec<u8>) -> BlockPos {
        let raw_bytes = pos_bits[0..12].to_vec();
        let coords: &[f32] = bytemuck::cast_slice(&raw_bytes);
        BlockPos::new(glam::Vec3::from_slice(coords))
    }

    pub fn as_vec3(&self) -> glam::Vec3 {
        self.pos.clone()
    }
}
impl PosTrait for BlockPos {
    fn serialize(&self) -> BitVec<u8> {
        let mut bits = BitVec::new();
        bits.extend_from_raw_slice(bytemuck::cast_slice(&self.to_array()));
        bits
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Add<BlockPos> for BlockPos {
    type Output = BlockPos;

    fn add(self, rhs: BlockPos) -> Self::Output {
        BlockPos::new(self.pos + rhs.pos)
    }
}

impl AddAssign<BlockPos> for BlockPos {
    fn add_assign(&mut self, rhs: Self) {
        self.pos = self.pos + rhs.pos
    }
}

impl AddAssign<Vec3> for BlockPos {
    fn add_assign(&mut self, rhs: Vec3) {
        self.pos = self.pos + rhs
    }
}
impl Add<glam::Vec3> for BlockPos {
    type Output = BlockPos;

    fn add(self, rhs: Vec3) -> Self::Output {
        BlockPos::new(self.pos + rhs)
    }
}


impl Deref for BlockPos {
    type Target = glam::Vec3;
    fn deref(&self) -> &Self::Target {
        &self.pos
    }
}