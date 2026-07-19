use shared::print_base;
use crate::server::world::ServerWorld;

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;
#[path="../server/mod.rs"] mod server;
pub fn main() -> Result<(), String> {
    // #[cfg(feature = "dhat-heap")]
    // let _profiler = dhat::Profiler::new_heap();
    print_base!("Starting server at {}", chrono::offset::Local::now());
    let mut server = ServerWorld::new();
    server.listen();
    Ok(())
}