use std::io::{Error, ErrorKind};
use std::ops::Mul;
use std::sync::Arc;
use bitvec::macros::internal::funty::Fundamental;
use noise::{NoiseFn, Perlin};
use shared::common::network::client::login_packet::LoginPacket;
use shared::common::network::l5_packet::L5Packet;
use shared::common::network::packet_type::L5PacketType;
use shared::common::network::packet_type::UdpPacketType::{Ack, Reliable};
use shared::common::network::reliable_packets::{AckPacket, ReliablePacket};
use shared::common::network::server::chunk_packet::ChunkPacket;
use shared::common::network::server::connection_packet::ConnectionPacket;
use shared::common::network::udp_packet::UdpPacket;
use shared::common::world::block::block::BlockType;
use shared::common::world::chunk::chunk::Chunk;
use shared::common::world::chunk::chunkmap::ChunkMap;
use shared::common::world::pos::chunkpos::{ChunkPos, CHUNK_SIZE};
use shared::print_base;
use crate::client::world_data::chunks::chunk_map::ClientChunkMap;
use crate::server::world_data::chunk::chunk_map::ServerChunkMap;

#[path="../client/mod.rs"] pub mod client;
#[path="../server/mod.rs"] pub mod server;


#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

pub fn main() -> Result<(), String> {
    // #[cfg(feature = "dhat-heap")]
    // let _profiler = dhat::Profiler::new_heap();
    test_l5_serialization()?;
    test_udp_serialization()?;
    test_chunk_loading()?;
    test_meshing()?;
    test_generation()?;
    Ok(())
}

pub fn test_l5_serialization() -> Result<(), String> {
    let packet : L5Packet = L5Packet::Login(LoginPacket::new(10,"aaaa"));
    let serialized = packet.encode().into_vec();
    let deserialized = L5Packet::decode(serialized).unwrap();
    assert_eq!(deserialized.get_packet_type(), L5PacketType::Login);
    if let L5Packet::Login(login) = deserialized {
        assert_eq!(login.get_puid().id(), 10);
        assert_eq!(login.get_password(), "aaaa");
    }
    Ok(())
}

pub fn test_generation() -> Result<(), String> {
    let perlin = Perlin::new(1);
    let frequency = 0.003;
    for x in 0..1000 {
        for y in 0..1000 {
            for z in 0..1000 {
                let noise = (perlin.get([x as f64 * frequency, y as f64 * frequency, z as f64 * frequency]) + 1.0).mul(10.0).log(20.0);
                let terrain_noise = noise * y as f64 / 1000.0;
                if terrain_noise > 0.9 {
                    print_base!("Value {},{},{}",x, y, z);
                    return Ok(())
                }
            }
        }
    }
    Err(String::from("Value above 0.95 not found"))
}

pub fn test_udp_serialization() -> Result<(), String> {
    let packet : UdpPacket = UdpPacket::Reliable(ReliablePacket::new(2_u32.pow(31) - 1,L5Packet::Login(LoginPacket::new(0,"bbbb"))));
    let serialized = packet.encode().into_vec();
    print_base!("Vec {:?}", serialized);
    let deserialized = UdpPacket::decode(&serialized).unwrap();
    assert!(matches!(deserialized, UdpPacket::Reliable(_)));
    if let UdpPacket::Reliable(reliable_packet) = deserialized {
        assert_eq!(reliable_packet.get_l5_packet().get_packet_type(),L5PacketType::Login);
    }

    let packet : UdpPacket = UdpPacket::Ack(AckPacket::new(100000));
    let serialized = packet.encode().into_vec();
    print_base!("Vec {:?}", serialized);
    let deserialized = UdpPacket::decode(&serialized).unwrap();
    assert_eq!(deserialized.get_packet_type(), Ack);
    if let UdpPacket::Ack(ack_packet) = deserialized {
        assert_eq!(ack_packet.get_ack(),100000);
    }
    Ok(())
}

pub fn test_meshing() -> Result<(), String> {
    let (sx, c_rx) = crossbeam::channel::bounded(100);
    let mut client_cm = ClientChunkMap::new(sx);
    for i in 0..27 {
        let pos = ChunkPos::new(i / 9 % 3, i / 3 % 3, i % 3);
        let mut vec = Vec::new();
        vec.resize(CHUNK_SIZE.pow(3), shared::common::world::block::block::BlockType::AIR.get_value());
        let chunk = Chunk::new(pos, Arc::new(vec));
        for packet in ChunkPacket::from_chunk_to_packets(&chunk) {
            client_cm.add_temp_chunk(packet);
        }
        c_rx.try_iter();
    }

    assert_eq!(ChunkMap::get_neighbours_chunks_pos(ChunkPos::new(1, 1, 1)).len(), 27);
    Ok(())
}

pub fn test_chunk_loading() -> Result<(), String> {
    let view_distance : i32 = 10;
    let (mut v1, mut v2) = ServerChunkMap::compute_chunk_diff(ChunkPos::new(0, 0, 0), ChunkPos::new(0, 1, 0), 0, view_distance);
    assert_eq!(v1.len(), (view_distance as usize * 2 + 1).pow(2)  * 13);
    v1.retain(|c| {
        view_distance < (*c).x && (*c).x < -view_distance && view_distance < (*c).z && (*c).z < -view_distance && 12 < (*c).y && (*c).x < 0
    });
    assert!(v1.is_empty());
    assert_eq!(v2.len(), 0);

    let view_distance : i32 = 5;
    let (mut v1, mut v2) = ServerChunkMap::compute_chunk_diff(ChunkPos::new(0, 0, 0), ChunkPos::new(0, 0, 0), 0, view_distance);
    for i in 0..v1.len()-1 {
        // print!("{}",v1[i].get_vec3().abs());
        let x_abs = v1[i].x.abs();
        let x1_abs = v1[i + 1].x.abs();
        let z_abs = v1[i].z.abs();
        let z1_abs = v1[i + 1].z.abs();
        // print_base!("{}<={}&&{}<={}",x_abs,x1_abs,z_abs,z1_abs);

        assert!((x_abs <= x1_abs || z_abs <= z1_abs));
    }
    assert_eq!(v2.len(), 0);
    Ok(())
}