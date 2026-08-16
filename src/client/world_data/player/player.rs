use std::cmp::min;
use std::collections::{HashMap};
use std::collections::hash_map::Entry;
use std::io::Error;
use std::ops::Deref;
use std::sync::Arc;
use glam::Vec3;
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta};
use winit::keyboard::{KeyCode, PhysicalKey};
use shared::common::network::client::block_packet::{BlockInteraction, BlockPacket};
use shared::common::network::l5_packet::L5Packet;
use shared::common::network::packet_type::UdpPacketType;
use shared::common::world::block::block::BlockType;
use shared::common::world::pos::blockpos::BlockPos;
use shared::common::world::pos::chunkpos::{ChunkPos};
use shared::common::world::pos::iblockpos::IBlockPos;
use shared::print_base;
use crate::client::world_data::chunks::chunk_map::ClientChunkMap;
use crate::client::world_data::client_data::ClientWorldData;
use crate::client::world_data::mesh_map::MeshMap;
use crate::client::world_data::player::camera::camera::Camera;
use crate::client::world_data::player::keyboard::Inputs;

pub struct ClientPlayer {
    camera : Camera,
    inputs: Inputs,
}



impl ClientPlayer {
    pub fn new(x : f32, y : f32, z : f32) -> Self {
        print_base!("Creating player at {},{},{}", x, y, z);
        Self {
            inputs : Inputs::new(),
            camera : Camera::new(x,y,z)

        }
    }
    pub fn get_keyboard(&mut self) -> &mut Inputs {
        &mut self.inputs
    }
    pub fn tick(&self, client_world_data: Arc<ClientWorldData>) {
    }
    pub fn get_camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }
    pub fn get_camera(&self) -> &Camera {
        &self.camera
    }
    pub fn poll_keys(&mut self, time : f32, client_world_data: Arc<ClientWorldData>, meshes : &mut MeshMap, client_chunk_map: &ClientChunkMap) {
        self.camera.poll_keys(&mut self.inputs, time, client_world_data.clone(), meshes, client_chunk_map);
    }
}






