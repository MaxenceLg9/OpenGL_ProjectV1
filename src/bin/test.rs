use std::ops::{Add, Deref, Mul};
use shared::common::world::pos::chunkpos::ChunkPos;
use shared::print_base;
use crate::server::world_data::chunk::chunk_map::ServerChunkMap;

#[path="../client/mod.rs"] pub mod client;
#[path="../server/mod.rs"] pub mod server;


#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

pub fn main() -> Result<(), String> {
    // #[cfg(feature = "dhat-heap")]
    // let _profiler = dhat::Profiler::new_heap();
    print_base!("Loaded {} chunks",ServerChunkMap::compute_chunk_diff(ChunkPos::from_i32(0,0,0), ChunkPos::from_i32(0,0,0),1,2).0.len());
    Ok(())
}