use std::hash::{Hash, Hasher};
use std::io::Error;
use glam::{IVec3, Vec3};
use tokio::sync::mpsc::error::TrySendError;
use shared::common::account::puid::PUID;
use shared::common::network::default_packet::ServerPacket;
use shared::common::world::chunk::chunk::Chunk;
use shared::common::world::pos::blockpos::BlockPos;
use shared::common::world::pos::chunkpos::{ChunkPos, CHUNK_SIZE};
use shared::print_base;

pub struct ServerPlayer {
    last_pos : BlockPos,
    pos : BlockPos,
    puid : PUID,
    sender: tokio::sync::mpsc::Sender<ServerPacket>,
}

impl ServerPlayer {

    pub fn get_chunk_pos(&self) -> IVec3 {
        self.pos.as_ivec3() / CHUNK_SIZE as i32
    }

    pub fn get_sender(&self) -> &tokio::sync::mpsc::Sender<ServerPacket> {
        &self.sender
    }

    pub fn new(pos : BlockPos, sender : tokio::sync::mpsc::Sender<ServerPacket>, puid : PUID) -> Self {
        print_base!("Creating player at {},{},{}", pos.x, pos.y, pos.z);
        Self {
            pos,
            last_pos : pos,
            puid,
            sender,
        }
    }

    pub fn move_to(&mut self, pos : BlockPos) {
        self.last_pos = self.pos;
        self.pos = pos;
    }

    pub fn send_packet(&self, packet : ServerPacket) -> Result<(), TrySendError<ServerPacket>> {
         self.sender.try_send(packet)
    }

    pub fn get_coords(&self) -> BlockPos {
        self.pos
    }

    pub fn register_chunk(&mut self, chunk : &Chunk) {

    }

    pub fn set_pos(&mut self, pos : BlockPos) {
        if pos == self.pos {
            return;
        }
        self.last_pos = self.pos;
        self.pos = pos;
        // print_base!("New pos : {} from {}", self.pos.deref(), self.last_pos.deref());
    }

}

impl Hash for ServerPlayer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.puid.hash(state)
    }
}

impl PartialEq for ServerPlayer {
    fn eq(&self, other: &Self) -> bool {
        self.puid == other.puid
    }
}

impl Eq for ServerPlayer {
    
}




