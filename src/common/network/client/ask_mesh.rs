use std::fmt::{Display, Formatter};
use bitvec::field::BitField;
use bitvec::order::Lsb0;
use bitvec::prelude::BitVec;
use bitvec::view::BitView;
use crate::common::account::puid::PUID;
use crate::common::network::network_traits::{ClientMessage, Message};
use crate::common::network::packet_type::ClientPacketType;
use crate::common::world::pos::blockpos::BlockPos;
use crate::common::world::pos::chunkpos::ChunkPos;
use crate::common::world::pos::pos_trait::PosTrait;

pub struct AskChunkPacket {
    puid: PUID,
    chunk_pos : ChunkPos,
    player_pos : BlockPos
}

impl AskChunkPacket {
    pub fn new(puid: PUID, chunk_pos: ChunkPos, player_pos : BlockPos) -> Self {
        Self {
            puid,
            chunk_pos,
            player_pos
        }
    }

    pub fn from_bits(bits : BitVec<u8>) -> Self {
        let (packet_type, header) = bits.split_at(8);
        let (header , pos) = header.split_at(4 * 8usize);
        let puid = header[8..40].load_le::<u32>();
        let chunk_pos : ChunkPos = ChunkPos::deserialize(pos[0..96].to_bitvec()).as_any().downcast_ref::<ChunkPos>().unwrap().clone();
        let player_pos : BlockPos = BlockPos::deserialize(pos[96..192].to_bitvec()).as_any().downcast_ref::<BlockPos>().unwrap().clone();

        Self::new(PUID::new(puid), chunk_pos, player_pos)
    }

}

impl Message for AskChunkPacket {
    fn serialize(&self, type_val: u8) -> BitVec<u8, Lsb0> {
        let mut bits = BitVec::new();
        bits.extend_from_bitslice(type_val.view_bits::<Lsb0>());
        bits.extend_from_bitslice(self.puid.id().view_bits::<Lsb0>());
        bits.extend(self.chunk_pos.serialize());
        bits.extend(self.player_pos.serialize());
        bits
    }
}

impl Display for AskChunkPacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl ClientMessage for AskChunkPacket {
    fn get_puid(&self) -> PUID {
        todo!()
    }

    fn get_packet_type(&self) -> ClientPacketType {
        todo!()
    }
}