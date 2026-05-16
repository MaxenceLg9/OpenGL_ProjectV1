use winit::window::Window;
use shared::common::display::vertex::vertex::Vertex;
use crate::client::display::renderer::mesh::mesh::Mesh;
use crate::client::display::renderer::mesh::shader::shader::Shader;

pub struct Cursor {
    shader: Shader,
    mesh: Mesh
}


impl Cursor {
    pub unsafe fn new() -> Cursor {
        let shader = Shader::new("/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/shaders/cursor/vertex.vert", "/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/shaders/cursor/fragment.frag");
        let mesh = Mesh::new(Cursor::vertices(),Cursor::indices(),Vec::new());
        Self {
            shader,
            mesh
        }
    }


    pub fn vertices() -> Vec<Vertex> {
        let mut vertices = Vec::new();
        //vertical
        vertices.push(Vertex::new(glam::vec3(-0.003, 0.02, 0.0), glam::vec3(1.0, 1.0, 1.0), glam::ivec2(0, 1)));
        vertices.push(Vertex::new(glam::vec3(0.003, 0.02, 0.0), glam::vec3(1.0, 1.0, 1.0), glam::ivec2(1, 1)));
        vertices.push(Vertex::new(glam::vec3(0.003, -0.02, 0.0), glam::vec3(0.0, 0.0, 0.0), glam::ivec2(1, 0)));
        vertices.push(Vertex::new(glam::vec3(-0.003, -0.02, 0.0), glam::vec3(0.0, 0.0, 0.0), glam::ivec2(0, 0)));

        //horizontal
        vertices.push(Vertex::new(glam::vec3(-0.02, 0.003, -0.0), glam::vec3(0.0, 0.0, 0.0), glam::ivec2(0, 1)));
        vertices.push(Vertex::new(glam::vec3(0.02, 0.003, -0.0), glam::vec3(1.0, 1.0, 1.0), glam::ivec2(1, 1)));
        vertices.push(Vertex::new(glam::vec3(0.02, -0.003, -0.0), glam::vec3(1.0, 1.0, 1.0), glam::ivec2(1, 0)));
        vertices.push(Vertex::new(glam::vec3(-0.02, -0.003, -0.0), glam::vec3(0.0, 0.0, 0.0), glam::ivec2(0, 0)));

        vertices
    }

    pub fn indices() -> Vec<u32> {
        let mut indices = Vec::new();

        //vertical
        indices.push(0);
        indices.push(1);
        indices.push(2);
        indices.push(0);
        indices.push(2);
        indices.push(3);

        //horizontal
        indices.push(4);
        indices.push(5);
        indices.push(6);
        indices.push(4);
        indices.push(6);
        indices.push(7);

        indices
    }


    pub unsafe fn draw_cursor(&mut self, window: &Window){
        self.shader.use_shader();
        let aspect = window.inner_size().width as f32 / window.inner_size().height as f32;
        let projection = glam::Mat4::orthographic_rh(
            -1.0,           // left
            1.0,            // right
            -1.0 / aspect,  // bottom
            1.0 / aspect,   // top
            -1.0,           // near
            1.0             // far
        );

        // 2. Upload to the shader
        // We use .to_cols_array_2d() or .as_ref() depending on your GL wrapper
        self.shader.set_matrix4fv("projection", projection);
        gl::DepthFunc(gl::ALWAYS); // Always pass the depth test (same effect as glDisable(GL_DEPTH_TEST))
        self.mesh.draw(&self.shader);
    }
}
