use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc};
use std::time::Instant;
use shared::common::world::chunk::chunk::Chunk;
use shared::common::world::pos::chunkpos::{ChunkPos, CHUNK_SIZE};
use shared::math::noised_terrain_default;
use crate::server::generation::mesh::chunk_mesh::ServerChunkMesh;
use crate::server::world_data::block::block::BlockType;

pub struct ServerChunk {
    chunk: Chunk
}

impl Drop for ServerChunk {
    fn drop(&mut self) {
    }
}

impl ServerChunk {

    pub fn generate_chunk(chunk_pos: ChunkPos) -> Chunk {
        let time : Instant = Instant::now();
        let mut blocks = Vec::new();
        blocks.resize(CHUNK_SIZE.pow(3), shared::common::world::block::block::BlockType::AIR.get_value());
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let block_x = x as i32 + chunk_pos.x * CHUNK_SIZE as i32;
                let block_z = z as i32 + chunk_pos.z * CHUNK_SIZE as i32;
                let max_h : i32 = (noised_terrain_default(block_x, block_z) * 100.0 + 150.0) as i32;
                let local_max_height = max_h - chunk_pos.y * CHUNK_SIZE as i32;
                for y in 0..local_max_height {
                    if y > (CHUNK_SIZE - 1) as i32 {
                        break;
                    }
                    blocks[x * CHUNK_SIZE * CHUNK_SIZE + y as usize * CHUNK_SIZE + z] =  Self::generate_block((y + chunk_pos.y * CHUNK_SIZE as i32) as u16);
                }
                for y in local_max_height.max(0) as usize..CHUNK_SIZE - 1 {
                    blocks[x * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + z] = BlockType::AIR.get_value();
                }
            }
        }
        Chunk::new(chunk_pos,blocks)
        // print_debug!("Chunk created in {}ms",Instant::now().duration_since(time).as_millis());
    }

    pub fn generate_block(y : u16) -> u16 {
        if y < 100 {
            BlockType::DEEPSLATE.get_value(); // Deepslate
        }
        if y < 200 || y > 400 {
            BlockType::STONE.get_value() // Stone
        }
        else {
            BlockType::DIRT.get_value() // Dirt;
        }
    }


}

impl Deref for ServerChunk {
    type Target = Chunk;

    fn deref(&self) -> &Self::Target {
        &self.chunk
    }
}

impl DerefMut for ServerChunk {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.chunk
    }
}

impl Clone for ServerChunk {
    fn clone(&self) -> Self {
        Self {
            chunk: self.chunk.clone()
        }
    }
}