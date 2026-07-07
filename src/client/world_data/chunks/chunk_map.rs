use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, RwLock};
use crossbeam::channel;
use shared::common::network::server::chunk_packet::ChunkPacket;
use shared::common::world::block::block::BlockType;
use shared::common::world::chunk::chunk::Chunk;
use shared::common::world::chunk::chunkmap::ChunkMap;
use shared::common::world::pos::chunkpos::ChunkPos;
use shared::common::world::pos::iblockpos::IBlockPos;
use crate::client::display::renderer::gui::text::text::MeshText;
use crate::client::generation::mesh::chunk_mesh::ChunkMesh;
use crate::client::world_data::mesh_map::MeshMap;

pub struct ClientChunkMap {
    chunk_map: ChunkMap,
    to_mesh : channel::Sender<ChunkPos>,
    chunk_packet_receiver: channel::Receiver<ChunkPacket>,
    temp_chunks : HashMap<ChunkPos,HashMap<u8, ChunkPacket>>,
}

impl ClientChunkMap {
    pub fn new(to_mesh: channel::Sender<ChunkPos>, chunk_receiver : channel::Receiver<ChunkPacket>) -> ClientChunkMap {
        Self {
            to_mesh,
            chunk_map : ChunkMap::new(),
            temp_chunks: HashMap::new(),
            // temp_chunks: HashMap::new(),
            chunk_packet_receiver: chunk_receiver
        }
    }

    pub fn tick(&mut self) {
        while let Ok(chunk) = self.chunk_packet_receiver.try_recv() {
            self.add_temp_chunk(chunk)
        }
    }
    fn add_temp_chunk(&mut self, chunk_packet: ChunkPacket) {
        let total = chunk_packet.get_total();
        let chunk_pos = chunk_packet.get_chunk_pos();
        if self.chunk_map.contains_chunk(&chunk_pos) {
            return;
        }
        match self.temp_chunks.entry(chunk_packet.get_chunk_pos()) {
            Entry::Occupied(mut e) => {
                e.get_mut().insert(chunk_packet.get_indice(), chunk_packet.clone());
            },
            Entry::Vacant(e) => {
                let mut submap = HashMap::new();
                submap.insert(chunk_packet.get_indice(), chunk_packet.clone());
                e.insert(submap);
            }
        }
        if self.temp_chunks.get(&chunk_pos).unwrap().len() as u8 == total {
            let c = ChunkPacket::from_packets_to_chunk(self.temp_chunks.remove(&chunk_pos).expect("Error when getting"), chunk_pos);
            // self.client_world_data.get_chunks().write().unwrap().add_chunk(c.clone());
            self.add_chunk(c);
        }
    }

    fn add_neighbours_if_exist(&self, chunk_pos: &ChunkPos, chunks : &mut HashMap<ChunkPos,Vec<u16>>) -> u8 {
        let mut count = 0;
        let pos_vec = ChunkMap::get_neighbours_chunks_pos(chunk_pos);

        // checking every side of the chunk
        for p in pos_vec.iter() {
            // the position is already in the map, no need to check if it exists
            if !chunks.contains_key(p) && p.y >= -2 && p.y <= 9 {
                // checking if the chunk exists in the world_data data as it doesn't exist in the map
                let result = self.chunk_map.get_chunk(p);
                // getting the object associated with the pos, checking that the chunk exists and adding it into the map
                if result.is_none() {
                    continue;
                }
                // the result is some, adding it into the map
                chunks.entry(*p).or_insert(result.unwrap().get_blocks().clone());
            }
            count += 1;
        }
        count
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

    pub fn set_block(&mut self, iblock_pos: IBlockPos, block_type: BlockType) -> Vec<Option<(ChunkMesh,MeshText)>> {
        let chunk_pos = iblock_pos.get_chunk_pos();
        if self.chunk_map.set_block(iblock_pos, block_type) {
            return self.redraw_chunks(chunk_pos, iblock_pos.get_block_pos());
        }
        Vec::new()
    }

    fn redraw_chunks(&self, chunk_pos: ChunkPos, relative_block_pos : IBlockPos) -> Vec<Option<(ChunkMesh,MeshText)>> {
        let mut vec = Vec::new();
        let mut chunks_map = HashMap::new();

        vec.push(self.rebuild_mesh(chunk_pos,&mut chunks_map));
        if relative_block_pos.x == 0 || relative_block_pos.x == 63 {
            let offset_x = relative_block_pos.x / 63 * 2 - 1;
            let pos = chunk_pos + ChunkPos::from_i32(offset_x, 0, 0);
            vec.push(self.rebuild_mesh(pos, &mut chunks_map));
        }
        if relative_block_pos.y == 0 || relative_block_pos.y == 63 {
            let offset_y = relative_block_pos.y / 63 * 2 - 1;
            let pos = chunk_pos + ChunkPos::from_i32(0, offset_y, 0);
            vec.push(self.rebuild_mesh(pos, &mut chunks_map));
        }
        if relative_block_pos.z == 0 || relative_block_pos.z == 63 {
            let offset_z = relative_block_pos.z / 63 * 2 - 1;
            let pos = chunk_pos + ChunkPos::from_i32(0, 0, offset_z);
            vec.push(self.rebuild_mesh(pos, &mut chunks_map));
        }
        vec
    }

    pub fn rebuild_mesh(&self, pos : ChunkPos, chunks_map : &mut HashMap<ChunkPos, Vec<u16>>) -> Option<(ChunkMesh, MeshText)> {
        if self.add_neighbours_if_exist(&pos, chunks_map) == 7 {
            if let Some(meshes) = ChunkMesh::build_mesh(&chunks_map, pos) {
                return Some(meshes);
            }
        }
        None
    }

}

impl Deref for ClientChunkMap {
    type Target = ChunkMap;

    fn deref(&self) -> &Self::Target {
        &self.chunk_map
    }
}