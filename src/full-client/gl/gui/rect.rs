use crate::client::gl::mesh::shader::shader::Shader;
use gl::types::GLuint;
use shared::common::error::err_type::GameError;
use shared::common::file::path::AssetsReader;

/// Axis aligned rectangle in pixels, origin at the bottom-left of the screen.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    /// Rectangle of the given size centered on (cx, cy).
    pub fn centered(cx: f32, cy: f32, width: f32, height: f32) -> Self {
        Self::new(cx - width / 2.0, cy - height / 2.0, width, height)
    }

    pub fn contains(&self, point: glam::Vec2) -> bool {
        point.x >= self.x && point.x <= self.x + self.width && point.y >= self.y && point.y <= self.y + self.height
    }

    pub fn center(&self) -> glam::Vec2 {
        glam::vec2(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }
}

/// Draws flat colored rectangles: one unit quad, moved and scaled by the shader.
pub struct RectRenderer {
    shader: Shader,
    vao: GLuint,
    vbo: GLuint,
}

impl RectRenderer {
    pub unsafe fn new() -> Result<Self, GameError> {
        unsafe {
            let shader = Shader::new(AssetsReader::new("shaders/ui/vertex.vert"), AssetsReader::new("shaders/ui/fragment.frag"))?;
            // two triangles covering (0,0)-(1,1), clockwise like the rest of the renderer
            let vertices: [f32; 12] = [0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0];
            let (mut vao, mut vbo) = (0, 0);
            gl::CreateBuffers(1, &mut vbo);
            gl::NamedBufferData(vbo, size_of_val(&vertices) as isize, vertices.as_ptr() as *const _, gl::STATIC_DRAW);
            gl::CreateVertexArrays(1, &mut vao);
            gl::VertexArrayVertexBuffer(vao, 0, vbo, 0, 2 * size_of::<f32>() as i32);
            gl::EnableVertexArrayAttrib(vao, 0);
            gl::VertexArrayAttribFormat(vao, 0, 2, gl::FLOAT, gl::FALSE, 0);
            gl::VertexArrayAttribBinding(vao, 0, 0);
            Ok(Self { shader, vao, vbo })
        }
    }

    /// Draws `rect` filled with `color` (RGBA, alpha blended) on a screen of `screen` pixels.
    pub unsafe fn draw(&self, rect: Rect, color: glam::Vec4, screen: glam::Vec2) {
        unsafe {
            self.shader.use_shader();
            self.shader
                .set_matrix4fv("projection", glam::Mat4::orthographic_rh_gl(0.0, screen.x, 0.0, screen.y, -1.0, 1.0));
            self.shader.set_vec4("rect", glam::vec4(rect.x, rect.y, rect.width, rect.height));
            self.shader.set_vec4("color", color);
            gl::BindVertexArray(self.vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
            gl::BindVertexArray(0);
        }
    }
}

impl Drop for RectRenderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.vbo);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_includes_the_borders() {
        let rect = Rect::new(10.0, 20.0, 100.0, 50.0);
        assert!(rect.contains(glam::vec2(10.0, 20.0)));
        assert!(rect.contains(glam::vec2(110.0, 70.0)));
        assert!(rect.contains(glam::vec2(60.0, 45.0)));
        assert!(!rect.contains(glam::vec2(9.9, 45.0)));
        assert!(!rect.contains(glam::vec2(60.0, 70.1)));
    }

    #[test]
    fn centered_rect_is_centered() {
        let rect = Rect::centered(100.0, 50.0, 40.0, 20.0);
        assert_eq!(rect, Rect::new(80.0, 40.0, 40.0, 20.0));
        assert_eq!(rect.center(), glam::vec2(100.0, 50.0));
    }
}
