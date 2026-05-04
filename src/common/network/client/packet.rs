use std::fmt::{Formatter};
use std::fmt::Display;
use bitvec::view::BitView;
use crate::common::network::network_traits::ClientNetPacket;
use crate::common::network::bit_cursor::BitCursor;
use bitvec::order::Lsb0;
use bitvec::prelude::{BitVec};
use crate::common::network::client::ask_chunk::AskChunkPacket;
use crate::common::network::client::login_packet::LoginPacket;
use crate::common::network::client::player_packet::UpdatePlayerPacket;
use crate::common::network::packet_type::ClientPacketType;
use crate::common::network::packet_type::ClientPacketType::{AskChunk, Login, UpdatePlayer, Quit};

macro_rules! register_packets {
    ($enum_name:ident, { $($variant_name:ident = {$struct_type:ident, $packet_type:ident}),* $(,)? }) => {
        pub enum $enum_name {
            $($variant_name($struct_type),)*
        }

        impl $enum_name {
            /// Takes the ID and the cursor, returns the specific packet variant
            pub fn decode(vec: &Vec<u8>) -> Option<Self> {
                let mut cursor = BitCursor::new(vec.view_bits::<Lsb0>());
                let result = ClientPacketType::from_repr(cursor.read_bits::<u8>(8));
                if result.is_none() {
                    return None
                }
                let client_packet_type = result.unwrap();
                match client_packet_type {
                    $( $struct_type::P_TYPE => Some($enum_name::$variant_name($struct_type::deserialize(&mut cursor))), )*
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

            pub fn get_packet_type(&self) -> ClientPacketType {

                match self {
                    $( $enum_name::$variant_name(p) => $packet_type, )*
                }
            }
        }

        impl Display for $enum_name {

                fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
                    match self {
                        $( $enum_name::$variant_name(p) => write!(f, "ClientPacket::{} -> {}", stringify!($variant_name), p), )*
                    }
                }
        }
    };
}

// Usage:
register_packets!(ClientPacket, {
    Login = {LoginPacket, Login},
    AskChunk = {AskChunkPacket, AskChunk},
    Quit = {AskChunkPacket, Quit},
    UpdatePlayer = {UpdatePlayerPacket, UpdatePlayer},
});