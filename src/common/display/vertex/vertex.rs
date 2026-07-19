use std::io;

pub struct Vertex {
    pub position: glam::Vec3,
    pub normal: glam::Vec3,
    pub tex_coords: glam::IVec2,
}

impl Vertex {

    pub fn new(position: glam::Vec3, normal : glam::Vec3, tex_coords: glam::IVec2) -> Vertex {
        Self { position, normal, tex_coords }
    }
}