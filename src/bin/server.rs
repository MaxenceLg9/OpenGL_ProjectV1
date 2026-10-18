use std::io::Error;

use crate::server::world::ServerWorld;
use shared::print_base;

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;
#[path = "../server/mod.rs"]
mod server;
pub fn main() -> Result<(), Error> {
    // #[cfg(feature = "dhat-heap")]
    // let _profiler = dhat::Profiler::new_heap();
    print_base!("Starting server at {}", chrono::offset::Local::now());
    let mut server = ServerWorld::new()?;
    print_base!("Listening server at {}", chrono::offset::Local::now());
    server.listen()
}
