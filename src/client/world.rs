use std::collections::HashMap;
use std::net::{Ipv6Addr, SocketAddrV6};
use std::sync::{Arc, RwLock};
use winit::window::Window;
use shared::common::world::block::block::BlockType;
use shared::common::world::pos::chunkpos::{ChunkPos, CHUNK_SIZE};
use shared::print_base;
use crate::client::display::renderer::mesh::chunk_mesh::ChunkMesh;
use crate::client::display::renderer::mesh::shader::shader::Shader;
use crate::client::display::renderer::mesh::texture::texture_array::TextureArray;
use crate::client::network::socket::ClientSocket;
use crate::client::world_data::player::player::ClientPlayer;

pub struct ClientWorld {
    socket : Option<ClientSocket>,
    player: ClientPlayer,
    chunk_shader : Shader,
    meshes : Arc<RwLock<HashMap<ChunkPos,ChunkMesh>>>,
    texture_array: TextureArray
}

impl ClientWorld {
    pub unsafe fn new() -> ClientWorld {
        let texture_array = TextureArray::new("textures".to_string());
        texture_array.add_texture("/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/textures/blocks/stone.png", BlockType::STONE.get_value() - 1).expect("Cannot add block to texture array");
        texture_array.add_texture("/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/textures/blocks/dirt.png", BlockType::DIRT.get_value() - 1).expect("Cannot add block to texture array");
        texture_array.add_texture("/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/textures/blocks/deepslate.png", BlockType::DEEPSLATE.get_value() - 1).expect("Cannot add block to texture array");
        Self {
            socket : None,
            player: ClientPlayer::new(1.0,1.0,1.0),
            texture_array,
            meshes: Arc::new(RwLock::new(HashMap::new())),
            chunk_shader: Shader::new("/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/shaders/chunk/vertex.vert".to_string(), "/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/shaders/chunk/fragment.frag".to_string()),
        }
    }

    pub fn connect_to(&mut self){
        let ipv6_address = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1);
        self.socket = Some(ClientSocket::new(SocketAddrV6::new(ipv6_address, 25000, 0, 0).into(), self.meshes.clone()));
        self.socket.as_mut().unwrap().send();
    }

    pub fn get_player(&mut self) -> &mut ClientPlayer {
        &mut self.player
    }

    pub unsafe fn render(&self, window: &Window){
        //    Logs::debug("Rendering world_data");
        let camera_pos: glam::Vec3 = self.player.get_coords();
        let camera_target: glam::Vec3 = camera_pos + self.player.get_direction();

        // build view matrix
        let view: glam::Mat4 = glam::Mat4::look_at_lh(camera_pos, camera_target, self.player.get_up());
        let projection: glam::Mat4 = glam::Mat4::perspective_lh(self.player.get_fov().to_radians(), window.inner_size().width as f32 / window.inner_size().height as f32,
                                                                0.01_f32, 1000.0_f32);
        let pro_view = projection * view;

        gl::DepthFunc(gl::LESS);
        let light_pos = glam::vec3(100.0 + 100.0 * 10.0, 1000.0, 100.0 + 100.0 * 10.0);
        // light.render(pro_view, player.get_coords() + let(0.0f, 100.0f, 0.0f));

        self.chunk_shader.use_shader();
        self.texture_array.use_textures(&self.chunk_shader);
        // camera/view transformation
        //    let color = light.getColor();
        let color = glam::Vec3::new(1.0_f32, 1.0_f32, 1.0_f32); // Default color for debugging
        self.chunk_shader.set_vec3("color", color);
        self.chunk_shader.set_vec3("uniformLightColor", glam::vec3(1.0, 1.0, 1.0));
        self.chunk_shader.set_vec3("uniformLightPos", light_pos);
        self.chunk_shader.set_vec3("uniformViewPos", camera_pos);
        self.chunk_shader.set_matrix4fv("uniform_projection_view", pro_view);


        // iterating over the hashmap
        // let mut pos_to_remove = Vec::new();
        // iterating over all the meshes to render them / or remove the ones that are too far from the player
        for (pos, mesh) in self.meshes.write().unwrap().iter_mut() {
            // small calculations for the 3d rendering
            let mut model = glam::Mat4::IDENTITY;
            model = model * glam::Mat4::from_translation(pos.as_vec3() * CHUNK_SIZE as f32);
            self.chunk_shader.set_matrix4fv("uniform_model", model);
            if !mesh.is_linked() {
                mesh.link();
            }
            mesh.draw();
        }
        print_base!("len of meshes {}", self.meshes.read().unwrap().len());

    }
}