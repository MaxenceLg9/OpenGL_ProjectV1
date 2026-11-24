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
use std::ops::Deref;
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

                let mut in_buffer = buffer.contains_key(chunk_pos);
                // if not in buffer but in the map, adding it
                if !in_buffer && self.chunk_map.contains_chunk(chunk_pos) {
                    buffer.insert(*chunk_pos, ChunkPacket::from_chunk_to_packets(self.chunk_map.get_chunk(chunk_pos).unwrap()).unwrap());
                    in_buffer = true;
                }
                // if the chunk exists (so in the buffer)
                if in_buffer {
                    // creating packets
                    let packets = buffer.get(chunk_pos).expect("Cannot get the chunk packet in the buffer");
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
                                        players_to_remove.push(*player);
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
        Self::cube_range(pos, range, |p| !chunks_generated.contains(p))
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
        (p.x - center.x).abs() <= vd && (p.z - center.z).abs() <= vd
    }

    pub fn interact_block(&mut self, iblock_pos: IBlockPos, block_interaction: BlockInteraction) -> BlockId {
        match block_interaction {
            BlockInteraction::LEFT => self.chunk_map.set_block(iblock_pos, 0),
            BlockInteraction::RIGHT => self.chunk_map.set_block(iblock_pos, 1),
        }
    }
}

impl Default for ServerChunkMap {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for ServerChunkMap {
    type Target = ChunkMap;

