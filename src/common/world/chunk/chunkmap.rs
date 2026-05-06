use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use crate::common::network::server::chunk_packet::ChunkPacket;
use crate::common::world::chunk::chunk::Chunk;
use crate::common::world::pos::chunkpos::ChunkPos;

pub struct ChunkMap {
    chunks : HashMap<ChunkPos, Arc<Chunk>>,

}

impl ChunkMap {
    pub fn new() -> Self {
        Self {
            chunks : HashMap::new(),
        }
    }

    pub fn add_chunk(&mut self, chunk: Chunk) -> bool {
        match self.chunks.entry(chunk.get_chunk_pos()) {
            Entry::Occupied(_) => {
                false
            }
            Entry::Vacant(slot) => {
                // print_base!("Inserted {} chunk", chunk.get_chunk_pos().deref());
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