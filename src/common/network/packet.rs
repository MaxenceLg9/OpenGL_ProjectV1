pub trait Packet{
    fn packet_type(&self) -> PacketType;

    fn serialize(&self) -> String;
}

pub trait P  {

}

pub enum PacketType{
    Correction,
    Update,
    Connect,
    Quit,
}


pub trait Aa: Packet + P {

}