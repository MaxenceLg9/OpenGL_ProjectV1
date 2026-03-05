use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use crossbeam::channel as channel;
use crossbeam::channel::RecvTimeoutError;
use shared::common::world::pos::chunkpos::ChunkPos;
use shared::print_base;
use crate::server::generation::mesh::chunk_mesh::ServerChunkMesh;
use crate::server::world_data::chunk::chunk::Chunk;
use crate::server::world_data::chunk::chunkmap::ChunkMap;
use crate::server::world_data::data::ServerWorldData;

const WORLD_THREADS : u32 = 8;

pub struct ChunkGenerator {
    gen_crossbeam_sx: channel::Sender<ChunkPos>,
    pos_register: Arc<Mutex<HashSet<ChunkPos>>>,
}

struct PackedData {
    pos: glam::IVec3,
    mesh : ServerChunkMesh,
}

impl ChunkGenerator {
    pub fn new(chunk_map: Arc<RwLock<ChunkMap>>, sender : channel::Sender<ServerChunkMesh>) -> ChunkGenerator {
        let (gen_crossbeam_sx, gen_crossbeam_rx) : (channel::Sender<ChunkPos>, channel::Receiver<ChunkPos>) = channel::unbounded::<ChunkPos>();
        let (chunk_sx, chunk_rx) = channel::unbounded::<Chunk>();
        let (pos_tx, pos_rx) = channel::unbounded::<ChunkPos>();
        let (meshes_sender, meshes_receiver) = channel::unbounded::<ServerChunkMesh>();
        let pos_register = Arc::new(Mutex::new(HashSet::new()));

        Self::start_generate_chunk(gen_crossbeam_rx, chunk_sx);
        Self::start_receive_chunks(chunk_rx, pos_tx, chunk_map.clone(), pos_register.clone());
        Self::start_build_meshes(pos_rx, meshes_sender, chunk_map);
        Self::start_send_meshes(meshes_receiver, sender);

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
            let chunk = Chunk::new(elt);
            if let Err(e) = sender.send(chunk) {
                print_base!("Error sending chunk: {:?}", e);
                return;
            }
            // print_base!("Generated chunk {}", elt.deref());
        }
    }

    fn start_receive_chunks(chunk_receiver : channel::Receiver<Chunk>, pos_sender : channel::Sender<ChunkPos>, chunk_map: Arc<RwLock<ChunkMap>>, position_register : Arc<Mutex<HashSet<ChunkPos>>>) {
        std::thread::Builder::new().name("chunk_receiver".to_string()).spawn(move || {
            Self::receive_chunks(chunk_receiver, pos_sender, chunk_map, position_register);
        }).unwrap();
    }

    /**
    Function to get the chunks newly generated and push them into the world_data and tick them
    */
    fn receive_chunks(chunk_receiver : channel::Receiver<Chunk>, pos_sender : channel::Sender<ChunkPos>, chunk_map: Arc<RwLock<ChunkMap>>, position_register : Arc<Mutex<HashSet<ChunkPos>>>) {
        let mut time = Instant::now();
        let mut chunks_vec : Vec<Chunk> = Vec::new();
        // waiting for a chunk to be sent
        while let result = chunk_receiver.recv_timeout(Duration::from_millis(20)) {
            match result {
                Ok(chunk) => {
                    chunks_vec.push(chunk);
                    chunks_vec.extend(chunk_receiver.try_iter());
                }
                Err(RecvTimeoutError::Timeout) => {
                    // No chunks, check if we need to do other cleanup
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
                    if lock.add_chunk(chunk) {
                        // if the chunk has been added, sending it to build the ChunkMesh
                        if let Err(e) = pos_sender.send(pos) {
                            print_base!("Error sending chunk: {:?}", e);
                            return;
                        }
                    }
                    // removing the position from the register
                    position_register.lock().unwrap().remove(&pos);
                }
                drop(lock);
                time = Instant::now();
            }
        }
    }
    /// Method call to push the ChunkPos into the channel to generates the associated chunk
    pub fn create_chunk(&mut self, chunk_pos: ChunkPos) {
        let mut result = match self.pos_register.lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };
        if result.insert(chunk_pos) {
            self.gen_crossbeam_sx.send(chunk_pos).expect("Error when sending chunk");
        }
        drop(result);
    }

    fn start_build_meshes(pos_receiver: channel::Receiver<ChunkPos>, mesh_sender: channel::Sender<ServerChunkMesh>, chunk_map: Arc<RwLock<ChunkMap>>) {
        std::thread::Builder::new()
            .name("ChunkMesh_generator".to_string())
            .spawn(move || {
                Self::build_meshes(pos_receiver, mesh_sender, chunk_map);
            }).unwrap();
    }

    fn build_meshes(pos_receiver: channel::Receiver<ChunkPos>, mesh_sender: channel::Sender<ServerChunkMesh>, server_world_data: Arc<RwLock<ChunkMap>>) {
        let mut chunks = HashMap::new();
        let mut hash_set = HashSet::new();
        while let Ok(elt) = pos_receiver.recv() {

            hash_set.insert(elt);
            for pos in pos_receiver.try_iter() {
                hash_set.insert(pos);
            }

            // iterating over the positions to build if possible the associated chunk
            for chunk_pos in hash_set.clone().iter() {
                // add the neighbours in chunks
                let count = Self::add_neighbours_if_exist(chunk_pos, server_world_data.clone(), &mut chunks);
                if count == 7 {
                    let mesh = chunks.get(chunk_pos).unwrap().build_mesh(&chunks);
                    hash_set.remove(chunk_pos);
                    if let Err(e) = mesh_sender.send(mesh) {
                        print_base!("Error sending mesh: {:?}", e);
                        return;
                    }
                    print_base!("Meshed chunk {}", chunk_pos.deref());
                } else if count == 0 {
                    hash_set.remove(chunk_pos);
                }
            }

            chunks.clear();

            // hash_set.retain(|p| {
            //     Self::has_neighbours(p,server_world_data.clone()) == 0
            // });
        }
    }

    fn add_neighbours_if_exist(chunk_pos: &ChunkPos, chunk_map: Arc<RwLock<ChunkMap>>, chunks : &mut HashMap<ChunkPos,Arc<Chunk>>) -> u8 {
        let mut count = 0;
        let pos_vec = Self::get_neighbours_chunks_pos(chunk_pos);

        // checking every side of the chunk
        let data = chunk_map.read().unwrap();
        for p in pos_vec.iter() {
            // the position is already in the map, no need to check if it exists
            if !chunks.contains_key(p) {
                // checking if the chunk exists in the world_data data as it doesn't exist in the map
                let result = data.get(p);
                // getting the object associated with the pos, checking that the chunk exists and adding it into the map
                if result.is_none() {
                    continue;
                }
                // the result is some, adding it into the map
                chunks.entry(*p).or_insert(result.unwrap().clone());
            }
            count += 1;
        }
        for p in pos_vec.iter() {
            if !chunks.contains_key(p) && count == 7 {
                print_base!("Bug on pos {} at {}",chunk_pos.deref(), p.deref())
            }
        }


        count
    }

    fn get_neighbours_chunks_pos(pos: &ChunkPos) -> Vec<ChunkPos> {
        let mut v = Vec::new();
        v.push(ChunkPos::new(glam::ivec3(pos.x - 1, pos.y, pos.z)));
        v.push(ChunkPos::new(glam::ivec3(pos.x + 1, pos.y, pos.z)));
        v.push(ChunkPos::new(glam::ivec3(pos.x, pos.y - 1, pos.z)));
        v.push(ChunkPos::new(glam::ivec3(pos.x, pos.y + 1, pos.z)));
        v.push(ChunkPos::new(glam::ivec3(pos.x, pos.y, pos.z - 1)));
        v.push(ChunkPos::new(glam::ivec3(pos.x, pos.y, pos.z + 1)));
        v.push(pos.clone());
        v
    }

    fn start_send_meshes(receiver: channel::Receiver<ServerChunkMesh>, sender : channel::Sender<ServerChunkMesh>) {
        std::thread::Builder::new().name("mesh_sender".to_string()).spawn(move || {
            Self::send_meshes(receiver, sender);
        }).unwrap();
    }

    fn send_meshes(receiver : channel::Receiver<ServerChunkMesh>, sender : channel::Sender<ServerChunkMesh>) {
        while let Ok(mesh) = receiver.recv() {
            if let Err(e) = sender.send(mesh) {
                print_base!("Error sending mesh: {:?}", e);
                return;
            }
            // print_base!("Sent mesh");
        }
    }
}
