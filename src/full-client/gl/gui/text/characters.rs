use gl::types::{GLsizei, GLsizeiptr, GLuint};
use shared::common::error::err_type::{CustomErrorKind, GameError};
use shared::custom_error;
use std::collections::HashMap;
use std::ffi::CString;
use std::ptr::null;
use std::sync::OnceLock;

use crate::client::gl::mesh::shader::shader::Shader;

const FONT_PATH: &str = "src/assets/fonts/IBM_Plex_Mono/IBMPlexMono-Bold.ttf";
/// Size the glyphs are rasterized at; `scale` in `render_text` is relative to it.
pub const FONT_PX_SIZE: f32 = 32.0;
/// Virtual screen used by `Text::render_text` (the in-game HUD is laid out for it).
pub const HUD_SCREEN: glam::Vec2 = glam::vec2(1920.0, 1080.0);

/// ASCII glyphs, loaded once by `load_characters`.
pub static CHARACTERS: OnceLock<HashMap<char, Character>> = OnceLock::new();

/// Rasterizes the 128 ASCII characters into one texture each. Needs a current GL context.
/// Calling it again does nothing.
pub unsafe fn load_characters() -> Result<(), GameError> {
    if CHARACTERS.get().is_some() {
        return Ok(());
    }
    let font_data = std::fs::read(FONT_PATH).map_err(|e| custom_error!(CustomErrorKind::Io, format!("Cannot read font {}: {}", FONT_PATH, e)))?;
    let font = fontdue::Font::from_bytes(font_data, fontdue::FontSettings::default()).map_err(|e| custom_error!(CustomErrorKind::Other, format!("Cannot parse font {}: {}", FONT_PATH, e)))?;

    let mut characters = HashMap::new();
    unsafe {
        // glyph bitmaps are 1 byte per pixel, rows are not 4-byte aligned
        gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
    }
    for chr in (0..128u8).map(char::from) {
        let (metrics, bitmap) = font.rasterize(chr, FONT_PX_SIZE);
        let mut texture_id = 0;
        unsafe {
            gl::GenTextures(1, &mut texture_id);
            gl::BindTexture(gl::TEXTURE_2D, texture_id);
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RED as i32, metrics.width as i32, metrics.height as i32, 0, gl::RED, gl::UNSIGNED_BYTE, bitmap.as_ptr() as *const _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        }
        let character = Character::new(texture_id, glam::ivec2(metrics.width as i32, metrics.height as i32), glam::ivec2(metrics.xmin, metrics.ymin), metrics.advance_width as u32);
        characters.insert(chr, character);
    }
    unsafe {
        gl::BindTexture(gl::TEXTURE_2D, 0);
    }
    let _ = CHARACTERS.set(characters);
    Ok(())
}

fn glyph(c: char) -> &'static Character {
    let characters = CHARACTERS.get().expect("load_characters must be called before rendering text");
    characters.get(&c).unwrap_or_else(|| &characters[&'?'])
}

pub struct Character {
    texture_id: GLuint,   // ID handle of the glyph texture
    size: glam::IVec2,    // size of glyph
    bearing: glam::IVec2, // Offset from baseline to left/top of glyph
    advance: u32,         // Offset to advance to next glyph
}

impl Character {
    pub fn new(
        texture_id: GLuint,   // ID handle of the glyph texture
        size: glam::IVec2,    // size of glyph
        bearing: glam::IVec2, // Offset from baseline to left/top of glyph
        advance: u32,
    ) -> Self {
        Self {
            texture_id, // ID handle of the glyph texture
            size,       // size of glyph
            bearing,    // Offset from baseline to left/top of glyph
            advance,
        }
    }
}

pub struct Text {
    vao: u32,
    vbo: u32,
}

impl Text {
    pub unsafe fn new() -> Text {
        let mut vao = 0;
        let mut vbo = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(gl::ARRAY_BUFFER, (size_of::<f32>() * 6 * 4) as GLsizeiptr, null(), gl::DYNAMIC_DRAW);
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 4, gl::FLOAT, gl::FALSE, 4 * size_of::<f32>() as GLsizei, null());
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }
        Self { vao, vbo }
    }
    /// Width in pixels of the rendered text.
    pub fn text_width(text: &str, scale: f32) -> f32 {
        text.chars().map(|c| glyph(c).advance as f32 * scale).sum()
    }

    /// Renders text on the 1920x1080 HUD virtual screen. (x, y) is the bottom-left of the baseline.
    pub unsafe fn render_text(&self, s: &Shader, text: &str, x: f32, y: f32, scale: f32, color: glam::Vec3) {
        unsafe { self.render_text_on(s, text, x, y, scale, color, HUD_SCREEN) }
    }

    /// Renders text on a screen of `screen` pixels, origin at the bottom-left.
    pub unsafe fn render_text_on(&self, s: &Shader, text: &str, mut x: f32, y: f32, scale: f32, color: glam::Vec3, screen: glam::Vec2) {
        unsafe {
            s.use_shader();

            let projection = glam::Mat4::orthographic_rh_gl(0.0, screen.x, 0.0, screen.y, -1.0, 1.0);
            s.set_matrix4fv("projection", projection);
            gl::Uniform3f(gl::GetUniformLocation(s.program, CString::new("textColor").unwrap().as_bytes().as_ptr() as *const i8), color.x, color.y, color.z);

            // print_base!("After");
            gl::DepthFunc(gl::ALWAYS);

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindVertexArray(self.vao);
        }

        // iterate through all characters
        for c in text.chars() {
            let ch = glyph(c);

            // Calculate bottom-left of the glyph quad
            let xpos = x + ch.bearing.x as f32 * scale;
            let ypos = y + ch.bearing.y as f32 * scale; // Add bearing in Y-up

            let w = ch.size.x as f32 * scale;
            let h = ch.size.y as f32 * scale;

            // Flipped UVs:
            // V=1.0 is the top of the texture (which is the first byte of fontdue data)
            // V=0.0 is the bottom of the texture (which is the last byte of fontdue data)
            let vertices: [[f32; 4]; 6] = [
                [xpos + w, ypos, 1.0, 1.0],     // Bottom-Right
                [xpos, ypos, 0.0, 1.0],         // Bottom-Left
                [xpos, ypos + h, 0.0, 0.0],     // Top-Left
                [xpos + w, ypos + h, 1.0, 0.0], // Top-Right
                [xpos + w, ypos, 1.0, 1.0],     // Bottom-Right
                [xpos, ypos + h, 0.0, 0.0],     // Top-Left
            ];
            unsafe {
                gl::BindTexture(gl::TEXTURE_2D, ch.texture_id);
                gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
                gl::BufferSubData(gl::ARRAY_BUFFER, 0, (vertices.len() * 16) as isize, vertices.as_ptr() as *const _);

                gl::DrawArrays(gl::TRIANGLES, 0, 6);
            }
            x += ch.advance as f32 * scale;
        }
        unsafe {
            gl::BindVertexArray(0);
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }
}
