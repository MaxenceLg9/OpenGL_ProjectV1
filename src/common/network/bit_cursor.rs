use bitvec::field::BitField;
use bitvec::macros::internal::funty::Integral;
use bitvec::prelude::*;

pub struct BitCursor<'a> {
    bits: &'a BitSlice<u8, Lsb0>,
    pos: usize,
}

impl<'a> BitCursor<'a> {
    pub fn new(bits: &'a BitSlice<u8, Lsb0>) -> Self {
        Self { bits, pos: 0 }
    }

    /// Read any type that bitvec can "load" (u8, u16, u32, etc.)
    pub fn read_bits<T: bitvec::macros::internal::funty::Integral>(&mut self, bits: usize) -> T {
        let val = self.bits[self.pos..self.pos + bits].load_le::<T>();
        self.pos += bits;
        val
    }

    /// Read a fixed number of bytes into a Vec
    pub fn read_bytes(&mut self, byte_count: usize) -> Vec<u8> {
        let bit_count = byte_count * 8;
        let vec = self.bits[self.pos..self.pos + bit_count].to_bitvec().into_vec();
        self.pos += bit_count;
        vec
    }

    /// Check how many bits are left
    pub fn remaining(&self) -> usize {
        self.bits.len() - self.pos
    }
}