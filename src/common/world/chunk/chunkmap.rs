use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use crate::common::network::server::chunk_packet::ChunkPacket;
use crate::common::world::chunk::chunk::Chunk;
use crate::common::world::pos::chunkpos::ChunkPos;

pub struct ChunkMap {
    chunks : HashMap<ChunkPos, Arc<Chunk>>,
    temp_chunks : HashMap<ChunkPos,HashMap<u8, ChunkPacket>>
}

impl ChunkMap {
    pub fn new() -> Self {
        Self {
            chunks : HashMap::new(),
            temp_chunks: HashMap::new()
        }
    }

    pub fn add_temp(&mut self, m : ChunkPacket) {
        let total = m.get_total();
        let chunk_pos = m.get_chunk_pos();
        match self.temp_chunks.entry(m.get_chunk_pos()) {
            Entry::Occupied(mut e) => {
                e.get_mut().insert(m.get_indice(),m);
            },
            Entry::Vacant(e) => {
                let mut submap = HashMap::new();
                submap.insert(m.get_indice(),m);
                e.insert(submap);
            }
        }
        if self.temp_chunks.get(&chunk_pos).unwrap().len() as u8 == total {
            let c = ChunkPacket::from_packets_to_chunk(self.temp_chunks.get(&chunk_pos).expect("Error when getting"), chunk_pos);
            self.add_chunk(c);
        }
    }

    pub fn add_chunk(&mut self, chunk: Chunk) -> bool {
        match self.chunks.entry(chunk.get_chunk_pos()) {
            Entry::Occupied(_) => {
                false
            }
            Entry::Vacant(slot) => {
                slot.insert(Arc::new(chunk));
                true
            }
        }
    }

    pub fn contains_chunk(&self, chunk_pos: &ChunkPos) -> bool {
        self.chunks.contains_key(chunk_pos)
    }

    pub fn get_chunk(&self, chunk_pos: &ChunkPos) -> Arc<Chunk> {
        self.chunks.get(chunk_pos).unwrap().clone()
    }

    pub fn remove_chunk(&mut self, chunk_pos : &ChunkPos) {
        self.chunks.remove(chunk_pos);
    }
}

impl Deref for ChunkMap {
    type Target = HashMap<ChunkPos,Arc<Chunk>>;

    fn deref(&self) -> &HashMap<ChunkPos,Arc<Chunk>> {
        &self.chunks
    }

}

impl DerefMut for ChunkMap {
    fn deref_mut(&mut self) -> &mut HashMap<ChunkPos,Arc<Chunk>> {
        &mut self.chunks
    }
}