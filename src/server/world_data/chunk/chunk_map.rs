use shared::common::account::puid::PUID;
use shared::common::error::err_type::GameError;
use shared::common::network::client::block_packet::BlockInteraction;
use shared::common::network::l5_packet::L5Packet;
use shared::common::network::packet_type::UdpPacketType::Reliable;
use shared::common::network::server::chunk_packet::ChunkPacket;
use shared::common::world::block::block::BlockId;
use shared::common::world::chunk::chunk::Chunk;
use shared::common::world::chunk::chunkmap::{ChunkMap, MAX_Y_CHUNK, MIN_Y_CHUNK};
use shared::common::world::player::PacketSender;
use shared::common::world::pos::chunkpos::ChunkPos;
use shared::common::world::pos::iblockpos::IBlockPos;
use shared::print_base;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};
use tokio::sync::mpsc::error::TrySendError;

pub struct ServerChunkMap {
    chunk_map: ChunkMap,
    asking_for_chunks: HashMap<PUID, (Vec<ChunkPos>, PacketSender)>,
}

impl ServerChunkMap {
    pub fn new() -> Self {
        Self {
            chunk_map: ChunkMap::new(),
            asking_for_chunks: HashMap::new(),
        }
    }

    pub fn remove_player_from_asking(&mut self, player: &PUID) {
        self.asking_for_chunks.remove(player);
    }

    pub fn ask_for_chunks(&mut self, vec_pos: Vec<ChunkPos>, player_sender: PacketSender, puid: PUID) {
        match self.asking_for_chunks.entry(puid) {
            Entry::Occupied(mut e) => {
                e.get_mut().0.extend(vec_pos);
            }
            Entry::Vacant(e) => {
                let mut entry = Vec::new();
                entry.extend(vec_pos);
                e.insert((entry, player_sender));
            }
        }
    }

    pub fn add_chunk(&mut self, chunk: Chunk) -> bool {
        self.chunk_map.add_chunk(chunk)
    }

    pub fn iter_send_chunks(&mut self) -> Result<Vec<PUID>, GameError> {
        let mut buffer: HashMap<ChunkPos, Vec<ChunkPacket>> = HashMap::new();
        let mut players_to_remove = Vec::new();

        for (player, (vec, sender)) in self.asking_for_chunks.iter_mut() {
            if vec.is_empty() {
                continue;
            }

            // iterating over chunk_pos associated to the player
            let mut i = 0;
            let mut flag = false;
            vec.retain(|chunk_pos| {
                if flag {
                    return false;
                }
                if i > 10 {
                    return true;
                }

                let mut in_buffer = buffer.contains_key(&chunk_pos);
                // if not in buffer but in the map, adding it
                if !in_buffer && self.chunk_map.contains_chunk(&chunk_pos) {
                    buffer.insert(chunk_pos.clone(), ChunkPacket::from_chunk_to_packets(self.chunk_map.get_chunk(&chunk_pos).unwrap()).unwrap());
                    in_buffer = true;
                }
                // if the chunk exists (so in the buffer)
                if in_buffer {
                    // creating packets
                    let packets = buffer.get(&chunk_pos).expect("Cannot get the chunk packet in the buffer");
                    // sending packets
                    for packet in packets {
                        let server_packet = L5Packet::Chunk(packet.clone());
                        match sender.try_send((server_packet.clone(), Reliable)) {
                            Ok(()) => {}
                            Err(e) => {
                                flag = true;
                                match e {
                                    TrySendError::Full(_) => {
                                        print_base!("Channel is full, skipping...")
                                    }
                                    TrySendError::Closed(_) => {
                                        print_base!("Channel is closed, removing the player : {}", e.to_string());
                                        players_to_remove.push(player.clone());
                                    }
                                }
                                return false;
                            }
                        }
                    }
                    i += 1;
                    // remove the position because already sent
                    return false;
                }
                true
            });
        }
        Ok(players_to_remove)
    }

    pub fn poll(&mut self) -> Result<(), GameError> {
        let players_to_remove = self.iter_send_chunks()?;
        for player in players_to_remove.iter() {
            self.asking_for_chunks.remove(player);
        }
        Ok(())
    }

    pub fn compute_chunks(pos: ChunkPos, range: i32, chunks_generated: &HashSet<ChunkPos>) -> Vec<ChunkPos> {
        let vec = Self::cube_range(pos, range, |p| !chunks_generated.contains(p));
        vec
    }

    pub fn compute_chunk_diff(last_pos: ChunkPos, new_pos: ChunkPos, old_view_distance: i32, new_view_distance: i32) -> (Vec<ChunkPos>, Vec<ChunkPos>) {
        // A cube is defined as all positions within [-vd, +vd] of a center.
        // (2*vd + 1)^3 total chunks, center always included.
        // let old_range = |pos: ChunkPos| cube_range(pos, old_view_distance);
        let old_range = Self::cube_range(last_pos, old_view_distance, |p| !Self::in_cube(new_pos, new_view_distance, *p));
        // let new_range = |pos: ChunkPos| cube_range(pos, new_view_distance);
        let new_range = Self::cube_range(new_pos, new_view_distance, |p| !Self::in_cube(last_pos, old_view_distance, *p));

        (new_range, old_range)
    }

    /// Iterator over all ChunkPos in a cube centered on `center` with half-size `vd`.
    fn cube_range<F>(center: ChunkPos, vd: i32, function: F) -> Vec<ChunkPos>
    where
        F: Fn(&ChunkPos) -> bool,
    {
        let (cx, cz) = (center.x, center.z);
        let mut vec: Vec<ChunkPos> = (-vd..=vd)
            .flat_map(move |dx| (MIN_Y_CHUNK..=MAX_Y_CHUNK).flat_map(move |dy| (-vd..=vd).map(move |dz| ChunkPos::new(cx + dx, dy, cz + dz))))
            .filter(function)
            .collect();
        vec.sort_by_key(|p| {
            let relative = *p - center;
            relative.x.abs().pow(2) + relative.z.abs().pow(2)
        });
        vec
    }

    /// O(1) membership test — no allocation, no iteration.
    fn in_cube(center: ChunkPos, vd: i32, p: ChunkPos) -> bool {
        (p.x - center.x).abs() < vd && (p.z - center.z).abs() < vd
    }

    pub fn interact_block(&mut self, iblock_pos: IBlockPos, block_interaction: BlockInteraction) -> BlockId {
        match block_interaction {
            BlockInteraction::LEFT => self.chunk_map.set_block(iblock_pos, 0),
            BlockInteraction::RIGHT => self.chunk_map.set_block(iblock_pos, 1),
        }
    }
}

impl Deref for ServerChunkMap {
    type Target = ChunkMap;

    fn deref(&self) -> &Self::Target {
        &self.chunk_map
    }
}
