use gl::types::GLuint;
use gl::{TRIANGLES, UNSIGNED_INT};
use glam::*;
use shared::common::world::pos::chunkpos::{CHUNK_SIZE, ChunkPos};

use crate::client::gl::mesh::shader::shader::Shader;

#[derive(Clone)]
pub struct Mesh {
    vao: GLuint,
    nb_indices: i32,
}

impl Mesh {
    pub fn new(vao: GLuint, _vbo: GLuint, _ebo: GLuint, nb_indices: i32) -> Mesh {
        Self { vao, nb_indices }
    }

    pub unsafe fn draw(&self, chunk_shader: &Shader, pos: &ChunkPos) {
        unsafe {
            if self.nb_indices == 0 {
                return;
            }
            let mut model = glam::Mat4::IDENTITY;
            model *= glam::Mat4::from_translation(pos.as_vec3() * CHUNK_SIZE as f32);
            chunk_shader.set_matrix4fv("uniform_model", model);
            gl::BindVertexArray(self.vao);
            gl::DrawElementsBaseVertex(TRIANGLES, self.nb_indices, UNSIGNED_INT, std::ptr::null(), 0);
        }
    }
}
