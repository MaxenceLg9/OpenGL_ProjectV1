use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use glam::Vec3;
use shared::common::account::puid::PUID;
use shared::{print_base, print_debug};
use shared::common::network::l5_packet::L5Packet;
use shared::common::network::packet_type::UdpPacketType;
use crate::server::world_data::player::player::ServerPlayer;
use shared::common::world::pos::blockpos::BlockPos;
use crate::server::generation::chunk_generator::ChunkGenerator;
use crate::server::world_data::chunk::chunk_map::ServerChunkMap;
use crate::server::world_data::properties::{Difficulty, ServerWorldProperties};

pub struct ServerWorldData {
    properties : ServerWorldProperties,
    generator : Arc<RwLock<ChunkGenerator>>,
    chunks : Arc<RwLock<ServerChunkMap>>,
    players : Arc<RwLock<HashMap<PUID, Arc<RwLock<ServerPlayer>>>>>,
}

impl ServerWorldData {
    pub fn new() -> Self {
        let chunk_map = Arc::new(RwLock::new(ServerChunkMap::new()));
        Self {
            properties : ServerWorldProperties::new("debug".to_string(),Difficulty::Easy),
            chunks : chunk_map.clone(),
            generator : Arc::new(RwLock::new(ChunkGenerator::new(chunk_map, 1))),
            players : Arc::new(RwLock::new(HashMap::new()))
        }
    }
    pub fn get_chunk_map(&self) -> Arc<RwLock<ServerChunkMap>> {
        self.chunks.clone()
    }

    pub fn tick(&self) {
        self.chunks.write().unwrap().tick();
    }
    pub fn get_players(&self) -> Arc<RwLock<HashMap<PUID, Arc<RwLock<ServerPlayer>>>>> {
        self.players.clone()
    }

    pub fn get_generator(&self) -> Arc<RwLock<ChunkGenerator>> {
        self.generator.clone()
    }

    pub fn connect_player(&self, puid : PUID, sx : tokio::sync::mpsc::Sender<(L5Packet, UdpPacketType)>) -> Result<(BlockPos,Arc<RwLock<ServerPlayer>>) ,String> {
        // check if the player is new or already has data
        if false {
            Err(format!("Chut {}", puid))
        } else {
            match self.players.write().unwrap().entry(puid) {
                Entry::Occupied(_) => Err(format!("Player {} already exist", puid)),
                Entry::Vacant(e) => {
                    let pos = BlockPos::new(Vec3::new(32.0,160.0,32.0));
                    print_base!("Created player with {}", puid);
                    let player = Arc::new(RwLock::new(ServerPlayer::new(pos,sx, puid)));
                    e.insert(player.clone());
                    Ok((pos,player))
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