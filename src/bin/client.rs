use std::net::{Ipv6Addr, SocketAddrV6};
use winit::event_loop::{ControlFlow, EventLoop};
use glutin::config::{ConfigTemplateBuilder};
use client::display::app::App;
use shared::common::network::network_traits::{PacketTrait};
use shared::common::network::client::connection_packet::ConnectionPacket;
use shared::common::network::client::packet::ClientPacket;
use shared::print_base;
use crate::client::network::socket::ClientSocket;
use client::world_data::world::ClientWorld;

#[path="../client/mod.rs"] pub mod client;

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

pub fn main() -> Result<(), String> {
    // #[cfg(feature = "dhat-heap")]
    // let _profiler = dhat::Profiler::new_heap();

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    // The template will match only the configurations supporting rendering
    // to windows.
    //
    // XXX We force transparency only on macOS, given that EGL on X11 doesn't
    // have it, but we still want to show window. The macOS situation is like
    // that, because we can query only one config at a time on it, but all
    // normal platforms will return multiple configs, so we can find the config
    // with transparency ourselves inside the `reduce`.
    let template = ConfigTemplateBuilder::new().with_alpha_size(8).with_transparency(false);
    let mut app = App::new(template);
    event_loop.run_app(&mut app).expect("Failed to run the event loop");

    app.exit_state()
}

// pub fn main() -> Result<(), String> {
//     #[cfg(feature = "dhat-heap")]
//     let _profiler = dhat::Profiler::new_heap();
//     let ipv6_address = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1);
//
//     let mut c = ClientWorld::new();
//     c.connect_to();
//     Ok(())
// }