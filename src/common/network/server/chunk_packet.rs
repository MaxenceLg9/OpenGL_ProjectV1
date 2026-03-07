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
use crate::common::world::chunk::chunk::Chunk;
use crate::common::world::pos::chunkpos::ChunkPos;
use crate::print_base;

pub struct ChunkPacket {
    chunk_pos: ChunkPos,
    indice : u8,
    total : u8,
    len : u32,
    bits : BitVec<u8>
}

impl ChunkPacket {

    pub fn new(chunk_pos: ChunkPos, i : u8, total : u8, len : u32, bits : &BitVec<u8>) -> Self {
        Self {
            chunk_pos,
            indice: i,
            len,
            total,
            bits: bits.clone(),
        }
    }

    pub fn get_indice(&self) -> u8 {
        self.indice
    }

    pub fn get_total(&self) -> u8 {
        self.total
    }

    pub fn get_chunk_pos(&self) -> ChunkPos {
        self.chunk_pos
    }
    pub fn from_bits(bits : BitVec<u8, Lsb0>) -> ChunkPacket {
        let (packet_type, packet) = bits.split_at(8);
        let (header, content_bits) = packet.split_at(Self::get_header_size() * 8);
        let (pos_bits , header) = header.split_at(12 * 8usize);
        let raw_bytes = pos_bits[0..96].to_bitvec().into_vec();
        let coords: &[i32] = bytemuck::cast_slice(&raw_bytes);
        let chunk_pos = ChunkPos::new(glam::IVec3::from_slice(coords));
        let i = header[0..8].load_le::<u8>();
        let total = header[8..16].load_le::<u8>();
        let slice = header[16..32].load_le::<u16>();
        let len = header[32..64].load_le::<u32>();
        // print_base!("Packet: {}/{}, ChunkPos : {}, Ilen : {}, Vlen {}, bytes {}", i + 1, len, chunk_pos.deref(), ilen, vlen, slice);
        // let uncompressed = content_bits;
        Self {
            chunk_pos,
            indice: i,
            total,
            len,
            bits: content_bits.to_bitvec().clone(),
        }
    }
    pub fn get_len(&self) -> u32 {
        self.len
    }

    pub fn get_bits(&self) -> BitVec<u8> {
        self.bits.clone()
    }

    pub fn get_header_size() -> usize {
        20
    }

    pub fn get_body_size(header : &BitVec<u8>) -> usize {
        let (pos_bits , header) = header.split_at(12 * 8usize);
        header[16..32].load_le::<u16>() as usize
    }

    fn compress(chunk: &Chunk) -> Vec<BitVec<u8>> {
        let mut vec = Vec::new();
        let mut bits: BitVec<u8> = BitVec::new();

        let level = if compression_level_range().contains(&10) { 10 } else { compression_level_range().max().unwrap() };
        let blocks_u8 = chunk.get_blocks().iter().flat_map(|&e| e.to_le_bytes()).collect::<Vec<u8>>();
        let data = zstd::bulk::compress(blocks_u8.as_slice(),level).expect("Cannot compress");

        bits.extend_from_bitslice(data.view_bits::<Lsb0>());
        for bit_chunk in bits.chunks(8000) {
            vec.push(bit_chunk.to_bitvec());
        }
        vec
    }

    pub fn from_chunk_to_packets(chunk : &Chunk) -> HashMap<u8, ChunkPacket> {
        let mut packets = HashMap::new();
        let len = chunk.get_blocks().len();
        let chunk_pos : ChunkPos = chunk.get_chunk_pos();
        let bytes_vec: Vec<BitVec<u8>> = Self::compress(chunk);

        let total = bytes_vec.len() as u8;

        for i in 0..total {
            let slice= bytes_vec.get(i as usize).expect("Cannot get vector");
            let packet = ChunkPacket::new(chunk_pos, i, total, len as u32, slice);
            packets.insert(i,packet);
        }
        packets
    }

    fn decompress(vecs_bits: Vec<BitVec<u8>>, len : u32) -> Vec<u16> {
        let mut bits : BitVec<u8> = BitVec::new();
        for vec_bits in vecs_bits {
            bits.extend(vec_bits);
        }
        let data = zstd::bulk::decompress(bits.as_raw_slice(),len as usize * 8usize).expect("Cannot decompress");
        let vec: Vec<u16> = bytemuck::cast_slice::<u8, u16>(&data).to_vec();

        // 3. Split the Vec into two

        vec
    }

    pub fn from_packets_to_chunk(packets : &HashMap<u8,ChunkPacket>, chunk_pos: ChunkPos) -> Chunk {
        let mut v = Vec::new();
        let mut len = 0;
        // print_base!("Decompressing");
        for i in 0..packets.len() as u8 {
            let packet = packets.get(&i).unwrap();
            v.push(packet.get_bits());
            len = packet.get_len();
        }
        let indices = Self::decompress(v, len);
        Chunk::new(chunk_pos,indices)
    }

    fn serialize_header(&self) -> BitVec<u8> {
        let mut bits = BitVec::new();
        bits.extend_from_raw_slice(bytemuck::cast_slice(&self.chunk_pos.to_array()));// 12 bytes
        bits.extend_from_bitslice(self.indice.view_bits::<Lsb0>()); // 1 byte
        bits.extend_from_bitslice(self.total.view_bits::<Lsb0>()); // 1 byte
        bits.extend_from_bitslice((self.bits.len() as u16 / 8).view_bits::<Lsb0>()); // 2 bytes
        bits.extend_from_bitslice((self.len).view_bits::<Lsb0>()); // 4 bytes
        bits
    }
}

impl Display for ChunkPacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Message for ChunkPacket {
    fn serialize(&self, packet_type: u8) -> BitVec<u8> {
        let mut bits: BitVec<u8> = BitVec::new();

        bits.extend_from_bitslice(packet_type.view_bits::<Lsb0>());// 1 byte (ServerPacketType)
        bits.extend(self.serialize_header());
        // print_base!("Packet: {}/{}, ChunkPos : {}, Ilen : {}, Vlen {}, bytes {}", self.indice + 1, self.len, self.chunk_pos.deref(), self.ilen, self.vlen, self.bits.len() / 8);
        bits.extend(self.bits.clone()); // 1000 bytes or fewer
        bits
    }
}

impl ServerMessage for ChunkPacket {
    fn get_packet_type(&self) -> ServerPacketType {
        ServerPacketType::Mesh
    }
}