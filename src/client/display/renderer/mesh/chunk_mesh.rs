use std::collections::HashMap;
use std::sync::Arc;
use std::vec::Vec;
use bitvec::order::Lsb0;
use bitvec::prelude::BitVec;
use bitvec::view::BitView;
use gl::types::{GLuint};
use gl::{TRIANGLES, UNSIGNED_INT};
use glam::*;
use zstd::compression_level_range;
use shared::common::display::vertex::vertex::Vertex;
use shared::common::network::server::chunk_packet::ChunkPacket;
use shared::common::world::chunk::chunk::Chunk;
use shared::common::world::pos::chunkpos::{ChunkPos, CHUNK_SIZE};
use shared::common::world::pos::iblockpos::IBlockPos;
use shared::print_debug;

#[derive(Clone)]
pub struct Mesh {
    vao : GLuint,
    nb_indices: i32,
}


impl Mesh {
    pub fn new(vao : GLuint, vbo : GLuint, ebo : GLuint, nb_indices : i32) -> Mesh {
        Self {
            vao,
            nb_indices,
        }
    }




    pub unsafe fn draw(&self,) {
        if self.nb_indices == 0 {
            return;
        }
        gl::BindVertexArray(self.vao);
        gl::DrawElementsBaseVertex(TRIANGLES,self.nb_indices,UNSIGNED_INT, std::ptr::null(),0);
        gl::BindVertexArray(0);
    }
}