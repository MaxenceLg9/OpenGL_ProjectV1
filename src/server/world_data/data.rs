use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::fmt::format;
use std::sync::{Arc, RwLock};
use std::sync::mpsc::Receiver;
use crossbeam::channel;
use glam::Vec3;
use shared::common::account::puid::PUID;
use shared::{print_base, print_debug};
use shared::common::network::server::packet::ServerPacket;
use crate::server::world_data::player::player::ServerPlayer;
use shared::common::world::chunk::chunkmap::ChunkMap;
use shared::common::world::pos::blockpos::BlockPos;
use shared::common::world::pos::chunkpos::ChunkPos;
use crate::server::generation::chunk_generator::ChunkGenerator;
use crate::server::world_data::properties::{Difficulty, ServerWorldProperties};

pub struct ServerWorldData {
    properties : ServerWorldProperties,
    generator : Arc<RwLock<ChunkGenerator>>,
    chunks : Arc<RwLock<ChunkMap>>,
    players : Arc<RwLock<HashMap<PUID, Arc<RwLock<ServerPlayer>>>>>,
}

impl ServerWorldData {
    pub fn new() -> Self {
        let chunk_map = Arc::new(RwLock::new(ChunkMap::new()));
        Self {
            properties : ServerWorldProperties::new("debug".to_string(),Difficulty::Easy),
            chunks : chunk_map.clone(),
            generator : Arc::new(RwLock::new(ChunkGenerator::new(chunk_map))),
            players : Arc::new(RwLock::new(HashMap::new()))
        }
    }
    pub fn get_chunk_map(&self) -> Arc<RwLock<ChunkMap>> {
        self.chunks.clone()
    }
    pub fn get_players(&self) -> Arc<RwLock<HashMap<PUID, Arc<RwLock<ServerPlayer>>>>> {
        self.players.clone()
    }

    pub fn get_generator(&self) -> Arc<RwLock<ChunkGenerator>> {
        self.generator.clone()
    }

    pub fn connect_player(&self, puid : PUID, sx : tokio::sync::mpsc::Sender<ServerPacket>) -> Result<(BlockPos,Arc<RwLock<ServerPlayer>>, HashSet<ChunkPos>) ,String> {
        // check if the player is new or already has data
        if false {
            Err(format!("Chut {}", puid))
        } else {
            match self.players.write().unwrap().entry(puid) {
                Entry::Occupied(_) => Err(format!("Player {} already exist", puid)),
                Entry::Vacant(e) => {
                    let pos = BlockPos::new(Vec3::new(100.0,200.0,100.0));
                    let mut hashset = HashSet::new();
                    for i in 0..20*20*20 {
                        hashset.insert(ChunkPos::from_single_value(i) + (pos.get_chunk_pos() * -1));
                    }
                    print_base!("Created player with {}", puid);
                    let player = Arc::new(RwLock::new(ServerPlayer::new(pos,sx)));
                    e.insert(player.clone());
                    Ok((pos,player,hashset))
                }
            }
        }
    }
    pub fn disconnect_player(&self, puid : &PUID) {
        // check if the player is new or already has data
        self.players.write().unwrap().remove(puid);
        print_base!("Disconnecting player {}", puid);
    }
}