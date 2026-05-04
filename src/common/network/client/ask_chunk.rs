use std::fmt::{Display, Formatter};
use bitvec::order::Lsb0;
use bitvec::prelude::BitVec;
use bitvec::view::BitView;
use crate::common::account::puid::PUID;
use crate::common::network::bit_cursor::BitCursor;
use crate::common::network::network_traits::{ClientNetPacket};
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

    pub fn player_pos(&self) -> BlockPos {
        self.player_pos
    }

    pub fn get_chunk_pos(&self) -> ChunkPos {
        self.chunk_pos
    }

}

impl ClientNetPacket for AskChunkPacket {
    const P_TYPE: ClientPacketType = ClientPacketType::AskChunk;

    fn serialize(&self) -> BitVec<u8, Lsb0> {
        let mut bits = BitVec::new();
        bits.extend_from_bitslice(self.puid.id().view_bits::<Lsb0>());
        bits.extend(self.chunk_pos.serialize());
        bits.extend(self.player_pos.serialize());
        bits
    }

    fn deserialize(cursor: &mut BitCursor) -> Self {
        let puid = cursor.read_bits::<u32>(32);
        let chunk_pos : ChunkPos = ChunkPos::deserialize(cursor.read_bytes(12)).as_any().downcast_ref::<ChunkPos>().unwrap().clone();
        let player_pos : BlockPos = BlockPos::deserialize(cursor.read_bytes(12)).as_any().downcast_ref::<BlockPos>().unwrap().clone();

        Self::new(PUID::new(puid), chunk_pos, player_pos)
    }

    fn get_packet_type() -> ClientPacketType {
        Self::P_TYPE
    }
}

impl Display for AskChunkPacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("")
    }
}