use std::fmt::{Display, Formatter};
use bitvec::field::BitField;
use bitvec::order::Lsb0;
use bitvec::vec::BitVec;
use bitvec::view::{BitView};
use crate::common::account::puid::PUID;
use crate::common::network::network_traits::{ClientMessage, Message};
use crate::common::network::packet_type::ClientPacketType;
use crate::print_base;

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
        let (packet_type , packet_content) = bits.split_at(8);
        let (header, password_bits) = packet_content.split_at(64);
        let puid = header[0..32].load_le::<u32>();
        let password_size = header[32..64].load_le::<u32>();
        let password = String::from_utf8(password_bits.to_bitvec().into_vec()[0..].to_vec()).unwrap();
        Self {
            puid : PUID::new(puid),
            password
        }
    }

    pub fn get_uuid(&self) -> &PUID {
        &self.puid
    }

    pub(crate) fn get_header_size() -> usize {
        8
    }

    pub fn get_body_size(header : &BitVec<u8>) -> usize {
        header[32..64].load_le::<u32>() as usize
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
        bits.extend_from_bitslice(type_val.view_bits::<Lsb0>()); // 1B
        bits.extend_from_bitslice(self.puid.id().view_bits::<Lsb0>()); // 4B
        bits.extend_from_bitslice((self.password.len() as u32).view_bits::<Lsb0>()); // 4B
        print_base!("Len of passwd {}", self.password.len());
        bits.extend_from_bitslice(self.password.as_bytes().view_bits::<Lsb0>()); // We don't know
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