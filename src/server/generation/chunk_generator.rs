use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use crossbeam::channel as channel;
use crossbeam::channel::RecvTimeoutError;
use shared::common::world::chunk::chunk::Chunk;
use shared::common::world::pos::chunkpos::ChunkPos;
use shared::print_base;
use crate::server::generation::mesh::chunk_mesh::ServerChunkMesh;
use crate::server::world_data::chunk::chunk::ServerChunk;
use shared::common::world::chunk::chunkmap::ChunkMap;
use crate::server::world_data::data::ServerWorldData;

const WORLD_THREADS : u32 = 8;

pub struct ChunkGenerator {
    gen_crossbeam_sx: channel::Sender<ChunkPos>,
    pos_register: HashSet<ChunkPos>,
}

impl ChunkGenerator {
    pub fn new(chunk_map: Arc<RwLock<ChunkMap>>) -> ChunkGenerator {
        let (gen_crossbeam_sx, gen_crossbeam_rx) : (channel::Sender<ChunkPos>, channel::Receiver<ChunkPos>) = channel::bounded::<ChunkPos>(1000);
        let (chunk_sx, chunk_rx) = channel::unbounded::<Chunk>();
        let pos_register = HashSet::new();

        Self::start_generate_chunk(gen_crossbeam_rx, chunk_sx);
        Self::start_receive_chunks(chunk_rx, chunk_map.clone());

        Self {
            gen_crossbeam_sx,
            pos_register,
        }
    }

    /// Function that creates 8 threads that will generate the chunks when needed
    fn start_generate_chunk(gen_crossbeam_rx: channel::Receiver<ChunkPos>, chunk_sender: channel::Sender<Chunk>) {
        for i in 0..WORLD_THREADS {
            let crossbeam_receiver = gen_crossbeam_rx.clone();
            let sender = chunk_sender.clone();
            std::thread::Builder::new()
                .name(format!("chunk_generator_{}", i).to_string())
                .spawn(move || {
                    Self::thread_generate_chunk(crossbeam_receiver, sender);
                }).unwrap();
        }
    }

    /// Thread that pulls the position from the multi-crossbeam receiver and generates chunks and send them back to another channel
    fn thread_generate_chunk(crossbeam_receiver: channel::Receiver<ChunkPos>, sender: channel::Sender<Chunk>) {
        while let Ok(elt) = crossbeam_receiver.recv() {
            let chunk = ServerChunk::generate_chunk(elt);
            if let Err(e) = sender.send(chunk) {
                print_base!("Error sending chunk: {:?}", e);
                return;
            }
            // print_base!("Generated chunk {}", elt.deref());
        }
    }

    fn start_receive_chunks(chunk_receiver : channel::Receiver<Chunk>, chunk_map: Arc<RwLock<ChunkMap>>) {
        std::thread::Builder::new().name("chunk_receiver".to_string()).spawn(move || {
            Self::receive_chunks(chunk_receiver, chunk_map);
        }).unwrap();
    }

    /**
    Function to get the chunks newly generated and push them into the world_data and tick them
    */
    fn receive_chunks(chunk_receiver : channel::Receiver<Chunk>, chunk_map: Arc<RwLock<ChunkMap>>) {
        let mut time = Instant::now();
        let mut chunks_vec : Vec<Chunk> = Vec::new();
        // waiting for a chunk to be sent
        loop {
            match chunk_receiver.recv_timeout(Duration::from_millis(20)) {
                Ok(chunk) => {
                    chunks_vec.push(chunk);
                    chunks_vec.extend(chunk_receiver.try_iter());
                }
                Err(RecvTimeoutError::Timeout) => {
                    // No chunks, check if we need to do other clean-up
                }
                Err(RecvTimeoutError::Disconnected) => return,
            }
            // timer to avoid looping too much
            if Instant::now().duration_since(time).as_millis() > 100 && !chunks_vec.is_empty() {
                // borrowing the lock
                let mut lock = chunk_map.write().unwrap();
                // iterating in the vector
                for i in 0..chunks_vec.len() {
                    let chunk = chunks_vec.pop().unwrap();
                    let pos = chunk.get_chunk_pos();
                    // trying to add the chunk into the world_data map
                    if lock.add_chunk(chunk.clone()) {
                        // if the chunk has been added, sending it to build the ChunkMesh
                    }
                    // removing the position from the register
                }
                drop(lock);
                time = Instant::now();
            }
        }
    }
    /// Method call to push the ChunkPos into the channel to generates the associated chunk
    pub fn schedule_chunks(&mut self, chunks_pos : [ChunkPos; 20*20*20]) {
        for chunk_pos in chunks_pos {
            if !self.pos_register.contains(&chunk_pos) {
                if self.pos_register.insert(chunk_pos) {
                    self.gen_crossbeam_sx.send(chunk_pos).expect("Error when sending chunk");
                    // print_base!("Chunk generated {}", chunk_pos.deref());
                }
            }
        }
    }
}
