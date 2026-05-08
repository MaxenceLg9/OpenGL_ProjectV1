use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, RwLock};
use shared::common::network::server::chunk_packet::ChunkPacket;
use shared::common::network::server::packet::ServerPacket;
use shared::common::world::chunk::chunk::Chunk;
use shared::common::world::chunk::chunkmap::ChunkMap;
use shared::common::world::pos::chunkpos::ChunkPos;
use crate::server::world_data::player::player::ServerPlayer;

pub struct ServerChunkMap {
    chunk_map: ChunkMap,
    asking_for_chunks : HashMap<ChunkPos, Vec<Arc<RwLock<ServerPlayer>>>>
}

impl ServerChunkMap {
    pub fn new() -> Self {
        Self {
            chunk_map: ChunkMap::new(),
            asking_for_chunks : HashMap::new()
        }
    }

    pub fn ask_for_chunks(&mut self, chunk_pos: ChunkPos, server_player: Arc<RwLock<ServerPlayer>>) {
        match self.asking_for_chunks.entry(chunk_pos) {
            Entry::Occupied(mut e) => {
                e.get_mut().push(server_player);
            }
            Entry::Vacant(e) => {
                let mut entry = Vec::new();
                entry.push(server_player);
                e.insert(entry);
            }
        }
    }

    pub fn add_chunk(&mut self, chunk : Chunk) -> bool {
        match self.asking_for_chunks.get(&chunk.get_chunk_pos()) {
            Some(v) => {
                for (_, packet) in ChunkPacket::from_chunk_to_packets(&chunk) {
                    let server_packet = ServerPacket::Chunk(packet);
                    for player in v {
                        player.read().unwrap().send_packet(server_packet.clone());
                    }
                }
                self.asking_for_chunks.remove(&chunk.get_chunk_pos());
            },
            None => {}
        }
        self.chunk_map.add_chunk(chunk)
    }

    pub fn tick(&mut self) {
        for (pos, v) in self.asking_for_chunks.clone().iter() {
            match self.chunk_map.get(pos) {
                None => {}
                Some(chunk) => {
                    for (_, packet) in ChunkPacket::from_chunk_to_packets(&chunk) {
                        let server_packet = ServerPacket::Chunk(packet);
                        for player in v {
                            player.read().unwrap().send_packet(server_packet.clone());
                        }
                    }
                    self.asking_for_chunks.remove(&chunk.get_chunk_pos());
                }
            }
        }
    }

}

impl Deref for ServerChunkMap {
    type Target = ChunkMap;

    fn deref(&self) -> &Self::Target {
        &self.chunk_map
    }
}