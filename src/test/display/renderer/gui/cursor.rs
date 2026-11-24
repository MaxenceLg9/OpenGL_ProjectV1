use shared::common::display::vertex::vertex::Vertex;
use winit::window::Window;

use crate::test::display::renderer::mesh::{mesh::Mesh, shader::shader::Shader};

pub struct Cursor {
    shader: Shader,
    mesh: Mesh,
}

impl Cursor {
    pub unsafe fn new() -> Cursor {
        unsafe {
            let shader = Shader::new("src/assets/shaders/cursor/vertex.vert", "src/assets/shaders/cursor/fragment.frag");
            let mesh = Mesh::new(Cursor::vertices(), Cursor::indices(), Vec::new());
            Self { shader, mesh }
        }
    }

    pub fn vertices() -> Vec<Vertex> {
        vec![
            //vertical
            Vertex::new(glam::vec3(-0.003, 0.02, 0.0), glam::vec3(1.0, 1.0, 1.0), glam::ivec2(0, 1)),
            Vertex::new(glam::vec3(0.003, 0.02, 0.0), glam::vec3(1.0, 1.0, 1.0), glam::ivec2(1, 1)),
            Vertex::new(glam::vec3(0.003, -0.02, 0.0), glam::vec3(0.0, 0.0, 0.0), glam::ivec2(1, 0)),
            Vertex::new(glam::vec3(-0.003, -0.02, 0.0), glam::vec3(0.0, 0.0, 0.0), glam::ivec2(0, 0)),
            //horizontal
            Vertex::new(glam::vec3(-0.02, 0.003, -0.0), glam::vec3(0.0, 0.0, 0.0), glam::ivec2(0, 1)),
            Vertex::new(glam::vec3(0.02, 0.003, -0.0), glam::vec3(1.0, 1.0, 1.0), glam::ivec2(1, 1)),
            Vertex::new(glam::vec3(0.02, -0.003, -0.0), glam::vec3(1.0, 1.0, 1.0), glam::ivec2(1, 0)),
            Vertex::new(glam::vec3(-0.02, -0.003, -0.0), glam::vec3(0.0, 0.0, 0.0), glam::ivec2(0, 0)),
        ]
    }

    pub fn indices() -> Vec<u32> {
        vec![
            //vertical
            0, 1, 2, 0, 2, 3, //horizontal
            4, 5, 6, 4, 6, 7,
        ]
    }

    pub unsafe fn draw_cursor(&mut self, window: &Window) {
        unsafe {
            self.shader.use_shader();
            let aspect = window.inner_size().width as f32 / window.inner_size().height as f32;
            let projection = glam::Mat4::orthographic_rh(
                -1.0,          // left
                1.0,           // right
                -1.0 / aspect, // bottom
                1.0 / aspect,  // top
                -1.0,          // near
                1.0,           // far
            );

            // 2. Upload to the shader
            // We use .to_cols_array_2d() or .as_ref() depending on your GL wrapper
            self.shader.set_matrix4fv("projection", projection);
            gl::DepthFunc(gl::ALWAYS); // Always pass the depth test (same effect as glDisable(GL_DEPTH_TEST))
            self.mesh.draw(&self.shader);
        }
    }
}
