use std::collections::HashMap;
use std::sync::Arc;
use shared::common::account::puid::PUID;
use shared::{print_base, print_debug};
use crate::server::world_data::player::player::ServerPlayer;
use crate::server::world_data::chunk::chunkmap::ChunkMap;
use crate::server::world_data::properties::{Difficulty, ServerWorldProperties};

pub struct ServerWorldData {
    properties : ServerWorldProperties,
    chunks : ChunkMap,
    players : HashMap<PUID, Arc<ServerPlayer>>
}

impl ServerWorldData {
    pub fn new() -> Self {
        Self {
            properties : ServerWorldProperties::new("debug".to_string(),Difficulty::Easy),
            chunks : ChunkMap::new(),
            players : HashMap::new()
        }
    }
    pub fn get_chunk_map(&self) -> &ChunkMap {
        &self.chunks
    }
    

    pub fn get_mut_chunk_map(&mut self) -> &mut ChunkMap {
        &mut self.chunks
    }

    pub fn connect_player(&mut self, puid : PUID) {
        // check if the player is new or already has data
        if false {

        } else {
            let player = Arc::new(ServerPlayer::new(1.0,1.0,1.0));
            print_base!("Connection initialized from {}", puid);
            self.players.insert(puid, player);
        }
    }

    pub fn disconnect_player(&mut self, puid : &PUID) {
        // check if the player is new or already has data
        self.players.remove(puid);
    }
}