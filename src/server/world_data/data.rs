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
use crossbeam::channel as cb;
use shared::common::network::client::block_packet::BlockPacket;
use shared::common::world::pos::chunkpos::ChunkPos;
use crate::server::world_data::event::events::{Event, EventType};

pub struct ServerWorldData {
    properties : ServerWorldProperties,
    event_rx: cb::Receiver<Event>,
    generator : Arc<RwLock<ChunkGenerator>>,
    chunks : Arc<RwLock<ServerChunkMap>>,
    players : Arc<RwLock<HashMap<PUID, Arc<RwLock<ServerPlayer>>>>>,
}

impl ServerWorldData {
    pub fn new(event_rx: cb::Receiver<Event>) -> Self {
        let chunk_map = Arc::new(RwLock::new(ServerChunkMap::new()));
        Self {
            properties : ServerWorldProperties::new("debug".to_string(),Difficulty::Easy),
            chunks : chunk_map.clone(),
            event_rx,
            generator : Arc::new(RwLock::new(ChunkGenerator::new(chunk_map, 1))),
            players : Arc::new(RwLock::new(HashMap::new()))
        }
    }
    pub fn get_chunk_map(&self) -> Arc<RwLock<ServerChunkMap>> {
        self.chunks.clone()
    }

    pub fn poll(&self) {
        self.handle_events();
        self.chunks.write().unwrap().poll();
    }

    fn handle_events(&self) {
        while let Ok(event) = self.event_rx.try_recv() {
            match event.event_type {
                EventType::BlockInteraction(p) => {
                    self.get_chunk_map().write().unwrap().interact_block(p.get_pos(), p.get_interaction_type());
                    event.player.write().unwrap().send_packet(L5Packet::Block(BlockPacket::new(p.get_interaction_type(), p.get_pos())), UdpPacketType::Reliable);
                }
                EventType::AskChunk(vec_pos) => {
                    self.get_chunk_map().write().unwrap().ask_for_chunks(vec_pos,event.player.clone());
                }
                EventType::GenerateChunk(chunk_pos) => {
                    let array: Vec<ChunkPos> = ServerChunkMap::compute_chunks(chunk_pos, 8, self.get_generator().read().unwrap().get_chunks_generated());
                    self.get_generator().write().unwrap().schedule_chunks(array);
                }
                EventType::EntityInteraction() => {}
            }
        }
    }

    pub fn tick(&self) {
    }
    pub fn get_players(&self) -> Arc<RwLock<HashMap<PUID, Arc<RwLock<ServerPlayer>>>>> {
        self.players.clone()
    }

    pub fn get_generator(&self) -> Arc<RwLock<ChunkGenerator>> {
        self.generator.clone()
    }

    pub fn connect_player(&self, puid : PUID, sx : tokio::sync::mpsc::Sender<(L5Packet, UdpPacketType)>) -> Result<(BlockPos,Arc<RwLock<ServerPlayer>>), String> {
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