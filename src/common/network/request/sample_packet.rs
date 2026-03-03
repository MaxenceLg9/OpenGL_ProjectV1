use std::sync::Arc;
use bitvec::prelude::BitVec;
use crate::common::network::packet::{PacketTrait, PacketHeader};

pub struct  SamplePacket {
    header : PacketHeader,
}

impl SamplePacket {
    pub fn new() -> Self {
        Self {
            header : PacketHeader::new(0)
        }
    }

    pub(crate) fn from_bits(bits: BitVec<u8>) -> Self {
        Self::new()
    }
    pub fn serialize(&self) -> BitVec<u8> {
        BitVec::new()
    }

    pub fn get_header(&self) -> &PacketHeader {
        &self.header
    }
}