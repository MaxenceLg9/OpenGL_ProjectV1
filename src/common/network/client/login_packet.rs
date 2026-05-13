use std::fmt::{Display, Formatter};
use bitvec::order::Lsb0;
use bitvec::vec::BitVec;
use bitvec::view::BitView;
use crate::common::account::puid::PUID;
use crate::common::network::bit_cursor::BitCursor;
use crate::common::network::network_traits::{ClientNetPacket, NetPacket};
use crate::common::network::packet_type::ClientPacketType;
#[derive(Clone)]
pub struct LoginPacket {
    puid: PUID,
    password: String,
}

impl LoginPacket {
    pub fn new(id: u32, password: String) -> Self {
        Self {
            puid: PUID::new(id),
            password,
        }
    }

    pub fn get_puid(&self) -> &PUID {
        &self.puid
    }
}

impl Display for LoginPacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}, Password: \"{}\"",self.puid, self.password)
    }
}

impl NetPacket for LoginPacket {

    fn serialize(&self) -> BitVec<u8, Lsb0> {
        let mut bits = BitVec::new();
        bits.extend_from_bitslice(self.puid.id().view_bits::<Lsb0>());
        let pass_bytes = self.password.as_bytes();
        bits.extend_from_bitslice(&(pass_bytes.len() as u32).view_bits::<Lsb0>());
        bits.extend_from_bitslice(pass_bytes.view_bits::<Lsb0>());
        bits
    }

    fn deserialize(cursor: &mut BitCursor) -> Self {
        let puid = cursor.read_bits::<u32>(32);
        let pass_len = cursor.read_bits::<u32>(32) as usize;
        let password = String::from_utf8(cursor.read_bytes(pass_len)).unwrap();

        Self { puid: PUID::new(puid), password }
    }

}