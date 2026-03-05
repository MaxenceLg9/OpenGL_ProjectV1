use std::fmt::{Display, Formatter};
use bitvec::field::BitField;
use bitvec::order::Lsb0;
use bitvec::vec::BitVec;
use bitvec::view::{BitView};
use crate::common::account::puid::PUID;
use crate::common::network::network_traits::{ClientMessage, Message};
use crate::common::network::packet_type::ClientPacketType;

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

    pub fn get_uuid(&self) -> &PUID {
        &self.puid
    }



}

impl Display for ConnectionPacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("ConnectionPacket {{ \"PUID\": {}, \"Password\": {} }}", self.puid, self.password))
    }
}

impl Message for ConnectionPacket {
    fn serialize(&self, type_val : u8) -> BitVec<u8> {
        let mut bits = BitVec::new();
        bits.extend_from_bitslice(type_val.view_bits::<Lsb0>());
        bits.extend_from_bitslice(self.puid.id().view_bits::<Lsb0>());
        bits.extend_from_bitslice(self.password.as_bytes().view_bits::<Lsb0>());
        bits
    }
}

impl ClientMessage for ConnectionPacket {
    fn get_puid(&self) -> PUID {
        self.puid
    }

    fn get_packet_type(&self) -> ClientPacketType {
        ClientPacketType::Connect
    }
}