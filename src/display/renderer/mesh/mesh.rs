//
// Created by Sinis on 08/05/2025.
//

use gl::types::{GLsizei, GLuint};
use glutin_winit::finalize_window;
use crate::display::renderer::mesh::shader::shader::Shader;
use crate::display::renderer::mesh::texture::texture::Texture;
use crate::display::renderer::mesh::vertex::vertex::Vertex;

pub struct Mesh {
    vao : GLuint,
    vbo : GLuint,
    ebo : GLuint,
    vertices : Vec<Vertex>,
    indices : Vec<u32>,
    textures: Vec<Texture>
}

impl Mesh {
    pub unsafe fn new(vertices: Vec<Vertex>, indices: Vec<u32>, textures: Vec<Texture>) -> Mesh {
        let (vao, vbo, ebo) = (0, 0, 0);
        let mut mesh = Self {
            vao,
            vbo,
            ebo,
            vertices,
            indices,
            textures
        };
        mesh.setup_mesh();
        mesh
    }

    pub unsafe fn setup_mesh(&mut self) {
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

    pub unsafe fn draw(&mut self, shader: &Shader) {     // render the mesh
        // bind appropriate textures
        let diffuse_nr = 1;
        let specular_nr = 1;
        let normal_nr = 1;
        let height_nr = 1;
        for i in 0..self.textures.len() {
            gl::ActiveTexture(self.textures.get(i).unwrap().get_code()); // active proper texture unit before binding
            // retrieve texture number (the N in diffuse_textureN)
            let number: String;
            let name = self.textures[i].get_t_type();
            if (name == "texture_diffuse") {
                number = (diffuse_nr + 1).to_string();
            } else if (name == "texture_specular") {
                number = (specular_nr + 1).to_string(); // transfer unsigned int to string
            } else if (name == "texture_normal") {
                number = (normal_nr + 1).to_string(); // transfer unsigned int to string
            } else if (name == "texture_height") {
                number = (height_nr + 1).to_string(); // transfer unsigned int to string
            }

            // now set the sampler to the correct texture unit
            shader.set_int(name.as_str(), i as i32);
            // and finally bind the texture
            gl::BindTexture(gl::TEXTURE_2D, self.textures[i].get_texture());
        }

        // draw mesh
        gl::BindVertexArray(self.vao);
        gl::DrawElementsBaseVertex(gl::TRIANGLES, self.indices.len() as GLsizei, gl::UNSIGNED_INT, std::ptr::null(), 0);
        gl::BindVertexArray(0);


        // always good practice to set everything back to defaults once configured.
        gl::ActiveTexture(gl::TEXTURE0);
    }

    pub unsafe fn load_textures(&mut self, filename : String, t_code: u32, name : String) {
        let texture: Texture = Texture::new(filename, t_code, name).unwrap();
        self.textures.push(texture);
    }

    pub unsafe fn init_textures(&self) {
        // set the texture wrapping/filtering options (on the currently bound texture object)
        gl::TextureParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl::TextureParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);

        // Note: You had REPEAT for filters in your snippet.
        // Usually, you want NEAREST or LINEAR for filters.
        gl::TextureParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TextureParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
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