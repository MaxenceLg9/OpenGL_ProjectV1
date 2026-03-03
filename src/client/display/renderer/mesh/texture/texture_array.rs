use std::io;
use std::sync::Arc;
use gl::types::{GLuint, GLvoid};
use image::{ImageReader};
use image::imageops::FilterType;
use crate::client::display::renderer::mesh::shader::shader::Shader;

pub const TEXTURE_ARRAY_SIZE : i32 = 256;
pub const TEXTURE_SIZE : i32 = 64;
pub struct TextureArray {
    texture : GLuint,
    name: String
}

impl Drop for TextureArray {
    fn drop(&mut self) {

    }
}
impl TextureArray {
    pub unsafe fn new(name: String) -> TextureArray {
        let mut texture: u32 = 0;

        // 1. Create the texture object specifically as a 2D_ARRAY
        gl::CreateTextures(gl::TEXTURE_2D_ARRAY, 1, &mut texture);

        // 2. Set parameters directly on the texture ID (No binding required!)
        gl::TextureParameteri(texture, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl::TextureParameteri(texture, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);

        // Note: You had REPEAT for filters in your snippet.
        // Usually, you want NEAREST or LINEAR for filters.
        gl::TextureParameteri(texture, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TextureParameteri(texture, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

        // 3. Allocate immutable storage for the array
        gl::TextureStorage3D(texture,
                             1,// Mipmap levels
                             gl::RGBA8,       // Internal format
                             TEXTURE_SIZE,    // Width
                             TEXTURE_SIZE,    // Height
                             TEXTURE_ARRAY_SIZE // Depth (Number of layers)
        );

        Self { texture, name }
    }

    pub unsafe fn add_texture(&self, filename : &str, index : u16) -> io::Result<()> {
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
        gl::TextureParameteri(self.texture, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl::TextureParameteri(self.texture, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);

        // Note: You had REPEAT for filters in your snippet.
        // Usually, you want NEAREST or LINEAR for filters.
        gl::TextureParameteri(self.texture, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TextureParameteri(self.texture, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

        // let data:  = PixelUnpackData::Slice(Some(image.as_bytes()));
        gl::TextureSubImage3D(self.texture, 0, 0, 0, index as i32, TEXTURE_SIZE, TEXTURE_SIZE, 1, gl::RGBA, gl::UNSIGNED_BYTE, image.to_rgba8().as_raw().as_ptr() as *const GLvoid);
        Ok(())
    }
    //
    pub unsafe fn use_textures(&self,shader : &Shader) {
        gl::ActiveTexture(gl::TEXTURE0);
        shader.set_int(self.name.as_str(), 0);
        gl::BindTexture(gl::TEXTURE_2D_ARRAY, self.texture);
    }
}