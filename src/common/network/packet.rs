use crate::common::network::packet_type::ServerPacketType;

const TYPE_SIZE: u8 = 1;
const HEADER_SIZE: u8 = 44;
pub struct Packet {
    packet_type: u8,
    header: Header,
    body : Vec<u8>,
}

impl Packet {
    pub fn new(packet_type: u8, token : [u8; 40], body : Vec<u8>) -> Packet {
        Packet {
            packet_type,
            header : Header::new(token,body.len() as u32),
            body
        }
    }

    pub fn get_header(&self) -> &Header {
        &self.header
    }

    pub fn get_body(&self) -> &Vec<u8> {
        &self.body
    }
}

pub struct Header {
    token: [u8; 40],
    body_size : u32,
}

impl Header {
    pub fn new(token : [u8;40], body_size : u32) -> Header {
        Header {
            token,
            body_size
        }
    }

    pub fn get_token(&self) -> &[u8; 40] {
        &self.token
    }

    pub fn get_body_size(&self) -> u32 {
        self.body_size
    }
}