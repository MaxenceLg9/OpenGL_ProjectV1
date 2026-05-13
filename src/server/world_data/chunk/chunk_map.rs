use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc::error::TrySendError;
use shared::common::network::server::chunk_packet::ChunkPacket;
use shared::common::network::default_packet::ServerPacket;
use shared::common::world::chunk::chunk::Chunk;
use shared::common::world::chunk::chunkmap::ChunkMap;
use shared::common::world::pos::chunkpos::ChunkPos;
use shared::print_base;
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
                        match player.read().unwrap().send_packet(server_packet.clone()) {
                            Ok(_) => {},
                            Err(e) => {
                                print_base!("Error {}",e)
                            }
                        };
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
                            match player.read().unwrap().send_packet(server_packet.clone()) {
                                Ok(_) => {}
                                Err(e) => {
                                    print_base!("Channel is full : {}",e.to_string());
                                }
                            }
                        }
                    }
                    self.asking_for_chunks.remove(&chunk.get_chunk_pos());
                }
            }
        }
    }

    pub fn compute_chunk_diff(
        last_pos:          ChunkPos,
        new_pos:           ChunkPos,
        old_view_distance: i32,
        new_view_distance: i32,
    ) -> (Vec<ChunkPos> , Vec<ChunkPos>) {
        // A cube is defined as all positions within [-vd, +vd] of a center.
        // (2*vd + 1)^3 total chunks, center always included.
        // let old_range = |pos: ChunkPos| cube_range(pos, old_view_distance);
        let old_range = Self::cube_range(last_pos, old_view_distance);
        // let new_range = |pos: ChunkPos| cube_range(pos, new_view_distance);
        let new_range = Self::cube_range(new_pos, new_view_distance);

        let to_load = new_range
            .filter(|p| !Self::in_cube(last_pos, old_view_distance, *p))
            .collect();

        let to_unload = old_range
            .filter(|p| !Self::in_cube(new_pos, new_view_distance, *p))
            .collect();

        ( to_load, to_unload )
    }

    /// Iterator over all ChunkPos in a cube centered on `center` with half-size `vd`.
    fn cube_range(center: ChunkPos, vd: i32) -> impl Iterator<Item = ChunkPos> {
        let (cx, cy, cz) = (center.x, center.y, center.z);
        (-vd..=vd).flat_map(move |dx|
            (-vd..=vd).flat_map(move |dy|
                (-vd..=vd).map(move |dz|
                    ChunkPos::from_i32(cx + dx, cy + dy, cz + dz)
                )
            )
        )
    }

    /// O(1) membership test — no allocation, no iteration.
    fn in_cube(center: ChunkPos, vd: i32, p: ChunkPos) -> bool {
        (p.x - center.x).abs() <= vd &&
            (p.y - center.y).abs() <= vd &&
            (p.z - center.z).abs() <= vd
    }
}

impl Deref for ServerChunkMap {
    type Target = ChunkMap;

    fn deref(&self) -> &Self::Target {
        &self.chunk_map
    }
}