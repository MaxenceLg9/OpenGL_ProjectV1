use std::sync::{Arc, RwLock};
use strum::Display;
use shared::common::account::puid::PUID;
use shared::common::network::client::block_packet::BlockPacket;
use shared::common::world::chunk::chunk::Chunk;
use shared::common::world::pos::chunkpos::ChunkPos;
use crate::server::world_data::player::player::ServerPlayer;

#[derive(Display)]
pub enum ServerEvent {
    PlayerEvent{e_type : PlayerEvent, player : Arc<RwLock<ServerPlayer>>},
    InternalEvent(InternalEvent)
}
#[derive(Display)]
pub enum PlayerEvent {
    BlockInteraction(BlockPacket),
    AskChunk(Vec<ChunkPos>),
    GenerateChunk(ChunkPos),
    EntityInteraction(),
    ConnectPlayer(),
    DisconnectPlayer(PUID)
}
#[derive(Display)]
pub enum InternalEvent {
    GeneratedChunk(Chunk),
}