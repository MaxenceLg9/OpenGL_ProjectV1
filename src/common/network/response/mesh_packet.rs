use std::fmt::{Display, Formatter};
use bitvec::prelude::BitVec;
use crate::common::generation::chunk_mesh::CommonChunkMesh;
use crate::common::network::packet::{PacketHeader, Message, HeaderMessage};

pub struct MeshPacket {
    header : PacketHeader,
    mesh : CommonChunkMesh,
}

impl Display for MeshPacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Message for MeshPacket {
    fn serialize(&self, value: u8) -> BitVec<u8> {
        todo!()
    }
}

impl HeaderMessage for MeshPacket {

    fn get_header(&self) -> &PacketHeader {
        todo!()
    }
}