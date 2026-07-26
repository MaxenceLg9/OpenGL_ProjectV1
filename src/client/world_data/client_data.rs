use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use shared::common::network::l5_packet::L5Packet;
use shared::common::network::packet_type::UdpPacketType;
use crate::client::display::renderer::gui::text::text::MeshText;
use crate::client::generation::mesh::chunk_mesh::ChunkMesh;
use crate::client::world_data::chunks::chunk_map::ClientChunkMap;
use crate::client::world_data::mesh_map::MeshMap;
use crate::client::world_data::player::player::ClientPlayer;

pub struct ClientWorldData {
    pub player: Arc<RwLock<ClientPlayer>>,
    // meshes : Arc<RwLock<MeshMap>>,
    pub sender: tokio::sync::mpsc::Sender<(L5Packet, UdpPacketType)>,
    pub mesh_sender : crossbeam::channel::Sender<(ChunkMesh, MeshText)>,
    pub debug : AtomicBool
}

impl ClientWorldData {
    pub fn new(sender: tokio::sync::mpsc::Sender<(L5Packet, UdpPacketType)>, mesh_sender : crossbeam::channel::Sender<(ChunkMesh, MeshText)>) -> ClientWorldData {
        let bool = AtomicBool::new(false);
        bool.store(false,Ordering::Relaxed);
        Self {
            player: Arc::new(RwLock::new(ClientPlayer::new(1.0,1.0,1.0))),
            // meshes: Arc::new(RwLock::new(MeshMap::new())),
            sender,
            mesh_sender,
            debug: bool
        }
    }

    pub fn toggle_debug(&self) {
        self.debug.store(!self.debug.load(Ordering::Relaxed),Ordering::Relaxed);
    }

    pub fn send(&self, l5packet: L5Packet, udp_packet_type: UdpPacketType) {
        self.sender.try_send((l5packet, udp_packet_type));
    }

    pub fn get_player(&self) -> Arc<RwLock<ClientPlayer>> {
        self.player.clone()
    }

    // pub fn get_meshes(&self) -> Arc<RwLock<MeshMap>> {
    //     self.meshes.clone()
    // }
}