    fn deref(&self) -> &Self::Target {
        &self.chunk_map
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::common::network::packet_type::UdpPacketType;
    use shared::common::world::pos::chunkpos::CHUNK_SIZE;
    use std::sync::Arc;

    const WORLD_HEIGHT: usize = (MAX_Y_CHUNK - MIN_Y_CHUNK + 1) as usize;

    fn empty_chunk(pos: ChunkPos) -> Chunk {
        Chunk::new(pos, Arc::new(vec![0; CHUNK_SIZE.pow(3)]))
    }

    /// Reference definition of the loaded area: every chunk within `vd` on x and z, all heights.
    fn cube(center: ChunkPos, vd: i32) -> HashSet<ChunkPos> {
        let mut set = HashSet::new();
        for x in -vd..=vd {
            for z in -vd..=vd {
                for y in MIN_Y_CHUNK..=MAX_Y_CHUNK {
                    set.insert(ChunkPos::new(center.x + x, y, center.z + z));
                }
            }
        }
        set
    }

    fn as_set(v: &[ChunkPos]) -> HashSet<ChunkPos> {
        v.iter().copied().collect()
    }

    // ------------------------------------------------------------ chunk diff

    #[test]
    fn initial_load_sends_the_whole_view_cube() {
        // a negative old view distance means "nothing was loaded before"
        let center = ChunkPos::new(3, 0, -2);
        let (to_load, to_unload) = ServerChunkMap::compute_chunk_diff(center, center, -1, 5);

        assert_eq!(to_load.len(), 11 * 11 * WORLD_HEIGHT);
        assert_eq!(as_set(&to_load), cube(center, 5));
        assert!(to_unload.is_empty());
    }

    #[test]
    fn initial_load_is_sorted_from_the_closest_column() {
        let center = ChunkPos::new(0, 0, 0);
        let (to_load, _) = ServerChunkMap::compute_chunk_diff(center, center, -1, 5);

        let distance = |p: &ChunkPos| (p.x - center.x).pow(2) + (p.z - center.z).pow(2);
        assert!(to_load.windows(2).all(|w| distance(&w[0]) <= distance(&w[1])));
        assert_eq!(distance(&to_load[0]), 0);
        assert!(to_load.contains(&ChunkPos::new(1, -4, 1)));
    }

    #[test]
    fn standing_still_loads_and_unloads_nothing() {
        let center = ChunkPos::new(10, 2, 10);
        let (to_load, to_unload) = ServerChunkMap::compute_chunk_diff(center, center, 4, 4);
        assert!(to_load.is_empty(), "{} chunks re-sent", to_load.len());
        assert!(to_unload.is_empty(), "{} chunks unloaded", to_unload.len());
    }

    #[test]
    fn moving_one_chunk_only_loads_and_unloads_one_slice() {
        let (vd, last, new) = (2, ChunkPos::new(0, 0, 0), ChunkPos::new(1, 0, 0));
        let (to_load, to_unload) = ServerChunkMap::compute_chunk_diff(last, new, vd, vd);

        let expected_load: HashSet<_> = cube(new, vd).difference(&cube(last, vd)).copied().collect();
        let expected_unload: HashSet<_> = cube(last, vd).difference(&cube(new, vd)).copied().collect();
        assert_eq!(as_set(&to_load), expected_load);
        assert_eq!(as_set(&to_unload), expected_unload);
        assert_eq!(to_load.len(), (2 * vd as usize + 1) * WORLD_HEIGHT);
    }

    #[test]
    fn changing_view_distance_loads_only_the_new_ring() {
        let center = ChunkPos::new(-5, 0, 5);
        let (to_load, to_unload) = ServerChunkMap::compute_chunk_diff(center, center, 4, 6);

        let expected: HashSet<_> = cube(center, 6).difference(&cube(center, 4)).copied().collect();
        assert_eq!(as_set(&to_load), expected);
        assert!(to_unload.is_empty());

        // and shrinking it back unloads that same ring
        let (to_load, to_unload) = ServerChunkMap::compute_chunk_diff(center, center, 6, 4);
        assert!(to_load.is_empty());
        assert_eq!(as_set(&to_unload), expected);
    }

    #[test]
    fn compute_chunks_skips_already_generated_positions() {
        let center = ChunkPos::new(0, 0, 0);
        let generated: HashSet<_> = cube(center, 1);
        let to_generate = ServerChunkMap::compute_chunks(center, 2, &generated);

        assert_eq!(to_generate.len(), (5 * 5 - 3 * 3) * WORLD_HEIGHT);
        assert!(to_generate.iter().all(|p| !generated.contains(p)));
    }

    // ------------------------------------------------------- block edits

    #[test]
    fn left_click_breaks_and_right_click_places() {
        let mut map = ServerChunkMap::new();
        map.add_chunk(empty_chunk(ChunkPos::new(0, 0, 0)));
        let pos = IBlockPos::new(5, 6, 7);

        assert_eq!(map.interact_block(pos, BlockInteraction::RIGHT), 1);
        assert_eq!(map.get_block_at(pos), 1);
        assert_eq!(map.interact_block(pos, BlockInteraction::LEFT), 0);
        assert_eq!(map.get_block_at(pos), 0);
    }

    // --------------------------------------------------- chunk streaming

    fn drain(rx: &mut tokio::sync::mpsc::Receiver<(L5Packet, UdpPacketType)>) -> Vec<ChunkPos> {
        let mut received = Vec::new();
        while let Ok((packet, udp_type)) = rx.try_recv() {
            assert_eq!(udp_type, Reliable, "chunks must be sent reliably");
            let L5Packet::Chunk(chunk_packet) = packet else {
                panic!("only chunk packets are expected");
            };
            received.push(chunk_packet.get_chunk_pos());
        }
        received
    }

    #[test]
    fn asked_chunks_are_sent_once_available() {
        let (sx, mut rx) = tokio::sync::mpsc::channel(1000);
        let puid = PUID::new(1);
        let (ready, pending) = (ChunkPos::new(0, 0, 0), ChunkPos::new(1, 0, 0));

        let mut map = ServerChunkMap::new();
        map.add_chunk(empty_chunk(ready));
        map.ask_for_chunks(vec![ready, pending], sx, puid);

        map.poll().unwrap();
        assert_eq!(drain(&mut rx), vec![ready]);

        // the pending chunk stays queued until it gets generated
        map.poll().unwrap();
        assert!(drain(&mut rx).is_empty());

        map.add_chunk(empty_chunk(pending));
        map.poll().unwrap();
        assert_eq!(drain(&mut rx), vec![pending]);
    }

    #[test]
    fn chunks_are_streamed_in_batches() {
        let (sx, mut rx) = tokio::sync::mpsc::channel(10_000);
        let mut map = ServerChunkMap::new();
        let positions: Vec<ChunkPos> = (0..30).map(|x| ChunkPos::new(x, 0, 0)).collect();
        for pos in &positions {
            map.add_chunk(empty_chunk(*pos));
        }
        map.ask_for_chunks(positions.clone(), sx, PUID::new(1));

        let mut received = Vec::new();
        for _ in 0..10 {
            map.poll().unwrap();
            let batch = drain(&mut rx);
            assert!(batch.len() <= 11, "a single poll sent {} chunks", batch.len());
            received.extend(batch);
        }
        assert_eq!(received, positions, "every chunk is sent once, in the asked order");
    }

    #[test]
    fn disconnected_players_are_dropped_from_the_queue() {
        let (sx, rx) = tokio::sync::mpsc::channel(1000);
        let puid = PUID::new(1);
        let mut map = ServerChunkMap::new();
        map.add_chunk(empty_chunk(ChunkPos::new(0, 0, 0)));
        map.ask_for_chunks(vec![ChunkPos::new(0, 0, 0)], sx, puid);
        drop(rx);

        map.poll().unwrap();
        assert!(!map.asking_for_chunks.contains_key(&puid));
    }
}
