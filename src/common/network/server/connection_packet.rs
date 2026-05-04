use std::fmt::{Display, Formatter};
use bitvec::order::Lsb0;
use bitvec::prelude::BitVec;
use bitvec::view::BitView;
use crate::common::network::bit_cursor::BitCursor;
use crate::common::network::network_traits::ServerNetPacket;
use crate::common::network::packet_type::ServerPacketType;
use crate::common::network::packet_type::ServerPacketType::BlockDestroyed;
use crate::common::world::pos::blockpos::BlockPos;
use crate::common::world::pos::pos_trait::PosTrait;

pub struct ConnectionPacket {
    token : [u8; 40],
    pos : BlockPos,
}

impl ConnectionPacket {
    pub fn new() -> ConnectionPacket {
        Self {
            token: [0;40],
            pos: BlockPos::from_floats([0.0,0.0,0.0])
        }
    }

    pub fn get_token(&self) -> &[u8; 40] {
        &self.token
    }

    pub fn get_pos(&self) -> &BlockPos {
        &self.pos
    }
}

impl Display for ConnectionPacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"{:?}",self.token)
    }
}

impl ServerNetPacket for ConnectionPacket {
    const P_TYPE: ServerPacketType = ServerPacketType::Connect;

    fn serialize(&self) -> BitVec<u8, Lsb0> {
        let mut bits = BitVec::new();
        bits.extend_from_bitslice(self.token.view_bits::<Lsb0>());
        bits.extend(self.pos.serialize());
        bits
    }

    fn deserialize(cursor: &mut BitCursor) -> Self {
        Self {
            token: cursor.read_bytes(40).as_array().unwrap().to_owned(),
            pos: BlockPos::deserialize(cursor.read_bytes(12))
        }
    }

    fn get_packet_type(&self) -> ServerPacketType {
        Self::P_TYPE
    }
}