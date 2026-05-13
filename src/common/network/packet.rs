use crate::common::network::network_traits::NetPacket;
use bitvec::view::BitView;
use crate::common::network::bit_cursor::BitCursor;
use bitvec::prelude::Lsb0;
use bitvec::prelude::BitVec;
use crate::print_base;
use crate::common::network::packet_type::PacketType;
use crate::common::network::packet_type::PacketType::Reliable;
use crate::common::network::reliable_packets::ReliablePacket;

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


register_packets!(Packet, PacketType, {
    Reliable = {ReliablePacket, Reliable},
});