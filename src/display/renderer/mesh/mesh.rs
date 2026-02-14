//
// Created by Sinis on 08/05/2025.
//

use crate::display::renderer::mesh::texture::texture::Texture;
use crate::display::renderer::mesh::vertex::vertex::Vertex;

pub struct Mesh {
    vao: i32,
    vbo: i32,
    ebo: i32,
    vertices : Vec<Vertex>,
    indices : Vec<u32>,
    textures: Vec<Texture>
}