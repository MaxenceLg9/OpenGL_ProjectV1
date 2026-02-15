use std::cmp::max;
use glam::UVec3;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock, RwLockReadGuard};
use std::time::Instant;
use crate::display::renderer::mesh::chunk_mesh::ChunkMesh;
use crate::game::world::chunk::block::block::BlockType;
use crate::game::world::world::{World, WorldData};
use crate::math::noised_terrain_default;

pub const CHUNK_SIZE : usize = 64;
pub struct Chunk {
    blocks : Vec<u16>,
    chunk_pos: glam::IVec3
}

impl Drop for Chunk {
    fn drop(&mut self) {

    }
}

impl Chunk {
    pub fn new(chunk_pos : glam::IVec3) -> Chunk{
        let mut chunk = Self {
            blocks: Vec::new(),
            chunk_pos
        };
        chunk.blocks.resize(CHUNK_SIZE.pow(3),BlockType::AIR.get_value());
        // println!("Generating the chunk");
        chunk.generate_chunk();
        chunk
    }

    pub fn generate_chunk(&mut self){
        let time : Instant = Instant::now();
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let block_x = x as i32 + self.chunk_pos.x * CHUNK_SIZE as i32;
                let block_z = z as i32 + self.chunk_pos.z * CHUNK_SIZE as i32;
                let max_h : i32 = noised_terrain_default(block_x, block_z) as i32 * 100 + 400;
                let local_max_height = max_h - self.chunk_pos.y * CHUNK_SIZE as i32;
                //            Logs::debug("MaxH: " + std::to_string(max_h));
                for y in 0..local_max_height {
                    if (y > (CHUNK_SIZE - 1) as i32) {
                        break;
                    }
                    self.blocks[x * CHUNK_SIZE * CHUNK_SIZE + y as usize * CHUNK_SIZE + z] =  self.generate_block((y + self.chunk_pos.y * CHUNK_SIZE as i32) as u16);
                }
                for y in local_max_height.max(0) as usize..CHUNK_SIZE - 1 {
                    self.blocks[x * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + z] = BlockType::AIR.get_value();
                }
            }

        }

    }

    pub fn generate_block(&self,y : u16) -> u16 {
        if (y < 100) {
            BlockType::DEEPSLATE.get_value(); // Deepslate
        }
        if (y < 200 || y > 400) {
            BlockType::STONE.get_value() // Stone
        }
        else {
            BlockType::DIRT.get_value() // Dirt;
        }
    }

    pub fn get_chunk_pos(&self) -> glam::IVec3 {
        self.chunk_pos
    }

    pub fn get_block_at(&self, block_pos: glam::IVec3) -> u16 {
        if (block_pos.x < 0 || block_pos.x >= CHUNK_SIZE as i32 || block_pos.y < 0 || block_pos.y >= CHUNK_SIZE as i32 || block_pos.z < 0 || block_pos.z >= CHUNK_SIZE as i32) {
            return 0; // out of bounds
        }
        self.blocks[ block_pos.x as usize * CHUNK_SIZE * CHUNK_SIZE + block_pos.y as usize * CHUNK_SIZE + block_pos.z as usize]
    }

    pub fn build_mesh(&self, world : &RwLockReadGuard<WorldData>) -> ChunkMesh {
        ChunkMesh::new(world, self.chunk_pos, &self.blocks)
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