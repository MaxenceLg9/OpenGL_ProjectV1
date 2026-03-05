use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use bitvec::field::BitField;
use bitvec::order::Lsb0;
use bitvec::prelude::BitVec;
use bitvec::view::BitView;
use zstd::compression_level_range;
use crate::common::network::network_traits::{Message, ServerMessage};
use crate::common::network::packet_type::ServerPacketType;
use crate::common::world::pos::chunkpos::ChunkPos;
use crate::print_base;

pub struct MeshPacket {
    chunk_pos: ChunkPos,
    indice : u8,
    len : u8,
    ilen : u32,
    vlen : u32,
    bits : BitVec<u8>
}

impl MeshPacket {

    pub fn new(chunk_pos: ChunkPos, i : u8, len : u8, ilen : u32, vlen : u32, bits : &BitVec<u8>) -> Self {
        Self {
            chunk_pos,
            indice: i,
            len,
            ilen,
            vlen,
            bits: bits.clone(),
        }
    }
    
    pub fn get_indice(&self) -> u8 {
        self.indice
    }
    
    pub fn get_total(&self) -> u8 {
        self.len
    }
    
    pub fn get_chunk_pos(&self) -> ChunkPos {
        self.chunk_pos
    }
    pub fn from_bits(bits : BitVec<u8, Lsb0>) -> MeshPacket {
        let (header, content_bits) = bits.split_at(24 * 8usize);
        let (pos_bits , header) = header.split_at(12 * 8usize);
        let raw_bytes = pos_bits[0..96].to_bitvec().into_vec();
        let coords: &[i32] = bytemuck::cast_slice(&raw_bytes);
        let chunk_pos = ChunkPos::new(glam::IVec3::from_slice(coords));
        let i = header[0..8].load_le::<u8>();
        let len = header[8..16].load_le::<u8>();
        let slice = header[16..32].load_le::<u16>();
        let ilen = header[32..64].load_le::<u32>();
        let vlen = header[64..96].load_le::<u32>();
        print_base!("Packet: {}/{}, ChunkPos : {}, Ilen : {}, Vlen {}, bytes {}", i + 1, len, chunk_pos.deref(), ilen, vlen, slice);
        // let uncompressed = content_bits;
        Self {
            chunk_pos,
            indice: i,
            len,
            ilen,
            vlen,
            bits: content_bits.to_bitvec().clone(),
        }
    }
    pub fn get_lens(&self) -> (u32,u32) {
        (self.vlen,self.ilen)
    }

    pub fn get_bits(&self) -> BitVec<u8> {
        self.bits.clone()
    }

    pub fn get_header_size() -> usize {
        24
    }

    pub fn get_body_size(header : BitVec<u8>) -> usize {
        let (pos_bits , header) = header.split_at(12 * 8usize);
        header[16..32].load_le::<u16>() as usize
    }
}

impl Display for MeshPacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Message for MeshPacket {
    fn serialize(&self, packet_type: u8) -> BitVec<u8> {
        let mut bits: BitVec<u8> = BitVec::new();

        bits.extend_from_bitslice(packet_type.view_bits::<Lsb0>());// 1 byte (ServerPacketType)
        bits.extend_from_raw_slice(bytemuck::cast_slice(&self.chunk_pos.to_array()));// 12 bytes
        bits.extend_from_bitslice(self.indice.view_bits::<Lsb0>()); // 1 byte
        bits.extend_from_bitslice(self.len.view_bits::<Lsb0>()); // 1 byte
        bits.extend_from_bitslice((self.bits.len() as u16 / 8).view_bits::<Lsb0>()); // 2 bytes
        bits.extend_from_bitslice((self.ilen).view_bits::<Lsb0>()); // 4 bytes
        bits.extend_from_bitslice((self.vlen).view_bits::<Lsb0>()); // 4 bytes
        // print_base!("Packet: {}/{}, ChunkPos : {}, Ilen : {}, Vlen {}, bytes {}", self.indice + 1, self.len, self.chunk_pos.deref(), self.ilen, self.vlen, self.bits.len() / 8);
        bits.extend(self.bits.clone()); // 1000 bytes or fewer
        bits
    }
}

impl ServerMessage for MeshPacket {
    fn get_packet_type(&self) -> ServerPacketType {
        ServerPacketType::Mesh
    }
}