use std::vec::Vec;
use gl::types::{GLuint};
use gl::{TRIANGLES, UNSIGNED_INT};
use glam::*;

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
    pub fn new(vertices : Vec<u32>, indices : Vec<u32>) -> ChunkMesh{
        let mut chunk_mesh = Self {
            vao: 0,
            vbo: 0,
            ebo: 0,
            nb_indices: 0,
            vertices,
            indices,
            linked: false,
        };
        chunk_mesh.linked = false;
        chunk_mesh
    }

    pub unsafe fn link(&mut self) {
        self.nb_indices = self.indices.len() as i32;
        self.setup_mesh();
        self.bind_data();
        self.linked = true;
    }

    pub fn is_linked(&self) -> bool {
        self.linked
    }

    unsafe fn setup_mesh(&mut self) {
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

    unsafe fn bind_data(&self) {
        gl::NamedBufferData(self.vbo, self.vertices.len().cast_signed() * 4, self.vertices.as_ptr() as *const _ , gl::STATIC_DRAW);
        gl::NamedBufferData(self.ebo, self.indices.len().cast_signed() * 4, self.indices.as_ptr() as *const _, gl::STATIC_DRAW);
    }

    pub unsafe fn draw(&self,) {
        if !self.linked || self.nb_indices == 0 { return; }
        gl::BindVertexArray(self.vao);
        gl::DrawElementsBaseVertex(TRIANGLES,self.nb_indices,UNSIGNED_INT, std::ptr::null(),0);
        gl::BindVertexArray(0);
    }

}