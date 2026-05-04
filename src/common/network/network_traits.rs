use std::fmt::Display;
use bitvec::order::Lsb0;
use bitvec::prelude::BitVec;
use crate::common::network::bit_cursor::BitCursor;
use crate::common::network::packet_type::{ClientPacketType, ServerPacketType};

pub trait ClientNetPacket: Display {
    const P_TYPE: ClientPacketType;
    fn serialize(&self) -> BitVec<u8, Lsb0>;
    fn deserialize(cursor: &mut BitCursor) -> Self;

    fn get_packet_type() -> ClientPacketType;
}

pub trait ServerNetPacket: Display {
    const P_TYPE: ServerPacketType;
    fn serialize(&self) -> BitVec<u8, Lsb0>;
    fn deserialize(cursor: &mut BitCursor) -> Self;

    fn get_packet_type(&self) -> ServerPacketType;
}