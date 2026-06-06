use shared::print_base;
use std::collections::HashMap;
use std::sync::Arc;
use gl::types::{GLint, GLuint};
use gl::UNSIGNED_INT;
use shared::common::display::vertex::vertex::Vertex;
use shared::common::world::chunk::chunk::Chunk;
use shared::common::world::pos::chunkpos::{ChunkPos, CHUNK_SIZE};
use shared::common::world::pos::iblockpos::IBlockPos;
use shared::print_debug;
use crate::client::display::renderer::gui::text::text::MeshText;
use crate::client::display::renderer::mesh::chunk_mesh::Mesh;

pub struct ChunkMesh {
    vertices: Vec<u32>,
    indices: Vec<u32>,
    chunk_pos: ChunkPos
}


impl ChunkMesh {

    pub fn get_chunk_pos(&self) -> ChunkPos {
        self.chunk_pos
    }

    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty() && self.indices.is_empty()
    }

    pub unsafe fn link(&mut self) -> Mesh {
        let (mut vao, mut vbo, mut ebo) = (0,0,0);
        Self::setup_mesh(&mut vao,&mut vbo,&mut ebo);
        self.bind_data(&mut vbo,&mut ebo);
        Mesh::new(vao,vbo,ebo, self.indices.len() as i32)
    }



    unsafe fn setup_mesh(vao: &mut GLuint, vbo: &mut GLuint, ebo: &mut GLuint) {

        // 1. Create the objects (DSA style)
        gl::CreateBuffers(1, vbo);
        gl::CreateBuffers(1, ebo);
        gl::CreateVertexArrays(1, vao);

        // 2. Configure Attribute 0
        gl::VertexArrayVertexBuffer(*vao, 0, *vbo, 0, 8);

        gl::EnableVertexArrayAttrib(*vao, 0);
        gl::EnableVertexArrayAttrib(*vao, 1);

        // 3. Configure Attribute 1
        gl::VertexArrayAttribIFormat(*vao, 0, 1, UNSIGNED_INT, 0);
        gl::VertexArrayAttribIFormat(*vao, 1, 1, UNSIGNED_INT, 4);

        gl::VertexArrayAttribBinding(*vao,0,0);
        gl::VertexArrayAttribBinding(*vao,1,0);

        gl::VertexArrayElementBuffer(*vao, *ebo);

    }

    unsafe fn bind_data(&self, vbo: &mut GLuint, ebo: &mut GLuint) {
        gl::NamedBufferData(*vbo, self.vertices.len().cast_signed() * 4, self.vertices.as_ptr() as *const _ , gl::STATIC_DRAW);
        gl::NamedBufferData(*ebo, self.indices.len().cast_signed() * 4, self.indices.as_ptr() as *const _, gl::STATIC_DRAW);
    }

    pub fn build_mesh(chunks_map : &HashMap<ChunkPos,Vec<u16>>, chunk_pos: ChunkPos) -> Option<(ChunkMesh, MeshText)> {
        let mut chunk_mesh = Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            chunk_pos
        };
        let blocks = chunks_map.get(&chunk_pos)?;
        chunk_mesh.vertices.reserve(CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE * 6 * 4 * 2); // 6 faces, 4 vertices per face
        chunk_mesh.indices.reserve(CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE * 6 * 6); // 6 faces, 6 nbIndices per face
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
                    if chunk_mesh.is_void(IBlockPos::from_ints(ux, uy, uz + 1), blocks, chunks_map, chunk_pos) {

                        v[0] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux, uy, uz + 1), 1, 3).unwrap();
                        v[1] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux, uy + 1, uz + 1), 1, 2).unwrap();
                        v[2] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy + 1, uz + 1), 1, 0).unwrap();
                        v[3] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy, uz + 1), 1, 1).unwrap();

                        index = chunk_mesh.add_data(v, index);
                    }
                    // back face
                    if chunk_mesh.is_void(IBlockPos::from_ints(ux, uy, uz - 1), blocks, chunks_map, chunk_pos) {

                        v[0] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy + 1, uz), 4, 2).unwrap();
                        v[1] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux, uy + 1, uz), 4, 0).unwrap();
                        v[2] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux, uy, uz), 4, 1).unwrap();
                        v[3] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy, uz), 4, 3).unwrap();

                        index = chunk_mesh.add_data(v, index);
                    }
                    //top face
                    if chunk_mesh.is_void(IBlockPos::from_ints(ux, uy + 1, uz), blocks, chunks_map, chunk_pos) {

                        v[0] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux, uy + 1, uz), 0, 2).unwrap();
                        v[1] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy + 1, uz), 0, 0).unwrap();
                        v[2] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy + 1, uz + 1), 0, 1).unwrap();
                        v[3] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux, uy + 1, uz + 1), 0, 3).unwrap();

                        index = chunk_mesh.add_data(v, index);
                    }
                    // bottom face
                    if chunk_mesh.is_void(IBlockPos::from_ints(ux, uy - 1, uz), blocks, chunks_map, chunk_pos) {

                        v[0] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux, uy, uz), 5, 1).unwrap();
                        v[1] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux, uy, uz + 1), 5, 3).unwrap();
                        v[2] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy, uz + 1), 5, 2).unwrap();
                        v[3] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy, uz), 5, 0).unwrap();

                        index = chunk_mesh.add_data(v, index);
                    }

                    // right face
                    if chunk_mesh.is_void(IBlockPos::from_ints(ux + 1, uy, uz), blocks, chunks_map, chunk_pos) {

                        v[0] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy, uz), 2, 1).unwrap();
                        v[1] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy, uz + 1), 2, 3).unwrap();
                        v[2] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy + 1, uz + 1), 2, 2).unwrap();
                        v[3] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy + 1, uz), 2, 0).unwrap();

                        index = chunk_mesh.add_data(v, index);
                    }

                    // left face
                    if chunk_mesh.is_void(IBlockPos::from_ints(ux - 1, uy, uz), blocks, chunks_map, chunk_pos) {

                        v[0] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux, uy, uz), 3, 3).unwrap();
                        v[1] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux, uy + 1, uz), 3, 2).unwrap();
                        v[2] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux, uy + 1, uz + 1), 3, 0).unwrap();
                        v[3] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux, uy, uz + 1), 3, 1).unwrap();

                        index = chunk_mesh.add_data(v, index);
                    }
                }
            }
        }
        if chunk_mesh.indices.len() == 0 {
            return None
        };
        print_debug!("Created {} vertices in chunks", chunk_mesh.indices.len());
        let mesh_text = unsafe { MeshText::new(chunk_mesh.vertices.clone(), chunk_mesh.indices.clone()) };
        Some((chunk_mesh, mesh_text))
    }

    fn is_void(&self, block_pos: IBlockPos, blocks : &Vec<u16>, chunks_map : &HashMap<ChunkPos,Vec<u16>>, chunk_pos: ChunkPos) -> bool {
        if block_pos.x < 0 || block_pos.x >= CHUNK_SIZE as i32 ||
            block_pos.y < 0 || block_pos.y >= CHUNK_SIZE as i32 ||
            block_pos.z < 0 || block_pos.z >= CHUNK_SIZE as i32 {

            return self.get_block_at(chunk_pos + block_pos, chunks_map) == 0;
        }
        blocks[block_pos.x as usize * CHUNK_SIZE * CHUNK_SIZE + block_pos.y as usize * CHUNK_SIZE + block_pos.z as usize] == 0
    }

    pub fn get_block_at(&self, ipos : IBlockPos, chunks_map : &HashMap<ChunkPos,Vec<u16>>,) -> u16 {
        let sz = CHUNK_SIZE as i32;

        let (block_pos, chunk_pos) = ipos.as_split_pos();
        if chunk_pos.y < -2 || chunk_pos.y > 9 {
            return 0
        }
        chunks_map.get(&chunk_pos).unwrap()[block_pos.get_offset()]
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