use bitvec::vec::BitVec;
use strum::{Display, FromRepr};
use crate::common::network::server::mesh_packet::MeshPacket;

#[derive(Clone, Copy, Debug, PartialEq, FromRepr, Display)]
#[repr(u8)]
pub enum ClientPacketType {
    Connect = 0,
    Quit = 1,
    PlayerMove = 2,
}

impl ClientPacketType {
    pub fn from_u8(t: u8) -> Option<Self> {
        // TryFromPrimitive generates this logic for you!
        Self::from_repr(t)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, FromRepr, Display)]
#[repr(u8)]
pub enum ServerPacketType {
    Mesh = 0,
}

impl ServerPacketType {
    pub fn get_header_size(t : ServerPacketType) -> usize {
        match t {
            ServerPacketType::Mesh => MeshPacket::get_header_size()
        }
    }

    pub fn get_body_size(t : ServerPacketType, header : BitVec<u8>) -> usize {
        match t {
            ServerPacketType::Mesh => MeshPacket::get_body_size(header)
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