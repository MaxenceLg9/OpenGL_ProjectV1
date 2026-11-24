//! Integration tests for the network packet layer of the `shared` library:
//! every packet must survive an encode -> decode round trip unchanged.

use shared::common::error::err_type::GameError;
use shared::common::network::client::block_packet::{BlockInteraction, BlockPacket};
use shared::common::network::client::login_packet::LoginPacket;
use shared::common::network::client::player_packet::UpdatePlayerPacket;
use shared::common::network::l5_packet::L5Packet;
use shared::common::network::packet_type::{L5PacketType, UdpPacketType};
use shared::common::network::reliable_packets::{AckPacket, ReliablePacket, SimplePacket};
use shared::common::network::server::chunk_packet::ChunkPacket;
use shared::common::network::server::connection_packet::ConnectionPacket;
use shared::common::network::server::quit_packet::QuitPacket;
use shared::common::network::server::tick_packet::GetPlayerPacket;
use shared::common::network::udp_packet::UdpPacket;
use shared::common::world::chunk::chunk::Chunk;
use shared::common::world::pos::blockpos::BlockPos;
use shared::common::world::pos::chunkpos::{CHUNK_SIZE, ChunkPos};
use shared::common::world::pos::iblockpos::IBlockPos;
use std::collections::HashMap;
use std::sync::Arc;

/// Size of the receive buffer in `CommonSocket`; any datagram bigger than this is truncated.
const SOCKET_BUFFER_SIZE: usize = 1300;

fn round_trip(packet: L5Packet) -> Result<L5Packet, GameError> {
    L5Packet::decode(packet.encode()?.into_vec())
}

/// Deterministic pseudo-random chunk (xorshift), with block ids in `0..max_id`.
fn noisy_chunk(pos: ChunkPos, max_id: u16) -> Chunk {
    let mut state: u32 = 0x9E37_79B9;
    let blocks = (0..CHUNK_SIZE.pow(3))
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            (state % max_id as u32) as u16
        })
        .collect();
    Chunk::new(pos, Arc::new(blocks))
}

// ---------------------------------------------------------------- L5 packets

#[test]
fn login_packet_round_trip() -> Result<(), GameError> {
    let L5Packet::Login(login) = round_trip(L5Packet::Login(LoginPacket::new(10, "aaaa")))? else {
        panic!("expected a Login packet");
    };
    assert_eq!(login.get_puid().id(), 10);
    assert_eq!(login.get_password(), "aaaa");
    Ok(())
}

#[test]
fn login_packet_keeps_utf8_and_empty_passwords() -> Result<(), GameError> {
    for password in ["", "mot de passe é ü 🔑"] {
        let L5Packet::Login(login) = round_trip(L5Packet::Login(LoginPacket::new(u32::MAX, password)))? else {
            panic!("expected a Login packet");
        };
        assert_eq!(login.get_puid().id(), u32::MAX);
        assert_eq!(login.get_password(), password);
    }
    Ok(())
}

#[test]
fn block_packet_round_trip() -> Result<(), GameError> {
    for interaction in [BlockInteraction::LEFT, BlockInteraction::RIGHT] {
        let pos = IBlockPos::new(-140, 108, 476);
        let L5Packet::Block(block) = round_trip(L5Packet::Block(BlockPacket::new(interaction, pos, 42)))? else {
            panic!("expected a Block packet");
        };
        assert_eq!(block.get_interaction_type(), interaction);
        assert_eq!(block.get_pos(), pos);
        assert_eq!(block.get_block_type(), 42);
    }
    Ok(())
}

#[test]
fn update_player_packet_round_trip() -> Result<(), GameError> {
    let pos = BlockPos::new(-12.25, 160.5, 1e6);
    let L5Packet::UpdatePlayer(update) = round_trip(L5Packet::UpdatePlayer(UpdatePlayerPacket::new(pos, 8, 65535)))? else {
        panic!("expected an UpdatePlayer packet");
    };
    assert_eq!(update.get_pos(), pos);
    assert_eq!(update.get_view_distance(), 8);
    assert_eq!(update.get_id(), 65535);
    Ok(())
}

#[test]
fn get_player_and_quit_packets_round_trip() -> Result<(), GameError> {
    let L5Packet::GetPlayer(get) = round_trip(L5Packet::GetPlayer(GetPlayerPacket::new(1234)))? else {
        panic!("expected a GetPlayer packet");
    };
    assert_eq!(get.get_id(), 1234);

    assert_eq!(round_trip(L5Packet::Quit(QuitPacket::new()))?.get_packet_type(), L5PacketType::Quit);
    Ok(())
}

#[test]
fn connection_packet_round_trip() -> Result<(), GameError> {
    let table = vec![("minecraft:air".to_owned(), 0), ("minecraft:grass".to_owned(), 1), ("minecraft:platinum_ore".to_owned(), 7)];
    let pos = BlockPos::new(32.0, 160.0, 32.0);
    let L5Packet::Connect(connect) = round_trip(L5Packet::Connect(ConnectionPacket::new(pos, table.clone())))? else {
        panic!("expected a Connect packet");
    };
    assert_eq!(connect.get_pos(), &pos);
    assert_eq!(connect.get_table(), &table);
    assert_eq!(connect.get_token(), &[0; 40]);
    Ok(())
}

