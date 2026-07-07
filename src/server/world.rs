use std::net::{Ipv6Addr, SocketAddrV6};
use std::sync::{Arc};
use std::time::{Duration, Instant};
use crate::server::network::server_socket::Socket;
use crate::server::world_data::data::ServerWorldData;
use crossbeam::channel as cb;
use shared::print_base;

pub const FRAME_DURATION : Duration = Duration::from_millis(20);

pub struct ServerWorld {
    data : Arc<ServerWorldData>,
    last_frame : Instant
}

impl ServerWorld {
    pub fn new() -> Self {
        let (event_sx, event_rx) = cb::bounded(1000);
        let data = Arc::new(ServerWorldData::new(event_rx));
        let ipv6_address = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1);
        Socket::listen(data.clone(), SocketAddrV6::new(ipv6_address, 25000, 0, 0,).into(), event_sx);
        Self {
            data,
            last_frame : Instant::now()
        }
    }

    pub fn listen(&mut self) {
        loop {
            let current_frame = std::time::Instant::now();
            if current_frame - self.last_frame > FRAME_DURATION {
                self.last_frame = current_frame;
                self.data.tick();
            }
            self.data.poll();
            // print_base!("Len of chunks {}", self.data.get_chunk_map().read().unwrap().len());
        }
    }

    pub fn get_data(&self) -> Arc<ServerWorldData> {
        Arc::clone(&self.data)
    }

}