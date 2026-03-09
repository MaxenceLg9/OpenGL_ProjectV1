use std::net::{Ipv6Addr, SocketAddrV6};
use std::ops::Deref;
use std::sync::{Arc};
use crate::server::generation::chunk_generator::ChunkGenerator;
use crate::server::network::socket::ServerSocket;
use crossbeam::channel;
use shared::common::world::pos::chunkpos::ChunkPos;
use crate::server::world_data::data::ServerWorldData;

pub struct ServerWorld {
    data : Arc<ServerWorldData>,
    generator : ChunkGenerator,
    socket : ServerSocket,
}

impl ServerWorld {
    pub fn new() -> Self {
        let data = Arc::new(ServerWorldData::new());
        let (sx, rx) = channel::unbounded();
        let chunk_map = data.get_chunk_map().clone();

        Self {
            data,
            generator : ChunkGenerator::new(chunk_map, sx),
            socket : ServerSocket::new(),
        }
    }

    pub fn listen(&mut self) {
        let ipv6_address = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1);

        self.socket.listen(self.data.clone(), SocketAddrV6::new(ipv6_address, 25000, 0, 0).into());
        loop {
            std::thread::sleep(std::time::Duration::from_millis(1000)); // ~20 ticks per second
        }
    }

    pub fn get_data(&self) -> Arc<ServerWorldData> {
        Arc::clone(&self.data)
    }

}