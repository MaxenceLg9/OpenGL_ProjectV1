use std::collections::HashMap;
use std::ffi::CString;
use std::net::{Ipv6Addr};
use std::ops::Deref;
use std::process::exit;
use std::sync::{Arc, RwLock};
use std::thread::sleep;
use std::time::Duration;
use bitvec::macros::internal::funty::Fundamental;
use gl::types::GLint;
use glam::IVec3;
use image::EncodableLayout;
use noise::Perlin;
use winit::window::Window;
use shared::common::account::puid::PUID;
use shared::common::world::block::block::BlockType;
use shared::common::world::pos::chunkpos::{ChunkPos, CHUNK_SIZE};
use shared::{print_base, print_debug};
use shared::common::network::default_packet::ClientPacket;
use shared::math::{get_continentalness, get_erosion, get_terrain_height, noised_terrain_default};
use crate::client::display::renderer::gui::text::characters::{Character, Text};
use crate::client::display::renderer::mesh::shader::shader::Shader;
use crate::client::display::renderer::mesh::texture::texture_array::TextureArray;
use crate::client::generation::mesh::chunk_mesh::ChunkMesh;
use crate::client::generation::mesh_generator::MeshGenerator;
use crate::client::network::server_connection::ServerConnection;
use crate::client::world_data::chunks::chunk_map::ClientChunkMap;
use crate::client::world_data::client_data::ClientWorldData;
use crate::client::world_data::player::player::ClientPlayer;

pub struct ClientWorld {
    socket : tokio::sync::mpsc::Sender<ClientPacket>,
    chunk_shader : Shader,
    texture_array: TextureArray,
    client_world_data: Arc<ClientWorldData>,
    last_player_pos : ChunkPos,
    mesh_receiver: crossbeam::channel::Receiver<ChunkMesh>,
    text_shader : Shader,
    text : Text,
    puid : PUID,
    characters : HashMap<char,Character>
}

