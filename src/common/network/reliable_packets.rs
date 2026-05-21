use std::fmt::{Display, Formatter};
use bitvec::order::Lsb0;
use bitvec::prelude::BitVec;
use bitvec::view::BitView;
use crate::common::network::bit_cursor::BitCursor;
use crate::common::network::network_traits::UdpPacketTrait;
use crate::common::network::l5_packet::L5Packet;

#[derive(Clone)]
pub struct ReliablePacket {
    ack: u32,
    l5_packet: L5Packet,
}

impl ReliablePacket {

    pub fn new(ack : u32, l5_packet : L5Packet) -> ReliablePacket {
        Self {
            ack,
            l5_packet
        }
    }

    pub fn get_ack(&self) -> u32 {
        self.ack
    }
    pub fn get_l5_packet(&self) -> L5Packet {
        self.l5_packet.clone()
    }
}

impl Display for ReliablePacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"Reliable packet")
    }
}

impl UdpPacketTrait for ReliablePacket {
    fn serialize(&self, vec: &mut BitVec<u8, Lsb0>) {
        vec.extend_from_bitslice(self.ack.view_bits::<Lsb0>());
        vec.extend(self.l5_packet.encode());
    }

    fn deserialize(cursor: &mut BitCursor) -> Self {
        let ack = cursor.read_bits::<u32>(32);
        Self {
            ack,
            l5_packet : L5Packet::decode(cursor.read_all()).unwrap(),
        }

    }
}

#[derive(Clone)]
pub struct SimplePacket {
    l5_packet : L5Packet
}

impl SimplePacket {

    pub fn new(l5_packet : L5Packet) -> SimplePacket {
        Self {
            l5_packet
        }
    }
    pub fn get_l5_packet(&self) -> L5Packet {
        self.l5_packet.clone()
    }
}

impl Display for SimplePacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"Reliable packet")
    }

}

impl UdpPacketTrait for SimplePacket {
    fn serialize(&self, vec: &mut BitVec<u8, Lsb0>) {
        vec.extend(self.l5_packet.encode())
    }

    fn deserialize(cursor: &mut BitCursor) -> Self {
        Self {
            l5_packet: L5Packet::decode(cursor.read_all()).unwrap(),
        }
    }
}


#[derive(Clone)]
pub struct AckPacket {
    ack : u32
}

impl AckPacket {

    pub fn new(ack : u32) -> AckPacket {
        Self {
            ack
        }
    }

    pub fn get_ack(&self) -> u32 {
        self.ack
    }
}

impl Display for AckPacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"Reliable packet")
    }
}

impl UdpPacketTrait for AckPacket {
    fn serialize(&self, vec: &mut BitVec<u8, Lsb0>) {
        vec.extend(self.ack.view_bits::<Lsb0>());
    }

    fn deserialize(cursor: &mut BitCursor) -> Self {
        Self {
            ack: cursor.read_bits::<u32>(32)
        }
    }
}