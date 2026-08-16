use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::time::Instant;
use noise::Perlin;
use shared::common::world::block::block::BlockType;
use shared::common::world::chunk::chunk::Chunk;
use shared::common::world::pos::chunkpos::{ChunkPos, CHUNK_SIZE};
use shared::worldgen::Generator;

pub struct ServerChunk {
    chunk: Chunk
}

impl Drop for ServerChunk {
    fn drop(&mut self) {
    }
}

impl ServerChunk {

    pub fn generate_chunk(generator: Arc<Generator>, chunk_pos: ChunkPos) -> Chunk {
        let time : Instant = Instant::now();
        let mut blocks = Vec::new();
        blocks.resize(CHUNK_SIZE.pow(3), shared::common::world::block::block::BlockType::AIR.get_value());
        let weight_bias = 0.3;
        let height_bias = 1.5;
        for x in 0..CHUNK_SIZE {

            let block_x = x as i32 + chunk_pos.x * CHUNK_SIZE as i32;

            for z in 0..CHUNK_SIZE {

                let block_z = z as i32 + chunk_pos.z * CHUNK_SIZE as i32;
                let height : f64 = generator.get_terrain_height(block_x, block_z);
                // let height = generator.get_perlin_height(block_x as f64, block_z as f64);

                for y in 0..CHUNK_SIZE {
                    let y_absolute = y as i32 + chunk_pos.y * CHUNK_SIZE as i32;
                    // blocks[x * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + z] = generator.get_block(x as f64, y_absolute as f64, z as f64, max_h as f64).get_value();
                    blocks[x * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + z] = generator.density_of(block_x as f64, y_absolute as f64, block_z as f64, height, weight_bias, height_bias).get_value();
                }
            }
        }
        Chunk::new(chunk_pos, Arc::new(blocks))
        // print_debug!("Chunk created in {}ms",Instant::now().duration_since(time).as_millis());
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