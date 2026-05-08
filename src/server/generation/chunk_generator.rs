use std::collections::{HashSet};
use std::sync::{Arc, RwLock};
use std::time::{Duration};
use crossbeam::channel as channel;
use shared::common::world::chunk::chunk::Chunk;
use shared::common::world::pos::chunkpos::ChunkPos;
use shared::print_base;
use crate::server::world_data::chunk::chunk::ServerChunk;
use crate::server::world_data::chunk::chunk_map::ServerChunkMap;

const WORLD_THREADS : u32 = 8;

pub struct ChunkGenerator {
    gen_crossbeam_sx: async_channel::Sender<ChunkPos>,
    pos_register: HashSet<ChunkPos>,
}

impl ChunkGenerator {
    pub fn new(chunk_map: Arc<RwLock<ServerChunkMap>>) -> ChunkGenerator {
        let (gen_crossbeam_sx, gen_crossbeam_rx) : (async_channel::Sender<ChunkPos>, async_channel::Receiver<ChunkPos>) = async_channel::bounded::<ChunkPos>(1000);
        let mut pos_register = HashSet::new();

        std::thread::Builder::new()
            .name("chunk_generator_thread".to_string())
            .spawn(move || {
                Self::start_async_thread(gen_crossbeam_rx, chunk_map);
            }).unwrap();

        // Self::start_generate_chunk(gen_crossbeam_rx, chunk_sx);
        // Self::start_receive_chunks(chunk_rx, chunk_map.clone());
        Self::generate_base_chunks(&mut pos_register, &gen_crossbeam_sx);
        Self {
            gen_crossbeam_sx,
            pos_register,
        }
    }

    pub fn start_async_thread(gen_crossbeam_rx: async_channel::Receiver<ChunkPos>, chunk_map: Arc<RwLock<ServerChunkMap>>) {
        let (chunk_sx, chunk_rx) = tokio::sync::mpsc::channel::<Chunk>(1000);

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            Self::start_generate_chunk(gen_crossbeam_rx, chunk_sx);
            Self::start_receive_chunks(chunk_rx, chunk_map.clone());
        })
    }

    fn generate_base_chunks(pos_register : &mut HashSet<ChunkPos>, gen_crossbeam_sx : &async_channel::Sender<ChunkPos>){
        for i in 0..20*20*20 {
            let pos : ChunkPos = ChunkPos::from_single_value(i, 20);
            pos_register.insert(pos);
            gen_crossbeam_sx.try_send(pos).unwrap();
            // if let Err(e) = gen_crossbeam_sx.send(pos) {
            //     print_base!("Got error {}",e);
            // }
        }
    }

    /// Function that creates 8 threads that will generate the chunks from the pos sent through the channel
    fn start_generate_chunk(gen_async_rx: async_channel::Receiver<ChunkPos>, chunk_sender: tokio::sync::mpsc::Sender<Chunk>) {
        for i in 0..WORLD_THREADS {
            let crossbeam_receiver = gen_async_rx.clone();
            let sender = chunk_sender.clone();
            tokio::task::spawn(async {
                Self::thread_generate_chunk(crossbeam_receiver, sender).await;
            });
        }
    }

    /// Thread that pulls the position from the multi-crossbeam receiver and generates chunks and send them back to another channel
    async fn thread_generate_chunk(async_receiver: async_channel::Receiver<ChunkPos>, sender: tokio::sync::mpsc::Sender<Chunk>) {
        while let Ok(elt) = async_receiver.recv().await {
            let chunk = ServerChunk::generate_chunk(elt);
            if let Err(e) = sender.send(chunk).await {
                print_base!("Error sending chunk: {:?}", e);
                return;
            }
            // print_base!("Generated chunk {}", elt.deref());
        }
    }

    fn start_receive_chunks(chunk_receiver : tokio::sync::mpsc::Receiver<Chunk>, chunk_map: Arc<RwLock<ServerChunkMap>>) {
        tokio::task::spawn(async {
            Self::receive_chunks(chunk_receiver, chunk_map).await;
        });
    }

    /**
    Function to get the chunks newly generated and push them into the world_data and tick them
    */
    async fn receive_chunks(mut chunk_receiver: tokio::sync::mpsc::Receiver<Chunk>, arc_chunk_map: Arc<RwLock<ServerChunkMap>>) {
        let mut chunks_vec : Vec<Chunk> = Vec::new();
        // waiting for a chunk to be sent
        let mut tick = tokio::time::interval(Duration::from_millis(100));
        loop {
            tokio::select! {
                chunk = chunk_receiver.recv() => {
                    match chunk {
                        Some(chunk) => {
                            chunks_vec.push(chunk);
                        }
                        None => return,
                    }
                }
                // timer to avoid looping too much
                _ = tick.tick() => {
                    if !chunks_vec.is_empty() {
                        // borrowing the lock
                        let mut chunk_map = arc_chunk_map.write().unwrap();
                        // iterating in the vector
                        for i in 0..chunks_vec.len() {
                            let chunk = chunks_vec.pop().unwrap();
                            let pos = chunk.get_chunk_pos();
                            // trying to add the chunk into the world_data map
                            chunk_map.add_chunk(chunk.clone());
                            // removing the position from the register
                        }
                        drop(chunk_map);
                    }
                }
            }

        }
    }
    /// Method call to push the ChunkPos into the channel to generates the associated chunk
    pub fn schedule_chunks(&mut self, chunks_pos : [ChunkPos; 20*20*20]) {
        for chunk_pos in chunks_pos {
            if !self.pos_register.contains(&chunk_pos) {
                if self.pos_register.insert(chunk_pos) {
                    self.gen_crossbeam_sx.try_send(chunk_pos).expect("Error when sending chunk");
                    // print_base!("Chunk generated {}", chunk_pos.deref());
                }
            }
        }
    }
}
