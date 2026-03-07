use std::fmt::{Display, Formatter};
use bitvec::field::BitField;
use bitvec::order::Lsb0;
use bitvec::prelude::{BitSlice, BitVec};
use crate::common::account::puid::PUID;
use crate::common::network::client::connection_packet::ConnectionPacket;
use crate::common::network::client::sample_packet::SamplePacket;
use crate::common::network::network_traits::{ClientMessage, Message, ClientPacketTrait, PacketTrait};
use crate::common::network::packet_type::ClientPacketType;

pub enum ClientPacket {
    Connect(ConnectionPacket),
    PlayerMove(SamplePacket),
    Quit(SamplePacket),
}
impl ClientPacket {

    pub fn from_type(bits : &BitSlice<u8, Lsb0>) -> Self {
        let (packet_type, content) = bits.split_at(8);
        let mut aligned_bits = BitVec::<u8, Lsb0>::new();
        aligned_bits.extend_from_bitslice(content);

        let vec = aligned_bits.to_bitvec();
        let t = packet_type.load::<u8>();

        let p_type = ClientPacketType::from_repr(t)
            .expect("Received invalid packet type ID!");
        
        match p_type {
            ClientPacketType::Connect => ClientPacket::Connect(ConnectionPacket::from_bits(vec)),
            ClientPacketType::Quit => ClientPacket::Quit(SamplePacket::from_bits(vec)),
            ClientPacketType::PlayerMove => ClientPacket::PlayerMove(SamplePacket::from_bits(vec))
        }
    }
    pub fn inside(&self) -> &dyn ClientMessage {
        match self {
            ClientPacket::Connect(p) => p,
            ClientPacket::Quit(p) => p,
            ClientPacket::PlayerMove(p) => p,
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
            ClientPacket::PlayerMove(_) => ClientPacketType::PlayerMove
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