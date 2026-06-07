use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::time::Instant;
use noise::Perlin;
use shared::common::world::block::block::BlockType;
use shared::common::world::chunk::chunk::Chunk;
use shared::common::world::pos::chunkpos::{ChunkPos, CHUNK_SIZE};
use shared::math::Generator;

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
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let block_x = x as i32 + chunk_pos.x * CHUNK_SIZE as i32;
                let block_z = z as i32 + chunk_pos.z * CHUNK_SIZE as i32;
                let max_h : i32 = generator.get_terrain_height(block_x, block_z);
                // let max_h = (block_x.abs() + block_z.abs()) / 4;
                if block_x < -130 && block_x > -145 && block_z < 485 && block_z > 470 {
                    // print_base!("Chunk: {}, Max_h: {}, x: {}, z: {}",chunk_pos.deref(), max_h, block_x, block_z);
                    for y in 0..CHUNK_SIZE {
                        blocks[x * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + z] = BlockType::IKRINEBLOCK.get_value();
                    }
                    continue;
                }
                for y_relative in 0..CHUNK_SIZE {
                    let y_absolute = y_relative as i32 + chunk_pos.y * CHUNK_SIZE as i32;
                    if y_absolute > max_h {
                        if y_absolute < 85 {
                            blocks[x * CHUNK_SIZE * CHUNK_SIZE + y_relative * CHUNK_SIZE + z] = BlockType::DEEPSLATE.get_value();
                        } else {
                            blocks[x * CHUNK_SIZE * CHUNK_SIZE + y_relative * CHUNK_SIZE + z] = BlockType::AIR.get_value();
                        }
                    } else {
                        blocks[x * CHUNK_SIZE * CHUNK_SIZE + y_relative * CHUNK_SIZE + z] =  Self::generate_block(y_absolute, max_h);

                        if y_absolute > max_h - 3 {
                            blocks[x * CHUNK_SIZE * CHUNK_SIZE + y_relative * CHUNK_SIZE + z] = BlockType::DIRT.get_value();
                        }
                        if y_absolute > max_h - 1 {
                            blocks[x * CHUNK_SIZE * CHUNK_SIZE + y_relative * CHUNK_SIZE + z] = BlockType::GRASS.get_value();
                        }
                    }
                }
            }
        }
        Chunk::new(chunk_pos,blocks)
        // print_debug!("Chunk created in {}ms",Instant::now().duration_since(time).as_millis());
    }

    pub fn generate_block(y : i32, max_h : i32) -> u16 {
        if y > max_h - 1 {
            BlockType::GRASS.get_value();
        }
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