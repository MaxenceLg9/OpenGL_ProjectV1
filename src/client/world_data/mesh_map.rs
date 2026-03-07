use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use shared::common::world::pos::chunkpos::ChunkPos;
use crate::client::display::renderer::mesh::chunk_mesh::Mesh;

pub struct MeshMap {
    meshes: HashMap<ChunkPos, Mesh>,
}

impl MeshMap {
    pub fn new() -> Self {
        Self {
            meshes: HashMap::new(),
        }
    }

    pub fn add_mesh(&mut self, pos : ChunkPos, mesh: Mesh) -> bool {
        match self.meshes.entry(pos) {
            Entry::Occupied(_) => {
                false
            }
            Entry::Vacant(slot) => {
                slot.insert(mesh);
                true
            }
        }
    }

    pub fn contains_mesh(&self, mesh_pos: &ChunkPos) -> bool {
        self.meshes.contains_key(mesh_pos)
    }

    pub fn get_mesh(&self, mesh_pos: &ChunkPos) -> &Mesh {
        self.meshes.get(mesh_pos).unwrap()
    }

    pub fn remove_mesh(&mut self, mesh_pos : &ChunkPos) {
        self.meshes.remove(mesh_pos);
    }
}

impl Deref for MeshMap {
    type Target = HashMap<ChunkPos,Mesh>;

    fn deref(&self) -> &HashMap<ChunkPos,Mesh> {
        &self.meshes
    }

}

impl DerefMut for MeshMap {
    fn deref_mut(&mut self) -> &mut HashMap<ChunkPos,Mesh> {
        &mut self.meshes
    }
}