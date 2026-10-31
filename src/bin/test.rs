#[path = "../direct-client/mod.rs"]
pub mod client;
#[path = "../server/mod.rs"]
pub mod server;

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use shared::{
        common::{
            error::err_type::GameError,
            network::{
                client::login_packet::LoginPacket,
                l5_packet::L5Packet,
                packet_type::{L5PacketType, UdpPacketType::Ack},
                reliable_packets::{AckPacket, ReliablePacket},
                server::chunk_packet::ChunkPacket,
                udp_packet::UdpPacket,
            },
            world::{
                chunk::{chunk::Chunk, chunkmap::ChunkMap},
                pos::chunkpos::{CHUNK_SIZE, ChunkPos},
            },
        },
        print_base,
    };

    use crate::{client::world_data::chunks::chunk_map::ClientChunkMap, server::world_data::chunk::chunk_map::ServerChunkMap};

    #[test]
    fn test_l5_serialization() -> Result<(), GameError> {
        let packet: L5Packet = L5Packet::Login(LoginPacket::new(10, "aaaa"));
        let serialized = packet.encode().into_vec();
        let deserialized = L5Packet::decode(serialized)?;
        assert_eq!(deserialized.get_packet_type(), L5PacketType::Login);
        if let L5Packet::Login(login) = deserialized {
            assert_eq!(login.get_puid().id(), 10);
            assert_eq!(login.get_password(), "aaaa");
        }
        Ok(())
    }

    #[test]
    fn test_udp_serialization() -> Result<(), GameError> {
        let packet: UdpPacket = UdpPacket::Reliable(ReliablePacket::new(2_u32.pow(31) - 1, L5Packet::Login(LoginPacket::new(0, "bbbb"))));
        let serialized = packet.encode()?.into_vec();
        let deserialized = UdpPacket::decode(&serialized).unwrap();
        assert!(matches!(deserialized, UdpPacket::Reliable(_)));
        if let UdpPacket::Reliable(reliable_packet) = deserialized {
            assert_eq!(reliable_packet.get_l5_packet().get_packet_type(), L5PacketType::Login);
        }

        let packet: UdpPacket = UdpPacket::Ack(AckPacket::new(100000));
        let serialized = packet.encode()?.into_vec();
        let deserialized = UdpPacket::decode(&serialized).unwrap();
        assert_eq!(deserialized.get_packet_type(), Ack);
        if let UdpPacket::Ack(ack_packet) = deserialized {
            assert_eq!(ack_packet.get_ack(), 100000);
        }
        Ok(())
    }

    #[test]
    pub fn test_meshing() -> Result<(), String> {
        let (sx, c_rx) = crossbeam::channel::bounded(100);
        let mut client_cm = ClientChunkMap::new(sx);
        for i in 0..27 {
            let pos = ChunkPos::new(i / 9 % 3, i / 3 % 3, i % 3);
            let mut vec = Vec::new();
            vec.resize(CHUNK_SIZE.pow(3), 0);
            let chunk = Chunk::new(pos, Arc::new(vec));
            for packet in ChunkPacket::from_chunk_to_packets(&chunk).unwrap() {
                client_cm.add_temp_chunk(packet);
            }
            c_rx.try_iter();
        }

        assert_eq!(ChunkMap::get_neighbours_chunks_pos(ChunkPos::new(1, 1, 1)).len(), 27);
        Ok(())
    }

    #[test]
    fn test_chunk_loading() -> Result<(), String> {
        let view_distance: i32 = 10;
        let (mut v1, mut v2) = ServerChunkMap::compute_chunk_diff(ChunkPos::new(0, 0, 0), ChunkPos::new(0, 1, 0), 0, view_distance);
        // assert_eq!(v1.len(), (view_distance as usize * 2 + 1).pow(2) * 13);
        v1.retain(|c| view_distance < (*c).x && (*c).x < -view_distance && view_distance < (*c).z && (*c).z < -view_distance && 12 < (*c).y && (*c).x < 0);
        assert!(v1.is_empty());
        assert_eq!(v2.len(), 0);

        let view_distance: i32 = 5;
        let (mut v1, mut v2) = ServerChunkMap::compute_chunk_diff(ChunkPos::new(0, 0, 0), ChunkPos::new(0, 0, 0), 0, view_distance);
        for i in 0..v1.len() - 1 {
            // print!("{}",v1[i].get_vec3().abs());
            let x_abs = v1[i].x.abs();
            let x1_abs = v1[i + 1].x.abs();
            let z_abs = v1[i].z.abs();
            let z1_abs = v1[i + 1].z.abs();
            // print_base!("{}<={}&&{}<={}",x_abs,x1_abs,z_abs,z1_abs);

            assert!((x_abs <= x1_abs || z_abs <= z1_abs));
        }
        assert!(v1.contains(&ChunkPos::new(1, -4, 1)));
        assert_eq!(v2.len(), 0);
        Ok(())
    }
}
