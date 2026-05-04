use std::time::Instant;
use tokio::sync::mpsc::Receiver;
use shared::common::account::puid::PUID;
use shared::common::network::packet_type::ConnectionState;

pub struct CState {
    cstate: ConnectionState,
    time: Instant,
    puid: Option<PUID>,
    receiver: tokio::sync::mpsc::Receiver<Vec<u8>>
}

impl CState {
    pub fn new(prx : tokio::sync::mpsc::Receiver<Vec<u8>>) -> Self {
        Self {
            cstate: ConnectionState::Login,
            time: Instant::now(),
            receiver: prx,
            puid: None
        }
    }

    pub fn set_puid(&mut self, puid : PUID) {
        self.puid = Some(puid)
    }

    pub fn get_puid(&self) -> Option<PUID> {
        self.puid
    }

    pub fn set_state(&mut self, state : ConnectionState) {
        self.cstate = state;
    }

    pub fn get_state(&self) -> ConnectionState {
        self.cstate
    }

    pub fn get_receiver(&mut self) -> &mut Receiver<Vec<u8>> {
        &mut self.receiver
    }

    pub fn refresh_time(&mut self) {
        self.time = Instant::now();
    }
}