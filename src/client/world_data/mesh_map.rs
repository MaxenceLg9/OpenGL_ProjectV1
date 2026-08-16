use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ops::{Add, Deref, DerefMut, Div, Sub};
use std::sync::{Arc, RwLock};
use std::sync::atomic::Ordering;
use bitvec::macros::internal::funty::Fundamental;
use glam::Vec3;
use winit::window::Window;
use shared::common::world::block::block::BlockType;
use shared::common::world::chunk::chunkmap::ChunkMap;
use shared::common::world::pos::blockpos::BlockPos;
use shared::common::world::pos::chunkpos::{ChunkPos, CHUNK_SIZE};
use shared::print_base;
use crate::client::display::renderer::gui::text::characters::Character;
use crate::client::display::renderer::gui::text::text::MeshText;
use crate::client::display::renderer::mesh::chunk_mesh::Mesh;
use crate::client::display::renderer::mesh::shader::shader::Shader;
use crate::client::display::renderer::mesh::texture::texture_array::TextureArray;
use crate::client::generation::mesh::chunk_mesh::ChunkMesh;
use crate::client::world_data::chunks::chunk_map::ClientChunkMap;
use crate::client::world_data::player::player::ClientPlayer;

pub struct MeshMap {
    meshes: HashMap<ChunkPos, Mesh>,
    chunk_shader : Shader,
    blocks_textures: TextureArray,
    meshtext_shader : Shader,
    text_textures: TextureArray,
    text_meshes : HashMap<ChunkPos, MeshText>,
    chunk_radius: f32,
}

impl MeshMap {
    pub unsafe fn new() -> Self {
        let blocks_textures = TextureArray::new("textures".to_string(), gl::RGBA8);
        blocks_textures.add_texture("/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/textures/blocks/stone.png", BlockType::STONE.get_value() - 1).expect("Cannot add block to texture array");
        blocks_textures.add_texture("/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/textures/blocks/dirt.png", BlockType::DIRT.get_value() - 1).expect("Cannot add block to texture array");
        blocks_textures.add_texture("/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/textures/blocks/deepslate.png", BlockType::DEEPSLATE.get_value() - 1).expect("Cannot add block to texture array");
        blocks_textures.add_texture("/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/textures/blocks/ikrine_ore.png", BlockType::IkrineOre.get_value() - 1).expect("Cannot add block to texture array");
        blocks_textures.add_texture("/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/textures/blocks/ikrine_block.png", BlockType::IKRINEBLOCK.get_value() - 1).expect("Cannot add block to texture array");
        blocks_textures.add_texture("/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/textures/blocks/grass.png", BlockType::GRASS.get_value() - 1).expect("Cannot add block to texture array");

        let mut m = Self {
            blocks_textures,
            chunk_shader: Shader::new("/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/shaders/chunk/vertex.vert", "/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/shaders/chunk/fragment.frag"),
            meshes: HashMap::new(),
            text_meshes: HashMap::new(),
            chunk_radius : (32_f32.powf(2.0) * 3.0).sqrt(),
            text_textures : TextureArray::new_raw("characters".to_string(), gl::R8, 32, 32),
            meshtext_shader : Shader::new("/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/shaders/debug_mesh/vertex.vert","/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/shaders/debug_mesh/fragment.frag")
        };
        m.load_characters();
        m
    }

    pub unsafe fn load_characters(&mut self) {
        unsafe { gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1); }

        let font_data = std::fs::read("src/assets/fonts/IBM_Plex_Mono/IBMPlexMono-Bold.ttf")
            .expect("Failed to read font");

        let font = fontdue::Font::from_bytes(font_data, fontdue::FontSettings::default())
            .expect("Failed to parse font");

        let px_size = 32.0;

