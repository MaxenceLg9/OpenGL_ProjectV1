use crate::common::network::network_traits::NetPacket;
use crate::common::network::network_traits::ServerNetPacket;
use crate::common::network::bit_cursor::BitCursor;
use bitvec::order::Lsb0;
use bitvec::prelude::{BitVec};
use bitvec::view::BitView;
use crate::common::network::client::sample_packet::SamplePacket;
use crate::common::network::packet_type::{ServerPacketType};
use crate::common::network::packet_type::ServerPacketType::{BlockDestroyed, Chunk, Connect, Correction, GetPlayer, ServerQuit as ServerQuit};
use crate::common::network::server::chunk_packet::ChunkPacket;
use crate::common::network::server::connection_packet::ConnectionPacket;
use crate::common::network::server::quit_packet::QuitPacket;
use crate::common::network::server::tick_player::GetPlayerPacket;
use crate::print_base;

use crate::common::network::network_traits::ClientNetPacket;
use crate::common::network::client::default_packet::DefaultPacket;
use crate::common::network::client::login_packet::LoginPacket;
use crate::common::network::client::player_packet::UpdatePlayerPacket;
use crate::common::network::packet_type::ClientPacketType;
use crate::common::network::packet_type::ClientPacketType::{Login, UpdatePlayer, ClientQuit as ClientQuit};

macro_rules! register_packets {
    ($enum_name:ident, $enum_type:ident, { $($variant_name:ident = {$struct_type:ident, $packet_type:ident}),* $(,)? }) => {
        #[derive(Clone)]
        pub enum $enum_name {
            $($variant_name($struct_type),)*
        }

        impl $enum_name {
            /// Takes the ID and the cursor, returns the specific packet variant
            pub fn decode(vec: &Vec<u8>) -> Option<Self> {
                let mut cursor = BitCursor::new(vec.view_bits::<Lsb0>());
                let byte = cursor.read_bits::<u8>(8);
                let result = $enum_type::from_repr(byte);
                if result.is_none() {
                    print_base!("There is no ServerPacketType from value {}", byte);
                    return None
                }
                let packet_type = result.unwrap();

                match packet_type {
                    $( $packet_type => Some($enum_name::$variant_name($struct_type::deserialize(&mut cursor))), )*
                    _ => None,
                }
            }

            /// Dispatches to the specific struct's serialize implementation
            pub fn encode(&self) -> BitVec<u8, Lsb0> {
                let mut vec = BitVec::new();
                vec.extend_from_bitslice((self.get_packet_type() as u8).view_bits::<Lsb0>());
                match self {
                    $( $enum_name::$variant_name(p) => vec.extend(p.serialize()), )*
                };
                vec
            }

            pub fn get_packet_type(&self) -> $enum_type {
                match self {
                    $( $enum_name::$variant_name(p) => $packet_type, )*
                }
            }
        }
    };
}

// Usage:
register_packets!(ServerPacket, ServerPacketType, {
    Correction = {SamplePacket, Correction},
    BlockDestroyed = {SamplePacket, BlockDestroyed},
    Chunk = {ChunkPacket, Chunk},
    GetPlayer = {GetPlayerPacket, GetPlayer},
    Quit = {QuitPacket, ServerQuit},
    Connect = {ConnectionPacket, Connect}
});

register_packets!(ClientPacket, ClientPacketType, {
    Login = {LoginPacket, Login},
    Quit = {DefaultPacket, ClientQuit},
    UpdatePlayer = {UpdatePlayerPacket, UpdatePlayer},
});