use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use crossbeam::channel;
use shared::common::network::server::chunk_packet::ChunkPacket;
use shared::common::world::chunk::chunk::Chunk;
use shared::common::world::chunk::chunkmap::ChunkMap;
use shared::common::world::pos::chunkpos::ChunkPos;
use shared::print_base;

pub struct ClientChunkMap {
    chunk_map: ChunkMap,
    to_mesh : channel::Sender<ChunkPos>,
    // temp_chunks : HashMap<ChunkPos,HashMap<u8, ChunkPacket>>,
    chunk_receiver : channel::Receiver<Chunk>
}

impl ClientChunkMap {
    pub fn new(mesh_sender: channel::Sender<ChunkPos>, chunk_receiver : channel::Receiver<Chunk>) -> ClientChunkMap {
        Self {
            to_mesh : mesh_sender,
            chunk_map : ChunkMap::new(),
            // temp_chunks: HashMap::new(),
            chunk_receiver
        }
    }

    pub fn get_chunk(&self, pos : &ChunkPos) -> Option<&Arc<Chunk>> {
        self.chunk_map.get_chunk(pos)
    }

    pub fn tick(&mut self) {
        while let Ok(chunk) = self.chunk_receiver.try_recv() {
            self.add_chunk(chunk)
        }
    }

    fn add_chunk(&mut self, c : Chunk) {
        print_base!("Sent chunk {}", c.get_chunk_pos().get_vec3());
        self.to_mesh.send(c.get_chunk_pos()).expect("Cannot send pos to mesh the chunk");
        self.chunk_map.add_chunk(c);
        // print_base!("Len of chunk_map is {}",self.chunk_map.len());
    }

}

impl Deref for ClientChunkMap {
    type Target = ChunkMap;

    fn deref(&self) -> &Self::Target {
        &self.chunk_map
    }
}