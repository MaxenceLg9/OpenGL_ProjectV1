use std::collections::HashMap;
use std::sync::Arc;
use std::vec::Vec;
use gl::types::{GLuint};
use gl::{TRIANGLES, UNSIGNED_INT};
use glam::*;
use shared::common::world::pos::chunkpos::{ChunkPos, CHUNK_SIZE};
use shared::common::world::pos::iblockpos::IBlockPos;
use shared::print_debug;
use crate::client::display::renderer::mesh::shader::shader::Shader;
use shared::print_base;
use std::io;
use strum::{Display, FromRepr};
use crate::client::display::renderer::gui::text::text::MeshText;

#[derive(Clone)]
pub struct Mesh {
    vao : GLuint,
    nb_indices: i32,
}


impl Mesh {
    pub fn new(vao : GLuint, vbo : GLuint, ebo : GLuint, nb_indices : i32) -> Mesh {
        Self {
            vao,
            nb_indices,
        }
    }




    pub unsafe fn draw(&self, chunk_shader : &Shader, pos : &ChunkPos) {
        if self.nb_indices == 0 {
            return;
        }
        let mut model = glam::Mat4::IDENTITY;
        model = model * glam::Mat4::from_translation(pos.as_vec3() * CHUNK_SIZE as f32);
        chunk_shader.set_matrix4fv("uniform_model", model);
        gl::BindVertexArray(self.vao);
        gl::DrawElementsBaseVertex(TRIANGLES,self.nb_indices,UNSIGNED_INT, std::ptr::null(),0);
    }
}



pub struct ChunkMesh {
    vertices: Vec<u32>,
    indices: Vec<u32>,
    chunk_pos: ChunkPos
}
#[derive(Clone, Copy, Debug, PartialEq, FromRepr, Display)]
enum Plane{
    X,Y,Z
}


impl ChunkMesh {

