use std::io::Write;
use std::net::{SocketAddrV6, TcpStream};
use shared::print_base;

pub struct ClientSocket {
    socket : TcpStream
}

impl ClientSocket {
    pub fn new(socket_addr_v6: SocketAddrV6) -> Self {
        let socket = TcpStream::connect(socket_addr_v6).unwrap();
        print_base!("ServerSocket binding to {:?}", socket_addr_v6);
        Self {
            socket,
        }
    }

    pub fn send(&mut self, message: &str) {
        self.socket.write(message.to_string().as_bytes()).expect("aaa");
    }
}

impl Drop for ClientSocket {
    fn drop(&mut self) {
    }
}