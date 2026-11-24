use gl::UNSIGNED_INT;
use gl::types::GLuint;
use shared::common::world::chunk::chunkmap::{MAX_Y_CHUNK, MIN_Y_CHUNK};
use shared::common::world::pos::chunkpos::{CHUNK_SIZE, ChunkPos};
use shared::common::world::pos::iblockpos::IBlockPos;
use shared::print_base;
use shared::print_debug;
use std::collections::HashMap;
use std::io;
use std::sync::Arc;
use strum::{Display, FromRepr};

use crate::client::gl::gui::text::text::MeshText;
use crate::client::gl::mesh::chunk_mesh::Mesh;

pub struct ChunkMesh {
    vertices: Vec<u32>,
    indices: Vec<u32>,
    chunk_pos: ChunkPos,
}
#[derive(Clone, Copy, Debug, PartialEq, FromRepr, Display)]
enum Plane {
    X,
    Y,
    Z,
}

impl ChunkMesh {
    pub fn get_chunk_pos(&self) -> ChunkPos {
        self.chunk_pos
    }

    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty() && self.indices.is_empty()
    }

    pub unsafe fn link(&mut self) -> Mesh {
        unsafe {
            let (mut vao, mut vbo, mut ebo) = (0, 0, 0);
            Self::setup_mesh(&mut vao, &mut vbo, &mut ebo);
            self.bind_data(&mut vbo, &mut ebo);
            Mesh::new(vao, vbo, ebo, self.indices.len() as i32)
        }
    }

    unsafe fn setup_mesh(vao: &mut GLuint, vbo: &mut GLuint, ebo: &mut GLuint) {
        unsafe {
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

            gl::VertexArrayAttribBinding(*vao, 0, 0);
            gl::VertexArrayAttribBinding(*vao, 1, 0);

            gl::VertexArrayElementBuffer(*vao, *ebo);
        }
    }

    unsafe fn bind_data(&self, vbo: &mut GLuint, ebo: &mut GLuint) {
        unsafe {
            gl::NamedBufferData(*vbo, self.vertices.len().cast_signed() * 4, self.vertices.as_ptr() as *const _, gl::STATIC_DRAW);
            gl::NamedBufferData(*ebo, self.indices.len().cast_signed() * 4, self.indices.as_ptr() as *const _, gl::STATIC_DRAW);
        }
    }

    /// Construct the chunk mesh and text meshes from chunks
    pub fn build_mesh(chunks_map: &HashMap<ChunkPos, Arc<Vec<u16>>>, chunk_pos: ChunkPos) -> Option<(ChunkMesh, MeshText)> {
        let mut chunk_mesh = Self { vertices: Vec::new(), indices: Vec::new(), chunk_pos };
        let blocks = chunks_map.get(&chunk_pos)?;
        chunk_mesh.vertices.reserve(CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE * 6 * 4 * 2); // 6 faces, 4 vertices per face
        chunk_mesh.indices.reserve(CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE * 6 * 6); // 6 faces, 6 nbIndices per face
        let mut index = 0;
        let materials = [1.0, 1.0, 1.0];
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let voxel_id: u16 = blocks[x * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + z];

                    if voxel_id == 0 {
                        continue; // skip empty blocks
                    }
                    let mut v: [u64; 4] = [0; 4];
                    let (mut ao, mut flip_id);
                    let (ux, uy, uz) = (x as i32, y as i32, z as i32);
                    //front face
                    if chunk_mesh.is_void(IBlockPos::new(ux, uy, uz + 1), blocks, chunks_map, chunk_pos)? {
                        ao = chunk_mesh.get_ao(IBlockPos::new(ux, uy, uz + 1), Plane::Z, blocks, chunks_map, chunk_pos)?;

                        flip_id = ao[1] + ao[3] > ao[0] + ao[2];

                        v[0] = Self::pack_data(voxel_id, glam::IVec3::new(ux, uy, uz + 1), 1, 3, materials, ao[0]).unwrap();
                        v[1] = Self::pack_data(voxel_id, glam::IVec3::new(ux, uy + 1, uz + 1), 1, 2, materials, ao[1]).unwrap();
                        v[2] = Self::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy + 1, uz + 1), 1, 0, materials, ao[2]).unwrap();
                        v[3] = Self::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy, uz + 1), 1, 1, materials, ao[3]).unwrap();

                        index = chunk_mesh.add_data(v, index, flip_id);
                    }
                    // back face
                    if chunk_mesh.is_void(IBlockPos::new(ux, uy, uz - 1), blocks, chunks_map, chunk_pos)? {
                        ao = chunk_mesh.get_ao(IBlockPos::new(ux, uy, uz - 1), Plane::Z, blocks, chunks_map, chunk_pos)?;

                        flip_id = ao[1] + ao[3] > ao[0] + ao[2];

                        v[0] = Self::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy + 1, uz), 4, 2, materials, ao[2]).unwrap();
                        v[1] = Self::pack_data(voxel_id, glam::IVec3::new(ux, uy + 1, uz), 4, 0, materials, ao[1]).unwrap();
                        v[2] = Self::pack_data(voxel_id, glam::IVec3::new(ux, uy, uz), 4, 1, materials, ao[0]).unwrap();
                        v[3] = Self::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy, uz), 4, 3, materials, ao[3]).unwrap();

                        index = chunk_mesh.add_data(v, index, flip_id);
                    }
                    //top face
                    if chunk_mesh.is_void(IBlockPos::new(ux, uy + 1, uz), blocks, chunks_map, chunk_pos)? {
                        ao = chunk_mesh.get_ao(IBlockPos::new(ux, uy + 1, uz), Plane::Y, blocks, chunks_map, chunk_pos)?;

                        flip_id = ao[1] + ao[3] > ao[0] + ao[2];

                        v[0] = Self::pack_data(voxel_id, glam::IVec3::new(ux, uy + 1, uz), 0, 2, materials, ao[0]).unwrap();
                        v[1] = Self::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy + 1, uz), 0, 0, materials, ao[1]).unwrap();
                        v[2] = Self::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy + 1, uz + 1), 0, 1, materials, ao[2]).unwrap();
                        v[3] = Self::pack_data(voxel_id, glam::IVec3::new(ux, uy + 1, uz + 1), 0, 3, materials, ao[3]).unwrap();

                        index = chunk_mesh.add_data(v, index, flip_id);
                    }
                    // bottom face
                    if chunk_mesh.is_void(IBlockPos::new(ux, uy - 1, uz), blocks, chunks_map, chunk_pos)? {
                        ao = chunk_mesh.get_ao(IBlockPos::new(ux, uy - 1, uz), Plane::Y, blocks, chunks_map, chunk_pos)?;

                        flip_id = ao[1] + ao[3] > ao[0] + ao[2];

                        v[0] = Self::pack_data(voxel_id, glam::IVec3::new(ux, uy, uz), 5, 1, materials, ao[0]).unwrap();
                        v[1] = Self::pack_data(voxel_id, glam::IVec3::new(ux, uy, uz + 1), 5, 3, materials, ao[3]).unwrap();
                        v[2] = Self::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy, uz + 1), 5, 2, materials, ao[2]).unwrap();
                        v[3] = Self::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy, uz), 5, 0, materials, ao[1]).unwrap();

                        index = chunk_mesh.add_data(v, index, flip_id);
                    }

                    // right face
                    if chunk_mesh.is_void(IBlockPos::new(ux + 1, uy, uz), blocks, chunks_map, chunk_pos)? {
                        ao = chunk_mesh.get_ao(IBlockPos::new(ux + 1, uy, uz), Plane::X, blocks, chunks_map, chunk_pos)?;

                        flip_id = ao[1] + ao[3] > ao[0] + ao[2];

                        v[0] = Self::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy, uz), 2, 1, materials, ao[0]).unwrap();
                        v[1] = Self::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy, uz + 1), 2, 3, materials, ao[3]).unwrap();
                        v[2] = Self::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy + 1, uz + 1), 2, 2, materials, ao[2]).unwrap();
                        v[3] = Self::pack_data(voxel_id, glam::IVec3::new(ux + 1, uy + 1, uz), 2, 0, materials, ao[1]).unwrap();

                        index = chunk_mesh.add_data(v, index, flip_id);
                    }

                    // left face
                    if chunk_mesh.is_void(IBlockPos::new(ux - 1, uy, uz), blocks, chunks_map, chunk_pos)? {
                        ao = chunk_mesh.get_ao(IBlockPos::new(ux - 1, uy, uz), Plane::X, blocks, chunks_map, chunk_pos)?;

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
        if chunk_mesh.indices.is_empty() {
            return None;
        };
        print_debug!("Created {} vertices in chunks", chunk_mesh.indices.len());
        let mesh_text = unsafe { MeshText::new(chunk_mesh.vertices.clone(), chunk_mesh.indices.clone()) };
        Some((chunk_mesh, mesh_text))
    }

    fn is_void(&self, block_pos: IBlockPos, blocks: &[u16], chunks_map: &HashMap<ChunkPos, Arc<Vec<u16>>>, chunk_pos: ChunkPos) -> Option<bool> {
        if block_pos.x < 0 || block_pos.x >= CHUNK_SIZE as i32 || block_pos.y < 0 || block_pos.y >= CHUNK_SIZE as i32 || block_pos.z < 0 || block_pos.z >= CHUNK_SIZE as i32 {
            return Some(self.get_block_at(chunk_pos + block_pos, chunks_map)? == 0);
        }
        Some(blocks[block_pos.x as usize * CHUNK_SIZE * CHUNK_SIZE + block_pos.y as usize * CHUNK_SIZE + block_pos.z as usize] == 0)
    }

    pub fn get_block_at(&self, ipos: IBlockPos, chunks_map: &HashMap<ChunkPos, Arc<Vec<u16>>>) -> Option<u16> {
        let (block_pos, chunk_pos) = ipos.as_split_pos();
        if chunk_pos.y < MIN_Y_CHUNK || chunk_pos.y > MAX_Y_CHUNK {
            return Some(0);
        }
        Some(chunks_map.get(&chunk_pos)?[block_pos.get_index()])
    }

    /// Push the packed and serialized data into the buffers and update the indices
    fn add_data(&mut self, v: [u64; 4], index: u32, flip: bool) -> u32 {
        for vertex in v {
            self.vertices.push((vertex >> 32) as u32); // High 32 bits
            self.vertices.push((vertex & 0xFFFFFFFF) as u32); // Low 32 bits
        }

        if flip {
            self.indices.push(index + 3);
            self.indices.push(index + 1);
            self.indices.push(index);
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

    fn get_ao(&self, pos: IBlockPos, plane: Plane, blocks: &[u16], chunks_map: &HashMap<ChunkPos, Arc<Vec<u16>>>, chunk_pos: ChunkPos) -> Option<[u8; 4]> {
        let (a, b, c, d, e, f, g, h) = match plane {
            Plane::X => {
                (
                    // back
                    self.is_void(IBlockPos::new(pos.x, pos.y, pos.z - 1), blocks, chunks_map, chunk_pos)?,
                    // back bottom
                    self.is_void(IBlockPos::new(pos.x, pos.y - 1, pos.z - 1), blocks, chunks_map, chunk_pos)?,
                    // bottom
                    self.is_void(IBlockPos::new(pos.x, pos.y - 1, pos.z), blocks, chunks_map, chunk_pos)?,
                    // bottom front
                    self.is_void(IBlockPos::new(pos.x, pos.y - 1, pos.z + 1), blocks, chunks_map, chunk_pos)?,
                    // front
                    self.is_void(IBlockPos::new(pos.x, pos.y, pos.z + 1), blocks, chunks_map, chunk_pos)?,
                    // front top
                    self.is_void(IBlockPos::new(pos.x, pos.y + 1, pos.z + 1), blocks, chunks_map, chunk_pos)?,
                    // top
                    self.is_void(IBlockPos::new(pos.x, pos.y + 1, pos.z), blocks, chunks_map, chunk_pos)?,
                    // back top
                    self.is_void(IBlockPos::new(pos.x, pos.y + 1, pos.z - 1), blocks, chunks_map, chunk_pos)?,
                )
            }
            Plane::Y => {
                (
                    // back
                    self.is_void(IBlockPos::new(pos.x, pos.y, pos.z - 1), blocks, chunks_map, chunk_pos)?,
                    // back left
                    self.is_void(IBlockPos::new(pos.x - 1, pos.y, pos.z - 1), blocks, chunks_map, chunk_pos)?,
                    // left
                    self.is_void(IBlockPos::new(pos.x - 1, pos.y, pos.z), blocks, chunks_map, chunk_pos)?,
                    // front left
                    self.is_void(IBlockPos::new(pos.x - 1, pos.y, pos.z + 1), blocks, chunks_map, chunk_pos)?,
                    // front
                    self.is_void(IBlockPos::new(pos.x, pos.y, pos.z + 1), blocks, chunks_map, chunk_pos)?,
                    // right front
                    self.is_void(IBlockPos::new(pos.x + 1, pos.y, pos.z + 1), blocks, chunks_map, chunk_pos)?,
                    // right
                    self.is_void(IBlockPos::new(pos.x + 1, pos.y, pos.z), blocks, chunks_map, chunk_pos)?,
                    // right back
                    self.is_void(IBlockPos::new(pos.x + 1, pos.y, pos.z - 1), blocks, chunks_map, chunk_pos)?,
                )
            }
            Plane::Z => {
                (
                    // left
                    self.is_void(IBlockPos::new(pos.x - 1, pos.y, pos.z), blocks, chunks_map, chunk_pos)?,
                    // left bottom
                    self.is_void(IBlockPos::new(pos.x - 1, pos.y - 1, pos.z), blocks, chunks_map, chunk_pos)?,
                    // bottom
                    self.is_void(IBlockPos::new(pos.x, pos.y - 1, pos.z), blocks, chunks_map, chunk_pos)?,
                    // right bottom
                    self.is_void(IBlockPos::new(pos.x + 1, pos.y - 1, pos.z), blocks, chunks_map, chunk_pos)?,
                    // right
                    self.is_void(IBlockPos::new(pos.x + 1, pos.y, pos.z), blocks, chunks_map, chunk_pos)?,
                    // right top
                    self.is_void(IBlockPos::new(pos.x + 1, pos.y + 1, pos.z), blocks, chunks_map, chunk_pos)?,
                    // top
                    self.is_void(IBlockPos::new(pos.x, pos.y + 1, pos.z), blocks, chunks_map, chunk_pos)?,
                    // left top
                    self.is_void(IBlockPos::new(pos.x - 1, pos.y + 1, pos.z), blocks, chunks_map, chunk_pos)?,
                )
            }
        };

        Some([a as u8 + b as u8 + c as u8, g as u8 + h as u8 + a as u8, e as u8 + f as u8 + g as u8, c as u8 + d as u8 + e as u8])
    }

    fn pack_data(id: u16, pos: glam::IVec3, face_id: u32, tex_coords: u8, material: [f32; 3], ao: u8) -> io::Result<u64> {
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
        let mut packed: u64 = 0;
        packed |= (id as u64 & 0x3FFFF) << 46; // ID (18 bits)
        packed |= (pos.x as u64 & 0x7F) << 39; // position X (7 bits)
        packed |= (pos.y as u64 & 0x7F) << 32; // position Y (7 bits)
        packed |= (pos.z as u64 & 0x7F) << 25; // position Z (7 bits)
        packed |= (face_id as u64 & 0x7) << 22; // Face ID (3 bits)
        packed |= Self::pack_f32_to_bits(material[0], 0.0, 1.0, 0x3F) << 16; // Face ID (6 bits)
        packed |= Self::pack_f32_to_bits(material[1], 0.0, 1.0, 0x3F) << 10; // Face ID (6 bits)
        packed |= Self::pack_f32_to_bits(material[2], 0.0, 1.0, 0x3F) << 4; // Face ID (6 bits)
        packed |= (ao as u64 & 0x7) << 2; // Face ID (6 bits)
        packed |= ((tex_coords as u64 >> 1) & 0x1) << 1; // TexCoord X (1 bit)
        packed |= tex_coords as u64 & 0x1; // TexCoord Y (1 bit)
        Ok(packed)
    }

    fn pack_f32_to_bits(val: f32, min: f32, max: f32, bits: u64) -> u64 {
        // Clamp the value to ensure it stays within bounds
        let clamped = val.clamp(min, max);
        // Scale and round to a 0-127 integer
        (((clamped - min) / (max - min)) * bits as f32).round() as u64
    }

    #[cfg(test)]
    fn unpack_bits_to_f32(packed: u64, min: f32, max: f32, bits: u64) -> f32 {
        let normalized = (packed & bits) as f32 / bits as f32;
        min + normalized * (max - min)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::common::world::chunk::chunkmap::ChunkMap;

    const C: usize = CHUNK_SIZE;
    const FACE_INDICES: usize = 6;
    const FACE_VERTEX_WORDS: usize = 8; // 4 vertices, each packed in 2 u32

    /// 27 empty chunks around `center`, with `blocks` set (relative to `center`).
    fn neighbourhood(center: ChunkPos, blocks: &[(usize, usize, usize, u16)]) -> HashMap<ChunkPos, Arc<Vec<u16>>> {
        let mut map: HashMap<_, _> = ChunkMap::get_neighbours_chunks_pos(center)
            .into_iter()
            .map(|p| (p, Arc::new(vec![0u16; C * C * C])))
            .collect();
        let center_blocks = Arc::make_mut(map.get_mut(&center).unwrap());
        for &(x, y, z, id) in blocks {
            center_blocks[x * C * C + y * C + z] = id;
        }
        map
    }

    /// Builds the mesh without touching OpenGL.
    fn build(chunks: &HashMap<ChunkPos, Arc<Vec<u16>>>, pos: ChunkPos) -> Option<ChunkMesh> {
        let (mesh, text) = ChunkMesh::build_mesh(chunks, pos)?;
        // `MeshText::drop` calls OpenGL, which is not loaded in tests
        std::mem::forget(text);
        Some(mesh)
    }

    fn faces(mesh: &ChunkMesh) -> usize {
        assert_eq!(mesh.indices.len() % FACE_INDICES, 0);
        assert_eq!(mesh.vertices.len() / FACE_VERTEX_WORDS, mesh.indices.len() / FACE_INDICES);
        mesh.indices.len() / FACE_INDICES
    }

    #[test]
    fn empty_chunk_has_no_mesh() {
        let pos = ChunkPos::new(0, 0, 0);
        assert!(build(&neighbourhood(pos, &[]), pos).is_none());
    }

    #[test]
    fn lone_block_has_six_faces() {
        let pos = ChunkPos::new(0, 0, 0);
        let mesh = build(&neighbourhood(pos, &[(32, 32, 32, 1)]), pos).unwrap();
        assert_eq!(faces(&mesh), 6);
        assert_eq!(mesh.get_chunk_pos(), pos);
    }

    #[test]
    fn touching_blocks_hide_their_shared_faces() {
        let pos = ChunkPos::new(2, -1, 3);
        let two = build(&neighbourhood(pos, &[(10, 10, 10, 1), (11, 10, 10, 2)]), pos).unwrap();
        assert_eq!(faces(&two), 10);

        // 2x2x2 cube: only the outer 24 faces remain
        let cube: Vec<_> = (0..8).map(|i| (20 + (i & 1), 20 + (i >> 1 & 1), 20 + (i >> 2 & 1), 1)).collect();
        assert_eq!(faces(&build(&neighbourhood(pos, &cube), pos).unwrap()), 24);
    }

    #[test]
    fn faces_against_a_solid_neighbour_chunk_are_culled() {
        let pos = ChunkPos::new(0, 0, 0);
        let mut chunks = neighbourhood(pos, &[(C - 1, 5, 5, 1)]);
        // fill the chunk on +x with stone
        chunks.insert(ChunkPos::new(1, 0, 0), Arc::new(vec![1; C * C * C]));
        assert_eq!(faces(&build(&chunks, pos).unwrap()), 5);
    }

    #[test]
    fn border_block_without_its_neighbour_chunk_cannot_be_meshed() {
        let pos = ChunkPos::new(0, 0, 0);
        let mut chunks = neighbourhood(pos, &[(0, 5, 5, 1)]);
        chunks.remove(&ChunkPos::new(-1, 0, 0));
        assert!(build(&chunks, pos).is_none());
    }

    #[test]
    fn chunks_outside_the_world_height_count_as_air() {
        let pos = ChunkPos::new(0, MAX_Y_CHUNK, 0);
        let mut chunks = neighbourhood(pos, &[(5, C - 1, 5, 1)]);
        chunks.retain(|p, _| p.y <= MAX_Y_CHUNK);
        assert_eq!(faces(&build(&chunks, pos).unwrap()), 6);
    }

    #[test]
    fn ambient_occlusion_counts_solid_corner_neighbours() {
        let pos = ChunkPos::new(0, 0, 0);
        let lone = neighbourhood(pos, &[(10, 10, 10, 1)]);
        let mesh = ChunkMesh {
            vertices: Vec::new(),
            indices: Vec::new(),
            chunk_pos: pos,
        };
        let blocks = lone[&pos].clone();
        // a lone block: every neighbour of the top face is air
        assert_eq!(mesh.get_ao(IBlockPos::new(10, 11, 10), Plane::Y, &blocks, &lone, pos), Some([3, 3, 3, 3]));

        // a wall on -x above the block darkens the two -x corners of its top face
        let walled = neighbourhood(pos, &[(10, 10, 10, 1), (9, 11, 9, 1), (9, 11, 10, 1), (9, 11, 11, 1)]);
        let blocks = walled[&pos].clone();
        let ao = mesh.get_ao(IBlockPos::new(10, 11, 10), Plane::Y, &blocks, &walled, pos).unwrap();
        assert_eq!(ao, [1, 3, 3, 1]);
    }

    #[test]
    fn pack_data_layout() {
        let packed = ChunkMesh::pack_data(0xABCD, glam::IVec3::new(64, 1, 127), 5, 0b11, [1.0, 0.0, 1.0], 3).unwrap();
        assert_eq!(packed >> 46, 0xABCD, "block id");
        assert_eq!(packed >> 39 & 0x7F, 64, "x");
        assert_eq!(packed >> 32 & 0x7F, 1, "y");
        assert_eq!(packed >> 25 & 0x7F, 127, "z");
        assert_eq!(packed >> 22 & 0x7, 5, "face");
        assert_eq!(packed >> 16 & 0x3F, 0x3F, "material r");
        assert_eq!(packed >> 10 & 0x3F, 0, "material g");
        assert_eq!(packed >> 4 & 0x3F, 0x3F, "material b");
        assert_eq!(packed >> 2 & 0x3, 3, "ao");
        assert_eq!(packed & 0x3, 0b11, "uv");
    }

    #[test]
    fn pack_data_rejects_out_of_range_values() {
        let ok = |id, pos, face| ChunkMesh::pack_data(id, pos, face, 0, [1.0; 3], 0).is_ok();
        assert!(ok(1, glam::IVec3::splat(64), 0), "x/y/z = 64 is a valid vertex on a chunk border");
        assert!(!ok(1, glam::IVec3::new(128, 0, 0), 0));
        assert!(!ok(1, glam::IVec3::ZERO, 6));
    }

    #[test]
    fn f32_packing_round_trips_within_precision() {
        for value in [0.0, 0.25, 0.5, 1.0] {
            let bits = ChunkMesh::pack_f32_to_bits(value, 0.0, 1.0, 0x3F);
            let back = ChunkMesh::unpack_bits_to_f32(bits, 0.0, 1.0, 0x3F);
            assert!((back - value).abs() <= 1.0 / 63.0, "{value} -> {back}");
        }
        assert_eq!(ChunkMesh::pack_f32_to_bits(2.0, 0.0, 1.0, 0x3F), 0x3F, "values are clamped");
    }
}