    pub fn get_chunk_pos(&self) -> ChunkPos {
        self.chunk_pos
    }

    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty() && self.indices.is_empty()
    }

    pub unsafe fn link(&mut self) -> Mesh {
        let (mut vao, mut vbo, mut ebo) = (0,0,0);
        Self::setup_mesh(&mut vao,&mut vbo,&mut ebo);
        self.bind_data(&mut vbo,&mut ebo);
        Mesh::new(vao,vbo,ebo, self.indices.len() as i32)
    }



    unsafe fn setup_mesh(vao: &mut GLuint, vbo: &mut GLuint, ebo: &mut GLuint) {

        // 1. Create the objects (DSA style)
        gl::CreateBuffers(1, vbo);
        gl::CreateBuffers(1, ebo);
        gl::CreateVertexArrays(1, vao);

        // 2. Configure Attribute 0
        gl::VertexArrayVertexBuffer(*vao, 0, *vbo, 0, 8);

        gl::EnableVertexArrayAttrib(*vao, 0);
        gl::EnableVertexArrayAttrib(*vao, 1);

        // 3. Configure Attribute 1
        gl::VertexArrayAttribIFormat(*vao, 0, 1, UNSIGNED_INT, 0);
        gl::VertexArrayAttribIFormat(*vao, 1, 1, UNSIGNED_INT, 4);

        gl::VertexArrayAttribBinding(*vao,0,0);
        gl::VertexArrayAttribBinding(*vao,1,0);

        gl::VertexArrayElementBuffer(*vao, *ebo);

    }

    unsafe fn bind_data(&self, vbo: &mut GLuint, ebo: &mut GLuint) {
        gl::NamedBufferData(*vbo, self.vertices.len().cast_signed() * 4, self.vertices.as_ptr() as *const _ , gl::STATIC_DRAW);
        gl::NamedBufferData(*ebo, self.indices.len().cast_signed() * 4, self.indices.as_ptr() as *const _, gl::STATIC_DRAW);
    }

    /// Construct the chunk mesh and text meshes from chunks
    pub fn build_mesh(blocks: Arc<Vec<u16>>, chunk_pos: ChunkPos) -> Option<(ChunkMesh, MeshText)> {
        let mut chunk_mesh = Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            chunk_pos
        };
        chunk_mesh.vertices.reserve(CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE * 6 * 4 * 2); // 6 faces, 4 vertices per face
        chunk_mesh.indices.reserve(CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE * 6 * 6); // 6 faces, 6 nbIndices per face
        let mut index = 0;
        let materials = [1.0,1.0,1.0];
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE{
                    let voxel_id : u16 = blocks[x * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + z];

                    if voxel_id == 0 {
                        continue; // skip empty blocks
                    }
                    let mut v: [u64;4] = [0; 4];
                    let (mut ao, mut flip_id);
                    let (ux, uy, uz) = (x as i32, y as i32, z as i32);
                    //front face
                    if chunk_mesh.is_void(IBlockPos::new(ux, uy, uz + 1), &blocks, chunk_pos)? {
                        ao = chunk_mesh.get_ao(IBlockPos::new(ux, uy, uz + 1), Plane::Z, &blocks, chunk_pos)?;

                        flip_id = ao[1] + ao[3] > ao[0] + ao[2];

                        v[0] = Self::pack_data(voxel_id, glam::IVec3::new(ux, uy, uz + 1), 1, 3, materials, ao[0]).unwrap();
                        v[1] = Self::pack_data(voxel_id, glam::IVec3::new(ux, uy + 1, uz + 1), 1, 2, materials, ao[1]).unwrap();
                        v[2] = Self::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy + 1, uz + 1), 1, 0, materials, ao[2]).unwrap();
                        v[3] = Self::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy, uz + 1), 1, 1, materials, ao[3]).unwrap();

                        index = chunk_mesh.add_data(v, index, flip_id);
                    }
                    // back face
                    if chunk_mesh.is_void(IBlockPos::new(ux, uy, uz - 1), &blocks, chunk_pos)? {
                        ao = chunk_mesh.get_ao(IBlockPos::new(ux, uy, uz - 1), Plane::Z, &blocks, chunk_pos)?;

                        flip_id = ao[1] + ao[3] > ao[0] + ao[2];

                        v[0] = Self::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy + 1, uz), 4, 2, materials, ao[2]).unwrap();
                        v[1] = Self::pack_data(voxel_id, glam::IVec3::new(ux, uy + 1, uz), 4, 0, materials, ao[1]).unwrap();
                        v[2] = Self::pack_data(voxel_id, glam::IVec3::new(ux, uy, uz), 4, 1, materials, ao[0]).unwrap();
                        v[3] = Self::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy, uz), 4, 3, materials, ao[3]).unwrap();

                        index = chunk_mesh.add_data(v, index, flip_id);
                    }
                    //top face
                    if chunk_mesh.is_void(IBlockPos::new(ux, uy + 1, uz), &blocks, chunk_pos)? {
                        ao = chunk_mesh.get_ao(IBlockPos::new(ux, uy + 1, uz), Plane::Y, &blocks, chunk_pos)?;

                        flip_id = ao[1] + ao[3] > ao[0] + ao[2];

                        v[0] = Self::pack_data(voxel_id, glam::IVec3::new(ux, uy + 1, uz), 0, 2, materials, ao[0]).unwrap();
                        v[1] = Self::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy + 1, uz), 0, 0, materials, ao[1]).unwrap();
                        v[2] = Self::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy + 1, uz + 1), 0, 1, materials, ao[2]).unwrap();
                        v[3] = Self::pack_data(voxel_id, glam::IVec3::new(ux, uy + 1, uz + 1), 0, 3, materials, ao[3]).unwrap();

                        index = chunk_mesh.add_data(v, index, flip_id);
                    }
                    // bottom face
                    if chunk_mesh.is_void(IBlockPos::new(ux, uy - 1, uz), &blocks, chunk_pos)? {
                        ao = chunk_mesh.get_ao(IBlockPos::new(ux, uy - 1, uz), Plane::Y, &blocks, chunk_pos)?;

                        flip_id = ao[1] + ao[3] > ao[0] + ao[2];

                        v[0] = Self::pack_data(voxel_id, glam::IVec3::new(ux, uy, uz), 5, 1, materials, ao[0]).unwrap();
                        v[1] = Self::pack_data(voxel_id, glam::IVec3::new(ux, uy, uz + 1), 5, 3, materials, ao[1]).unwrap();
                        v[2] = Self::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy, uz + 1), 5, 2, materials, ao[2]).unwrap();
                        v[3] = Self::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy, uz), 5, 0, materials, ao[3]).unwrap();

                        index = chunk_mesh.add_data(v, index, flip_id);
                    }

                    // right face
                    if chunk_mesh.is_void(IBlockPos::new(ux + 1, uy, uz), &blocks, chunk_pos)? {
                        ao = chunk_mesh.get_ao(IBlockPos::new(ux + 1, uy, uz), Plane::X, &blocks, chunk_pos)?;

                        flip_id = ao[1] + ao[3] > ao[0] + ao[2];

                        v[0] = Self::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy, uz), 2, 1, materials, ao[0]).unwrap();
                        v[1] = Self::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy, uz + 1), 2, 3, materials, ao[3]).unwrap();
                        v[2] = Self::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy + 1, uz + 1), 2, 2, materials, ao[1]).unwrap();
                        v[3] = Self::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy + 1, uz), 2, 0, materials, ao[2]).unwrap();

                        index = chunk_mesh.add_data(v, index, flip_id);
                    }

                    // left face
                    if chunk_mesh.is_void(IBlockPos::new(ux - 1, uy, uz), &blocks, chunk_pos)? {
                        ao = chunk_mesh.get_ao(IBlockPos::new(ux - 1, uy, uz), Plane::X, &blocks, chunk_pos)?;

                        flip_id = ao[1] + ao[3] > ao[0] + ao[2];

                        v[0] = Self::pack_data(voxel_id, glam::IVec3::new(ux, uy, uz), 3, 3, materials, ao[0]).unwrap();
                        v[1] = Self::pack_data(voxel_id, glam::IVec3::new(ux, uy + 1, uz), 3, 2, materials, ao[1]).unwrap();
                        v[2] = Self::pack_data(voxel_id, glam::IVec3::new(ux, uy + 1, uz + 1), 3, 0, materials, ao[2]).unwrap();
                        v[3] = Self::pack_data(voxel_id, glam::IVec3::new(ux, uy, uz + 1), 3, 1, materials, ao[3]).unwrap();

                        index = chunk_mesh.add_data(v, index, flip_id);
                    }
                }
            }
        }
        if chunk_mesh.indices.len() == 0 {
            return None
        };
        print_debug!("Created {} vertices in chunks", chunk_mesh.indices.len());
        let mesh_text = unsafe { MeshText::new(chunk_mesh.vertices.clone(), chunk_mesh.indices.clone()) };
        Some((chunk_mesh, mesh_text))
    }

    fn is_void(&self, block_pos: IBlockPos, blocks : &Vec<u16>, chunk_pos: ChunkPos) -> Option<bool> {
        if block_pos.x < 0 || block_pos.x >= CHUNK_SIZE as i32 ||
            block_pos.y < 0 || block_pos.y >= CHUNK_SIZE as i32 ||
            block_pos.z < 0 || block_pos.z >= CHUNK_SIZE as i32 {

            return Some(true);
        }
        Some(blocks[block_pos.x as usize * CHUNK_SIZE * CHUNK_SIZE + block_pos.y as usize * CHUNK_SIZE + block_pos.z as usize] == 0)
    }

    /// Push the packed and serialized data into the buffers and update the indices
    fn add_data(&mut self, v : [u64;4], index : u32, flip : bool) -> u32 {

        for i in 0..4usize {
            self.vertices.push((v[i] >> 32) as u32);        // High 32 bits
            self.vertices.push((v[i] & 0xFFFFFFFF) as u32); // Low 32 bits
        }

        if flip {
            self.indices.push(index + 3);
            self.indices.push(index + 1);
            self.indices.push(index + 0);
            self.indices.push(index + 3);
            self.indices.push(index + 2);
            self.indices.push(index + 1);
        } else {

            self.indices.push(index);
            self.indices.push(index + 2);
            self.indices.push(index + 1);
            self.indices.push(index);
            self.indices.push(index + 3);
            self.indices.push(index + 2);
        }
        index + 4
    }

    fn get_ao(&self, pos : IBlockPos, plane : Plane, blocks : &Vec<u16>, chunk_pos: ChunkPos) -> Option<[u8; 4]> {
        let (a,b,c,d,e,f,g,h);
        if plane == Plane::Y {
            a = self.is_void(IBlockPos::new(pos.x, pos.y, pos.z - 1), blocks, chunk_pos)?;
            b = self.is_void(IBlockPos::new(pos.x - 1, pos.y, pos.z- 1), blocks, chunk_pos)?;
            c = self.is_void(IBlockPos::new(pos.x - 1, pos.y, pos.z), blocks, chunk_pos)?;
            d = self.is_void(IBlockPos::new(pos.x - 1, pos.y, pos.z + 1), blocks,  chunk_pos)?;
            e = self.is_void(IBlockPos::new(pos.x, pos.y, pos.z + 1), blocks,  chunk_pos)?;
            f = self.is_void(IBlockPos::new(pos.x + 1, pos.y, pos.z + 1), blocks,  chunk_pos)?;
            g = self.is_void(IBlockPos::new(pos.x + 1, pos.y, pos.z), blocks,  chunk_pos)?;
            h = self.is_void(IBlockPos::new(pos.x + 1, pos.y, pos.z - 1), blocks,  chunk_pos)?;
        }
        else if plane == Plane::X {
            a = self.is_void(IBlockPos::new(pos.x, pos.y, pos.z - 1), blocks,  chunk_pos)?;
            b = self.is_void(IBlockPos::new(pos.x, pos.y - 1, pos.z - 1), blocks,  chunk_pos)?;
            c = self.is_void(IBlockPos::new(pos.x, pos.y - 1, pos.z), blocks,  chunk_pos)?;
            d = self.is_void(IBlockPos::new(pos.x, pos.y - 1, pos.z + 1), blocks,  chunk_pos)?;
            e = self.is_void(IBlockPos::new(pos.x, pos.y, pos.z + 1), blocks,  chunk_pos)?;
            f = self.is_void(IBlockPos::new(pos.x, pos.y + 1, pos.z + 1), blocks,  chunk_pos)?;
            g = self.is_void(IBlockPos::new(pos.x, pos.y + 1, pos.z), blocks,  chunk_pos)?;
            h = self.is_void(IBlockPos::new(pos.x, pos.y + 1, pos.z + 1), blocks,  chunk_pos)?;
        }
        else  {// Z plane

            a = self.is_void(IBlockPos::new(pos.x - 1, pos.y, pos.z), blocks,  chunk_pos)?;
            b = self.is_void(IBlockPos::new(pos.x - 1, pos.y - 1, pos.z), blocks,  chunk_pos)?;
            c = self.is_void(IBlockPos::new(pos.x, pos.y - 1, pos.z), blocks,  chunk_pos)?;
            d = self.is_void(IBlockPos::new(pos.x + 1, pos.y - 1, pos.z), blocks,  chunk_pos)?;
            e = self.is_void(IBlockPos::new(pos.x + 1, pos.y, pos.z), blocks,  chunk_pos)?;
            f = self.is_void(IBlockPos::new(pos.x + 1, pos.y + 1, pos.z), blocks,  chunk_pos)?;
            g = self.is_void(IBlockPos::new(pos.x, pos.y + 1, pos.z), blocks,  chunk_pos)?;
            h = self.is_void(IBlockPos::new(pos.x - 1, pos.y + 1, pos.z), blocks,  chunk_pos)?;
        }


        Some([a as u8 + b as u8 + c as u8, g as u8 + h as u8 + a as u8, e as u8 + f as u8 + g as u8, c as u8 + d as u8 + e as u8])
    }

    fn pack_data(id : u16, pos: glam::IVec3, face_id : u32, tex_coords : u8, material : [f32; 3], ao : u8) -> io::Result<u64> {
        // println!("Input : {}, {}, {}",pos, face_id, tex_coords);
        if id as u32 >= 2_u32.pow(18) {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Self ID exceeds maximum value of 2^18 - 1".to_string()));
        }
        if pos.x >= 2_i32.pow(7) || pos.y >= 2_i32.pow(7) || pos.z >= 2_i32.pow(7) {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Self position exceeds maximum value of 2^7 - 1".to_string()));
        }
        if face_id > 5 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Face id has to be between 0 and 5".to_string()));
        }
        let mut packed : u64 = 0;
        packed |= (id as u64 & 0x3FFFF) << 46; // ID (18 bits)
        packed |= (pos.x as u64 & 0x7F) << 39; // position X (7 bits)
        packed |= (pos.y as u64 & 0x7F) << 32; // position Y (7 bits)
        packed |= (pos.z as u64 & 0x7F) << 25; // position Z (7 bits)
        packed |= (face_id as u64 & 0x7) << 22; // Face ID (3 bits)
        packed |= Self::pack_f32_to_bits(material[0],0.0,1.0,0x3F) << 16; // Face ID (6 bits)
        packed |= Self::pack_f32_to_bits(material[1],0.0,1.0,0x3F) << 10; // Face ID (6 bits)
        packed |= Self::pack_f32_to_bits(material[2],0.0,1.0,0x3F) << 4; // Face ID (6 bits)
        packed |= (ao as u64 & 0x7) << 2; // Face ID (6 bits)
        packed |= ((tex_coords as u64 >> 1) & 0x1) << 1; // TexCoord X (1 bit)
        packed |= tex_coords as u64 & 0x1; // TexCoord Y (1 bit)
        Ok(packed)
    }

    fn pack_f32_to_bits(val: f32, min: f32, max: f32, bits : u64) -> u64 {
        // Clamp the value to ensure it stays within bounds
        let clamped = val.clamp(min, max);
        // Scale and round to a 0-127 integer
        (((clamped - min) / (max - min)) * bits as f32).round() as u64
    }

    fn unpack_bits_to_f32(packed: u64, min: f32, max: f32, bits : u64) -> f32 {
        let normalized = (packed & bits) as f32 / bits as f32;
        min + normalized * (max - min)
    }
}