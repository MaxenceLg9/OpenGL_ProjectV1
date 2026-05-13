use strum::{Display, FromRepr};

#[derive(Clone, Copy, Debug, PartialEq, FromRepr, Display)]
#[repr(u8)]
pub enum ClientPacketType {
    Login = 0,
    ClientQuit = 1,
    UpdatePlayer    = 2,
}

#[derive(Clone, Copy, Debug, PartialEq, FromRepr, Display)]
#[repr(u8)]
pub enum ServerPacketType {
    Chunk = 3,
    GetPlayer = 2,
    TLS = 0,
    BlockDestroyed = 4,
    Correction = 5,
    ServerQuit = 6,
    Connect = 1
}
#[derive(Clone, Copy, Debug, PartialEq, FromRepr, Display)]
#[repr(u8)]
pub enum PacketType {
    Reliable,
    Simple,
}

#[derive(Clone, Copy, Debug, PartialEq, FromRepr, Display)]
#[repr(u8)]
pub enum ConnectionState {
    TLS,
    Login,
    Ok,
    Quit,
}