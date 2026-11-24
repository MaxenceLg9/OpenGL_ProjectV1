use crate::client::generation::mesh::chunk_mesh::ChunkMesh;
use crate::client::gl::gui::text::text::MeshText;
use shared::common::network::server::chunk_packet::ChunkPacket;
use shared::common::world::chunk::chunkmap::{ChunkMap, MAX_Y_CHUNK, MIN_Y_CHUNK};
use shared::common::world::pos::chunkpos::ChunkPos;
use shared::common::world::pos::iblockpos::IBlockPos;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::ops::Deref;
use std::sync::Arc;

pub struct ClientChunkMap {
    chunk_map: ChunkMap,
    temp_chunks: HashMap<ChunkPos, HashMap<u8, ChunkPacket>>,
}

impl ClientChunkMap {
    pub fn new() -> ClientChunkMap {
        Self {
            chunk_map: ChunkMap::new(),
            temp_chunks: HashMap::new(),
        }
    }

    pub fn remove_chunk(&mut self, chunk_pos: &ChunkPos) {
        self.chunk_map.remove_chunk(chunk_pos);
    }

    /// Add the slice of the Chunk contained in the packet to a temporary map
    /// If enough temporary chunks, instead instantiate the chunk and add it to the map
    pub fn add_temp_chunk(&mut self, chunk_packet: ChunkPacket) -> bool {
        let total = chunk_packet.get_total();
        let chunk_pos = chunk_packet.get_chunk_pos();
        if self.chunk_map.contains_chunk(&chunk_pos) {
            return false;
        }
        match self.temp_chunks.entry(chunk_packet.get_chunk_pos()) {
            Entry::Occupied(mut e) => {
                e.get_mut().insert(chunk_packet.get_indice(), chunk_packet.clone());
            }
            Entry::Vacant(e) => {
                let mut submap = HashMap::new();
                submap.insert(chunk_packet.get_indice(), chunk_packet.clone());
                e.insert(submap);
            }
        }
        if self.temp_chunks.get(&chunk_pos).unwrap().len() as u8 == total {
            let c = ChunkPacket::from_packets_to_chunk(self.temp_chunks.remove(&chunk_pos).expect("Error when getting"), chunk_pos);
            // self.client_world_data.get_chunks().write().unwrap().add_chunk(c.clone());
            return self.chunk_map.add_chunk(c);
        }
        false
    }

    pub fn add_neighbours_if_exist(&self, chunk_pos: ChunkPos, chunks: &mut HashMap<ChunkPos, Arc<Vec<u16>>>) -> u8 {
        let mut count = 0;
        let pos_vec = ChunkMap::get_neighbours_chunks_pos(chunk_pos);

        // checking every side of the chunk
        for p in pos_vec.iter() {
            // the position is already in the map, no need to check if it exists
            if !chunks.contains_key(p) && p.y >= MIN_Y_CHUNK && p.y <= MAX_Y_CHUNK {
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

    pub fn set_block(&mut self, iblock_pos: IBlockPos, block_type: u16) -> Vec<Option<(ChunkMesh, MeshText)>> {
        let chunk_pos = iblock_pos.get_chunk_pos();
        if self.chunk_map.set_block(iblock_pos, block_type) == block_type {
            return self.redraw_chunks(chunk_pos, iblock_pos.get_block_pos());
        }
        Vec::new()
    }

    fn redraw_chunks(&self, chunk_pos: ChunkPos, relative_block_pos: IBlockPos) -> Vec<Option<(ChunkMesh, MeshText)>> {
        let mut vec = Vec::new();
        let mut chunks_map = HashMap::new();

        vec.push(self.rebuild_mesh(chunk_pos, &mut chunks_map));
        if relative_block_pos.x == 0 || relative_block_pos.x == 63 {
            let offset_x = relative_block_pos.x / 63 * 2 - 1;
            let pos = chunk_pos + ChunkPos::new(offset_x, 0, 0);
            vec.push(self.rebuild_mesh(pos, &mut chunks_map));
        }
        if relative_block_pos.y == 0 || relative_block_pos.y == 63 {
            let offset_y = relative_block_pos.y / 63 * 2 - 1;
            let pos = chunk_pos + ChunkPos::new(0, offset_y, 0);
            vec.push(self.rebuild_mesh(pos, &mut chunks_map));
        }
        if relative_block_pos.z == 0 || relative_block_pos.z == 63 {
            let offset_z = relative_block_pos.z / 63 * 2 - 1;
            let pos = chunk_pos + ChunkPos::new(0, 0, offset_z);
            vec.push(self.rebuild_mesh(pos, &mut chunks_map));
        }
        vec
    }

    pub fn rebuild_mesh(&self, pos: ChunkPos, chunks_map: &mut HashMap<ChunkPos, Arc<Vec<u16>>>) -> Option<(ChunkMesh, MeshText)> {
        if self.add_neighbours_if_exist(pos, chunks_map) == 27
            && let Some(meshes) = ChunkMesh::build_mesh(chunks_map, pos)
        {
            return Some(meshes);
        }
        None
    }
}

impl Default for ClientChunkMap {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for ClientChunkMap {
    type Target = ChunkMap;

    fn deref(&self) -> &Self::Target {
        &self.chunk_map
    }
}
