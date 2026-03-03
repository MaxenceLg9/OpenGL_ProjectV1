use std::fmt::{Display, Formatter};
use bitvec::field::BitField;
use bitvec::order::Lsb0;
use bitvec::vec::BitVec;
use bitvec::view::{BitView};
use crate::common::account::puid::PUID;
use crate::common::network::packet::{Message};

pub struct ConnectionPacket {
    puid : PUID,
    password : String
}

impl ConnectionPacket {
    pub fn new(id : u32, password : String) -> Self {
        Self {
            puid : PUID::new(id),
            password,
        }
    }
    pub fn from_bits(bits : BitVec<u8, Lsb0>) -> Self {
        let (puid_bits, password_bits) = bits.split_at(32usize);

        let puid = puid_bits.get(0..puid_bits.len()).unwrap().load_le::<u32>();


        let vec = password_bits.to_bitvec().into_vec();
        let pos = vec.iter().position(|&e| e == 0).unwrap_or(vec.len());
        let password = String::from_utf8(vec[0..pos].to_vec()).unwrap();
        Self {
            puid : PUID::new(puid),
            password
        }
    }



}

impl Display for ConnectionPacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("ConnectionPacket {{ PUID: {}, Password: {} }}", self.puid, self.password))
    }
}

impl Message for ConnectionPacket {
    fn serialize(&self, packet_type : u8) -> BitVec<u8> {
        let mut bits = BitVec::new();
        let packet_type = packet_type as u32;
        bits.extend_from_bitslice(&packet_type.view_bits::<Lsb0>()[..3]);
        bits.extend_from_bitslice(self.puid.id().view_bits::<Lsb0>());
        bits.extend_from_bitslice(self.password.as_bytes().view_bits::<Lsb0>());
        bits
    }
}