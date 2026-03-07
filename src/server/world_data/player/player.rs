use std::collections::{HashMap, HashSet};
use std::sync::mpsc::Receiver;
use crossbeam::channel;
use glam::IVec3;
use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{KeyCode, PhysicalKey};
use shared::common::network::network_traits::PacketTrait;
use shared::common::network::server::chunk_packet::ChunkPacket;
use shared::common::network::server::packet::ServerPacket;
use shared::common::world::chunk::chunk::Chunk;
use shared::common::world::pos::chunkpos::{ChunkPos, CHUNK_SIZE};
use shared::print_base;

pub struct ServerPlayer {
    last_pos : glam::Vec3,
    pos : glam::Vec3,
    sender: tokio::sync::mpsc::Sender<Vec<u8>>,
    chunks_loaded : HashSet<ChunkPos>
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
            sender,
            chunks_loaded: HashSet::new(),
        }
    }

    pub fn move_to(&mut self, pos : glam::Vec3) {
        self.last_pos = self.pos;
        self.pos = pos;
    }

    pub fn send_packet(&self, packet : ServerPacket) {
        self.sender.try_send(packet.serialize().into_vec()).expect("Capacity is too small");
    }

    pub fn get_coords(&self) -> glam::Vec3 {
        self.pos
    }

    pub fn register_chunk(&mut self, chunk : &Chunk) {
        if self.chunks_loaded.insert(chunk.get_chunk_pos().clone()) {
            for (_,packet) in ChunkPacket::from_chunk_to_packets(&chunk) {
                self.send_packet(ServerPacket::Chunk(packet));
            }
        }
    }

}






