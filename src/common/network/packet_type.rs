use bitvec::vec::BitVec;
use strum::{Display, FromRepr};
use crate::common::network::client::connection_packet::ConnectionPacket;
use crate::common::network::client::packet::ClientPacket;
use crate::common::network::server::chunk_packet::ChunkPacket;
use crate::common::network::server::player_packet::AskPlayerPacket;

#[derive(Clone, Copy, Debug, PartialEq, FromRepr, Display)]
#[repr(u8)]
pub enum ClientPacketType {
    Connect = 0,
    Quit = 1,
    Player = 2,
    AskChunk = 3,
}

impl ClientPacketType {
    pub fn from_u8(t: u8) -> Option<Self> {
        // TryFromPrimitive generates this logic for you!
        Self::from_repr(t)
    }

    pub fn get_header_size(t : ClientPacketType) -> usize {
        match t {
            ClientPacketType::Connect => ConnectionPacket::get_header_size(),
            _ => 0
        }
    }

    pub fn get_body_size(t : ClientPacketType, header : &BitVec<u8>) -> usize {
        match t {
            ClientPacketType::Connect => ConnectionPacket::get_body_size(header),
            _ => 0
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, FromRepr, Display)]
#[repr(u8)]
pub enum ServerPacketType {
    Chunk = 0,
    PlayerPos = 1,
}

impl ServerPacketType {
    pub fn get_header_size(t : ServerPacketType) -> usize {
        match t {
            ServerPacketType::Chunk => ChunkPacket::get_header_size(),
            ServerPacketType::PlayerPos => AskPlayerPacket::get_header_size()
        }
    }

    pub fn get_body_size(t : ServerPacketType, header : &BitVec<u8>) -> usize {
        match t {
            ServerPacketType::Chunk => ChunkPacket::get_body_size(header),
            ServerPacketType::PlayerPos => AskPlayerPacket::get_body_size(header)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, FromRepr, Display)]
#[repr(u8)]
pub enum ConnectionState {
    TLS,
    Login,
    Stream,
    Quit
}