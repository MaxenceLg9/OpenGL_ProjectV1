use std::collections::{HashSet};
use std::ops::{Deref, Mul};
use std::sync::{Arc, RwLock};
use crossbeam::channel as channel;
use noise::Perlin;
use shared::common::world::chunk::chunk::Chunk;
use shared::common::world::pos::chunkpos::ChunkPos;
use shared::worldgen::{Generator, SpinePoint};
use shared::print_base;
use crate::server::world_data::chunk::chunk::ServerChunk;
use crate::server::world_data::chunk::chunk_map::ServerChunkMap;
use crate::server::world_data::event::events::{ServerEvent, PlayerEvent, InternalEvent};
use crossbeam::channel as cb;

const WORLD_THREADS : u32 = 8;

pub struct ChunkGenerator;

impl ChunkGenerator {
    pub fn new(gen_crossbeam_rx : channel::Receiver<ChunkPos>, event_sx: cb::Sender<ServerEvent>, seed : u32) {
        Self::start_generate_chunk(gen_crossbeam_rx, event_sx, Arc::new(Generator::new(seed)));
    }



    /// Function that creates 8 threads that will generate the chunks from the pos sent through the channel
    fn start_generate_chunk(gen_crossbeam_rx: channel::Receiver<ChunkPos>, chunk_sender: cb::Sender<ServerEvent>, generator: Arc<Generator>) {
        for i in 0..WORLD_THREADS {
            let crossbeam_receiver = gen_crossbeam_rx.clone();
            let sender = chunk_sender.clone();
            let clone = generator.clone();
            std::thread::Builder::new()
                .name(format!("chunk_generator_{}", i).to_string())
                .spawn(move || {
                    Self::thread_generate_chunk(crossbeam_receiver, sender, clone);
                }).unwrap();
        }
    }

    /// Thread that pulls the position from the multi-crossbeam receiver and generates chunks and send them back to another channel
    fn thread_generate_chunk(crossbeam_receiver: channel::Receiver<ChunkPos>, sender: cb::Sender<ServerEvent>, generator: Arc<Generator>) {
        while let Ok(elt) = crossbeam_receiver.recv() {
            let chunk = ServerChunk::generate_chunk(generator.clone(), elt);
            if let Err(e) = sender.send(ServerEvent::InternalEvent(InternalEvent::GeneratedChunk(chunk))) {
                print_base!("Error sending chunk: {:?}", e);
                return;
            }
            // print_base!("Generated chunk {}", elt.deref());
        }
    }

    fn start_receive_chunks(chunk_receiver : channel::Receiver<Chunk>, chunk_map: Arc<RwLock<ServerChunkMap>>) {
        std::thread::Builder::new().name("chunk_receiver".to_string()).spawn(move || {
            Self::receive_chunks(chunk_receiver, chunk_map);
            print_base!("Thread exited");
        }).unwrap();
    }

    /**
    Function to get the chunks newly generated and push them into the world_data and tick them
    */
    fn receive_chunks(chunk_receiver: channel::Receiver<Chunk>, arc_chunk_map: Arc<RwLock<ServerChunkMap>>) {
        // waiting for a chunk to be sent
        while let Ok(chunk) = chunk_receiver.recv() {
            let mut chunk_map = arc_chunk_map.write().unwrap();
            // iterating in the vector
            chunk_map.add_chunk(chunk.clone());
        }
    }
}