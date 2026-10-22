use crate::common::world::block::block::{BlockId, BlockState};
use crate::common::world::pos::chunkpos::{CHUNK_SIZE, ChunkPos};
use crate::common::world::pos::iblockpos::IBlockPos;
use std::sync::Arc;

pub struct Chunk {
    blocks: Arc<Vec<BlockId>>,
    chunk_pos: ChunkPos,
}

impl Chunk {
    pub fn get_blocks(&self) -> Arc<Vec<BlockId>> {
        self.blocks.clone()
    }

    pub fn new(chunk_pos: ChunkPos, blocks: Arc<Vec<BlockId>>) -> Chunk {
        Self { blocks, chunk_pos }
    }

    pub fn set_block(&mut self, iblock_pos: IBlockPos, block_type: BlockId) -> bool {
        if self.blocks[iblock_pos.get_block_pos().get_index()] == block_type {
            return false;
        }
        let blocks = Arc::make_mut(&mut self.blocks);
        blocks[iblock_pos.get_block_pos().get_index()] = block_type;
        true
    }
    pub fn get_chunk_pos(&self) -> ChunkPos {
        self.chunk_pos
    }

    pub fn get_block_at(&self, block_pos: IBlockPos) -> BlockId {
        if block_pos.x < 0 || block_pos.x >= CHUNK_SIZE as i32 || block_pos.y < 0 || block_pos.y >= CHUNK_SIZE as i32 || block_pos.z < 0 || block_pos.z >= CHUNK_SIZE as i32 {
            return 0; // out of bounds
        }
        self.blocks[block_pos.get_index()]
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
            chunk_pos: self.chunk_pos.clone(),
        }
    }
}
