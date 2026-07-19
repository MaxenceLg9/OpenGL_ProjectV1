use std::any::Any;
use std::ops::{Add, Deref, Div, Mul, Sub};
use bitvec::vec::BitVec;
use glam::{IVec3};
use crate::common::world::pos::blockpos::BlockPos;
use crate::common::world::pos::iblockpos::IBlockPos;
use crate::common::world::pos::pos_trait::PosTrait;

pub const CHUNK_SIZE : usize = 64;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct ChunkPos {
    pos : glam::IVec3,
}

impl ChunkPos {
    pub fn from_vec3(pos : IVec3) -> Self {
        Self { pos }
    }

    pub fn new(x : i32, y : i32, z : i32) -> Self {
        Self { pos : glam::ivec3(x,y,z) }
    }

    pub fn center(&self) -> BlockPos {
        self.to_block_pos() + glam::Vec3::new(32.0,32.0,32.0)
    }


    pub fn from_absolute(x : i32, y : i32, z : i32, range : i32) -> Self {
        Self { pos : glam::ivec3(x,y,z) - glam::ivec3(range,0,range)}
    }

    // translate a i absolute value to x,y,z coordinates such i = range * range * x + range * y + z
    pub fn from_single_value(i : i32, range : i32) -> ChunkPos {
        Self::from_absolute((i / range.pow(2)) % range, (i / range) % 7, i % range, range.add(-1) / 2)
    }

    pub fn from_block_pos(block_pos : &BlockPos) -> Self {
        Self::from_vec3(block_pos.as_ivec3().div_euclid(glam::IVec3::new(CHUNK_SIZE as i32, CHUNK_SIZE as i32, CHUNK_SIZE as i32)))
    }

    pub fn to_block_pos(&self) -> BlockPos {
        BlockPos::new((self.pos * CHUNK_SIZE as i32).as_vec3())
    }

    pub fn get_vec3(&self) -> IVec3 {
        self.pos
    }

    pub fn deserialize(pos_bits : Vec<u8>) -> ChunkPos {
        let raw_bytes = pos_bits[0..12].to_vec();
        let coords: &[i32] = bytemuck::cast_slice(&raw_bytes);
        ChunkPos::from_vec3(glam::IVec3::from_slice(coords))
    }

    pub fn lesser_then_or_equal(&self, chunk_pos: ChunkPos) -> bool {
        self.x <= chunk_pos.x && self.y <= chunk_pos.y && self.z <= chunk_pos.z
    }

    pub fn abs(&self) -> ChunkPos {
        Self {
            pos : self.pos.abs()
        }
    }

    pub fn flattened(&self) -> ChunkPos {
        Self {
            pos : glam::ivec3(self.x, 0, self.z)
        }
    }
}

impl PosTrait for ChunkPos {
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

impl Deref for ChunkPos {
    type Target = glam::IVec3;
    fn deref(&self) -> &Self::Target {
        &self.pos
    }
}

impl Mul<i32> for ChunkPos {

    type Output = ChunkPos;

    fn mul(self, other : i32) -> ChunkPos {
        ChunkPos::from_vec3(self.pos * other)
    }
}

impl Mul<usize> for ChunkPos {

    type Output = ChunkPos;

    fn mul(self, other : usize) -> ChunkPos {
        ChunkPos::from_vec3(self.pos * other as i32)
    }
}

impl Add<IBlockPos> for ChunkPos {
    type Output = IBlockPos;

    fn add(self, rhs: IBlockPos) -> IBlockPos {
        IBlockPos::from_vec3((self * CHUNK_SIZE).pos + rhs.deref())
    }
}

impl Add<ChunkPos> for ChunkPos {
    type Output = ChunkPos;

    fn add(self, rhs: ChunkPos) -> ChunkPos {
        ChunkPos::from_vec3(self.pos + rhs.deref())
    }
}

impl Sub<ChunkPos> for ChunkPos {
    type Output = ChunkPos;

    fn sub(self, rhs: Self) -> Self::Output {
        ChunkPos::from_vec3(self.pos - rhs.pos)
    }
}