#[test]
fn decoding_an_unknown_l5_packet_type_fails() {
    // 255 is not a valid L5PacketType discriminant
    assert!(L5Packet::decode(vec![255, 0, 0, 0]).is_err());
}

// --------------------------------------------------------------- UDP packets

#[test]
fn reliable_udp_packet_round_trip() -> Result<(), GameError> {
    let ack = u64::MAX - 1;
    let packet = UdpPacket::Reliable(ReliablePacket::new(ack, L5Packet::Login(LoginPacket::new(0, "bbbb"))));
    let UdpPacket::Reliable(reliable) = UdpPacket::decode(&packet.encode()?.into_vec())? else {
        panic!("expected a Reliable packet");
    };
    assert_eq!(reliable.get_ack(), ack);
    assert_eq!(reliable.get_l5_packet().get_packet_type(), L5PacketType::Login);
    Ok(())
}

#[test]
fn simple_udp_packet_round_trip() -> Result<(), GameError> {
    let packet = UdpPacket::Simple(SimplePacket::new(L5Packet::GetPlayer(GetPlayerPacket::new(3))));
    let UdpPacket::Simple(simple) = UdpPacket::decode(&packet.encode()?.into_vec())? else {
        panic!("expected a Simple packet");
    };
    let L5Packet::GetPlayer(get) = simple.get_l5_packet() else {
        panic!("expected a GetPlayer packet");
    };
    assert_eq!(get.get_id(), 3);
    Ok(())
}

#[test]
fn ack_udp_packet_round_trip() -> Result<(), GameError> {
    let packet = UdpPacket::Ack(AckPacket::new(100_000));
    let decoded = UdpPacket::decode(&packet.encode()?.into_vec())?;
    assert_eq!(decoded.get_packet_type(), UdpPacketType::Ack);
    let UdpPacket::Ack(ack) = decoded else { unreachable!() };
    assert_eq!(ack.get_ack(), 100_000);
    Ok(())
}

#[test]
fn decoding_an_unknown_udp_packet_type_fails() {
    assert!(UdpPacket::decode(&[200, 0, 0]).is_err());
}

// ------------------------------------------------------------- Chunk packets

#[test]
fn empty_chunk_fits_in_a_single_packet_and_round_trips() {
    let pos = ChunkPos::new(-3, 2, 7);
    let chunk = Chunk::new(pos, Arc::new(vec![0; CHUNK_SIZE.pow(3)]));

    let packets = ChunkPacket::from_chunk_to_packets(&chunk).expect("compression failed");
    assert_eq!(packets.len(), 1);
    assert_eq!(packets[0].get_total(), 1);
    assert_eq!(packets[0].get_chunk_pos(), pos);

    let map = packets.into_iter().map(|p| (p.get_indice(), p)).collect();
    let rebuilt = ChunkPacket::from_packets_to_chunk(map, pos);
    assert_eq!(rebuilt.get_chunk_pos(), pos);
    assert_eq!(*rebuilt.get_blocks(), *chunk.get_blocks());
}

#[test]
fn noisy_chunk_is_split_and_rebuilt_whatever_the_arrival_order() {
    let pos = ChunkPos::new(1, -4, -1);
    let chunk = noisy_chunk(pos, 8);

    let packets = ChunkPacket::from_chunk_to_packets(&chunk).expect("compression failed");
    assert!(packets.len() > 1, "a noisy chunk should need several packets");
    for (i, packet) in packets.iter().enumerate() {
        assert_eq!(packet.get_indice() as usize, i);
        assert_eq!(packet.get_total() as usize, packets.len());
    }

    // packets arrive in reverse order and go through the wire format
    let mut received = HashMap::new();
    for packet in packets.into_iter().rev() {
        let L5Packet::Chunk(decoded) = round_trip(L5Packet::Chunk(packet)).unwrap() else {
            panic!("expected a Chunk packet");
        };
        received.insert(decoded.get_indice(), decoded);
    }
    let rebuilt = ChunkPacket::from_packets_to_chunk(received, pos);
    assert_eq!(*rebuilt.get_blocks(), *chunk.get_blocks());
}

#[test]
fn every_encoded_chunk_packet_fits_in_the_socket_buffer() -> Result<(), GameError> {
    // worst realistic case: ids spread over 256 block types (8 bits of entropy per block)
    let chunk = noisy_chunk(ChunkPos::new(0, 0, 0), 256);
    let packets = ChunkPacket::from_chunk_to_packets(&chunk).expect("compression failed");
    assert!(packets.len() <= u8::MAX as usize, "`total` is stored on a u8");

    for packet in packets {
        // chunk packets are sent as Reliable, which adds the UDP header and the 64-bit ack
        let datagram = UdpPacket::Reliable(ReliablePacket::new(u64::MAX, L5Packet::Chunk(packet))).encode()?.into_vec();
        assert!(datagram.len() <= SOCKET_BUFFER_SIZE, "datagram of {} bytes would be truncated", datagram.len());
    }
    Ok(())
}
