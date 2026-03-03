pub struct CommonChunkMesh {
    vertices: Vec<u32>,
    indices: Vec<u32>,
}

impl CommonChunkMesh {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn vpush(&mut self, vertex: u32) {
        self.vertices.push(vertex);
    }

    pub fn ipush(&mut self, index: u32) {
        self.indices.push(index);
    }

    pub fn ilen(&self) -> usize {
        self.indices.len()
    }

    pub fn ireserve(&mut self, size : usize) {
        self.indices.reserve(size);
    }

    pub fn vreserve(&mut self, size : usize) {
        self.vertices.reserve(size);
    }
}