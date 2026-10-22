use shared::common::network::client::block_packet::BlockPacket;
use shared::common::network::server::chunk_packet::ChunkPacket;
use shared::common::network::server::connection_packet::ConnectionPacket;

pub enum ClientEventType {
    BlockInteraction(BlockPacket),
    ChunkPacketReceived(ChunkPacket),
    Connection(ConnectionPacket),
}

pub struct ClientEvent {
    pub client_event_type: ClientEventType,
}
