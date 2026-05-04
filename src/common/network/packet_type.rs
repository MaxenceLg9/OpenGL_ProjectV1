use bitvec::vec::BitVec;
use strum::{Display, FromRepr};
use crate::common::network::client::ask_chunk::AskChunkPacket;
use crate::common::network::client::login_packet::LoginPacket;
use crate::common::network::client::packet::ClientPacket;
use crate::common::network::server::chunk_packet::ChunkPacket;
use crate::common::network::server::tick_player::GetPlayerPacket;

#[derive(Clone, Copy, Debug, PartialEq, FromRepr, Display)]
#[repr(u8)]
pub enum ClientPacketType {
    Login = 0,
    Quit            = 1,
    UpdatePlayer    = 2,
    AskChunk        = 3,
}

#[derive(Clone, Copy, Debug, PartialEq, FromRepr, Display)]
#[repr(u8)]
pub enum ServerPacketType {
    Chunk = 3,
    GetPlayer = 2,
    TLS = 0,
    BlockDestroyed = 4,
    Correction = 5,
    Quit = 6,
    Connect = 1
}

#[derive(Clone, Copy, Debug, PartialEq, FromRepr, Display)]
#[repr(u8)]
pub enum ConnectionState {
    TLS,
    Login,
    Ok,
    Quit,
}