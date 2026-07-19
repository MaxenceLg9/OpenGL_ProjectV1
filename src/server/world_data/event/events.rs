use std::sync::{Arc, RwLock};
use shared::common::network::client::block_packet::BlockPacket;
use shared::common::world::pos::chunkpos::ChunkPos;
use crate::server::world_data::player::player::ServerPlayer;

pub enum EventType {
    BlockInteraction(BlockPacket),
    AskChunk(Vec<ChunkPos>),
    GenerateChunk(ChunkPos),
    EntityInteraction(),
    ConnectPlayer(),
    DisconnectPlayer()
}

pub struct Event {
    pub player : Arc<RwLock<ServerPlayer>>,
    pub event_type: EventType
}