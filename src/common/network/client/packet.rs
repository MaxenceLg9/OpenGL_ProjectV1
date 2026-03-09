use std::fmt::{Display, Formatter};
use bitvec::field::BitField;
use bitvec::order::Lsb0;
use bitvec::prelude::{BitSlice, BitVec};
use crate::common::account::puid::PUID;
use crate::common::network::client::ask_mesh::AskChunkPacket;
use crate::common::network::client::connection_packet::ConnectionPacket;
use crate::common::network::client::sample_packet::SamplePacket;
use crate::common::network::network_traits::{ClientMessage, Message, ClientPacketTrait, PacketTrait};
use crate::common::network::packet_type::{ClientPacketType, ServerPacketType};
use crate::common::network::server::chunk_packet::ChunkPacket;
use crate::common::network::server::packet::ServerPacket;
use crate::common::network::server::player_packet::AskPlayerPacket;

pub enum ClientPacket {
    Connect(ConnectionPacket),
    Player(SamplePacket),
    AskChunk(AskChunkPacket),
    Quit(SamplePacket),
}
impl ClientPacket {

    pub fn from_bits(p_type : ClientPacketType, bits : BitVec<u8>) -> Self {
        match p_type {
            ClientPacketType::Connect => ClientPacket::Connect(ConnectionPacket::from_bits(bits)),
            ClientPacketType::Quit => ClientPacket::Quit(SamplePacket::from_bits(bits)),
            ClientPacketType::Player => ClientPacket::Player(SamplePacket::from_bits(bits)),
            ClientPacketType::AskChunk => ClientPacket::AskChunk(AskChunkPacket::from_bits(bits))
        }
    }

    pub fn inside(&self) -> &dyn ClientMessage {
        match self {
            ClientPacket::Connect(p) => p,
            ClientPacket::Quit(p) => p,
            ClientPacket::Player(p) => p,
            ClientPacket::AskChunk(p) => p,
        }
    }
    pub fn get_puid(&self) -> PUID {
        self.inside().get_puid()
    }
}

impl ClientPacketTrait for ClientPacket {

    fn get_packet_type(&self) -> ClientPacketType {
        match self {
            ClientPacket::Connect(_) => ClientPacketType::Connect,
            ClientPacket::Quit(_) => ClientPacketType::Quit,
            ClientPacket::Player(_) => ClientPacketType::Player,
            ClientPacket::AskChunk(_) => ClientPacketType::AskChunk
        }
    }
}

impl PacketTrait for ClientPacket {
    fn serialize(&self) -> BitVec<u8> {

        // Use the discriminant value directly
        let type_val = self.get_packet_type() as u8;
        self.inside().serialize(type_val)
    }
}

impl Display for ClientPacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientPacket::Connect(p) => p.fmt(f),
            ClientPacket::Quit(p) => f.write_str(""),
            _ => f.write_str(""),
        }
    }
}