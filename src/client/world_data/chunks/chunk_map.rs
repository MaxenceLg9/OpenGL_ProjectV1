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
    temp_chunks : HashMap<ChunkPos,HashMap<u8, ChunkPacket>>,
}

impl ClientChunkMap {
    pub fn new(channel : channel::Sender<ChunkPos>) -> ClientChunkMap {
        Self {
            to_mesh : channel,
            chunk_map : ChunkMap::new(),
            temp_chunks: HashMap::new(),
        }
    }

    pub fn get_chunk(&self, pos : &ChunkPos) -> Option<&Arc<Chunk>> {
        self.chunk_map.get_chunk(pos)
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
            self.to_mesh.send(c.get_chunk_pos()).expect("Cannot send pos to mesh the chunk");
            self.chunk_map.add_chunk(c);
        }
    }

    pub fn add_chunk(&mut self, c : Chunk) {
        self.to_mesh.send(c.get_chunk_pos()).expect("Cannot send pos to mesh the chunk");
        self.chunk_map.add_chunk(c);
        // print_base!("Len of chunk_map is {}",self.chunk_map.len());
    }
}

// impl Deref for ClientChunkMap {
//     type Target = ChunkMap;
//
//     fn deref(&self) -> &Self::Target {
//         &self.chunk_map
//     }
// }

// impl DerefMut for ClientChunkMap {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.chunk_map
//     }
// }