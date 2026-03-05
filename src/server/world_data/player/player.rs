use std::collections::{HashMap};
use std::sync::mpsc::Receiver;
use crossbeam::channel;
use glam::IVec3;
use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{KeyCode, PhysicalKey};
use shared::common::world::pos::chunkpos::CHUNK_SIZE;
use shared::print_base;

pub struct ServerPlayer {
    last_pos : glam::Vec3,
    pos : glam::Vec3,
    sender: tokio::sync::mpsc::Sender<Vec<u8>>
}

impl ServerPlayer {

    pub fn get_chunk_pos(&self) -> IVec3 {
        self.pos.as_ivec3() / CHUNK_SIZE as i32
    }

    pub fn get_sender(&self) -> &tokio::sync::mpsc::Sender<Vec<u8>> {
        &self.sender
    }

    pub fn new(x : f32, y : f32, z : f32, sender : tokio::sync::mpsc::Sender<Vec<u8>>) -> Self {
        print_base!("Creating player at {},{},{}", x, y, z);
        Self {
            pos : glam::vec3(x, y, z),
            last_pos : glam::vec3(x,y,z),
            sender
        }
    }

    pub fn move_to(&mut self, pos : glam::Vec3) {
        self.last_pos = self.pos;
        self.pos = pos;
    }

    pub fn get_coords(&self) -> glam::Vec3 {
        self.pos
    }

}






