use crate::common::network::{l5_packet::L5Packet, packet_type::UdpPacketType};

pub type PacketSender = tokio::sync::mpsc::Sender<(L5Packet, UdpPacketType)>;
