use gl::types::{GLsizei, GLuint};
use shared::common::display::vertex::vertex::Vertex;

use crate::client::gl::mesh::{shader::shader::Shader, texture::texture::Texture};

pub struct Mesh {
    vao: GLuint,
    vbo: GLuint,
    ebo: GLuint,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    textures: Vec<Texture>,
}

impl Mesh {
    pub unsafe fn new(vertices: Vec<Vertex>, indices: Vec<u32>, textures: Vec<Texture>) -> Mesh {
        unsafe {
            let (vao, vbo, ebo) = (0, 0, 0);
            let mut mesh = Self { vao, vbo, ebo, vertices, indices, textures };
            mesh.setup_mesh();
            mesh
        }
    }

    pub unsafe fn setup_mesh(&mut self) {
        unsafe {
            gl::CreateBuffers(1, &mut self.vbo);
            gl::NamedBufferData(self.vbo, (self.vertices.len() * std::mem::size_of::<Vertex>()).cast_signed(), self.vertices.as_ptr() as *const _, gl::STATIC_DRAW);

            gl::CreateBuffers(1, &mut self.ebo);
            gl::NamedBufferData(self.ebo, (self.indices.len() * std::mem::size_of::<u32>()).cast_signed(), self.indices.as_ptr() as *const _, gl::STATIC_DRAW);

            gl::CreateVertexArrays(1, &mut self.vao);
            gl::VertexArrayVertexBuffer(self.vao, 0, self.vbo, 0, std::mem::size_of::<Vertex>() as i32);
            gl::VertexArrayElementBuffer(self.vao, self.ebo);

            //Enable vertex attributes (location = ?)
            gl::EnableVertexArrayAttrib(self.vao, 0);
            gl::EnableVertexArrayAttrib(self.vao, 1);
            gl::EnableVertexArrayAttrib(self.vao, 2);

            gl::VertexArrayAttribFormat(self.vao, 0, 3, gl::FLOAT, gl::FALSE, core::mem::offset_of!(Vertex, position) as u32);
            gl::VertexArrayAttribFormat(self.vao, 1, 3, gl::FLOAT, gl::FALSE, core::mem::offset_of!(Vertex, normal) as u32);
            gl::VertexArrayAttribFormat(self.vao, 2, 2, gl::FLOAT, gl::FALSE, core::mem::offset_of!(Vertex, tex_coords) as u32);

            gl::VertexArrayAttribBinding(self.vao, 0, 0);
            gl::VertexArrayAttribBinding(self.vao, 1, 0);
            gl::VertexArrayAttribBinding(self.vao, 2, 0);

            // Logs::debug("Mesh created with self.vbo: " + std::to_string(self.vbo) + ", self.ebo: " + std::to_string(self.ebo) + ", self.vao: " + std::to_string(self.vao));
            // check_opengl::error("mesh");
        }
    }

    pub unsafe fn draw(&mut self, shader: &Shader) {
        unsafe {
            // render the mesh
            // bind appropriate textures
            for (i, texture) in self.textures.iter().enumerate() {
                gl::ActiveTexture(texture.get_code()); // active proper texture unit before binding
                // set the sampler to the texture unit, then bind the texture
                shader.set_int(texture.get_t_type().as_str(), i as i32);
                gl::BindTexture(gl::TEXTURE_2D, texture.get_texture());
            }

            // draw mesh
            gl::BindVertexArray(self.vao);
            gl::DrawElementsBaseVertex(gl::TRIANGLES, self.indices.len() as GLsizei, gl::UNSIGNED_INT, std::ptr::null(), 0);
            gl::BindVertexArray(0);

            // always good practice to set everything back to defaults once configured.
            gl::ActiveTexture(gl::TEXTURE0);
        }
    }

    pub unsafe fn load_textures(&mut self, filename: String, t_code: u32, name: String) {
        unsafe {
            let texture: Texture = Texture::new(filename, t_code, name).unwrap();
            self.textures.push(texture);
        }
    }

    pub unsafe fn init_textures(&self) {
        unsafe {
            // set the texture wrapping/filtering options (on the currently bound texture object)
            gl::TextureParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TextureParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);

            // Note: You had REPEAT for filters in your snippet.
            // Usually, you want NEAREST or LINEAR for filters.
            gl::TextureParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TextureParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        }
    }
}
impl Drop for Mesh {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteBuffers(1, &self.ebo);
            for texture in self.textures.iter() {
                gl::DeleteTextures(1, &texture.get_texture());
            }
        }
    }
}
