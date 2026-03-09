use std::fmt::Display;
use bitvec::order::Lsb0;
use bitvec::prelude::BitVec;
use crate::common::account::puid::PUID;
use crate::common::network::packet_type::{ClientPacketType, ServerPacketType};
use crate::common::network::server::chunk_packet::ChunkPacket;

pub trait ClientPacketTrait: PacketTrait {
    fn get_packet_type(&self) -> ClientPacketType;
    fn get_packet_type_value(&self) -> u8 {
        self.get_packet_type() as u8
    }
}

pub trait PacketTrait : Send + Sync + Display {
    fn serialize(&self) -> BitVec<u8>;
}

pub trait ServerPacketTrait : PacketTrait {
    fn get_packet_type(&self) -> ServerPacketType;
    fn get_packet_type_value(&self) -> u8 {
        self.get_packet_type() as u8
    }
}


pub trait Message: Display {
    fn serialize(&self, type_val : u8) -> BitVec<u8,Lsb0>;
}

pub trait ClientMessage: Message {
    fn get_puid(&self) -> PUID;
    fn get_packet_type(&self) -> ClientPacketType;
}

pub trait ServerMessage : Message {
    fn get_packet_type(&self) -> ServerPacketType;
}