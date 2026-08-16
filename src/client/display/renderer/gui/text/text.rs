use gl::types::{GLsizei, GLuint};
use gl::UNSIGNED_INT;
use shared::common::display::vertex::vertex::Vertex;
use crate::client::display::renderer::mesh::chunk_mesh::Mesh;
use crate::client::display::renderer::mesh::shader::shader::Shader;
use crate::client::display::renderer::mesh::texture::texture_array::TextureArray;

pub struct MeshText {
    vao : GLuint,
    vbo : GLuint,
    ebo : GLuint,
    vertices : Vec<u32>,
    indices : Vec<u32>,
}

impl MeshText {
    pub unsafe fn new(vertices: Vec<u32>, indices: Vec<u32>) -> MeshText {
        let (vao, vbo, ebo) = (0, 0, 0);
        Self {
            vao,
            vbo,
            ebo,
            vertices,
            indices,
        }
    }

    pub unsafe fn link(mut self) -> MeshText {
        let (mut vao, mut vbo, mut ebo) = (0,0,0);
        Self::setup_mesh(&mut self.vao,&mut self.vbo,&mut self.ebo);
        self.bind_data();
        self
    }

    unsafe fn bind_data(&mut self) {
        gl::NamedBufferData(self.vbo, self.vertices.len().cast_signed() * 4, self.vertices.as_ptr() as *const _ , gl::STATIC_DRAW);
        gl::NamedBufferData(self.ebo, self.indices.len().cast_signed() * 4, self.indices.as_ptr() as *const _, gl::STATIC_DRAW);
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

    pub unsafe fn draw(&self) {     // render the mesh
        if self.indices.len() == 0 {
            return;
        }

        // bind appropriate textures
        // draw mesh
        gl::BindVertexArray(self.vao);
        gl::DrawElementsBaseVertex(gl::TRIANGLES, self.indices.len() as GLsizei, gl::UNSIGNED_INT, std::ptr::null(), 0);
        gl::BindVertexArray(0);


        // always good practice to set everything back to defaults once configured.
        gl::ActiveTexture(gl::TEXTURE0);
    }
}
impl Drop for MeshText {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteBuffers(1, &self.ebo);
        }
    }
}