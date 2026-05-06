use crate::common::world::pos::chunkpos::{ChunkPos, CHUNK_SIZE};
use crate::common::world::pos::iblockpos::IBlockPos;

pub struct Chunk {
    blocks : Vec<u16>,
    chunk_pos: ChunkPos
}

impl Chunk {

    pub fn get_blocks(&self) -> &Vec<u16> {
        &self.blocks
    }

    pub fn new(chunk_pos : ChunkPos, blocks : Vec<u16>) -> Chunk {
        Self {
            blocks,
            chunk_pos
        }
    }
    pub fn get_chunk_pos(&self) -> ChunkPos {
        self.chunk_pos
    }

    pub fn get_block_at(&self, block_pos: IBlockPos) -> u16 {
        if block_pos.x < 0 || block_pos.x >= CHUNK_SIZE as i32 || block_pos.y < 0 || block_pos.y >= CHUNK_SIZE as i32 || block_pos.z < 0 || block_pos.z >= CHUNK_SIZE as i32 {
            return 0; // out of bounds
        }
        self.blocks[ block_pos.x as usize * CHUNK_SIZE * CHUNK_SIZE + block_pos.y as usize * CHUNK_SIZE + block_pos.z as usize]
    }

    pub fn serialize(&self) -> Vec<u8> {
        let vec = self.blocks.iter().flat_map(|&e| e.to_le_bytes()).collect::<Vec<u8>>();
        vec
    }
}

impl Clone for Chunk {
    fn clone(&self) -> Self {
        Self {
            blocks: self.blocks.clone(),
            chunk_pos: self.chunk_pos.clone()
        }
    }
}