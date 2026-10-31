use crate::client::game::chunks::chunk_map::ClientChunkMap;
use crate::client::game::client_data::ClientWorldData;
use crate::client::game::mesh_map::MeshMap;
use crate::client::game::player::camera::camera::Camera;
use crate::client::game::player::keyboard::Inputs;
use shared::print_base;
use std::sync::Arc;

pub struct ClientPlayer {
    camera: Camera,
    inputs: Inputs,
}

impl ClientPlayer {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { inputs: Inputs::new(), camera: Camera::new(x, y, z) }
    }
    pub fn get_keyboard(&mut self) -> &mut Inputs {
        &mut self.inputs
    }
    pub fn tick(&self, client_world_data: Arc<ClientWorldData>) {}
    pub fn get_camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }
    pub fn get_camera(&self) -> &Camera {
        &self.camera
    }
    pub fn poll_keys(&mut self, time: f32, client_world_data: Arc<ClientWorldData>, meshes: &mut MeshMap, client_chunk_map: &ClientChunkMap) {
        self.camera.poll_keys(&mut self.inputs, time, client_world_data.clone(), meshes, client_chunk_map);
    }
}
