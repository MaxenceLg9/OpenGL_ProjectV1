use crate::common::network::l5_packet::L5Packet;
use crate::common::network::network_traits::UdpPacketTrait;
use crate::common::network::socket::common_socket::Ack;
use crate::common::{error::err_type::GameError, network::bit_cursor::BitCursor};
use bitvec::order::Lsb0;
use bitvec::prelude::BitVec;
use bitvec::view::BitView;
use std::fmt::{Display, Formatter};

#[derive(Clone)]
pub struct ReliablePacket {
    ack: Ack,
    l5_packet: L5Packet,
}

impl ReliablePacket {
    pub fn new(ack: Ack, l5_packet: L5Packet) -> ReliablePacket {
        Self { ack, l5_packet }
    }

    pub fn get_ack(&self) -> Ack {
        self.ack
    }
    pub fn get_l5_packet(&self) -> L5Packet {
        self.l5_packet.clone()
    }
}

impl Display for ReliablePacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Reliable packet")
    }
}

impl UdpPacketTrait for ReliablePacket {
    fn serialize(&self, vec: &mut BitVec<u8, Lsb0>) -> Result<(), GameError> {
        let b = Box::new(());
        vec.extend_from_bitslice(self.ack.view_bits::<Lsb0>());
        vec.extend(self.l5_packet.encode());
        Ok(())
    }

    fn deserialize(cursor: &mut BitCursor) -> Result<Self, GameError> {
        let ack = cursor.read_bits::<Ack>(64);
        Ok(Self { ack, l5_packet: L5Packet::decode(cursor.read_all())? })
    }
}

#[derive(Clone)]
pub struct SimplePacket {
    l5_packet: L5Packet,
}

impl SimplePacket {
    pub fn new(l5_packet: L5Packet) -> SimplePacket {
        Self { l5_packet }
    }
    pub fn get_l5_packet(&self) -> L5Packet {
        self.l5_packet.clone()
    }
}

impl Display for SimplePacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Reliable packet")
    }
}

impl UdpPacketTrait for SimplePacket {
    fn serialize(&self, vec: &mut BitVec<u8, Lsb0>) -> Result<(), GameError> {
        vec.extend(self.l5_packet.encode());
        Ok(())
    }

    fn deserialize(cursor: &mut BitCursor) -> Result<Self, GameError> {
        Ok(Self {
            l5_packet: L5Packet::decode(cursor.read_all()).unwrap(),
        })
    }
}

#[derive(Clone)]
pub struct AckPacket {
    ack: Ack,
}

impl AckPacket {
    pub fn new(ack: Ack) -> AckPacket {
        Self { ack }
    }

    pub fn get_ack(&self) -> Ack {
        self.ack
    }
}

impl Display for AckPacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Reliable packet")
    }
}

impl UdpPacketTrait for AckPacket {
    fn serialize(&self, vec: &mut BitVec<u8, Lsb0>) -> Result<(), GameError> {
        vec.extend(self.ack.view_bits::<Lsb0>());
        Ok(())
    }

    fn deserialize(cursor: &mut BitCursor) -> Result<Self, GameError> {
        Ok(Self { ack: cursor.read_bits::<Ack>(64) })
    }
}
