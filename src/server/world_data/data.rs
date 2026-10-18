use crate::server::world_data::player::player::ServerPlayer;
use crate::server::world_data::properties::{Difficulty, ServerWorldProperties};
use glam::Vec3;
use mlua::Lua;
use shared::common::account::puid::PUID;
use shared::common::network::l5_packet::L5Packet;
use shared::common::network::packet_type::UdpPacketType;
use shared::common::world::block::block::{BlockState, BlockTier, BlocksRegistries};
use shared::common::world::pos::blockpos::BlockPos;
use shared::{print_base, print_debug};
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::io::{Error, ErrorKind};
use std::net::{Ipv6Addr, SocketAddrV6};
use std::sync::{Arc, RwLock};

pub struct ServerWorldData {
    properties: ServerWorldProperties,
    players: Arc<RwLock<HashMap<PUID, Arc<RwLock<ServerPlayer>>>>>,
    registries: Arc<HashMap<String, BlocksRegistries>>,
}

impl ServerWorldData {
    pub fn new() -> Self {
        let lua = Lua::new();
        let env = lua.globals();
        let table = lua.create_table().unwrap();
        table.set("new", lua.create_function(|_, (tier, is_solid): (BlockTier, bool)| Ok(BlockState::new(tier, is_solid))).unwrap()).unwrap();
        env.set("BlockState", table).unwrap();
        let chunk = lua
            .load(std::fs::read_to_string("/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/lua/blocks.lua").unwrap())
            .set_environment(env.clone())
            .into_function()
            .unwrap();
        let map = chunk.call::<HashMap<String, HashMap<String, BlockState>>>(()).unwrap();
        let registers = HashMap::from([("blocks".to_owned(), BlocksRegistries::from_map(map))]);
        Self {
            properties: ServerWorldProperties::new("debug".to_string(), Difficulty::Easy),
            players: Arc::new(RwLock::new(HashMap::new())),
            registries: Arc::new(registers),
        }
    }
    pub fn tick(&self) {}
    pub fn get_players(&self) -> Arc<RwLock<HashMap<PUID, Arc<RwLock<ServerPlayer>>>>> {
        self.players.clone()
    }

    pub fn get_registries(&self) -> Arc<HashMap<String, BlocksRegistries>> {
        self.registries.clone()
    }

    pub fn connect_player(&self, puid: PUID, sx: tokio::sync::mpsc::Sender<(L5Packet, UdpPacketType)>) -> Result<(BlockPos, Arc<RwLock<ServerPlayer>>), Error> {
        // check if the player is new or already has data
        if false {
            Err(Error::new(ErrorKind::InvalidData, "I don't know"))
        } else {
            match self.players.write().unwrap().entry(puid) {
                Entry::Occupied(_) => Err(Error::new(ErrorKind::InvalidInput, format!("Player {} already exist", puid))),
                Entry::Vacant(e) => {
                    let pos = BlockPos::from_vec3(Vec3::new(32.0, 160.0, 32.0));
                    print_base!("Created player with {}", puid);
                    let player = Arc::new(RwLock::new(ServerPlayer::new(pos, sx, puid)));
                    e.insert(player.clone());
                    Ok((pos, player))
                }
            }
        }
    }
    pub fn disconnect_player(&self, puid: &PUID) {
        // check if the player is new or already has data
        self.players.write().unwrap().remove(puid);
        print_base!("Disconnecting player {}", puid);
    }
}
