use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::net::{Ipv6Addr, SocketAddrV6};
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
use crossbeam::channel as cb;
use shared::common::network::client::block_packet::BlockPacket;
use shared::common::world::pos::chunkpos::ChunkPos;
use crate::server::network::server_socket::Socket;
use crate::server::world_data::event::events::{InternalEvent, PlayerEvent, ServerEvent};

pub struct ServerWorldData {
    properties : ServerWorldProperties,
    players : Arc<RwLock<HashMap<PUID, Arc<RwLock<ServerPlayer>>>>>,
}

impl ServerWorldData {
    pub fn new() -> Self {
        Self {
            properties : ServerWorldProperties::new("debug".to_string(),Difficulty::Easy),
            players : Arc::new(RwLock::new(HashMap::new())),
        }
    }
    pub fn tick(&self) {
    }
    pub fn get_players(&self) -> Arc<RwLock<HashMap<PUID, Arc<RwLock<ServerPlayer>>>>> {
        self.players.clone()
    }

    pub fn connect_player(&self, puid : PUID, sx : tokio::sync::mpsc::Sender<(L5Packet, UdpPacketType)>) -> Result<(BlockPos,Arc<RwLock<ServerPlayer>>), String> {
        // check if the player is new or already has data
        if false {
            Err(format!("Chut {}", puid))
        } else {
            match self.players.write().unwrap().entry(puid) {
                Entry::Occupied(_) => Err(format!("Player {} already exist", puid)),
                Entry::Vacant(e) => {
                    let pos = BlockPos::from_vec3(Vec3::new(32.0, 160.0, 32.0));
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