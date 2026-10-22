use crate::server::world_data::chunk::chunk::ServerChunk;
use crate::server::world_data::event::events::{InternalEvent, ServerEvent};
use crossbeam::channel::{self as cb};
use shared::common::error::err_type::{CustomErrorKind, GameError};
use shared::common::world::block::block::BlocksRegistries;
use shared::common::world::pos::chunkpos::ChunkPos;
use shared::worldgen::Generator;
use shared::{custom_error, print_base};
use std::collections::HashMap;
use std::sync::Arc;

const WORLD_THREADS: u32 = 8;

pub struct ChunkGenerator;

impl ChunkGenerator {
    pub fn new(gen_crossbeam_rx: cb::Receiver<ChunkPos>, event_sx: cb::Sender<ServerEvent>, seed: u32, registries: Arc<HashMap<String, BlocksRegistries>>) -> Result<(), GameError> {
        Self::start_generate_chunk(gen_crossbeam_rx, event_sx, Arc::new(Generator::new(seed, registries)))
    }

    /// Function that creates 8 threads that will generate the chunks from the pos sent through the channel
    fn start_generate_chunk(gen_crossbeam_rx: cb::Receiver<ChunkPos>, event_sender: cb::Sender<ServerEvent>, generator: Arc<Generator>) -> Result<(), GameError> {
        for i in 0..WORLD_THREADS {
            let crossbeam_receiver = gen_crossbeam_rx.clone();
            let sender = event_sender.clone();
            let clone = generator.clone();

            std::thread::Builder::new()
                .name(format!("chunk_generator_{}", i).to_string())
                .spawn(move || {
                    if let Err(err) = Self::thread_generate_chunk(crossbeam_receiver, sender, clone) {
                        print_base!("Got error in thread : {}", err);
                    }
                })
                .expect("Cannot start threads to generate chunks");
        }
        Ok(())
    }

    /// Thread that pulls the position from the multi-crossbeam receiver and generates chunks and send them back to another channel
    fn thread_generate_chunk(crossbeam_receiver: cb::Receiver<ChunkPos>, event_sender: cb::Sender<ServerEvent>, generator: Arc<Generator>) -> Result<(), GameError> {
        loop {
            let elt = crossbeam_receiver.recv().map_err(|e| custom_error!(CustomErrorKind::SendError, e.to_string()))?;
            let chunk = ServerChunk::generate_chunk(generator.clone(), elt);
            event_sender
                .send(ServerEvent::InternalEvent(InternalEvent::GeneratedChunk(chunk)))
                .map_err(|e| custom_error!(CustomErrorKind::SendError, e.to_string()))?
            // print_base!("Generated chunk {}", elt.deref());
        }
    }

    //     fn start_receive_chunks(chunk_receiver: channel::Receiver<Chunk>, chunk_map: Arc<RwLock<ServerChunkMap>>) {
    //         std::thread::Builder::new()
    //             .name("chunk_receiver".to_string())
    //             .spawn(move || {
    //                 Self::receive_chunks(chunk_receiver, chunk_map);
    //                 print_base!("Thread exited");
    //             })
    //             .unwrap();
    //     }
    //
    //     /**
    //     Function to get the chunks newly generated and push them into the world_data and tick them
    //     */
    //     fn receive_chunks(chunk_receiver: channel::Receiver<Chunk>, arc_chunk_map: Arc<RwLock<ServerChunkMap>>) {
    //         // waiting for a chunk to be sent
    //         while let Ok(chunk) = chunk_receiver.recv() {
    //             let mut chunk_map = arc_chunk_map.write().unwrap();
    //             // iterating in the vector
    //             chunk_map.add_chunk(chunk.clone());
    //         }
    //     }
}
