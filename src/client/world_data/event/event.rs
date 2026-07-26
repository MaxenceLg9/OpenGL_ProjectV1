use shared::common::network::client::block_packet::BlockPacket;
use shared::common::network::server::chunk_packet::ChunkPacket;

pub enum ClientEventType {
    BlockInteraction(BlockPacket),
    ChunkPacketReceived(ChunkPacket)
}

pub struct ClientEvent {
    pub client_event_type: ClientEventType
}