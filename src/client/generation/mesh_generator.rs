use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use crossbeam::channel as cb;
use shared::common::world::pos::chunkpos::ChunkPos;
use shared::{print_base, print_debug};
use crate::client::display::renderer::gui::text::text::MeshText;
use crate::client::generation::mesh::chunk_mesh::ChunkMesh;

pub struct MeshGenerator;

impl MeshGenerator {

    pub fn start_build_meshes(pos_to_mesh_rx: cb::Receiver<(ChunkPos, HashMap<ChunkPos, Arc<Vec<u16>>>)>, mesh_sender: cb::Sender<(ChunkMesh, MeshText)>) {
        for i in 0..4 {
            let sender_clone = mesh_sender.clone();
            let receiver_clone = pos_to_mesh_rx.clone();
            std::thread::Builder::new()
                .name("ChunkMesh_generator".to_string())
                .spawn(move || {
                    Self::build_meshes(receiver_clone, sender_clone);
                }).unwrap();
        }
    }

    fn build_meshes(pos_to_mesh_rx: cb::Receiver<(ChunkPos, HashMap<ChunkPos, Arc<Vec<u16>>>)>, mesh_sender: cb::Sender<(ChunkMesh, MeshText)>) {
        while let Ok((chunk_pos, chunks)) = pos_to_mesh_rx.recv() {
            if let Some(meshes) = ChunkMesh::build_mesh(&chunks, chunk_pos) {
                if let Err(e) = mesh_sender.send(meshes) {
                    print_base!("Error sending mesh: {:?}", e);
                    return;
                }
            }
        }
    }
}

