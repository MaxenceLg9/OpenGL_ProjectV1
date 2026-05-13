use std::fmt::Display;
use bitvec::order::Lsb0;
use bitvec::prelude::BitVec;
use crate::common::network::bit_cursor::BitCursor;
use crate::common::network::packet_type::{ClientPacketType, ServerPacketType};

pub trait ClientNetPacket: Display {
    fn serialize(&self) -> BitVec<u8, Lsb0>;
    fn deserialize(cursor: &mut BitCursor) -> Self;

    fn get_packet_type() -> ClientPacketType;
}

pub trait ServerNetPacket: NetPacket {
    fn get_packet_type(&self) -> ServerPacketType;
}

pub trait NetPacket: Display {
    fn serialize(&self) -> BitVec<u8, Lsb0>;
    fn deserialize(cursor: &mut BitCursor) -> Self;
}