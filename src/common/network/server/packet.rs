use std::fmt::{Display, Formatter};
use bitvec::field::BitField;
use bitvec::order::Lsb0;
use bitvec::prelude::{BitSlice, BitVec};
use bitvec::view::BitView;
use crate::common::network::client::connection_packet::ConnectionPacket;
use crate::common::network::client::packet::ClientPacket;
use crate::common::network::client::sample_packet::SamplePacket;
use crate::common::network::network_traits::{ClientMessage, PacketTrait, ServerMessage, ServerPacketTrait};
use crate::common::network::packet_type::{ClientPacketType, ServerPacketType};
use crate::common::network::server::chunk_packet::ChunkPacket;
use crate::common::network::server::player_packet::AskPlayerPacket;

pub enum ServerPacket {
    Correction(SamplePacket),
    BlockDestroy(SamplePacket),
    Chunk(ChunkPacket),
    PlayerPos(AskPlayerPacket)
}

impl ServerPacket {
    pub fn from_bits(p_type : ServerPacketType, bits : &BitSlice<u8, Lsb0>) -> Self {

        // 4. Match and construct
        match p_type {
            ServerPacketType::Chunk => ServerPacket::Chunk(ChunkPacket::from_bits(bits.to_bitvec())),
            ServerPacketType::PlayerPos => ServerPacket::PlayerPos(AskPlayerPacket::from_bits(bits.to_bitvec()))
        }
    }
}

impl ServerPacket {
    pub fn inside(&self) -> &dyn ServerMessage {
        match self {
            ServerPacket::BlockDestroy(p) => p,
            ServerPacket::Correction(p) => p,
            ServerPacket::Chunk(p) => p,
            ServerPacket::PlayerPos(p) => p,
        }
    }
}

impl PacketTrait for ServerPacket {
    fn serialize(&self) -> BitVec<u8> {

        // Use the discriminant value directly
        let type_val = self.get_packet_type() as u8;
        self.inside().serialize(type_val)
    }
}

impl Display for ServerPacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl ServerPacketTrait for ServerPacket {
    fn get_packet_type(&self) -> ServerPacketType {
        self.inside().get_packet_type()
    }
}