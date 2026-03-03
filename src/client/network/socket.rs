use std::io::{Read, Write};
use std::net::{SocketAddrV6, TcpStream};
use bitvec::order::BitOrder;
use bitvec::vec::BitVec;
use shared::print_base;

pub struct ClientSocket {
    socket : TcpStream
}

impl ClientSocket {
    pub fn new(socket_addr_v6: SocketAddrV6) -> Self {
        let socket = TcpStream::connect(socket_addr_v6).unwrap();
        print_base!("ClientSocket connecting to {:?}", socket_addr_v6);
        Self {
            socket,
        }
    }

    pub fn send(&mut self, bits: BitVec<u8>) {
        self.socket.write(bits.as_raw_slice()).expect("aaa");
    }
}

impl Drop for ClientSocket {
    fn drop(&mut self) {
    }
}