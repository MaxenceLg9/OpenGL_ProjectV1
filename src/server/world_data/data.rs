use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::format;
use std::sync::{Arc, RwLock};
use std::sync::mpsc::Receiver;
use crossbeam::channel;
use shared::common::account::puid::PUID;
use shared::{print_base, print_debug};
use crate::server::world_data::player::player::ServerPlayer;
use shared::common::world::chunk::chunkmap::ChunkMap;
use crate::server::world_data::properties::{Difficulty, ServerWorldProperties};

pub struct ServerWorldData {
    properties : ServerWorldProperties,
    chunks : Arc<RwLock<ChunkMap>>,
    players : Arc<RwLock<HashMap<PUID, ServerPlayer>>>,
}

impl ServerWorldData {
    pub fn new() -> Self {
        Self {
            properties : ServerWorldProperties::new("debug".to_string(),Difficulty::Easy),
            chunks : Arc::new(RwLock::new(ChunkMap::new())),
            players : Arc::new(RwLock::new(HashMap::new()))
        }
    }
    pub fn get_chunk_map(&self) -> Arc<RwLock<ChunkMap>> {
        self.chunks.clone()
    }
    pub fn get_players(&self) -> Arc<RwLock<HashMap<PUID,ServerPlayer>>> {
        self.players.clone()
    }
    pub fn connect_player(&self, puid : PUID) -> Result<tokio::sync::mpsc::Receiver<Vec<u8>>,String> {
        // check if the player is new or already has data
        if false {
            Err(format!("Chut {}", puid))
        } else {

            match self.players.write().unwrap().entry(puid) {
                Entry::Occupied(_) => Err(format!("Player {} already exist", puid)),
                Entry::Vacant(e) => {
                    let (sx, rx) = tokio::sync::mpsc::channel(10000);
                    let player = ServerPlayer::new(1.0,1.0,1.0,sx);
                    // let arc_player = Arc::new(player);
                    e.insert(player);
                    Ok(rx)
                }
            }
        }
    }
    pub fn disconnect_player(&self, puid : &PUID) {
        // check if the player is new or already has data
        self.players.write().unwrap().remove(puid);
    }
}