use std::collections::HashMap;
use std::net::{Ipv6Addr};
use std::ops::{Add, Deref};
use std::sync::{Arc, RwLock};
use std::sync::atomic::Ordering;
use std::sync::mpsc::channel;
use std::thread::Thread;
use std::time::{Duration, Instant};
use crossbeam::channel::{Receiver, Sender};
use glam::Vec3;
use noise::{NoiseFn, Perlin};
use winit::window::Window;
use shared::common::account::puid::PUID;
use shared::{print_base, print_debug};
use shared::common::network::client::block_packet::BlockInteraction;
use shared::common::world::block::block::BlockType;
use shared::common::world::chunk::chunkmap::ChunkMap;
use shared::common::world::pos::blockpos::BlockPos;
use shared::common::world::pos::chunkpos::{ChunkPos, CHUNK_SIZE};
use shared::worldgen::{Generator};
use crate::client::display::renderer::gui::text::characters::{Character, Text};
use crate::client::display::renderer::mesh::shader::shader::Shader;
use crate::client::display::renderer::mesh::texture::texture_array::TextureArray;
use crate::test::display::renderer::mesh::chunk_mesh::ChunkMesh;
use crate::server::world_data::chunk::chunk::ServerChunk;
use crate::client::display::renderer::gui::text::text::MeshText;
use crate::test::display::renderer::mesh::chunk_mesh::Mesh;
use crate::test::display::renderer::player::player::ClientPlayer;

pub struct ClientWorld {
    text_shader : Shader,
    text : Text,
    puid : PUID,
    chunk_shader : Shader,
    blocks_textures: TextureArray,
    meshtext_shader : Shader,
    text_textures: TextureArray,
    debug : bool,
    characters : HashMap<char,Character>,
    last_frame : Instant,
    meshes : HashMap<ChunkPos, (Mesh, MeshText)>,
    generator: Generator,
    sender : Sender<(ChunkPos,Arc<Vec<u16>>)>,
    receiver : Receiver<(ChunkPos,Arc<Vec<u16>>)>,
    player : ClientPlayer,
}

impl ClientWorld {
    pub unsafe fn new() -> ClientWorld {
        let blocks_textures = TextureArray::new("textures".to_string(), gl::RGBA8);
        let (sx, rx) = crossbeam::channel::bounded(100);
        blocks_textures.add_texture("/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/textures/blocks/stone.png", BlockType::STONE.get_value() - 1).expect("Cannot add block to texture array");
        blocks_textures.add_texture("/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/textures/blocks/dirt.png", BlockType::DIRT.get_value() - 1).expect("Cannot add block to texture array");
        blocks_textures.add_texture("/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/textures/blocks/deepslate.png", BlockType::DEEPSLATE.get_value() - 1).expect("Cannot add block to texture array");
        blocks_textures.add_texture("/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/textures/blocks/ikrine_ore.png", BlockType::IkrineOre.get_value() - 1).expect("Cannot add block to texture array");
        blocks_textures.add_texture("/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/textures/blocks/ikrine_block.png", BlockType::IKRINEBLOCK.get_value() - 1).expect("Cannot add block to texture array");
        blocks_textures.add_texture("/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/textures/blocks/grass.png", BlockType::GRASS.get_value() - 1).expect("Cannot add block to texture array");

        Self {
            puid: PUID::new(0),
            blocks_textures,
            chunk_shader: Shader::new("/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/shaders/chunk/vertex.vert", "/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/shaders/chunk/fragment.frag"),
            text_textures : TextureArray::new_raw("characters".to_string(), gl::R8, 32, 32),
            meshtext_shader : Shader::new("/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/shaders/debug_mesh/vertex.vert","/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/shaders/debug_mesh/fragment.frag"),
            generator : Generator::new(0),
            text: Text::new(),
            sender : sx,
            receiver : rx,
            meshes : HashMap::new(),
            debug : false,
            last_frame : Instant::now(),
            characters : HashMap::new(),
            player: ClientPlayer::new(0.0,0.0,0.0),
            text_shader: Shader::new("/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/shaders/text/vertex.vert", "/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/shaders/text/fragment.frag"),
        }
    }

    /// Update the game with the inputs from keyboard and mouse
    pub fn poll_keys(&mut self, time : f32) {
        self.player.poll_keys(time, &mut self.debug);
    }

