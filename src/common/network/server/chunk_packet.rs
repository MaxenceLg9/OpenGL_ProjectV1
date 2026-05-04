use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use bitvec::order::Lsb0;
use bitvec::prelude::BitVec;
use bitvec::view::{AsBits, BitView};
use zstd::compression_level_range;
use crate::common::network::bit_cursor::BitCursor;
use crate::common::network::network_traits::{ServerNetPacket};
use crate::common::network::packet_type::ServerPacketType;
use crate::common::world::chunk::chunk::Chunk;
use crate::common::world::pos::chunkpos::ChunkPos;
use crate::common::world::pos::pos_trait::PosTrait;
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

    pub fn get_len(&self) -> u32 {
        self.len
    }

    pub fn get_bits(&self) -> BitVec<u8> {
        self.bits.clone()
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

    // what's the purpose of the u8 in the map ?
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
}

impl Clone for ChunkPacket {
    fn clone(&self) -> Self {
        Self {
            chunk_pos: self.chunk_pos.clone(),
            bits: self.bits.clone(),
            len: self.len,
            total: self.total,
            indice: self.indice
        }
    }
}

impl Display for ChunkPacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("aa")
    }
}

impl ServerNetPacket for ChunkPacket {
    const P_TYPE: ServerPacketType = ServerPacketType::Chunk;

    fn serialize(&self) -> BitVec<u8, Lsb0> {
        let mut bits = BitVec::new();
        bits.extend(self.chunk_pos.serialize());// 12 bytes
        bits.extend_from_bitslice(self.indice.view_bits::<Lsb0>()); // 1 byte
        bits.extend_from_bitslice(self.total.view_bits::<Lsb0>()); // 1 byte
        bits.extend_from_bitslice((self.bits.len() as u16 / 8).view_bits::<Lsb0>()); // 2 bytes
        bits.extend_from_bitslice((self.len).view_bits::<Lsb0>()); // 4 bytes
        bits.extend(self.bits.clone()); // 1000 bytes or fewer
        bits
    }

    fn deserialize(cursor: &mut BitCursor) -> Self {
        let raw_bytes = cursor.read_bytes(12).to_vec();
        let coords: &[i32] = bytemuck::cast_slice(&raw_bytes);
        let chunk_pos = ChunkPos::new(glam::IVec3::from_slice(coords));
        let i = cursor.read_bits::<u8>(8);
        let total = cursor.read_bits::<u8>(8);
        let slice = cursor.read_bits::<u16>(16);
        let len = cursor.read_bits::<u32>(32);
        print_base!("Packet: {}/{}, ChunkPos : {}, Len : {}, bytes {}", i + 1, total, chunk_pos.deref(), len, slice);
        // let uncompressed = content_bits;
        Self {
            chunk_pos,
            indice: i,
            total,
            len,
            bits: cursor.read_bytes(slice as usize).as_bits().to_bitvec(),
        }

    }

    fn get_packet_type(&self) -> ServerPacketType {
        Self::P_TYPE
    }
}