use std::collections::{HashSet};
use std::ops::{Deref, Mul};
use std::sync::{Arc, RwLock};
use crossbeam::channel as channel;
use noise::Perlin;
use shared::common::world::chunk::chunk::Chunk;
use shared::common::world::pos::chunkpos::ChunkPos;
use shared::math::{Generator, SpinePoint};
use shared::print_base;
use crate::server::world_data::chunk::chunk::ServerChunk;
use crate::server::world_data::chunk::chunk_map::ServerChunkMap;

const WORLD_THREADS : u32 = 8;

pub struct ChunkGenerator {
    gen_crossbeam_sx: channel::Sender<ChunkPos>,
    chunks_generated: HashSet<ChunkPos>,
}

impl ChunkGenerator {
    pub fn new(chunk_map: Arc<RwLock<ServerChunkMap>>, seed : u32) -> ChunkGenerator {
        let (gen_crossbeam_sx, gen_crossbeam_rx) : (channel::Sender<ChunkPos>, channel::Receiver<ChunkPos>) = channel::bounded::<ChunkPos>(1000);
        let mut chunks_generated = HashSet::new();
        let (chunk_sx, chunk_rx) = channel::bounded::<Chunk>(10000);

        Self::start_generate_chunk(gen_crossbeam_rx, chunk_sx, Arc::new(Generator::new(Perlin::new(seed))));
        Self::start_receive_chunks(chunk_rx, chunk_map.clone());
        Self::generate_base_chunks(&mut chunks_generated, gen_crossbeam_sx.clone());
        Self {
            gen_crossbeam_sx,
            chunks_generated,
        }
    }

    pub fn get_chunks_generated(&self) -> HashSet<ChunkPos> {
        self.chunks_generated.clone()
    }

    fn generate_base_chunks(pos_register : &mut HashSet<ChunkPos>, gen_crossbeam_sx : channel::Sender<ChunkPos>){
        let range: i32 = 20;
        for i in 0..range.pow(2).mul(7) {
            let pos : ChunkPos = ChunkPos::from_single_value(i, range);
            pos_register.insert(pos);
            gen_crossbeam_sx.send(pos).unwrap();
            // if let Err(e) = gen_crossbeam_sx.send(pos) {
            //     print_base!("Got error {}",e);
            // }
        }
    }

    /// Function that creates 8 threads that will generate the chunks from the pos sent through the channel
    fn start_generate_chunk(gen_crossbeam_rx: channel::Receiver<ChunkPos>, chunk_sender: channel::Sender<Chunk>, perlin: Arc<Generator>) {
        for i in 0..WORLD_THREADS {
            let crossbeam_receiver = gen_crossbeam_rx.clone();
            let sender = chunk_sender.clone();
            let noise = perlin.clone();
            std::thread::Builder::new()
                .name(format!("chunk_generator_{}", i).to_string())
                .spawn(move || {
                    Self::thread_generate_chunk(crossbeam_receiver, sender, noise);
                }).unwrap();
        }
    }

    /// Thread that pulls the position from the multi-crossbeam receiver and generates chunks and send them back to another channel
    fn thread_generate_chunk(crossbeam_receiver: channel::Receiver<ChunkPos>, sender: channel::Sender<Chunk>, generator: Arc<Generator>) {
        while let Ok(elt) = crossbeam_receiver.recv() {
            let chunk = ServerChunk::generate_chunk(generator.clone(), elt);
            if let Err(e) = sender.send(chunk) {
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

    /// Method call to push the ChunkPos into the channel to generates the associated chunk
    pub fn schedule_chunks(&mut self, chunks_pos : Vec<ChunkPos>) {
        for chunk_pos in chunks_pos {
            if !self.chunks_generated.contains(&chunk_pos) && chunk_pos.y >= -2 && chunk_pos.y <= 9 {
                if self.chunks_generated.insert(chunk_pos) {
                    self.gen_crossbeam_sx.send(chunk_pos).expect("Error when sending chunk");
                }
            }
        }
    }
}