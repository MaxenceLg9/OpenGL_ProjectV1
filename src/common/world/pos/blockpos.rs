use std::any::Any;
use std::ops::{Add, AddAssign, Deref, Sub};
use bitvec::macros::internal::funty::Fundamental;
use bitvec::vec::BitVec;
use glam::{IVec3, Vec3};
use crate::common::world::pos::chunkpos::{ChunkPos, CHUNK_SIZE};
use crate::common::world::pos::iblockpos::IBlockPos;
use crate::common::world::pos::pos_trait::PosTrait;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct BlockPos {
    pos : glam::Vec3,
}

impl BlockPos {
    pub fn from_vec3(pos : Vec3) -> Self {
        Self { pos }
    }
    
    pub fn new(x : f32, y : f32, z : f32) -> Self {
        Self {
            pos : glam::vec3(x,y,z)
        }
    }

    pub fn from_floats(pos : [f32; 3]) -> Self {
        Self::from_vec3(Vec3::from(pos))
    }

    pub fn get_chunk_pos(&self) -> ChunkPos {
        ChunkPos::from_block_pos(self)
    }

    pub fn get_relative_block_pos(&self) -> IBlockPos {
        IBlockPos::from_vec3(self.as_ivec3().rem_euclid(glam::IVec3::new(CHUNK_SIZE as i32, CHUNK_SIZE as i32, CHUNK_SIZE as i32)))
    }

    pub fn get_absolute_iblock_pos(&self) -> IBlockPos {
        IBlockPos::from_vec3(self.as_ivec3())
    }

    pub fn as_ivec3(&self) -> IVec3 {
        let mut pos = IVec3::new(0,0,0);
        if self.x < 0.0 {
            pos.x = self.x.floor() as i32;
        } else {
            pos.x = self.x.trunc() as i32;
        }
        if self.y < 0.0 {
            pos.y = self.y.floor() as i32;
        } else {
            pos.y = self.y.trunc() as i32;
        }
        if self.z < 0.0 {
            pos.z = self.z.floor() as i32;
        } else {
            pos.z = self.z.trunc() as i32;
        }
        pos
    }


    pub fn deserialize(pos_bits : Vec<u8>) -> BlockPos {
        let raw_bytes = pos_bits[0..12].to_vec();
        let coords: &[f32] = bytemuck::cast_slice(&raw_bytes);
        BlockPos::from_vec3(glam::Vec3::from_slice(coords))
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
        BlockPos::from_vec3(self.pos + rhs.pos)
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
        BlockPos::from_vec3(self.pos + rhs)
    }
}

impl Sub<BlockPos> for BlockPos{
    type Output = BlockPos;

    fn sub(self, rhs: BlockPos) -> Self::Output {
        BlockPos::from_vec3(self.pos - rhs.pos)
    }
}

impl Sub<Vec3> for BlockPos{
    type Output = Vec3;

    fn sub(self, rhs: Vec3) -> Self::Output {
        self.pos - rhs
    }
}


impl Deref for BlockPos {
    type Target = glam::Vec3;
    fn deref(&self) -> &Self::Target {
        &self.pos
    }
}