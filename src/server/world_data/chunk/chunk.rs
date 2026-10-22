use shared::common::world::chunk::chunk::Chunk;
use shared::common::world::pos::chunkpos::{CHUNK_SIZE, ChunkPos};
use shared::worldgen::Generator;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::time::Instant;

pub struct ServerChunk {
    chunk: Chunk,
}

impl Drop for ServerChunk {
    fn drop(&mut self) {}
}

impl ServerChunk {
    pub fn generate_chunk(generator: Arc<Generator>, chunk_pos: ChunkPos) -> Chunk {
        let time: Instant = Instant::now();
        let mut blocks = Vec::new();
        blocks.resize(CHUNK_SIZE.pow(3), 0);
        let weight_bias = 0.4;
        let height_bias = 1.5;
        for cx in 0..CHUNK_SIZE {
            let block_x = cx as i32 + chunk_pos.x * CHUNK_SIZE as i32;
            // env.set("x", block_x);

            for cz in 0..CHUNK_SIZE {
                let block_z = cz as i32 + chunk_pos.z * CHUNK_SIZE as i32;

                let height: f64 = generator.get_terrain_height(block_x, block_z);
                // env.set("z", block_z);
                // env.set("height", height);

                for cy in 0..CHUNK_SIZE {
                    let block_y = cy as i32 + chunk_pos.y * CHUNK_SIZE as i32;
                    // env.set("y", block_y);
                    blocks[cx * CHUNK_SIZE * CHUNK_SIZE + cy * CHUNK_SIZE + cz] = generator
                        .density_of(block_x as f64, block_y as f64, block_z as f64, height, weight_bias, height_bias)
                        .unwrap_or(0);
                    // blocks[cX * CHUNK_SIZE * CHUNK_SIZE + cY * CHUNK_SIZE + cZ] = block_type.get_value();
                }
            }
        }

        // print_base!("Chunk created in {}ms",Instant::now().duration_since(time).as_millis());

        Chunk::new(chunk_pos, Arc::new(blocks))
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
        Self { chunk: self.chunk.clone() }
    }
}
