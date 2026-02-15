use std::sync::{Arc, RwLockReadGuard};
use std::vec::Vec;
use gl::types::{GLint, GLuint};
use gl::{TRIANGLES, UNSIGNED_INT};
use glam::*;
use crate::game::world::world::{WorldData};
use crate::game::world::chunk::chunk::*;
use crate::display::renderer::mesh::vertex::vertex::Vertex;

pub struct ChunkMesh {
    vao : GLuint,
    vbo : GLuint,
    ebo : GLuint,
    nb_indices: i32,
    vertices: Vec<u32>,
    indices: Vec<u32>,
    linked : bool,
}

impl ChunkMesh {
    pub(crate) fn new(world : &RwLockReadGuard<WorldData>, chunk_pos: IVec3, blocks : &Vec<u16>) -> ChunkMesh{
        let mut chunk_mesh = Self {
            vao: 0,
            vbo: 0,
            ebo: 0,
            nb_indices: 0,
            vertices: Vec::new(),
            indices: Vec::new(),
            linked: false,
        };
        chunk_mesh.linked = false;
        chunk_mesh.build_mesh(world, chunk_pos, blocks);
        chunk_mesh
    }

    pub unsafe fn link(&mut self) {
        self.setup_mesh2();
        self.bind_data2();
        self.linked = true;
    }

    pub(crate) fn is_linked(&self) -> bool {
        self.linked
    }

    unsafe fn setup_mesh2(&mut self) {
        let (mut vbo, mut ebo, mut vao) = (0, 0, 0);

        // 1. Create the objects (DSA style)
        gl::CreateBuffers(1, &mut vbo);
        gl::CreateBuffers(1, &mut ebo);
        gl::CreateVertexArrays(1, &mut vao);

        // 2. Configure Attribute 0
        gl::VertexArrayVertexBuffer(vao, 0, vbo, 0, 8);

        gl::EnableVertexArrayAttrib(vao, 0);
        gl::EnableVertexArrayAttrib(vao, 1);

        // 3. Configure Attribute 1
        gl::VertexArrayAttribIFormat(vao, 0, 1, UNSIGNED_INT, 0);
        gl::VertexArrayAttribIFormat(vao, 1, 1, UNSIGNED_INT, 4);

        gl::VertexArrayAttribBinding(vao,0,0);
        gl::VertexArrayAttribBinding(vao,1,0);

        gl::VertexArrayElementBuffer(vao, ebo);

        self.vao = vao;
        self.vbo = vbo;
        self.ebo = ebo;

    }

    unsafe fn bind_data2(&self) {
        gl::NamedBufferData(self.vbo, self.vertices.len().cast_signed() * 4, self.vertices.as_ptr() as *const _ , gl::STATIC_DRAW);
        gl::NamedBufferData(self.ebo, self.indices.len().cast_signed() * 4, self.indices.as_ptr() as *const _, gl::STATIC_DRAW);
    }


    fn build_mesh(&mut self, world : &RwLockReadGuard<WorldData>, chunk_pos: IVec3, blocks : &Vec<u16>) {

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
                    if self.is_void(glam::IVec3::new(ux, uy, uz + 1), blocks, world, chunk_pos) {

                        v[0] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux, uy, uz + 1), 1, 0).unwrap();
                        v[1] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux, uy + 1, uz + 1), 1, 1).unwrap();
                        v[2] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy + 1, uz + 1), 1, 3).unwrap();
                        v[3] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy, uz + 1), 1, 2).unwrap();

                        index = self.add_data(v, index);
                    }
                    // back face
                    if self.is_void(glam::IVec3::new(ux, uy, uz - 1), blocks, world, chunk_pos) {

                        v[0] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux, uy, uz), 4, 2).unwrap();
                        v[1] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy, uz), 4, 0).unwrap();
                        v[2] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy + 1, uz), 4, 1).unwrap();
                        v[3] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux, uy + 1, uz), 4, 3).unwrap();

                        index = self.add_data(v, index);
                    }
                    //top face
                    if self.is_void(glam::IVec3::new(ux, uy + 1, uz), blocks, world, chunk_pos) {

                        v[0] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux, uy + 1, uz), 0, 2).unwrap();
                        v[1] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy + 1, uz), 0, 0).unwrap();
                        v[2] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy + 1, uz + 1), 0, 1).unwrap();
                        v[3] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux, uy + 1, uz + 1), 0, 3).unwrap();

                        index = self.add_data(v, index);
                    }
                    // bottom face
                    if self.is_void(glam::IVec3::new(ux, uy - 1, uz), blocks, world, chunk_pos) {

                        v[0] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux, uy, uz), 5, 1).unwrap();
                        v[1] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux, uy, uz + 1), 5, 3).unwrap();
                        v[2] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy, uz + 1), 5, 2).unwrap();
                        v[3] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy, uz), 5, 0).unwrap();

                        index = self.add_data(v, index);
                    }

                    // right face
                    if self.is_void(glam::IVec3::new(ux + 1, uy, uz), blocks, world, chunk_pos) {

                        v[0] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy, uz), 2, 2).unwrap();
                        v[1] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy, uz + 1), 2, 0).unwrap();
                        v[2] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy + 1, uz + 1), 2, 1).unwrap();
                        v[3] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy + 1, uz), 2, 3).unwrap();

                        index = self.add_data(v, index);
                    }

                    // left face
                    if self.is_void(glam::IVec3::new(ux - 1, uy, uz), blocks, world, chunk_pos) {

                        v[0] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux, uy, uz), 3, 0).unwrap();
                        v[1] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux, uy + 1, uz), 3, 1).unwrap();
                        v[2] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux, uy + 1, uz + 1), 3, 3).unwrap();
                        v[3] = Vertex::pack_data(voxel_id, glam::IVec3::new(ux, uy, uz + 1), 3, 2).unwrap();

                        index = self.add_data(v, index);
                    }
                }
            }
        }
        self.nb_indices = self.indices.len() as i32;
        if self.nb_indices == 0 {
            // println!("No vertices to draw");
            return;
        } else {
            println!("Created {} vertices in chunks", self.nb_indices);
        }
        //    Logs::debug("Size " + std::to_string(vertices->size()) + " : " + std::to_string(indices->size()));
        //    Logs::debug("Data bound to VBO and EBO");
    }

    fn is_void(&self, block_pos: glam::IVec3, blocks : &Vec<u16>, world : &RwLockReadGuard<WorldData>, chunk_pos: glam::IVec3) -> bool {
        if block_pos.x < 0 || block_pos.x >= CHUNK_SIZE as i32 ||
            block_pos.y < 0 || block_pos.y >= CHUNK_SIZE as i32 ||
            block_pos.z < 0 || block_pos.z >= CHUNK_SIZE as i32 {
            return world.get_block_at(chunk_pos * CHUNK_SIZE as i32 + block_pos) == 0;
        }
        blocks[block_pos.x as usize * CHUNK_SIZE * CHUNK_SIZE + block_pos.y as usize * CHUNK_SIZE + block_pos.z as usize] == 0
    }

    pub unsafe fn draw(&self,) {
        if !self.linked || self.nb_indices == 0 { return; }
        gl::BindVertexArray(self.vao);
        gl::DrawElementsBaseVertex(TRIANGLES,self.nb_indices,UNSIGNED_INT, std::ptr::null(),0);
        gl::BindVertexArray(0);
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