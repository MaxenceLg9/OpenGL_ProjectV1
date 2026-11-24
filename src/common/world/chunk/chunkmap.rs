use crate::common::world::block::block::{BlockId, BlockState};
use crate::common::world::chunk::chunk::Chunk;
use crate::common::world::pos::chunkpos::ChunkPos;
use crate::common::world::pos::iblockpos::IBlockPos;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

pub const MIN_Y_CHUNK: i32 = -4;
pub const MAX_Y_CHUNK: i32 = 9;

pub struct ChunkMap {
    chunks: HashMap<ChunkPos, Chunk>,
}

impl ChunkMap {
    pub fn new() -> Self {
        Self { chunks: HashMap::new() }
    }

    pub fn get_neighbours_chunks_pos(pos: ChunkPos) -> Vec<ChunkPos> {
        let mut v = Vec::new();
        // for i in 0..27 {
        //     let relative_pos = ChunkPos::new(i / 9 % 3 - 1, i / 3 % 3 - 1, i % 3 - 1);
        //     v.push(pos + relative_pos);
        // }
        for x in 0..3 {
            for y in 0..3 {
                for z in 0..3 {
                    let relative_pos = ChunkPos::new(pos.x + x - 1, pos.y + y - 1, pos.z + z - 1);
                    v.push(relative_pos);
                }
            }
        }
        v
    }

    pub fn add_chunk(&mut self, chunk: Chunk) -> bool {
        if chunk.get_chunk_pos().y < MIN_Y_CHUNK || chunk.get_chunk_pos().y > MAX_Y_CHUNK {
            return false;
        }
        self.chunks.insert(chunk.get_chunk_pos(), chunk);
        true
    }

    pub fn set_block(&mut self, iblock_pos: IBlockPos, block_type: BlockId) -> BlockId {
        self.chunks.get_mut(&iblock_pos.get_chunk_pos()).unwrap().set_block(iblock_pos, block_type);
        block_type
    }

    pub fn get_block_at(&self, block_pos: IBlockPos) -> u16 {
        match self.get_chunk(&block_pos.get_chunk_pos()) {
            Some(e) => e.get_block_at(block_pos.get_block_pos()),
            None => 0,
        }
    }

    pub fn contains_chunk(&self, chunk_pos: &ChunkPos) -> bool {
        self.chunks.contains_key(chunk_pos)
    }

    pub fn get_chunk(&self, chunk_pos: &ChunkPos) -> Option<&Chunk> {
        self.chunks.get(chunk_pos)
    }

    pub fn get_chunk_mut(&mut self, chunk_pos: &ChunkPos) -> Option<&mut Chunk> {
        self.chunks.get_mut(chunk_pos)
    }
    pub fn len(&self) -> usize {
        self.chunks.len()
    }
    pub fn remove_chunk(&mut self, chunk_pos: &ChunkPos) {
        self.chunks.remove(chunk_pos);
    }

    pub fn get_pos(&self) -> Vec<ChunkPos> {
        self.chunks.keys().copied().collect()
    }
}
