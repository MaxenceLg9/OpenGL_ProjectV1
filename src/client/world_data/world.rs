use std::collections::HashMap;
use std::net::{Ipv6Addr};
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};
use noise::{NoiseFn, Perlin};
use winit::window::Window;
use shared::common::account::puid::PUID;
use shared::{print_base, print_debug};
use shared::common::network::client::block_packet::BlockInteraction;
use shared::common::world::block::block::BlockType;
use shared::worldgen::{Generator};
use crate::client::display::renderer::gui::text::characters::{Character, Text};
use crate::client::display::renderer::gui::text::text::MeshText;
use crate::client::display::renderer::mesh::shader::shader::Shader;
use crate::client::generation::mesh::chunk_mesh::ChunkMesh;
use crate::client::generation::mesh_generator::MeshGenerator;
use crate::client::network::server_connection::ServerConnection;
use crate::client::world_data::chunks::chunk_map::ClientChunkMap;
use crate::client::world_data::client_data::ClientWorldData;
use crate::client::world_data::event::event::{ClientEvent, ClientEventType};
use crate::client::world_data::mesh_map::MeshMap;
use crate::client::world_data::player::player::ClientPlayer;

pub struct ClientWorld {
    chunks : ClientChunkMap,
    client_world_data: Arc<ClientWorldData>,
    mesh_receiver: crossbeam::channel::Receiver<(ChunkMesh, MeshText)>,
    text_shader : Shader,
    text : Text,
    puid : PUID,
    characters : HashMap<char,Character>,
    last_frame : Instant,
    meshes : MeshMap,
    generator: Generator,
    event_rx : crossbeam::channel::Receiver<ClientEvent>
}

impl ClientWorld {
    pub unsafe fn new() -> ClientWorld {
        let (mesh_sender, mesh_receiver) = crossbeam::channel::unbounded();
        let (sender_to_mesh, receiver_to_mesh) = crossbeam::channel::unbounded();
        let (event_sx, event_rx) = crossbeam::channel::bounded(100);
        let (sender, receiver) = tokio::sync::mpsc::channel(1000);

        let chunk_map = ClientChunkMap::new(sender_to_mesh);
        let client_world_data = Arc::new(ClientWorldData::new(sender, mesh_sender.clone()));

        MeshGenerator::start_build_meshes(receiver_to_mesh, mesh_sender);

        ServerConnection::start(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), receiver, client_world_data.clone(), event_sx);
        Self {
            client_world_data,
            mesh_receiver,
            puid: PUID::new(0),
            generator : Generator::new(1),
            text: Text::new(),
            meshes : MeshMap::new(),
            chunks : chunk_map,
            last_frame : Instant::now(),
            characters : HashMap::new(),
            event_rx,
            text_shader: Shader::new("/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/shaders/text/vertex.vert", "/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/shaders/text/fragment.frag"),
        }
    }

    /// Update the game with the inputs from keyboard and mouse
    pub fn poll_keys(&mut self, time : f32) {
        self.client_world_data.get_player().write().unwrap().poll_keys(time, self.client_world_data.clone(), &mut self.meshes, &self.chunks);
    }



    /// Load bitmap character into VRAM by creating buffer
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

    /// Draw vertices on the screen
    /// 
    /// Rendering the chunks and stuff
    pub unsafe fn render(&mut self, window: &Window, redraw_time : Duration) {
        let period = Instant::now() - self.last_frame;
        self.last_frame = Instant::now();
        let (camera_pos, direction, up, fov) = self.get_player().read().unwrap().get_camera().get_pos_info();

        let n = self.meshes.render(camera_pos, up, direction, fov, window, self.client_world_data.debug.load(Ordering::Relaxed), &mut self.chunks);


        // print_base!("Len of chunks {}", self.client_world_data.get_chunks().read().unwrap().len());
        // print_base!("len of meshes {}", self.meshes.read().unwrap().len());
        let x = camera_pos.x as f64;
        let z = camera_pos.z as f64;

        let c_noise = self.generator.get_c_noise(x, z);
        let continentalness = self.generator.get_continentalness(c_noise);

        let e_noise = self.generator.get_e_noise(x, z);
        let erosion = self.generator.get_erosion(e_noise, c_noise);

        let pv_noise = self.generator.get_pv_noise(x, z);
        let peaks_and_valleys = self.generator.get_peaks_and_valleys(pv_noise, c_noise);
        let height = self.generator.get_terrain_height(x as i32,z as i32) as i32;
        self.text.render_text(&self.text_shader, &format!("Erosion {:.5} + Noise {:.5}, Peaks & Valleys {:.5}, Contientalness : {:.5}, Height {}", erosion, e_noise, peaks_and_valleys, continentalness, height), 20.0, 20.0, 0.4, glam::vec3(1.0, 1.0, 1.0), &self.characters);
        self.text.render_text(&self.text_shader, &format!("Player : {:.2}, ChunkPos : {:.4}, Direction {:+.5}, Is the ChunkPos in the ChunkMap ? {} , block is {}", camera_pos.deref(),  camera_pos.get_chunk_pos().get_vec3(), direction, self.chunks.get_chunk(&camera_pos.get_chunk_pos()).is_some() ,self.chunks.get_block_at(camera_pos.get_absolute_iblock_pos())), 20.0, 1060.0, 0.4, glam::vec3(1.0, 1.0, 1.0), &self.characters);
        self.text.render_text(&self.text_shader, &format!("{:.4} FPS, Rendering {}/{}/{} chunk meshes", 1.0 / period.as_secs_f64(), n, self.meshes.len(), self.chunks.len()), 1550.0, 1060.0, 0.4, glam::vec3(1.0, 1.0, 1.0), &self.characters);
        // self.text.render_text(&self.text_shader, &format!("Redraw Time : {}, Render time {}, Tick time {}", redraw_time.as_micros(), self.stats.render_time.as_micros(), self.stats.tick_time.as_micros()), 1400.0, 20.0, 0.4, glam::vec3(1.0, 1.0, 1.0), &self.characters);
    }

    pub fn tick(&mut self) {
        let instant = Instant::now();
        unsafe {
            while !self.mesh_receiver.is_empty() && let Ok(cm) = self.mesh_receiver.recv() {
                self.meshes.add_mesh(cm);
                // print_base!("Linking {}", cm.get_chunk_pos().deref());
            }
        }
        while let Ok(event) = self.event_rx.try_recv() {
            match event.client_event_type {
                ClientEventType::ChunkPacketReceived(packet) => {
                    self.chunks.add_temp_chunk(packet);
                },
                ClientEventType::BlockInteraction(packet) => {
                    let meshes = match packet.get_interaction_type() {
                        BlockInteraction::RIGHT => {
                            self.chunks.set_block(packet.get_pos(), packet.get_block_type())
                        }
                        BlockInteraction::LEFT => {
                            self.chunks.set_block(packet.get_pos(), BlockType::AIR)
                        }
                    };
                    for mesh in meshes {
                        print_base!("Receiving packet");
                        if let Some(content) = mesh {
                            self.client_world_data.mesh_sender.send(content);
                        }
                    };
                }
            }
        }
        self.chunks.tick(self.get_player().read().unwrap().get_camera().get_chunk_pos());
    }
}