    pub unsafe fn generate_chunks(&mut self) {
        let sender = self.sender.clone();

        let thread = std::thread::spawn(move || {
            let generator = Arc::new(Generator::new(0));
            for x in -3..3 {
                for z in -3..3 {
                    for y in 0..8 {
                        sender.send((ChunkPos::new(x,y,z),ServerChunk::generate_chunk(generator.clone(),ChunkPos::new(x,y,z)).get_blocks()));
                    }
                }
            }
            print_base!("Generation Done, linking...");
        });

    }

    pub fn get_player(&mut self) -> &mut ClientPlayer {
        &mut self.player
    }

    /// Load bitmap character into VRAM by creating buffer
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
        // gl::UniformMatrix4fv(gl::GetUniformLocation(self.text_shader.program, CString::new("projection").unwrap().as_bytes().as_ptr() as *const i8), 1, gl::FALSE, projection.as_ref().as_ptr())
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

    /// Draw vertices on the screen
    ///
    /// Rendering the chunks and stuff
    pub unsafe fn render(&mut self, window: &Window, redraw_time : Duration) {
        let period = Instant::now() - self.last_frame;
        self.last_frame = Instant::now();
        let (camera_pos, direction, up, fov) = self.player.get_camera().get_pos_info();


        let camera_target: glam::Vec3 = camera_pos.as_vec3() + direction;

        // build view matrix
        let view: glam::Mat4 = glam::Mat4::look_at_lh(camera_pos.as_vec3(), camera_target, up);
        let projection: glam::Mat4 = glam::Mat4::perspective_lh(fov.to_radians(), window.inner_size().width as f32 / window.inner_size().height as f32,
                                                                0.01_f32, 1000.0_f32);
        let pro_view = projection * view;

        gl::DepthFunc(gl::LESS);
        let light_pos = glam::vec3(100.0 + 100.0 * 10.0, 1000.0, 100.0 + 100.0 * 10.0);
        // light.render(pro_view, player.get_coords() + let(0.0f, 100.0f, 0.0f));

        if self.debug {
            self.meshtext_shader.use_shader();
            self.text_textures.use_textures(&self.meshtext_shader);

            self.meshtext_shader.set_matrix4fv("uniform_projection_view", pro_view);
            for (pos, mesh) in self.meshes.iter() {
                let mut model = glam::Mat4::IDENTITY;
                model = model * glam::Mat4::from_translation(pos.as_vec3() * CHUNK_SIZE as f32);
                self.meshtext_shader.set_matrix4fv("uniform_model", model);
                mesh.1.draw();
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
                self.meshes.get_mut(&pos).unwrap().0.draw(&self.chunk_shader, &pos);
            }
        }

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
        for (pos, blocks) in self.receiver.try_recv() {
            if !self.meshes.contains_key(&pos) {
                if let Some(mut mesh) = ChunkMesh::build_mesh(blocks, pos.clone()) {
                    let (meshes) = (mesh.0.link(), mesh.1.link());
                    self.meshes.insert(pos, meshes);
                }
            }

        }
        self.text.render_text(&self.text_shader, &format!("Erosion {:.5} + Noise {:.5}, Peaks & Valleys {:.5}, Contientalness : {:.5}, Height {}", erosion, e_noise, peaks_and_valleys, continentalness, height as i32), 20.0, 20.0, 0.4, glam::vec3(1.0, 1.0, 1.0), &self.characters);
        self.text.render_text(&self.text_shader, &format!("Player : {:.2}, ChunkPos : {:.4}, Direction {:+.5}", camera_pos.deref(),  camera_pos.get_chunk_pos().get_vec3(), direction), 20.0, 1060.0, 0.4, glam::vec3(1.0, 1.0, 1.0), &self.characters);
        self.text.render_text(&self.text_shader, &format!("{:.4} FPS", 1.0 / period.as_secs_f64()), 1550.0, 1060.0, 0.4, glam::vec3(1.0, 1.0, 1.0), &self.characters);
        // self.text.render_text(&self.text_shader, &format!("Redraw Time : {}, Render time {}, Tick time {}", redraw_time.as_micros(), self.stats.render_time.as_micros(), self.stats.tick_time.as_micros()), 1400.0, 20.0, 0.4, glam::vec3(1.0, 1.0, 1.0), &self.characters);
    }

    pub fn tick(&mut self) {

    }
}