use std::collections::HashMap;
use std::net::{Ipv6Addr};
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use std::sync::atomic::Ordering;
use std::thread::sleep;
use std::time::{Duration, Instant};
use crossbeam::channel::internal::SelectHandle;
use noise::{NoiseFn, Perlin};
use winit::window::Window;
use shared::common::account::puid::PUID;
use shared::common::world::pos::chunkpos::{ChunkPos};
use shared::{print_base, print_debug};
use shared::common::network::l5_packet::L5Packet;
use shared::common::network::packet_type::UdpPacketType;
use shared::common::world::pos::blockpos::BlockPos;
use shared::common::world::pos::iblockpos::IBlockPos;
use shared::math::{get_continentalness, get_erosion, get_terrain_height, peaks_and_valleys, default_function};
use crate::client::display::renderer::gui::text::characters::{Character, Text};
use crate::client::display::renderer::gui::text::text::MeshText;
use crate::client::display::renderer::mesh::shader::shader::Shader;
use crate::client::generation::mesh::chunk_mesh::ChunkMesh;
use crate::client::generation::mesh_generator::MeshGenerator;
use crate::client::network::server_connection::ServerConnection;
use crate::client::world_data::chunks::chunk_map::ClientChunkMap;
use crate::client::world_data::client_data::ClientWorldData;
use crate::client::world_data::player::player::ClientPlayer;

pub struct ClientWorld {
    client_world_data: Arc<ClientWorldData>,
    last_player_pos : ChunkPos,
    mesh_receiver: crossbeam::channel::Receiver<(ChunkMesh, MeshText)>,
    text_shader : Shader,
    text : Text,
    puid : PUID,
    characters : HashMap<char,Character>,
    last_frame : Instant,
}

impl ClientWorld {
    pub unsafe fn new() -> ClientWorld {
        let (mesh_sender, mesh_receiver) = crossbeam::channel::unbounded();
        let (chunk_sender, chunk_receiver) = crossbeam::channel::unbounded();
        let (pos_to_mesh_sx, pos_to_mesh_rx) = crossbeam::channel::unbounded();
        let (sender, receiver) = tokio::sync::mpsc::channel(1000);

        let chunk_map = Arc::new(RwLock::new(ClientChunkMap::new(pos_to_mesh_sx, chunk_receiver)));

        MeshGenerator::start_build_meshes(pos_to_mesh_rx, mesh_sender, chunk_map.clone());
        let client_world_data = Arc::new(ClientWorldData::new(chunk_map, sender));

        ServerConnection::start(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), receiver, client_world_data.clone(), chunk_sender);
        Self {
            client_world_data,
            last_player_pos : ChunkPos::new(glam::ivec3(0,0,0)),
            mesh_receiver,
            puid: PUID::new(0),
            text: Text::new(),
            last_frame : Instant::now(),
            characters : HashMap::new(),
            text_shader: Shader::new("/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/shaders/text/vertex.vert", "/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/shaders/text/fragment.frag"),
        }
    }

    pub fn poll_keys(&self, time : f32) {
        self.client_world_data.get_player().write().unwrap().poll_keys(time, self.client_world_data.clone());
    }

    pub unsafe fn load_characters(&mut self) {
        // gl::UniformMatrix4fv(gl::GetUniformLocation(self.text_shader.program, CString::new("projection").unwrap().as_bytes().as_ptr() as *const i8), 1, gl::FALSE, projection.as_ref().as_ptr())
        print_base!("Debug");
        // Keep this! OpenGL needs it for 1-byte (RED) alignment
        unsafe { gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1); }

        let font_data = std::fs::read("src/assets/fonts/IBM_Plex_Mono/IBMPlexMono-Bold.ttf")
            .expect("Failed to read font");

        let font = fontdue::Font::from_bytes(font_data, fontdue::FontSettings::default())
            .expect("Failed to parse font");

        let px_size = 32.0;

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

    pub unsafe fn render(&mut self, window: &Window) {
        let period = Instant::now() - self.last_frame;
        self.last_frame = Instant::now();
        let camera_pos: BlockPos = self.get_player().read().unwrap().get_coords();

        self.client_world_data.get_meshes().write().unwrap().render(&self.get_player().read().unwrap(), window, self.client_world_data.debug.load(Ordering::Relaxed));


        // print_base!("Len of chunks {}", self.client_world_data.get_chunks().read().unwrap().len());
        // print_base!("len of meshes {}", self.meshes.read().unwrap().len());
        let perlin = Perlin::new(1);
        let x = camera_pos.x as f64;
        let z = camera_pos.z as f64;
        let erosion = get_erosion(&perlin, x, z);
        let continentalness = peaks_and_valleys(&perlin, x, z);
        let height = get_terrain_height(&perlin,x as i32,z as i32) as i32;
        let noise = perlin.get([x * 0.001, z * 0.001]);
        let function = default_function(noise.abs());
        let binding = self.client_world_data.get_chunks().clone();
        let chunk_map = binding.write().unwrap();
        self.text.render_text(&self.text_shader, &format!("Noise : {:.5}, Erosion {:.5}, Peaks & Valleys {:.5}, Default fn : {:.8}, Height {}", noise, erosion, continentalness, function, height), 20.0, 20.0, 0.5, glam::vec3(1.0, 1.0, 1.0), &self.characters);
        self.text.render_text(&self.text_shader, &format!("Player : {:.2}, ChunkPos : {:.4}, Is the ChunkPos in the ChunkMap ? {} , block is {}", camera_pos.deref(),  camera_pos.get_chunk_pos().get_vec3(), chunk_map.get_chunk(&camera_pos.get_chunk_pos()).is_some() ,chunk_map.get_block_at(camera_pos.get_absolute_iblock_pos())), 20.0, 1040.0, 0.5, glam::vec3(1.0, 1.0, 1.0), &self.characters);
        self.text.render_text(&self.text_shader, &format!("{} FPS",1.0 / period.as_secs_f64()), 1900.0, 1040.0, 0.5, glam::vec3(1.0, 1.0, 1.0), &self.characters);
    }

    pub fn tick(&mut self) {
        unsafe {
            while !self.mesh_receiver.is_empty() && let Ok(cm) = self.mesh_receiver.recv() {
                self.client_world_data.get_meshes().write().unwrap().add_mesh(cm);
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