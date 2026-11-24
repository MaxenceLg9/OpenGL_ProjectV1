use crate::server::generation::chunk_generator::ChunkGenerator;
use crate::server::network::server_socket::Socket;
use crate::server::world_data::chunk::chunk_map::ServerChunkMap;
use crate::server::world_data::data::ServerWorldData;
use crate::server::world_data::event::events::{InternalEvent, PlayerEvent, ServerEvent};
use crossbeam::{channel as cb, channel};
use shared::common::error::err_type::{CustomErrorKind, GameError};
use shared::common::network::client::block_packet::BlockPacket;
use shared::common::network::l5_packet::L5Packet;
use shared::common::network::packet_type::UdpPacketType;
use shared::common::world::chunk::chunkmap::{MAX_Y_CHUNK, MIN_Y_CHUNK};
use shared::common::world::pos::chunkpos::ChunkPos;
use shared::{custom_error, print_base};
use std::collections::HashSet;
use std::io::Error;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV6};
use std::ops::Mul;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub const FRAME_DURATION: Duration = Duration::from_millis(20);

pub struct ServerWorld {
    data: Arc<ServerWorldData>,
    chunks: ServerChunkMap,
    last_frame: Instant,
    event_rx: cb::Receiver<ServerEvent>,
    chunks_generated: HashSet<ChunkPos>,
    gen_crossbeam_sx: cb::Sender<ChunkPos>,
}

impl ServerWorld {
    pub fn new() -> Result<Self, GameError> {
        let (event_sx, event_rx) = cb::bounded(10000);
        let (gen_crossbeam_sx, gen_crossbeam_rx) = cb::bounded(10000);
        let data = Arc::new(ServerWorldData::new()?);

        let ipv6_address = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
        Socket::listen(data.clone(), SocketAddr::new(ipv6_address, 25000), event_sx.clone())?;

        ChunkGenerator::new(gen_crossbeam_rx, event_sx, 1, data.get_registries())?;
        Ok(Self {
            data,
            event_rx,
            chunks: ServerChunkMap::new(),
            gen_crossbeam_sx,
            chunks_generated: HashSet::new(),
            last_frame: Instant::now(),
        })
    }

    fn generate_base_chunks(pos_register: &mut HashSet<ChunkPos>, gen_crossbeam_sx: channel::Sender<ChunkPos>) -> Result<(), GameError> {
        let range: i32 = 20;
        for i in 0..range.pow(2).mul(14) {
            let pos: ChunkPos = ChunkPos::from_single_value(i, range);
            pos_register.insert(pos);
            gen_crossbeam_sx
                .send(pos)
                .map_err(|e| custom_error!(CustomErrorKind::SendError, format!("{}", e)))?;
            // if let Err(e) = gen_crossbeam_sx.send(pos) {
            //     print_base!("Got error {}",e);
            // }
        }
        Ok(())
    }

    pub fn listen(&mut self) -> Result<(), GameError> {
        Self::generate_base_chunks(&mut self.chunks_generated, self.gen_crossbeam_sx.clone())?;
        print_base!("Spawn generated");
        loop {
            let current_frame = std::time::Instant::now();
            if current_frame - self.last_frame > FRAME_DURATION {
                self.last_frame = current_frame;
                self.data.tick();
            }
            self.poll()?;
            std::thread::sleep(Duration::from_millis(10));
            // print_base!("Len of chunks {}", self.data.get_chunk_map().read().unwrap().len());
        }
    }
    pub fn get_chunk_map(&self) -> &ServerChunkMap {
        &self.chunks
    }
    pub fn poll(&mut self) -> Result<(), GameError> {
        self.handle_events()?;
        self.chunks.poll()
    }
    fn handle_events(&mut self) -> Result<(), GameError> {
        while let Ok(event) = self.event_rx.try_recv() {
            match event {
                ServerEvent::InternalEvent(internal_event) => match internal_event {
                    InternalEvent::GeneratedChunk(chunk) => {
                        self.chunks.add_chunk(chunk);
                    }
                },
                ServerEvent::PlayerEvent { e_type, player_sender: player } => {
                    // print_base!("Player event is {}", e_type);
                    match e_type {
                        PlayerEvent::BlockInteraction(p) => {
                            let bs = self.chunks.interact_block(p.get_pos(), p.get_interaction_type());
                            player
                                .try_send((L5Packet::Block(BlockPacket::new(p.get_interaction_type(), p.get_pos(), bs)), UdpPacketType::Reliable))
                                .expect("Cannot send packet for block interaction");
                        }
                        PlayerEvent::AskChunk(vec_pos, puid) => {
                            self.chunks.ask_for_chunks(vec_pos, player, puid);
                        }
                        PlayerEvent::GenerateChunk(chunk_pos) => {
                            let array: Vec<ChunkPos> = ServerChunkMap::compute_chunks(chunk_pos, 8, &self.chunks_generated);
                            for chunk_pos in array {
                                if !self.chunks_generated.contains(&chunk_pos) && chunk_pos.y >= MIN_Y_CHUNK && chunk_pos.y <= MAX_Y_CHUNK {
                                    if self.chunks_generated.insert(chunk_pos) {
                                        self.gen_crossbeam_sx
                                            .send(chunk_pos)
                                            .map_err(|e| custom_error!(CustomErrorKind::SendError, e.to_string()))?;
                                    }
                                }
                            }
                        }
                        PlayerEvent::EntityInteraction() => {}
                        PlayerEvent::ConnectPlayer() => {}
                        PlayerEvent::DisconnectPlayer(puid) => {
                            self.chunks.remove_player_from_asking(&puid);
                            self.data.disconnect_player(&puid);
                        }
                    }
                }
            }
        }
        Ok(())
    }
    pub fn get_data(&self) -> Arc<ServerWorldData> {
        Arc::clone(&self.data)
    }
}
