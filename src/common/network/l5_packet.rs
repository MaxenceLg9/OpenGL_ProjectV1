use std::io::ErrorKind;
use std::io::Error;
use crate::common::network::network_traits::L5PacketTrait;
use bitvec::view::BitView;
use crate::common::network::bit_cursor::BitCursor;
use bitvec::prelude::Lsb0;
use bitvec::prelude::BitVec;
use crate::common::network::client::login_packet::LoginPacket;
use crate::common::network::client::player_packet::UpdatePlayerPacket;
use crate::common::network::client::sample_packet::SamplePacket;
use crate::common::network::packet_type::L5PacketType;
use crate::common::network::packet_type::L5PacketType::{Chunk, Connect, Correction, GetPlayer, Quit, Login, TLS, UpdatePlayer, Block};
use crate::common::network::server::chunk_packet::ChunkPacket;
use crate::common::network::server::connection_packet::ConnectionPacket;
use crate::common::network::server::quit_packet::QuitPacket;
use crate::common::network::server::tick_packet::GetPlayerPacket;
use crate::print_base;


macro_rules! register_packets {
    ($enum_name:ident, $enum_type:ident, { $($variant_name:ident = {$struct_type:ident, $packet_type:ident}),* $(,)? }) => {
        #[derive(Clone)]
        pub enum $enum_name {
            $($variant_name($struct_type),)*
        }

        impl $enum_name {
            /// Takes the ID and the cursor, returns the specific packet variant
            pub fn decode(vec: Vec<u8>) -> Result<Self, Error> {
                let mut cursor = BitCursor::new(vec.view_bits::<Lsb0>());
                let byte = cursor.read_bits::<u8>(8);
                let result = $enum_type::from_repr(byte);
                if result.is_none() {
                    print_base!("There is no ServerPacketType from value {}", byte);
                    return Err(Error::new(ErrorKind::InvalidData, format!("There is no ServerPacketType from value {}", byte)));

                }
                let packet_type = result.unwrap();

                match packet_type {
                    $( $packet_type => Ok($enum_name::$variant_name($struct_type::deserialize(&mut cursor))), )*
                }
            }

            /// Dispatches to the specific struct's serialize implementation
            pub fn encode(&self) -> BitVec<u8, Lsb0> {
                let mut vec = BitVec::new();
                vec.extend_from_bitslice((self.get_packet_type() as u8).view_bits::<Lsb0>());
                match self {
                    $( $enum_name::$variant_name(p) => p.serialize(&mut vec), )*
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

register_packets!(L5Packet, L5PacketType, {
    Correction = {SamplePacket, Correction},
    Block = {SamplePacket, Block},
    Chunk = {ChunkPacket, Chunk},
    GetPlayer = {GetPlayerPacket, GetPlayer},
    Quit = {QuitPacket, Quit},
    Connect = {ConnectionPacket, Connect},
    Login = {LoginPacket, Login},
    UpdatePlayer = {UpdatePlayerPacket, UpdatePlayer},
    TLS = {SamplePacket, TLS},
});