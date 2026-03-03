use std::fmt::{Display, Formatter};
use std::todo;
use bitvec::field::BitField;
use bitvec::order::Lsb0;
use bitvec::prelude::BitSlice;
use bitvec::vec::BitVec;
use crate::common::account::puid::PUID;
use crate::common::network::request::connection_packet::ConnectionPacket;
use crate::common::network::response::mesh_packet::MeshPacket;
use crate::common::network::request::sample_packet::SamplePacket;
pub enum PacketType {
    Connect = 0,
    Sample = 5,
    Correction = 3,
    Update = 2,
    Quit = 4,
    Mesh = 1,
}

pub trait PacketTrait: Send + Sync + Display {
    fn serialize(&self) -> BitVec<u8>;

    fn get_packet_type(&self) -> PacketType;
    fn get_packet_type_value(&self) -> u8 {
        self.get_packet_type() as u8
    }
    fn get_header(&self) -> &PacketHeader;
}


pub trait Message: Display {
    fn serialize(&self, value : u8) -> BitVec<u8>;
}

pub trait HeaderMessage: Message {
    fn get_header(&self) -> &PacketHeader;
}

pub enum Packet {
    Connect(ConnectionPacket),
    Sample(SamplePacket),
    Correction(SamplePacket),
    Update(SamplePacket),
    Quit(SamplePacket),
    Mesh(MeshPacket),
}

impl Packet {

    pub fn from_type(bits : &BitSlice<u8, Lsb0>) -> Self {
        let (packet_type, content) = bits.split_at(3);
        let mut aligned_bits = BitVec::<u8, Lsb0>::new();
        aligned_bits.extend_from_bitslice(content);

        let vec = aligned_bits.to_bitvec();
        match packet_type.load::<u8>() {
            0 => Packet::Connect(ConnectionPacket::from_bits(vec)),
            _ => { Packet::Sample(SamplePacket::new())}
        }
    }
}

impl PacketTrait for Packet {

    fn serialize(&self) -> BitVec<u8> {
        let packet_type = self.get_packet_type_value();
        match self {
            Packet::Connect(p) => p.serialize(packet_type),
            Packet::Sample(p) => p.serialize(),
            Packet::Mesh(p) => p.serialize(packet_type),
            &Packet::Correction(_) | &Packet::Update(_) | &Packet::Quit(_) => todo!(),
        }
    }

    fn get_packet_type(&self) -> PacketType {
        match self {
            Packet::Connect(_) => PacketType::Connect,
            Packet::Mesh(_) => PacketType::Connect,
            Packet::Update(_) => PacketType::Connect,
            Packet::Correction(_) => PacketType::Connect,
            Packet::Quit(_) => PacketType::Connect,
            Packet::Sample(_) => PacketType::Connect,
        }
    }

    fn get_header(&self) -> &PacketHeader {
        match self {
            Packet::Connect(p) => panic!("Header doesn't exist for Connect"),
            Packet::Sample(p) | Packet::Correction(p) | Packet::Update(p) | Packet::Quit(p) => p.get_header(),
            Packet::Mesh(p) => p.get_header()
        }
    }
}

impl Display for Packet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Packet::Connect(p) => p.fmt(f),
            Packet::Correction(p) => f.write_str(""),
            Packet::Update(p) => f.write_str(""),
            Packet::Quit(p) => f.write_str(""),
            Packet::Sample(p) => f.write_str(""),
            Packet::Mesh(p) => p.fmt(f),
        }
    }
}

pub struct PacketHeader {
    puid : PUID
}

impl PacketHeader {
    pub fn new(id : u32) -> Self {
        Self {
            puid : PUID::new(id)
        }
    }

    pub fn puid(&self) -> &PUID {
        &self.puid
    }
}