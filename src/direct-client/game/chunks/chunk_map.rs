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
        let chunk_pos = chunk_packet.get_chunk_pos();
        if self.chunk_map.contains_chunk(&chunk_pos) {
            return false;
        }
        let total = chunk_packet.get_total();
        match self.temp_chunks.entry(chunk_packet.get_chunk_pos()) {
            Entry::Occupied(mut e) => {
                e.get_mut().insert(chunk_packet.get_indice(), chunk_packet);
            }
            Entry::Vacant(e) => {
                let mut submap = HashMap::new();
                submap.insert(chunk_packet.get_indice(), chunk_packet);
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
                chunks.insert(*p, result.unwrap().get_blocks().clone());
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

#[cfg(test)]
mod tests {
    use super::*;
    use shared::common::world::chunk::chunk::Chunk;
    use shared::common::world::pos::chunkpos::CHUNK_SIZE;
    use std::collections::HashSet;

    fn new_map() -> ClientChunkMap {
        ClientChunkMap::new()
    }

    fn packets(pos: ChunkPos, fill: impl Fn(usize) -> u16) -> Vec<ChunkPacket> {
        let chunk = Chunk::new(pos, Arc::new((0..CHUNK_SIZE.pow(3)).map(fill).collect()));
        ChunkPacket::from_chunk_to_packets(&chunk).unwrap()
    }

    #[test]
    fn chunk_is_added_only_when_every_part_arrived() {
        let pos = ChunkPos::new(0, 0, 0);
        // pseudo-random content (splitmix64 hash) so the chunk needs several packets
        let mut parts = packets(pos, |i| {
            let mut z = (i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            ((z ^ (z >> 31)) % 8) as u16
        });
        assert!(parts.len() > 1);
        let last = parts.pop().unwrap();

        let mut map = new_map();
        for part in parts {
            assert!(!map.add_temp_chunk(part));
            assert!(!map.contains_chunk(&pos));
        }
        assert!(map.add_temp_chunk(last));
        assert!(map.contains_chunk(&pos));
        assert!(map.temp_chunks.is_empty(), "temporary parts are freed");
    }

    #[test]
    fn packets_for_an_already_loaded_chunk_are_ignored() {
        let pos = ChunkPos::new(1, 1, 1);
        let mut map = new_map();
        for part in packets(pos, |_| 1) {
            map.add_temp_chunk(part);
        }
        for part in packets(pos, |_| 2) {
            assert!(!map.add_temp_chunk(part));
        }
        assert_eq!(map.get_block_at(pos + IBlockPos::new(0, 0, 0)), 1);
        assert!(map.temp_chunks.is_empty());
    }

    #[test]
    fn neighbours_count_includes_missing_chunks_outside_the_world() {
        let mut map = new_map();
        let top = ChunkPos::new(0, MAX_Y_CHUNK, 0);
        // only the 18 chunks inside the world exist; the 9 above are out of the world
        for pos in ChunkMap::get_neighbours_chunks_pos(top).into_iter().filter(|p| p.y <= MAX_Y_CHUNK) {
            for part in packets(pos, |_| 0) {
                map.add_temp_chunk(part);
            }
        }
        let mut chunks = HashMap::new();
        assert_eq!(map.add_neighbours_if_exist(top, &mut chunks), 27);
        assert_eq!(chunks.len(), 18);

        map.remove_chunk(&ChunkPos::new(1, MAX_Y_CHUNK, 1));
        let mut chunks = HashMap::new();
        assert_eq!(map.add_neighbours_if_exist(top, &mut chunks), 26, "a missing in-world chunk blocks meshing");
    }

    #[test]
    fn editing_a_border_block_remeshes_the_neighbour_chunks() {
        let mut map = new_map();
        let center = ChunkPos::new(0, 0, 0);
        let mut area = HashSet::new();
        for pos in ChunkMap::get_neighbours_chunks_pos(center) {
            area.extend(ChunkMap::get_neighbours_chunks_pos(pos));
        }
        for pos in area {
            for part in packets(pos, |_| 0) {
                map.add_temp_chunk(part);
            }
        }

        let forget = |meshes: Vec<Option<(ChunkMesh, MeshText)>>| -> Vec<Option<ChunkPos>> {
            meshes
                .into_iter()
                .map(|m| {
                    m.map(|(mesh, text)| {
                        std::mem::forget(text); // `MeshText::drop` needs OpenGL
                        mesh.get_chunk_pos()
                    })
                })
                .collect()
        };

        // inside the chunk: only this chunk is rebuilt
        assert_eq!(forget(map.set_block(IBlockPos::new(10, 10, 10), 1)), vec![Some(center)]);

        // on the x = 0 border: the -x chunk is rebuilt too (it has nothing to draw, hence None)
        let rebuilt = forget(map.set_block(IBlockPos::new(0, 10, 10), 1));
        assert_eq!(rebuilt, vec![Some(center), None]);

        // on a corner: 3 neighbours are concerned
        assert_eq!(forget(map.set_block(IBlockPos::new(63, 63, 63), 1)).len(), 4);
    }
}
