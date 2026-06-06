use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::sync::{Arc, Mutex, RwLock};
use crossbeam::channel as cb;
use shared::common::world::chunk::chunk::Chunk;
use shared::common::world::pos::chunkpos::ChunkPos;
use shared::{print_base, print_debug};
use shared::common::world::chunk::chunkmap::ChunkMap;
use crate::client::display::renderer::gui::text::text::MeshText;
use crate::client::generation::mesh::chunk_mesh::ChunkMesh;
use crate::client::world_data::chunks::chunk_map::ClientChunkMap;

pub struct MeshGenerator;

impl MeshGenerator {

    pub fn start_build_meshes(pos_to_mesh_rx: cb::Receiver<ChunkPos>, mesh_sender: cb::Sender<(ChunkMesh, MeshText)>, chunk_map: Arc<RwLock<ClientChunkMap>>) {
        for i in 0..4 {
            let clone_map = chunk_map.clone();
            let sender_clone = mesh_sender.clone();
            let receiver_clone = pos_to_mesh_rx.clone();
            std::thread::Builder::new()
                .name("ChunkMesh_generator".to_string())
                .spawn(move || {
                    Self::build_meshes(receiver_clone, sender_clone, clone_map);
                }).unwrap();
        }
    }

    fn build_meshes(pos_to_mesh_rx: cb::Receiver<ChunkPos>, mesh_sender: cb::Sender<(ChunkMesh, MeshText)>, server_world_data: Arc<RwLock<ClientChunkMap>>) {
        let mut chunks = HashMap::new();
        let mut hash_set = HashSet::new();
        let mut n = 0;
        while let Ok(elt) = pos_to_mesh_rx.recv() {

            hash_set.insert(elt);
            for pos in pos_to_mesh_rx.try_iter() {
                hash_set.insert(pos);
            }
            // iterating over the positions to build if possible the associated chunk
            for chunk_pos in hash_set.clone().iter() {
                // add the neighbours in chunks
                let count = Self::add_neighbours_if_exist(chunk_pos, server_world_data.clone(), &mut chunks);
                if count == 7 {
                    if let Some(meshes) = ChunkMesh::build_mesh(&chunks, chunk_pos.clone()) {
                        hash_set.remove(chunk_pos);
                        if let Err(e) = mesh_sender.send(meshes) {
                            print_base!("Error sending mesh: {:?}", e);
                            return;
                        }
                        n += 1;
                    }
                    // print_base!("Meshed chunk {}, {} chunks sent", chunk_pos.deref(), n);
                } else if count == 0 {
                    print_debug!("Deleted pos {}",chunk_pos.deref());
                    hash_set.remove(chunk_pos);
                }
            }

            chunks.clear();

            // hash_set.retain(|p| {
            //     Self::has_neighbours(p,server_world_data.clone()) == 0
            // });
        }
    }

    fn add_neighbours_if_exist(chunk_pos: &ChunkPos, arc_chunk_map: Arc<RwLock<ClientChunkMap>>, chunks : &mut HashMap<ChunkPos,Vec<u16>>) -> u8 {
        let mut count = 0;
        let pos_vec = ChunkMap::get_neighbours_chunks_pos(chunk_pos);

        // checking every side of the chunk
        let chunk_map = arc_chunk_map.read().unwrap();
        for p in pos_vec.iter() {
            // the position is already in the map, no need to check if it exists
            if !chunks.contains_key(p) && p.y >= -2 && p.y <= 9 {
                // checking if the chunk exists in the world_data data as it doesn't exist in the map
                let result = chunk_map.get_chunk(p);
                // getting the object associated with the pos, checking that the chunk exists and adding it into the map
                if result.is_none() {
                    continue;
                }
                // the result is some, adding it into the map
                chunks.entry(*p).or_insert(result.unwrap().get_blocks().clone());
            }
            count += 1;
        }
/*        for p in pos_vec.iter() {
            if !chunks.contains_key(p) && count == 7 {
                print_base!("Bug on pos {} at {}",chunk_pos.deref(), p.deref());
            }
        }*/
        count
    }
}