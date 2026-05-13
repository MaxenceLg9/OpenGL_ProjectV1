use std::collections::HashMap;
use std::ffi::CString;
use std::ptr::null;
use bitvec::macros::internal::funty::Fundamental;
use gl::types::{GLsizei, GLsizeiptr, GLuint};
use shared::print_base;
use crate::client::display::renderer::mesh::shader::shader::Shader;

pub struct Character {
    texture_id: GLuint,// ID handle of the glyph texture
    size: glam::IVec2,       // size of glyph
    bearing: glam::IVec2,    // Offset from baseline to left/top of glyph
    advance: u32    // Offset to advance to next glyph
}

impl Character {
    pub fn new(texture_id: GLuint,// ID handle of the glyph texture
                   size: glam::IVec2,       // size of glyph
                   bearing: glam::IVec2,    // Offset from baseline to left/top of glyph
                   advance: u32) -> Self {
        Self {
            texture_id,// ID handle of the glyph texture
            size,       // size of glyph
            bearing,    // Offset from baseline to left/top of glyph
            advance
        }
    }
}

pub struct Text {
    vao : u32,
    vbo : u32,
}


impl Text {
    pub unsafe fn new() -> Text {
        let mut vao = 0;
        let mut vbo = 0;
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(gl::ARRAY_BUFFER, (size_of::<f32>() * 6 * 4) as GLsizeiptr, null(), gl::DYNAMIC_DRAW);
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 4, gl::FLOAT, gl::FALSE, 4 * size_of::<f32>() as GLsizei, null());
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);
        Self {
            vao,
            vbo
        }
    }
    pub unsafe fn render_text(&self, s : &Shader, text : &str, mut x: f32, y : f32, scale : f32, color : glam::Vec3, characters : &HashMap<char,Character>) {
        // activate corresponding render state
        // print_base!("Before");

        s.use_shader();

        let projection = glam::Mat4::orthographic_rh_gl(0.0, 1920.0, 0.0, 1080.0, -1.0, 1.0);

        s.set_matrix4fv("projection", projection);
        gl::Uniform3f(gl::GetUniformLocation(s.program, CString::new("textColor").unwrap().as_bytes().as_ptr() as *const i8), color.x, color.y, color.z);

        // print_base!("After");
        gl::DepthFunc(gl::ALWAYS);

        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindVertexArray(self.vao);

        // iterate through all characters
        for c in text.chars() {
            let ch = characters.get(&c).unwrap_or_else(|| &characters[&'?']);

            // Calculate bottom-left of the glyph quad
            let xpos = x + ch.bearing.x as f32 * scale;
            let ypos = y + ch.bearing.y as f32 * scale; // Add bearing in Y-up

            let w = ch.size.x as f32 * scale;
            let h = ch.size.y as f32 * scale;

            // Flipped UVs:
            // V=1.0 is the top of the texture (which is the first byte of fontdue data)
            // V=0.0 is the bottom of the texture (which is the last byte of fontdue data)
            let vertices: [[f32; 4]; 6] = [
                [ xpos + w, ypos,       1.0, 1.0 ], // Bottom-Right
                [ xpos,     ypos,       0.0, 1.0 ], // Bottom-Left
                [ xpos,     ypos + h,   0.0, 0.0 ], // Top-Left

                [ xpos + w, ypos + h,   1.0, 0.0 ], // Top-Right
                [ xpos + w, ypos,       1.0, 1.0 ], // Bottom-Right
                [ xpos,     ypos + h,   0.0, 0.0 ], // Top-Left
            ];

            gl::BindTexture(gl::TEXTURE_2D, ch.texture_id);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferSubData(gl::ARRAY_BUFFER, 0, (vertices.len() * 16) as isize, vertices.as_ptr() as *const _);

            gl::DrawArrays(gl::TRIANGLES, 0, 6);

            x += ch.advance as f32 * scale;
        }
        gl::BindVertexArray(0);
        gl::BindTexture(gl::TEXTURE_2D, 0);
    }
}
