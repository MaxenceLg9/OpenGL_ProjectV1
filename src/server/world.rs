use std::net::{Ipv6Addr, SocketAddrV6};
use std::sync::{Arc, RwLock};
use shared::print_base;
use crate::server::generation::chunk_generator::ChunkGenerator;
use crate::server::network::socket::ServerSocket;
use crate::server::world_data::data::ServerWorldData;

pub struct ServerWorld {
    data : Arc<RwLock<ServerWorldData>>,
    generator : ChunkGenerator,
    socket : ServerSocket
}

impl ServerWorld {
    pub fn new() -> Self {
        let data = Arc::new(RwLock::new(ServerWorldData::new()));
        Self {
            data: data.clone(),
            generator : ChunkGenerator::new(data),
            socket : ServerSocket::new()
        }
    }

    pub fn listen(&mut self) {
        let ipv6_address = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1);

        self.socket.listen(self.data.clone(), SocketAddrV6::new(ipv6_address, 25000, 0, 0).into());

        loop {
            // server.tick();
            std::thread::sleep(std::time::Duration::from_millis(10000)); // ~20 ticks per second
            print_base!("Ticking world");
        }
    }

    pub fn get_data(&self) -> Arc<RwLock<ServerWorldData>> {
        Arc::clone(&self.data)
    }

}