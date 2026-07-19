use std::any::Any;
use std::ops::{Add, Deref, Div};
use bitvec::prelude::BitVec;
use glam::{IVec3};
use crate::common::world::pos::chunkpos::{ChunkPos, CHUNK_SIZE};
use crate::common::world::pos::pos_trait::PosTrait;
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct IBlockPos {
    pos : glam::IVec3,
}

/// BlockPos made of Integer, used to locate block position in the world
impl IBlockPos {
    pub fn from_vec3(pos : IVec3) -> Self {
        Self { pos }
    }

    pub fn new(i1 : i32, i2 : i32, i3 : i32) -> Self {
        Self::from_vec3(IVec3::new(i1, i2, i3))
    }

    pub fn from_array(pos : [i32; 3]) -> Self {
        Self::from_vec3(IVec3::from(pos))
    }

    /// Split the IBlockPos into a relative IBlockPos within the chunk boundaries and the ChunkPos
    pub fn as_split_pos(&self) -> (IBlockPos, ChunkPos) {
        (self.get_block_pos(),self.get_chunk_pos())
    }

    /// Returns the ChunkPos where the IBlockPos points to
    pub fn get_chunk_pos(&self) -> ChunkPos {
        ChunkPos::from_vec3(self.pos.div_euclid(glam::IVec3::new(CHUNK_SIZE as i32, CHUNK_SIZE as i32, CHUNK_SIZE as i32)))
    }

    /// Returns the relative IBlockPos within the chunk boundaries
    pub fn get_block_pos(&self) -> IBlockPos {
        IBlockPos::from_vec3(self.pos.rem_euclid(glam::IVec3::new(CHUNK_SIZE as i32, CHUNK_SIZE as i32, CHUNK_SIZE as i32)))
    }

    /// Returns the index of the IBlockPos coordinates in the block array of a chunk
    pub fn get_index(&self) -> usize {
        self.x as usize * CHUNK_SIZE * CHUNK_SIZE + self.y as usize * CHUNK_SIZE + self.z as usize
    }

    /// Extract IBlockPos from serialized bytes
    pub fn deserialize(pos_bits : Vec<u8>) -> Self {
        let raw_bytes = pos_bits[0..12].to_vec();
        let coords: &[i32] = bytemuck::cast_slice(&raw_bytes);
        IBlockPos::from_vec3(glam::IVec3::from_slice(coords))
    }
}
impl PosTrait for IBlockPos {
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
impl Div<i32> for &IBlockPos {
    type Output = IBlockPos;

    fn div(self, rhs: i32) -> IBlockPos {
        IBlockPos::from_vec3(self.pos / rhs)
    }
}

impl Deref for IBlockPos {
    type Target = glam::IVec3;
    fn deref(&self) -> &Self::Target {
        &self.pos
    }
}

impl Add<IVec3> for IBlockPos {
    type Output = IBlockPos;
    
    fn add(self, rhs: IVec3) -> Self::Output {
        IBlockPos::from_vec3(self.pos + rhs)
    }
}