impl ClientWorld {
    pub unsafe fn new() -> ClientWorld {
        let texture_array = TextureArray::new("textures".to_string());
        let (mesh_sender, mesh_receiver) = crossbeam::channel::unbounded();
        let (chunk_sender, chunk_receiver) = crossbeam::channel::unbounded();
        let (pos_to_mesh_sx, pos_to_mesh_rx) = crossbeam::channel::unbounded();
        let (sender, receiver) = tokio::sync::mpsc::channel(100000);

        let chunk_map = Arc::new(RwLock::new(ClientChunkMap::new(pos_to_mesh_sx, chunk_receiver)));

        texture_array.add_texture("/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/textures/blocks/stone.png", BlockType::STONE.get_value() - 1).expect("Cannot add block to texture array");
        texture_array.add_texture("/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/textures/blocks/dirt.png", BlockType::DIRT.get_value() - 1).expect("Cannot add block to texture array");
        texture_array.add_texture("/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/textures/blocks/deepslate.png", BlockType::DEEPSLATE.get_value() - 1).expect("Cannot add block to texture array");
        MeshGenerator::start_build_meshes(pos_to_mesh_rx, mesh_sender, chunk_map.clone());
        let client_world_data = Arc::new(ClientWorldData::new(chunk_map));

        ServerConnection::start(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), receiver, client_world_data.clone(), chunk_sender);
        Self {
            socket : sender,
            texture_array,
            client_world_data,
            last_player_pos : ChunkPos::new(glam::ivec3(0,0,0)),
            mesh_receiver,
            puid: PUID::new(0),
            text: Text::new(),
            characters : HashMap::new(),
            chunk_shader: Shader::new("/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/shaders/chunk/vertex.vert".to_string(), "/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/shaders/chunk/fragment.frag".to_string()),
            text_shader: Shader::new("/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/shaders/text/vertex.vert".to_string(), "/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/shaders/text/fragment.frag".to_string()),
        }
    }

    pub unsafe fn load_characters(&mut self) {
        // gl::UniformMatrix4fv(gl::GetUniformLocation(self.text_shader.program, CString::new("projection").unwrap().as_bytes().as_ptr() as *const i8), 1, gl::FALSE, projection.as_ref().as_ptr());

        // Keep this! OpenGL needs it for 1-byte (RED) alignment
        unsafe { gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1); }

        let font_data = std::fs::read("src/assets/fonts/IBM_Plex_Mono/IBMPlexMono-Bold.ttf")
            .expect("Failed to read font");

        let font = fontdue::Font::from_bytes(font_data, fontdue::FontSettings::default())
            .expect("Failed to parse font");

        // FreeType 40pt at 50 DPI is approx 27.7px.
        // If you just want 40px size, use 40.0.
        let px_size = 30.0;

        for i in 0..128u8 {
            let chr = i as char;

            // 1. Rasterize
            let (metrics, bitmap) = font.rasterize(chr, px_size);

            // 2. Generate Texture
            let mut texture_id = 0;
            unsafe {
                gl::GenTextures(1, &mut texture_id);
                gl::BindTexture(gl::TEXTURE_2D, texture_id);
                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RED as i32,
                    metrics.width as i32,
                    metrics.height as i32,
                    0,
                    gl::RED,
                    gl::UNSIGNED_BYTE,
                    bitmap.as_ptr() as *const _
                );

                // Texture parameters
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            }

            let character = Character::new(
                texture_id,
                glam::ivec2(metrics.width as i32, metrics.height as i32),
                glam::ivec2(metrics.xmin, metrics.ymin),
                metrics.advance_width as u32
            );

            self.characters.insert(chr, character);
        }
    }

    pub fn get_player(&self) -> Arc<RwLock<ClientPlayer>> {
        self.client_world_data.get_player()
    }

    pub unsafe fn render(&self, window: &Window) {
        //    Logs::debug("Rendering world_data");
        let camera_pos: glam::Vec3 = self.get_player().read().unwrap().get_coords().as_vec3();
        let camera_target: glam::Vec3 = camera_pos + self.get_player().read().unwrap().get_direction();

        // build view matrix
        let view: glam::Mat4 = glam::Mat4::look_at_lh(camera_pos, camera_target, self.get_player().read().unwrap().get_up());
        let projection: glam::Mat4 = glam::Mat4::perspective_lh(self.get_player().read().unwrap().get_fov().to_radians(), window.inner_size().width as f32 / window.inner_size().height as f32,
                                                                0.01_f32, 1000.0_f32);
        let pro_view = projection * view;

        gl::DepthFunc(gl::LESS);
        let light_pos = glam::vec3(100.0 + 100.0 * 10.0, 1000.0, 100.0 + 100.0 * 10.0);
        // light.render(pro_view, player.get_coords() + let(0.0f, 100.0f, 0.0f));

        self.chunk_shader.use_shader();
        self.texture_array.use_textures(&self.chunk_shader);
        // camera/view transformation
        //    let color = light.getColor();
        let color = glam::Vec3::new(1.0_f32, 1.0_f32, 1.0_f32); // Default colour for debugging
        self.chunk_shader.set_vec3("color", color);
        self.chunk_shader.set_vec3("uniformLightColor", glam::vec3(1.0, 1.0, 1.0));
        self.chunk_shader.set_vec3("uniformLightPos", light_pos);
        self.chunk_shader.set_vec3("uniformViewPos", camera_pos);
        self.chunk_shader.set_matrix4fv("uniform_projection_view", pro_view);

        // self.text.render_text(&self.text_shader, "(C) LearnOpenGL.com", 540.0, 570.0, 0.5, glam::vec3(0.3, 0.7, 0.9), &self.characters);

        // iterating over the hashmap
        // let mut pos_to_remove = Vec::new();
        // iterating over all the meshes to render them / or remove the ones that are too far from the player
        for (pos, mesh) in self.client_world_data.get_meshes().write().unwrap().iter() {
            // small calculations for the 3d rendering
            let mut model = glam::Mat4::IDENTITY;
            model = model * glam::Mat4::from_translation(pos.as_vec3() * CHUNK_SIZE as f32);
            self.chunk_shader.set_matrix4fv("uniform_model", model);
            mesh.draw();
        }
        sleep(Duration::from_millis(20));
        // print_base!("Len of chunks {}", self.client_world_data.get_chunks().read().unwrap().len());
        // print_base!("len of meshes {}", self.meshes.read().unwrap().len());
        let perlin = Perlin::new(1);
        let x = camera_pos.x.as_f64();
        let z = camera_pos.z.as_f64();
        let erosion = get_erosion(&perlin, x, z);
        let continentalness = noised_terrain_default(&perlin, x, z, 0.005);
        let height = get_terrain_height(&perlin,x.as_i32(),z.as_i32()).as_i32();
        self.text.render_text(&self.text_shader, &format!("Erosion {:.5}, Continentalness {:.5}, Height {}",erosion, continentalness, height), 20.0, 20.0, 0.5, glam::vec3(1.0, 1.0, 1.0), &self.characters);
        self.text.render_text(&self.text_shader, &format!("Player : {:.2}", camera_pos), 20.0, 1000.0, 0.5, glam::vec3(1.0, 1.0, 1.0), &self.characters);

    }

    pub fn tick(&mut self) {
        unsafe {
            while let Ok(mut cm) = self.mesh_receiver.try_recv() {
                let m = cm.link();
                self.client_world_data.get_meshes().write().unwrap().insert(cm.get_chunk_pos(),m);
                // print_base!("Linking {}", cm.get_chunk_pos().deref());
            }
        }
        self.client_world_data.get_chunks().write().unwrap().tick();
        if self.last_player_pos != self.get_player().read().unwrap().get_chunk_pos() {
            self.last_player_pos = self.get_player().read().unwrap().get_chunk_pos();

            print_debug!("New ChunkPos {}", self.last_player_pos.deref());
        }
    }
}