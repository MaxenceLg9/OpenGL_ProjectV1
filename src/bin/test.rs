use std::ops::{Add, Deref, Mul};
use bitvec::macros::internal::funty::Fundamental;
use noise::{NoiseFn, Perlin};
use shared::common::world::pos::chunkpos::{ChunkPos, CHUNK_SIZE};
use shared::common::world::pos::iblockpos::IBlockPos;
use shared::math::get_terrain_height;
use shared::print_base;
use crate::server::world_data::chunk::chunk::ServerChunk;
use crate::server::world_data::chunk::chunk_map::ServerChunkMap;

#[path="../client/mod.rs"] pub mod client;
#[path="../server/mod.rs"] pub mod server;


#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

pub fn main() -> Result<(), String> {
    // #[cfg(feature = "dhat-heap")]
    // let _profiler = dhat::Profiler::new_heap();
    let test : f64 = -10.0;
    print_base!("Value : {}, casted value {}", test, test.as_f32());
    Ok(())
}