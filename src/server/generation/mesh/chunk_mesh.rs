use shared::print_base;
use std::collections::HashMap;
use std::sync::{Arc};
use std::vec::Vec;
use glam::*;
use shared::common::display::vertex::vertex::Vertex;
use shared::common::world::pos::chunkpos::{ChunkPos, CHUNK_SIZE};
use shared::common::world::pos::iblockpos::IBlockPos;
use shared::print_debug;
use crate::server::world_data::chunk::chunk::Chunk;

pub struct ServerChunkMesh {
    vertices: Vec<u32>,
    indices: Vec<u32>,
    linked : bool,
}

impl ServerChunkMesh {
    pub fn new(chunks_map : &HashMap<ChunkPos,Arc<Chunk>>, chunk_pos: ChunkPos, blocks : &Vec<u16>) -> ServerChunkMesh {
        let mut chunk_mesh = Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            linked: false,
        };
        chunk_mesh.linked = false;
        chunk_mesh.build_mesh(chunks_map, chunk_pos, blocks);
        chunk_mesh
    }

    fn build_mesh(&mut self, chunks_map : &HashMap<ChunkPos,Arc<Chunk>>, chunk_pos: ChunkPos, blocks : &Vec<u16>) {

        self.vertices.reserve(CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE * 6 * 4 * 2); // 6 faces, 4 vertices per face
        self.indices.reserve(CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE * 6 * 6); // 6 faces, 6 nbIndices per face
        let mut index = 0;
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE{
                    let voxel_id : u16 = blocks[x * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + z];

                    if voxel_id == 0 {
                        continue; // skip empty blocks
                    }
                    let mut v: [u64;4] = [0; 4];
                    let (ux, uy, uz) = (x as i32, y as i32, z as i32);
                    //front face
                    if self.is_void(IBlockPos::from_ints(ux, uy, uz + 1), blocks, chunks_map, chunk_pos) {

                        v[0] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux, uy, uz + 1), 1, 0).unwrap();
                        v[1] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux, uy + 1, uz + 1), 1, 1).unwrap();
                        v[2] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy + 1, uz + 1), 1, 3).unwrap();
                        v[3] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy, uz + 1), 1, 2).unwrap();

                        index = self.add_data(v, index);
                    }
                    // back face
                    if self.is_void(IBlockPos::from_ints(ux, uy, uz - 1), blocks, chunks_map, chunk_pos) {

                        v[0] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux, uy, uz), 4, 2).unwrap();
                        v[1] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy, uz), 4, 0).unwrap();
                        v[2] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy + 1, uz), 4, 1).unwrap();
                        v[3] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux, uy + 1, uz), 4, 3).unwrap();

                        index = self.add_data(v, index);
                    }
                    //top face
                    if self.is_void(IBlockPos::from_ints(ux, uy + 1, uz), blocks, chunks_map, chunk_pos) {

                        v[0] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux, uy + 1, uz), 0, 2).unwrap();
                        v[1] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy + 1, uz), 0, 0).unwrap();
                        v[2] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy + 1, uz + 1), 0, 1).unwrap();
                        v[3] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux, uy + 1, uz + 1), 0, 3).unwrap();

                        index = self.add_data(v, index);
                    }
                    // bottom face
                    if self.is_void(IBlockPos::from_ints(ux, uy - 1, uz), blocks, chunks_map, chunk_pos) {

                        v[0] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux, uy, uz), 5, 1).unwrap();
                        v[1] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux, uy, uz + 1), 5, 3).unwrap();
                        v[2] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy, uz + 1), 5, 2).unwrap();
                        v[3] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy, uz), 5, 0).unwrap();

                        index = self.add_data(v, index);
                    }

                    // right face
                    if self.is_void(IBlockPos::from_ints(ux + 1, uy, uz), blocks, chunks_map, chunk_pos) {

                        v[0] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy, uz), 2, 2).unwrap();
                        v[1] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy, uz + 1), 2, 0).unwrap();
                        v[2] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy + 1, uz + 1), 2, 1).unwrap();
                        v[3] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy + 1, uz), 2, 3).unwrap();

                        index = self.add_data(v, index);
                    }

                    // left face
                    if self.is_void(IBlockPos::from_ints(ux - 1, uy, uz), blocks, chunks_map, chunk_pos) {

                        v[0] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux, uy, uz), 3, 0).unwrap();
                        v[1] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux, uy + 1, uz), 3, 1).unwrap();
                        v[2] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux, uy + 1, uz + 1), 3, 3).unwrap();
                        v[3] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux, uy, uz + 1), 3, 2).unwrap();

                        index = self.add_data(v, index);
                    }
                }
            }
        }
        if self.indices.len() == 0 {
            return;
        } else {
            print_debug!("Created {} vertices in chunks", self.indices.len());
        }
    }

    fn is_void(&self, block_pos: IBlockPos, blocks : &Vec<u16>, chunks_map : &HashMap<ChunkPos,Arc<Chunk>>, chunk_pos: ChunkPos) -> bool {
        if block_pos.x < 0 || block_pos.x >= CHUNK_SIZE as i32 ||
            block_pos.y < 0 || block_pos.y >= CHUNK_SIZE as i32 ||
            block_pos.z < 0 || block_pos.z >= CHUNK_SIZE as i32 {

            return self.get_block_at(chunk_pos * CHUNK_SIZE as i32 + block_pos, chunks_map) == 0;
        }
        blocks[block_pos.x as usize * CHUNK_SIZE * CHUNK_SIZE + block_pos.y as usize * CHUNK_SIZE + block_pos.z as usize] == 0
    }

    pub fn get_block_at(&self, ipos : IBlockPos, chunks_map : &HashMap<ChunkPos,Arc<Chunk>>,) -> u16 {
        let sz = CHUNK_SIZE as i32;

        let (block_pos, chunk_pos) = ipos.as_split_pos();

        chunks_map.get(&chunk_pos).unwrap().get_block_at(block_pos)
    }

    fn add_data(&mut self, v : [u64;4], index : u32) -> u32 {

        for i in 0..4usize {
            self.vertices.push((v[i] >> 32) as u32);        // High 32 bits
            self.vertices.push((v[i] & 0xFFFFFFFF) as u32); // Low 32 bits
        }

        self.indices.push(index);
        self.indices.push(index + 2);
        self.indices.push(index + 1);
        self.indices.push(index);
        self.indices.push(index + 3);
        self.indices.push(index + 2);

        index + 4
    }
}