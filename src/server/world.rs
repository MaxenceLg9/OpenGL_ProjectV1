use std::net::{Ipv6Addr, SocketAddrV6};
use std::sync::{Arc};
use crate::server::generation::chunk_generator::ChunkGenerator;
use crate::server::network::socket::ServerSocket;
use crossbeam::channel;
use shared::common::network::network_traits::PacketTrait;
use shared::common::network::server::packet::ServerPacket;
use shared::common::world::pos::chunkpos::ChunkPos;
use crate::server::generation::mesh::chunk_mesh::ServerChunkMesh;
use crate::server::world_data::data::ServerWorldData;

pub struct ServerWorld {
    data : Arc<ServerWorldData>,
    generator : ChunkGenerator,
    socket : ServerSocket,
    mesh_receiver : channel::Receiver<ServerChunkMesh>,
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
            mesh_receiver: rx,
        }
    }

    pub fn listen(&mut self) {
        let ipv6_address = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1);

        self.socket.listen(self.data.clone(), SocketAddrV6::new(ipv6_address, 25000, 0, 0).into());
        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    self.generator.create_chunk(ChunkPos::new(glam::ivec3(i,j,k)));
                }
            }
        }
        loop {
            // server.tick();

            for e in self.data.get_players().read().unwrap().iter() {
                while let Ok(m) = self.mesh_receiver.try_recv(){
                    if m.is_empty() {
                        continue;
                    }
                    for packet in m.to_packets() {
                        let sx = e.1.get_sender().clone();
                        sx.try_send(ServerPacket::Mesh(packet.1).serialize().into_vec());
                    }
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(1000)); // ~20 ticks per second
        }
    }

    pub fn get_data(&self) -> Arc<ServerWorldData> {
        Arc::clone(&self.data)
    }

}