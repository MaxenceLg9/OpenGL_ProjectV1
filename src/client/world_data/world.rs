use std::collections::HashMap;
use std::net::{Ipv6Addr, SocketAddrV6};
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use glam::IVec3;
use tokio::sync::mpsc::channel;
use winit::window::Window;
use shared::common::account::puid::PUID;
use shared::common::network::client::ask_chunk::AskChunkPacket;
use shared::common::network::client::packet::ClientPacket;
use shared::common::world::block::block::BlockType;
use shared::common::world::chunk::chunkmap::ChunkMap;
use shared::common::world::pos::chunkpos::{ChunkPos, CHUNK_SIZE};
use shared::{print_base, print_debug};
use crate::client::display::renderer::mesh::chunk_mesh::Mesh;
use crate::client::display::renderer::mesh::shader::shader::Shader;
use crate::client::display::renderer::mesh::texture::texture_array::TextureArray;
use crate::client::generation::mesh::chunk_mesh::ChunkMesh;
use crate::client::generation::mesh_generator::MeshGenerator;
use crate::client::network::server_connection::ServerConnection;
use crate::client::network::socket::ClientSocket;
use crate::client::world_data::client_data::ClientWorldData;
use crate::client::world_data::mesh_map::MeshMap;
use crate::client::world_data::player::player::ClientPlayer;

pub struct ClientWorld {
    socket : Option<ClientSocket>,
    chunk_shader : Shader,
    texture_array: TextureArray,
    client_world_data: Arc<ClientWorldData>,
    last_player_pos : IVec3,
    generator: MeshGenerator,
    mesh_receiver: crossbeam::channel::Receiver<ChunkMesh>,
    puid : PUID,
}

impl ClientWorld {
    pub unsafe fn new() -> ClientWorld {
        let texture_array = TextureArray::new("textures".to_string());
        let cm = Arc::new(RwLock::new(ChunkMap::new()));
        let (sx, rx) = crossbeam::channel::unbounded();
        texture_array.add_texture("/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/textures/blocks/stone.png", BlockType::STONE.get_value() - 1).expect("Cannot add block to texture array");
        texture_array.add_texture("/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/textures/blocks/dirt.png", BlockType::DIRT.get_value() - 1).expect("Cannot add block to texture array");
        texture_array.add_texture("/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/textures/blocks/deepslate.png", BlockType::DEEPSLATE.get_value() - 1).expect("Cannot add block to texture array");
        Self {
            socket : None,
            texture_array,
            client_world_data: Arc::new(ClientWorldData::new(cm.clone())),
            last_player_pos : glam::ivec3(0,0,0),
            generator: MeshGenerator::new(cm,sx),
            mesh_receiver: rx,
            puid: PUID::new(0),
            chunk_shader: Shader::new("/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/shaders/chunk/vertex.vert".to_string(), "/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/shaders/chunk/fragment.frag".to_string()),
        }
    }

    pub fn connect_to(&mut self) {
        self.socket = Some(ClientSocket::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1),self.client_world_data.clone()));

        for x in -5..10 {
            for y in 0..10 {
                for z in -5..10 {
                    self.socket.as_mut().unwrap().send(ClientPacket::AskChunk(AskChunkPacket::new(self.puid,ChunkPos::new(glam::ivec3(x,y,z)),self.client_world_data.get_player().read().unwrap().get_block_pos())))
                }
            }
        }

    }

    pub fn get_player(&self) -> Arc<RwLock<ClientPlayer>> {
        self.client_world_data.get_player()
    }

    pub unsafe fn render(&self, window: &Window) {
        //    Logs::debug("Rendering world_data");
        let camera_pos: glam::Vec3 = self.get_player().read().unwrap().get_coords();
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
        // print_base!("Len of chunks {}", self.client_world_data.get_chunks().read().unwrap().len());
        // print_base!("len of meshes {}", self.meshes.read().unwrap().len());
    }
    
    pub fn tick(&mut self) {
        unsafe {
            while let Ok(mut cm) = self.mesh_receiver.try_recv() {
                let m = cm.link();
                self.client_world_data.get_meshes().write().unwrap().insert(cm.get_chunk_pos(),m);
                // print_base!("Linking {}", cm.get_chunk_pos().deref());
            }
        }
        for (pos,chunk) in self.client_world_data.get_chunks().read().unwrap().iter() {
            if !self.client_world_data.get_meshes().read().unwrap().contains_mesh(&pos) {
                self.generator.create_mesh(pos.clone());
            }
        }
        let pos = self.get_player().read().unwrap().get_chunk_pos();
        let bpos = self.get_player().read().unwrap().get_block_pos();
        for x in -5..10 {
            for y in 0..10 {
                for z in -5..10 {
                    let pos = ChunkPos::new(glam::ivec3(x + pos.x,y,z + pos.z));
                    if !self.client_world_data.get_chunks().read().unwrap().contains_chunk(&pos) {
                        self.socket.as_mut().unwrap().send(ClientPacket::AskChunk(AskChunkPacket::new(self.puid, pos, bpos)));
                    }
                }
            }
        }
        if self.last_player_pos != self.get_player().read().unwrap().get_chunk_pos() {
            self.last_player_pos = self.get_player().read().unwrap().get_chunk_pos();

            print_debug!("New ChunkPos {}", self.last_player_pos);
        }
    }
}