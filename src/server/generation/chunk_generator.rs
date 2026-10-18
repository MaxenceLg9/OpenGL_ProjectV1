use crate::server::world_data::chunk::chunk::ServerChunk;
use crate::server::world_data::event::events::{InternalEvent, ServerEvent};
use crossbeam::channel;
use crossbeam::channel as cb;
use shared::common::world::block::block::BlocksRegistries;
use shared::common::world::pos::chunkpos::ChunkPos;
use shared::print_base;
use shared::worldgen::Generator;
use std::collections::HashMap;
use std::io::Error;
use std::sync::Arc;

const WORLD_THREADS: u32 = 8;

pub struct ChunkGenerator;

impl ChunkGenerator {
    pub fn new(gen_crossbeam_rx: channel::Receiver<ChunkPos>, event_sx: cb::Sender<ServerEvent>, seed: u32, registries: Arc<HashMap<String, BlocksRegistries>>) -> Result<(), Error> {
        Self::start_generate_chunk(gen_crossbeam_rx, event_sx, Arc::new(Generator::new(seed, registries)))
    }

    /// Function that creates 8 threads that will generate the chunks from the pos sent through the channel
    fn start_generate_chunk(gen_crossbeam_rx: channel::Receiver<ChunkPos>, event_sender: cb::Sender<ServerEvent>, generator: Arc<Generator>) -> Result<(), Error> {
        for i in 0..WORLD_THREADS {
            let crossbeam_receiver = gen_crossbeam_rx.clone();
            let sender = event_sender.clone();
            let clone = generator.clone();

            std::thread::Builder::new().name(format!("chunk_generator_{}", i).to_string()).spawn(move || {
                let err = Self::thread_generate_chunk(crossbeam_receiver, sender, clone).unwrap_err();
                print_base!("Got error in thread : {}", err);
            })?;
        }
        Ok(())
    }

    /// Thread that pulls the position from the multi-crossbeam receiver and generates chunks and send them back to another channel
    fn thread_generate_chunk(crossbeam_receiver: channel::Receiver<ChunkPos>, event_sender: cb::Sender<ServerEvent>, generator: Arc<Generator>) -> Result<(), Error> {
        loop {
            let elt = crossbeam_receiver.recv().map_err(Error::other)?;
            let chunk = ServerChunk::generate_chunk(generator.clone(), elt);
            event_sender.send(ServerEvent::InternalEvent(InternalEvent::GeneratedChunk(chunk))).map_err(Error::other)?
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
