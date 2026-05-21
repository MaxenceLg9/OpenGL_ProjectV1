use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use crossbeam::channel;
use shared::common::network::server::chunk_packet::ChunkPacket;
use shared::common::world::block::block::BlockType;
use shared::common::world::chunk::chunk::Chunk;
use shared::common::world::chunk::chunkmap::ChunkMap;
use shared::common::world::pos::chunkpos::ChunkPos;
use shared::common::world::pos::iblockpos::IBlockPos;
use shared::print_base;

pub struct ClientChunkMap {
    chunk_map: ChunkMap,
    to_mesh : channel::Sender<ChunkPos>,
    // temp_chunks : HashMap<ChunkPos,HashMap<u8, ChunkPacket>>,
    chunk_receiver : channel::Receiver<Chunk>
}

impl ClientChunkMap {
    pub fn new(to_mesh: channel::Sender<ChunkPos>, chunk_receiver : channel::Receiver<Chunk>) -> ClientChunkMap {
        Self {
            to_mesh,
            chunk_map : ChunkMap::new(),
            // temp_chunks: HashMap::new(),
            chunk_receiver
        }
    }

    pub fn tick(&mut self) {
        while let Ok(chunk) = self.chunk_receiver.try_recv() {
            self.add_chunk(chunk)
        }
    }

    fn add_chunk(&mut self, chunk: Chunk) {
        let pos = chunk.get_chunk_pos();
        if !self.chunk_map.add_chunk(chunk) {
            return;
        }
        // print_base!("Sent chunk {}", c.get_chunk_pos().get_vec3());
        self.to_mesh.send(pos).expect("Cannot send pos to mesh the chunk");
        // print_base!("Len of chunk_map is {}",self.chunk_map.len());
    }

    pub fn set_block(&mut self, iblock_pos: IBlockPos, block_type: BlockType) {
        self.chunk_map.set_block(iblock_pos, block_type);
        self.to_mesh.send(iblock_pos.get_chunk_pos()).expect("Cannot send pos to re-mesh the chunk");
    }

}

impl Deref for ClientChunkMap {
    type Target = ChunkMap;

    fn deref(&self) -> &Self::Target {
        &self.chunk_map
    }
}