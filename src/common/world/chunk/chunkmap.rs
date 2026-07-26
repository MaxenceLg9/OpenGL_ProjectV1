use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use crate::common::world::block::block::BlockType;
use crate::common::world::chunk::chunk::Chunk;
use crate::common::world::pos::chunkpos::ChunkPos;
use crate::common::world::pos::iblockpos::IBlockPos;

pub struct ChunkMap {
    chunks : HashMap<ChunkPos, Chunk>,
}

impl ChunkMap {
    pub fn new() -> Self {
        Self {
            chunks : HashMap::new(),
        }
    }
    
    pub fn get_neighbours_chunks_pos(pos: ChunkPos) -> Vec<ChunkPos> {
        let mut v = Vec::new();
        for i in 0..27 {
            let relative_pos = ChunkPos::new(i / 9 % 3 - 1, i / 3 % 3 - 1, i % 3 - 1);
            v.push(pos + relative_pos);
        }
        v
    }

    pub fn add_chunk(&mut self, chunk: Chunk) -> bool {
        if chunk.get_chunk_pos().y < 0 || chunk.get_chunk_pos().y > 11 {
            return false;
        }
        match self.chunks.entry(chunk.get_chunk_pos()) {
            Entry::Occupied(_) => {
                false
            }
            Entry::Vacant(slot) => {
                // print_base!("Inserted {} chunk", chunk.get_chunk_pos().deref());
                slot.insert(chunk);
                true
            }
        }
    }

    pub fn set_block(&mut self, iblock_pos: IBlockPos, block_type: BlockType) -> bool {
        self.chunks.get_mut(&iblock_pos.get_chunk_pos()).unwrap().set_block(iblock_pos,block_type)
    }

    pub fn get_block_at(&self, block_pos : IBlockPos) -> u16 {
        match self.get_chunk(&block_pos.get_chunk_pos()) {
            Some(e) => {
                e.get_block_at(block_pos.get_block_pos())
            },
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

    pub fn remove_chunk(&mut self, chunk_pos : &ChunkPos) {
        self.chunks.remove(chunk_pos);
    }
}