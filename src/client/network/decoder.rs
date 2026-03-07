use bitvec::order::Lsb0;
use bitvec::view::BitView;
use bytes::{BytesMut, Buf};
use shared::common::network::server::packet::ServerPacket;
use tokio_util::codec::{Decoder, FramedRead};
use shared::common::network::packet_type::ServerPacketType;
use shared::print_base;

pub struct PacketCodec;

impl Decoder for PacketCodec {
    type Item = ServerPacket;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Step 1: Need at least 1 byte for Packet Type
        if src.is_empty() { return Ok(None); }

        // Step 2: Peek at type to find header size
        let packet_type = ServerPacketType::from_repr(src[0]).unwrap();
        let header_size = ServerPacketType::get_header_size(packet_type);

        // Step 3: Do we have the full header?
        if src.len() < 1 + header_size {
            return Ok(None); // Tell Tokio: "Wait for more data"
        }

        // Step 4: Peek at header to find body size
        // (Assuming body size is at a fixed offset in your header)
        let body_size = ServerPacketType::get_body_size(packet_type, &src[1..header_size+1].view_bits::<Lsb0>().to_bitvec());

        // Step 5: Do we have the full packet?
        if src.len() < 1 + header_size + body_size {
            return Ok(None); // Still not enough
        }
        // print_base!("Decoding packet");
        // Step 6: We have everything! Remove bytes from buffer and parse.
        let full_packet_data = src.split_to(header_size + body_size + 1);
        let packet = ServerPacket::from_bits(packet_type, full_packet_data.view_bits::<Lsb0>());

        Ok(Some(packet))
    }
}