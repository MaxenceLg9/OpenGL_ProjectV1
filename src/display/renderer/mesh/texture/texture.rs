use std::io;
use gl::{RGBA, TEXTURE0};
use gl::types::GLuint;
use image::imageops::FilterType;
use image::ImageReader;
use crate::display::renderer::mesh::shader::shader::Shader;
use crate::display::renderer::mesh::texture::texture_array::{TEXTURE_ARRAY_SIZE, TEXTURE_SIZE};

pub struct Texture {
    t_type: String,
    code: u32,
    texture: GLuint,
}

impl Texture {
    pub unsafe fn new(filename : String, code: u32, t_type: String) -> io::Result<Texture> {

        let mut texture: u32 = 0;

        gl::CreateTextures(gl::TEXTURE_2D, 1, &mut texture);

        // 2. Set parameters directly on the texture ID (No binding required!)
        gl::TextureParameteri(texture, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl::TextureParameteri(texture, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);

        // Note: You had REPEAT for filters in your snippet.
        // Usually, you want NEAREST or LINEAR for filters.
        gl::TextureParameteri(texture, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TextureParameteri(texture, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

        let mut image = match ImageReader::open(filename)?.decode() {
            Ok(f) => f,
            Err(e) => return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Error decoding image: {}", e)))
        };
        if image.width() != image.height() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Image dimensions do not match"))
        }
        if (image.width() != TEXTURE_SIZE as u32) || (image.height() != TEXTURE_SIZE as u32) {
            image = image.resize(TEXTURE_SIZE as u32, TEXTURE_SIZE as u32, FilterType::Nearest);
        }

        gl::TextureParameteri(texture, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl::TextureParameteri(texture, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);

        // Note: You had REPEAT for filters in your snippet.
        // Usually, you want NEAREST or LINEAR for filters.
        gl::TextureParameteri(texture, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TextureParameteri(texture, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

        gl::TextureStorage2D(texture,
                             1,// Mipmap levels
                             gl::RGBA8,       // Internal format
                             TEXTURE_SIZE,    // Width
                             TEXTURE_SIZE,    // Height
        );

        // let data:  = PixelUnpackData::Slice(Some(image.as_bytes()));
        gl::TexSubImage2D(texture, 0, gl::RGBA8 as i32, TEXTURE_SIZE, TEXTURE_SIZE, 0, gl::RGBA, gl::UNSIGNED_BYTE, image.as_bytes().as_ptr() as *const _);

        Result::Ok(Self {
            t_type,
            code,
            texture,
        })
    }

    pub unsafe fn use_texture(&self, shader: &Shader) {
        gl::ActiveTexture(TEXTURE0 + self.code);
        shader.set_int(self.t_type.clone(), self.code as i32);
        gl::BindTexture(gl::TEXTURE_2D, self.texture);
    }

    pub fn get_code(&self) -> GLuint {
        self.code
    }
    pub fn get_t_type(&self) -> String {
        self.t_type.clone()
    }

    pub fn get_texture(&self) -> GLuint {
        self.texture
    }


}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.texture);
        }
    }
}
