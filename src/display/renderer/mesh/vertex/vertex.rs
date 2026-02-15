use std::io;
use glam::IVec3;

pub struct Vertex {
    pub position: glam::Vec3,
    pub normal: glam::Vec3,
    pub tex_coords: glam::IVec2,
}

impl Vertex {

    pub fn new(position: glam::Vec3, normal : glam::Vec3, tex_coords: glam::IVec2) -> Vertex {
        Self { position, normal, tex_coords }
    }

    pub fn pack_data(id : u16, pos: glam::IVec3, face_id : u32, tex_coords : u8) -> io::Result<u64> {
        // println!("Input : {}, {}, {}",pos, face_id, tex_coords);
        if id as u32 >= 2_u32.pow(18) {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Vertex ID exceeds maximum value of 2^18 - 1".to_string()));
        }
        if pos.x >= 2_i32.pow(7) || pos.y >= 2_i32.pow(7) || pos.z >= 2_i32.pow(7) {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Vertex position exceeds maximum value of 2^7 - 1".to_string()));
        }
        if face_id > 5 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Face id has to be between 0 and 5".to_string()));
        }
        let mut packed : u64 = 0;
        packed |= (id as u64 & 0x3FFFF) << 46; // ID (18 bits)
        packed |= (pos.x as u64 & 0x7F) << 39; // position X (7 bits)
        packed |= (pos.y as u64 & 0x7F) << 32; // position Y (7 bits)
        packed |= (pos.z as u64 & 0x7F) << 25; // position Z (7 bits)
        packed |= (face_id as u64 & 0x7) << 18; // Face ID (3 bits)
        packed |= ((tex_coords as u64 >> 1) & 0x1) << 1; // TexCoord X (1 bit)
        packed |= tex_coords as u64 & 0x1; // TexCoord Y (1 bit)
        Ok(packed)
    }
}