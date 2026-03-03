use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use shared::common::world::pos::chunkpos::ChunkPos;
use crate::client::display::renderer::mesh::chunk_mesh::ChunkMesh;

pub struct MeshMap {
    meshes: HashMap<ChunkPos, Arc<ChunkMesh>>,
}

impl MeshMap {
    pub fn new() -> Self {
        Self {
            meshes: HashMap::new(),
        }
    }

    pub fn add_mesh(&mut self, pos : ChunkPos, mesh: ChunkMesh) -> bool {
        match self.meshes.entry(pos) {
            Entry::Occupied(_) => {
                false
            }
            Entry::Vacant(slot) => {
                slot.insert(Arc::new(mesh));
                true
            }
        }
    }

    pub fn contains_mesh(&self, mesh_pos: &ChunkPos) -> bool {
        self.meshes.contains_key(mesh_pos)
    }

    pub fn get_mesh(&self, mesh_pos: &ChunkPos) -> Arc<ChunkMesh> {
        self.meshes.get(mesh_pos).unwrap().clone()
    }

    pub fn remove_mesh(&mut self, mesh_pos : &ChunkPos) {
        self.meshes.remove(mesh_pos);
    }
}

impl Deref for MeshMap {
    type Target = HashMap<ChunkPos,Arc<ChunkMesh>>;

    fn deref(&self) -> &HashMap<ChunkPos,Arc<ChunkMesh>> {
        &self.meshes
    }

}

impl DerefMut for MeshMap {
    fn deref_mut(&mut self) -> &mut HashMap<ChunkPos,Arc<ChunkMesh>> {
        &mut self.meshes
    }
}