use std::sync::{Arc, RwLock};
use crate::client::world_data::chunks::chunk_map::ClientChunkMap;
use crate::client::world_data::mesh_map::MeshMap;
use crate::client::world_data::player::player::ClientPlayer;

pub struct ClientWorldData {
    player: Arc<RwLock<ClientPlayer>>,
    meshes : Arc<RwLock<MeshMap>>,
    chunks : Arc<RwLock<ClientChunkMap>>,
}

impl ClientWorldData {
    pub fn new(cm: Arc<RwLock<ClientChunkMap>>) -> ClientWorldData {
        Self {
            player: Arc::new(RwLock::new(ClientPlayer::new(1.0,1.0,1.0))),
            meshes: Arc::new(RwLock::new(MeshMap::new())),
            chunks: cm.clone(),
        }
    }

    pub fn get_player(&self) -> Arc<RwLock<ClientPlayer>> {
        self.player.clone()
    }

    pub fn get_meshes(&self) -> Arc<RwLock<MeshMap>> {
        self.meshes.clone()
    }

    pub fn get_chunks(&self) -> Arc<RwLock<ClientChunkMap>> {
        self.chunks.clone()
    }
}