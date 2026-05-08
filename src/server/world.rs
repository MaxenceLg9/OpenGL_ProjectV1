use std::net::{Ipv6Addr, SocketAddrV6};
use std::sync::{Arc};
use crate::server::network::socket::ServerSocket;
use crate::server::world_data::data::ServerWorldData;

pub struct ServerWorld {
    data : Arc<ServerWorldData>,
    socket : ServerSocket,
}

impl ServerWorld {
    pub fn new() -> Self {
        let data = Arc::new(ServerWorldData::new());
        Self {
            data,
            socket : ServerSocket::new(),
        }
    }

    pub fn listen(&mut self) {
        let ipv6_address = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1);
        self.socket.listen(self.data.clone(), SocketAddrV6::new(ipv6_address, 25000, 0, 0).into());
        loop {
            std::thread::sleep(std::time::Duration::from_millis(1000)); // ~20 ticks per second
            self.data.tick();
            // print_base!("Len of chunks {}", self.data.get_chunk_map().read().unwrap().len());
        }
    }

    pub fn get_data(&self) -> Arc<ServerWorldData> {
        Arc::clone(&self.data)
    }

}