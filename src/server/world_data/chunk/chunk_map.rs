use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::iter::FlatMap;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, RwLock};
use shared::common::account::puid::PUID;
use shared::common::network::client::block_packet::BlockInteraction;
use shared::common::network::server::chunk_packet::ChunkPacket;
use shared::common::network::l5_packet::L5Packet;
use shared::common::network::packet_type::UdpPacketType;
use shared::common::network::packet_type::UdpPacketType::Reliable;
use shared::common::world::block::block::BlockType;
use shared::common::world::chunk::chunk::Chunk;
use shared::common::world::chunk::chunkmap::ChunkMap;
use shared::common::world::pos::chunkpos::ChunkPos;
use shared::common::world::pos::iblockpos::IBlockPos;
use shared::print_base;
use crate::server::world_data::player::player::ServerPlayer;

pub struct ServerChunkMap {
    chunk_map: ChunkMap,
    asking_for_chunks : HashMap<PUID, (Vec<ChunkPos>,tokio::sync::mpsc::Sender<(L5Packet, UdpPacketType)>)>
}

impl ServerChunkMap {
    pub fn new() -> Self {
        Self {
            chunk_map: ChunkMap::new(),
            asking_for_chunks : HashMap::new()
        }
    }

    pub fn ask_for_chunks(&mut self, vec_pos: Vec<ChunkPos>, server_player: Arc<RwLock<ServerPlayer>>) {
        let borrow = server_player.read().unwrap();
        match self.asking_for_chunks.entry(borrow.get_puid()) {
            Entry::Occupied(mut e) => {
                e.get_mut().0.extend(vec_pos);
            }
            Entry::Vacant(e) => {
                let mut entry = Vec::new();
                entry.extend(vec_pos);
                e.insert((entry, borrow.get_sender().clone()));
            }
        }
    }

    pub fn add_chunk(&mut self, chunk : Chunk) -> bool {
        self.chunk_map.add_chunk(chunk)
    }

    pub fn poll(&mut self) {
        let mut buffer : HashMap<ChunkPos, Vec<ChunkPacket>> = HashMap::new();
        'players: for (player, (mut vec, sender)) in self.asking_for_chunks.clone() {
            if vec.is_empty() {
                continue
            }

            // iterating over chunk_pos associated to the player
            vec.retain(|chunk_pos| {
                let mut in_buffer = buffer.contains_key(&chunk_pos);
                // if not in buffer but in the map, adding it
                if !in_buffer && self.contains_chunk(&chunk_pos) {
                    buffer.insert(chunk_pos.clone(), ChunkPacket::from_chunk_to_packets(self.get_chunk(&chunk_pos).unwrap()));
                    in_buffer = true;
                }
                // if the chunk exists (so in the buffer)
                if in_buffer {
                    // creating packets
                    let packets = buffer.get(&chunk_pos).unwrap();
                    // sending packets
                    for packet in packets {
                        let server_packet = L5Packet::Chunk(packet.clone());
                        match sender.try_send((server_packet.clone(), Reliable)) {
                            Ok(_) => {}
                            Err(e) => {
                                print_base!("Polling : Channel is full : {}",e.to_string());
                                self.asking_for_chunks.remove(&player);
                                return false;
                            }
                        }
                    }
                    // remove the position because already sent
                    return false;
                }
                true
            });
            self.asking_for_chunks.get_mut(&player).unwrap().0 = vec;
        }
    }

    pub fn compute_chunks(pos:ChunkPos, range: i32, chunks_generated : HashSet<ChunkPos>) -> Vec<ChunkPos> {
        let vec = Self::cube_range(pos, range, |p| !chunks_generated.contains(p));
        vec
    }

    pub fn compute_chunk_diff(last_pos:ChunkPos,new_pos:ChunkPos,old_view_distance: i32,new_view_distance: i32) -> (Vec<ChunkPos> , Vec<ChunkPos>) {
        // A cube is defined as all positions within [-vd, +vd] of a center.
        // (2*vd + 1)^3 total chunks, center always included.
        // let old_range = |pos: ChunkPos| cube_range(pos, old_view_distance);
        let old_range = Self::cube_range(last_pos, old_view_distance, |p| !Self::in_cube(new_pos, new_view_distance, *p));
        // let new_range = |pos: ChunkPos| cube_range(pos, new_view_distance);
        let new_range = Self::cube_range(new_pos, new_view_distance, |p| !Self::in_cube(last_pos, old_view_distance, *p));

        ( new_range, old_range )
    }

    /// Iterator over all ChunkPos in a cube centered on `center` with half-size `vd`.
    fn cube_range<F>(center: ChunkPos, vd: i32, function : F) -> Vec<ChunkPos>
    where F: Fn(&ChunkPos) -> bool,{
        let (cx, cz) = (center.x, center.z);
        let mut vec : Vec<ChunkPos> = (-vd..=vd).flat_map(move |dx|
            (-2..=10).flat_map(move |dy|
                (-vd..=vd).map(move |dz|
                    ChunkPos::from_i32(cx + dx, dy, cz + dz)
                )
            )
        ).filter(function).collect();
        vec.sort_by_key(|p| {
            let relative = *p - center;
            relative.x.abs().pow(2) + relative.z.abs().pow(2)
        });
        vec
    }

    /// O(1) membership test — no allocation, no iteration.
    fn in_cube(center: ChunkPos, vd: i32, p: ChunkPos) -> bool {
        (p.x - center.x).abs() < vd &&
            (p.z - center.z).abs() < vd
    }

    pub fn interact_block(&mut self, iblock_pos: IBlockPos, block_interaction: BlockInteraction) {
        match block_interaction {
            BlockInteraction::LEFT => {
                self.chunk_map.set_block(iblock_pos,BlockType::AIR);
            }
            BlockInteraction::RIGHT => {
                self.chunk_map.set_block(iblock_pos, BlockType::DEEPSLATE);
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