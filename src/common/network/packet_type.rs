use strum::{Display, FromRepr};
#[derive(Clone, Copy, Debug, PartialEq, FromRepr, Display)]
#[repr(u8)]
pub enum UdpPacketType {
    Reliable,
    Simple,
    Ack
}

#[derive(Clone, Copy, Debug, PartialEq, FromRepr, Display)]
#[repr(u8)]
pub enum ConnectionState {
    TLS,
    Login,
    Ok,
    Quit,
}
#[derive(Clone, Copy, Debug, PartialEq, FromRepr, Display)]
#[repr(u8)]
pub enum L5PacketType {
    TLS = 0,
    Login = 10,
    Connect = 11,
    GetPlayer = 20,
    UpdatePlayer = 21,
    Chunk = 30,
    Block = 40,
    Correction = 50,
    Quit = 90,
}