        for i in 0..10 {
            let chr = (i as u8).add(48) as char;

            // 1. Rasterize
            let (metrics, bitmap) = font.rasterize(chr, px_size);

            // 2. Generate Texture
            self.text_textures.add_raw(&bitmap,i as u16,gl::RED, metrics.width as i32, metrics.height as i32);
        }
    }

    pub unsafe fn add_mesh(&mut self, mut meshes : (ChunkMesh, MeshText)) -> bool {
        let pos = meshes.0.get_chunk_pos();

        match self.text_meshes.entry(pos) {
            Entry::Occupied(mut slot) => {
                print_base!("Reinserting mesh");
                slot.insert(meshes.1.link());
            }
            Entry::Vacant(slot) => {
                slot.insert(meshes.1.link());
            }
        };
        match self.meshes.entry(pos) {
            Entry::Occupied(mut slot) => {
                slot.insert(meshes.0.link());
                true
            }
            Entry::Vacant(slot) => {
                slot.insert(meshes.0.link());
                true
            }
        }


    }

    pub fn contains_mesh(&self, mesh_pos: &ChunkPos) -> bool {
        self.meshes.contains_key(mesh_pos)
    }

    pub fn get_mesh(&self, mesh_pos: &ChunkPos) -> &Mesh {
        self.meshes.get(mesh_pos).unwrap()
    }

    pub fn remove_mesh(&mut self, mesh_pos : &ChunkPos) {
        self.meshes.remove(mesh_pos);
        self.text_meshes.remove(mesh_pos);
    }

    pub unsafe fn render(&mut self, camera_pos: BlockPos, up : Vec3, direction : Vec3, fov : f32, window : &Window, debug : bool, chunk_map: &mut ClientChunkMap) -> u16 {

        let mut n = 0;
        let camera_target: glam::Vec3 = camera_pos.as_vec3() + direction;

        // build view matrix
        let view: glam::Mat4 = glam::Mat4::look_at_lh(camera_pos.as_vec3(), camera_target, up);
        let projection: glam::Mat4 = glam::Mat4::perspective_lh(fov.to_radians(), window.inner_size().width as f32 / window.inner_size().height as f32,
                                                                0.01_f32, 1000.0_f32);
        let pro_view = projection * view;

        gl::DepthFunc(gl::LESS);
        let light_pos = glam::vec3(100.0 + 100.0 * 10.0, 1000.0, 100.0 + 100.0 * 10.0);
        // light.render(pro_view, player.get_coords() + let(0.0f, 100.0f, 0.0f));

        if debug {
            self.meshtext_shader.use_shader();
            self.text_textures.use_textures(&self.meshtext_shader);

            self.meshtext_shader.set_matrix4fv("uniform_projection_view", pro_view);
            for (pos, mesh) in self.text_meshes.iter() {
                let mut model = glam::Mat4::IDENTITY;
                model = model * glam::Mat4::from_translation(pos.as_vec3() * CHUNK_SIZE as f32);
                self.meshtext_shader.set_matrix4fv("uniform_model", model);
                mesh.draw();
            }
        } else {

            self.chunk_shader.use_shader();
            self.blocks_textures.use_textures(&self.chunk_shader);
            // self.text_textures.use_textures(&self.chunk_shader);
            // camera/view transformation
            //    let color = light.getColor();
            let color = glam::Vec3::new(1.0_f32, 1.0_f32, 1.0_f32); // Default colour for debugging
            self.chunk_shader.set_vec3("color", color);
            self.chunk_shader.set_vec3("uniformLightColor", glam::vec3(1.0, 1.0, 1.0));
            self.chunk_shader.set_vec3("uniformLightPos", light_pos);
            self.chunk_shader.set_vec3("uniformViewPos", camera_pos.as_vec3());
            self.chunk_shader.set_matrix4fv("uniform_projection_view", pro_view);

            // let block_pos = IBlockPos::from_ints(-140,108,476);
            // let b1 = self.client_world_data.get_chunks().write().unwrap().get_block_at(block_pos);
            // print_base!("Block at {} is {}", block_pos.deref(), b1);
            // let block_pos = IBlockPos::from_ints(-140,109,476);
            // let b2 = self.client_world_data.get_chunks().write().unwrap().get_block_at(block_pos);
            // print_base!("Block at {} is {}", block_pos.deref(), b2);

            let keys = self.meshes.keys().cloned().collect::<Vec<_>>();
            for pos in keys {
                // small calculations for the 3d rendering
                if self.is_visible(&pos, camera_pos.as_vec3(), direction, up, fov, window.inner_size().width as f32 / window.inner_size().height as f32) {
                    self.meshes.get_mut(&pos).unwrap().draw(&self.chunk_shader, &pos);
                    n += 1;
                } else {

                    let camera_ipos = camera_pos.get_chunk_pos();
                    if (pos.x.sub(camera_ipos.x).pow(2) + pos.z.sub(camera_ipos.z).pow(2)).isqrt() > 10 {
                        self.remove_mesh(&pos);
                        chunk_map.remove_chunk(&pos);
                    }
                }
            }
        }
        n
    }

    fn is_visible(&self, chunk_pos : &ChunkPos, player_pos : Vec3, direction_normalized: glam::Vec3, up : glam::Vec3, fov : f32, aspect : f32) -> bool {
        let relative_chunk_pos = chunk_pos.center() - player_pos;
        let sz = direction_normalized.dot(relative_chunk_pos);

        if sz < -self.chunk_radius {
            return false;
        }
        if fov < 165.0 {
            let half_y_fov = (fov * 0.5).to_radians();
            let tany = half_y_fov.tan();
            let factor_y = 1.0 / half_y_fov.cos();

            let sy = relative_chunk_pos.dot(up);
            let dist = factor_y * self.chunk_radius + sz * tany;
            if sy.abs() > dist {
                return false;
            }

            let tanx = tany * aspect;
            let factor_x = (1.0 + tanx * tanx).sqrt();

            let sx = relative_chunk_pos.dot(up.cross(direction_normalized));
            let dist_side = factor_x * self.chunk_radius + sz * tanx;
            if sx.abs() > dist_side {
                return false
            }
        }

        true
    }
}

impl Deref for MeshMap {
    type Target = HashMap<ChunkPos,Mesh>;

    fn deref(&self) -> &HashMap<ChunkPos,Mesh> {
        &self.meshes
    }

}

impl DerefMut for MeshMap {
    fn deref_mut(&mut self) -> &mut HashMap<ChunkPos,Mesh> {
        &mut self.meshes
    }
}