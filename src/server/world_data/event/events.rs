use shared::common::account::puid::PUID;
use shared::common::network::client::block_packet::BlockPacket;
use shared::common::world::chunk::chunk::Chunk;
use shared::common::world::player::PacketSender;
use shared::common::world::pos::chunkpos::ChunkPos;
use strum::Display;

#[derive(Display)]
pub enum ServerEvent {
    PlayerEvent { e_type: PlayerEvent, player_sender: PacketSender },
    InternalEvent(InternalEvent),
}
#[derive(Display)]
pub enum PlayerEvent {
    BlockInteraction(BlockPacket),
    AskChunk(Vec<ChunkPos>, PUID),
    GenerateChunk(ChunkPos),
    EntityInteraction(),
    ConnectPlayer(),
    DisconnectPlayer(PUID),
}
#[derive(Display)]
pub enum InternalEvent {
    GeneratedChunk(Chunk),
}
