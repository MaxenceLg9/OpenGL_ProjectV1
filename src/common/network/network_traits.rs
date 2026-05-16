use std::fmt::Display;
use bitvec::order::Lsb0;
use bitvec::prelude::BitVec;
use crate::common::network::bit_cursor::BitCursor;

pub trait L5PacketTrait: Display {
    fn serialize(&self) -> BitVec<u8, Lsb0>;
    fn deserialize(cursor: &mut BitCursor) -> Self;

}

pub trait UdpPacketTrait: Display {
    fn serialize(&self) -> BitVec<u8, Lsb0>;
    fn deserialize(cursor: &mut BitCursor) -> Self;
}