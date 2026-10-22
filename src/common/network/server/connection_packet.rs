use crate::common::error::err_type::GameError;
use crate::common::network::network_traits::L5PacketTrait;
use crate::common::world::pos::blockpos::BlockPos;
use crate::common::world::pos::pos_trait::PosTrait;
use crate::common::{compression::compress, network::bit_cursor::BitCursor};
use crate::print_base;
use bitvec::order::Lsb0;
use bitvec::prelude::BitVec;
use bitvec::view::BitView;
use std::fmt::{Display, Formatter};
#[derive(Clone)]
pub struct ConnectionPacket {
    token: [u8; 40],
    pos: BlockPos,
    block_table: Vec<(String, u16)>,
}

impl ConnectionPacket {
    pub fn new(pos: BlockPos, block_table: Vec<(String, u16)>) -> ConnectionPacket {
        Self { token: [0; 40], pos, block_table }
    }

    pub fn get_token(&self) -> &[u8; 40] {
        &self.token
    }

    pub fn get_pos(&self) -> &BlockPos {
        &self.pos
    }

    pub fn get_table(&self) -> &Vec<(String, u16)> {
        &self.block_table
    }

    pub fn serialize_table(&self) -> Vec<u8> {
        let mut vec = Vec::new();
        for (string, index) in self.block_table.iter() {
            vec.push(string.len() as u8);
            vec.extend(string.as_bytes());
            vec.extend(index.to_le_bytes());
        }
        vec
    }

    pub fn deserialize_table(serialized: Vec<u8>) -> Vec<(String, u16)> {
        let mut reader = BitCursor::new(serialized.view_bits::<Lsb0>());
        let mut vec = Vec::new();
        loop {
            let len = reader.read_bits::<u8>(8);
            let string = String::from_utf8(reader.read_bytes(len as usize)).unwrap();
            let index = reader.read_bits::<u16>(16);
            print_base!("Got {}:{}", string, index);
            vec.push((string, index));

            if reader.remaining() == 0 {
                break;
            }
        }
        vec
    }
}

impl Display for ConnectionPacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.token)
    }
}

impl L5PacketTrait for ConnectionPacket {
    fn serialize(&self, vec: &mut BitVec<u8, Lsb0>) -> Result<(), GameError> {
        vec.extend_from_bitslice(self.token.view_bits::<Lsb0>());
        vec.extend(self.pos.serialize());

        let serialized = self.serialize_table();
        let compressed = compress::compress(serialized, 10)?;
        vec.extend_from_bitslice((compressed.len() as u32).view_bits::<Lsb0>());
        vec.extend(compressed);
        Ok(())
        // vec.extend_from_bitslice((self.block_table.len() as u32).view_bits::<Lsb0>());
    }

    fn deserialize(cursor: &mut BitCursor) -> Result<Self, GameError> {
        let token = cursor.read_bytes(40).as_array().unwrap().to_owned();
        let pos = BlockPos::deserialize(cursor.read_bytes(12));
        let size = cursor.read_bits::<u32>(32);
        let compressed = cursor.read_bytes(size as usize);
        let serialized = compress::decompress(compressed, size)?;
        let block_table = Self::deserialize_table(serialized);
        Ok(Self { token, pos, block_table })
    }
}
