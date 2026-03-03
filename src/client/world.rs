use std::net::{Ipv6Addr, SocketAddrV6};
use bitvec::vec::BitVec;
use winit::window::Window;
use crate::client::network::socket::ClientSocket;
use crate::client::world_data::player::player::ClientPlayer;

pub struct ClientWorld {
    socket : Option<ClientSocket>,
    player: ClientPlayer,
}

impl ClientWorld {
    pub fn new() -> ClientWorld {
        Self {
            socket : None,
            player: ClientPlayer::new(1.0,1.0,1.0)
        }
    }

    pub fn connect_to(&mut self){
        let ipv6_address = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1);
        self.socket = Some(ClientSocket::new(SocketAddrV6::new(ipv6_address, 25000, 0, 0).into()));
        let bits = BitVec::new();
        "J'aime la galette";
        self.socket.as_mut().unwrap().send(bits);
    }

    pub fn get_player(&mut self) -> &mut ClientPlayer {
        &mut self.player
    }

    pub fn render(&self, window: &Window){

    }
}