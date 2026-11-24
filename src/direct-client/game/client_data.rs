use crate::client::game::player::player::ClientPlayer;
use crate::client::generation::mesh::chunk_mesh::ChunkMesh;
use crate::client::gl::gui::text::text::MeshText;
use shared::common::network::l5_packet::L5Packet;
use shared::common::network::packet_type::UdpPacketType;
use shared::common::world::player::PacketSender;
use shared::print_base;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

pub struct ClientWorldData {
    pub player: Arc<RwLock<ClientPlayer>>,
    pub sender: PacketSender,
    pub mesh_sender: crossbeam::channel::Sender<(ChunkMesh, MeshText)>,
    pub debug: AtomicBool,
}

impl ClientWorldData {
    pub fn new(sender: PacketSender, mesh_sender: crossbeam::channel::Sender<(ChunkMesh, MeshText)>) -> ClientWorldData {
        let bool = AtomicBool::new(false);
        bool.store(false, Ordering::Relaxed);
        Self {
            player: Arc::new(RwLock::new(ClientPlayer::new(1.0, 1.0, 1.0))),
            // meshes: Arc::new(RwLock::new(MeshMap::new())),
            sender,
            mesh_sender,
            debug: bool,
        }
    }

    pub fn toggle_debug(&self) {
        self.debug.store(!self.debug.load(Ordering::Relaxed), Ordering::Relaxed);
    }

    pub fn send(&self, l5packet: L5Packet, udp_packet_type: UdpPacketType) {
        if let Err(e) = self.sender.try_send((l5packet, udp_packet_type)) {
            print_base!("Cannot send packet to the network thread: {}", e);
        }
    }

    pub fn get_player(&self) -> Arc<RwLock<ClientPlayer>> {
        self.player.clone()
    }

    // pub fn get_meshes(&self) -> Arc<RwLock<MeshMap>> {
    //     self.meshes.clone()
    // }
}
