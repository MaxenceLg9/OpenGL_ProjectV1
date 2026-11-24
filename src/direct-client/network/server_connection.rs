use crate::client::game::client_data::ClientWorldData;
use crate::client::game::event::event::{ClientEvent, ClientEventType};
use shared::common::error::err_type::{CustomErrorKind, GameError};
use shared::common::network::client::login_packet::LoginPacket;
use shared::common::network::client::player_packet::UpdatePlayerPacket;
use shared::common::network::l5_packet::L5Packet;
use shared::common::network::packet_type::UdpPacketType;
use shared::common::network::socket::common_socket::CommonSocket;
use shared::common::network::udp_packet::TIMEOUT_DURATION;
use shared::{custom_error, print_base};
use std::io::Error;
use std::net::{Ipv4Addr, SocketAddr};
use std::ops::Deref;
use std::sync::Arc;
use std::time::Instant;
use tokio::time::timeout;

pub struct ServerConnection {
    socket: CommonSocket,
    client_world_data: Arc<ClientWorldData>,
    rx: tokio::sync::mpsc::Receiver<(L5Packet, UdpPacketType)>,
    start_time: Instant,
    event_sx: crossbeam::channel::Sender<ClientEvent>,
}

impl ServerConnection {
    pub fn start(sockaddr: SocketAddr, rx: tokio::sync::mpsc::Receiver<(L5Packet, UdpPacketType)>, client_world_data: Arc<ClientWorldData>, event_sx: crossbeam::channel::Sender<ClientEvent>) -> Result<(), Error> {
        std::thread::Builder::new()
            .name("network_thread".to_string())
            .spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();

                rt.block_on(async {
                    let mut socket = ServerConnection::new(rx, client_world_data, event_sx).await.expect("Failed to create a socket");
                    let addr = socket.connect(sockaddr).await.expect("Socket failed to connect");
                    socket.handle_server(&addr).await.expect("Error while listening");
                    print_base!("Closed connection from {} at {}", addr, chrono::offset::Local::now());
                });
            })
            .map(|_| ())
    }
    pub async fn new(rx: tokio::sync::mpsc::Receiver<(L5Packet, UdpPacketType)>, client_world_data: Arc<ClientWorldData>, event_sx: crossbeam::channel::Sender<ClientEvent>) -> Result<Self, GameError> {
        let socket = CommonSocket::new(SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), 50003)).await?;
        Ok(Self {
            socket,
            rx,
            start_time: Instant::now(),
            client_world_data,
            event_sx,
        })
    }

    async fn connect(&mut self, sockaddr: SocketAddr) -> Result<SocketAddr, GameError> {
        self.socket.connect(sockaddr).await?;
        // let (psx, prx) = tokio::sync::mpsc::channel(10000);
        print_base!("ClientSocket connecting to {:?}", sockaddr);

        self.socket.flush();
        self.socket
            .send_to(L5Packet::Login(LoginPacket::new(2000, "maxence")), &sockaddr, UdpPacketType::Reliable)
            .await
            .expect("Panic when sending connection packet");

        let (packet, _addr) = self.socket.recv_from().await?;
        let L5Packet::Connect(packet) = packet else {
            self.socket.flush();
            return Err(custom_error!(CustomErrorKind::InvalidData, "Expected Connect packet, got something else"));
        };
        self.event_sx
            .send(ClientEvent { client_event_type: ClientEventType::Connection(packet) })
            .map_err(|e| custom_error!(CustomErrorKind::SendError, e.to_string()))?;

        self.socket.peer_addr().map_err(|e| custom_error!(CustomErrorKind::NetworkSocket, e.to_string()))
    }

    async fn handle_server(&mut self, sockaddr: &SocketAddr) -> Result<(), GameError> {
        self.start_time = Instant::now();
        loop {
            tokio::select! {
                frame = timeout(TIMEOUT_DURATION, self.socket.recv_from()) => {
                    self.receive(frame).await?
                }
                Some((packet, udp_type)) = self.rx.recv() => {
                    if let L5Packet::Block(_blockpacket) = &packet {
                        print_base!("Sending block packet")
                    }
                    // print_base!("Sending packet {}",packet.get_packet_type());
                    self.socket.send_to(packet, sockaddr, udp_type).await.unwrap();
                }
            }
        }
    }

    async fn receive(&mut self, frame: Result<Result<(L5Packet, SocketAddr), GameError>, tokio::time::error::Elapsed>) -> Result<(), GameError> {
        match frame {
            Ok(Ok((packet, addr))) => self.handle_packet(packet, addr).await,
            Ok(Err(e)) => Err(e),
            Err(e) => Err(custom_error!(CustomErrorKind::NetworkSocket, e.to_string())),
        }
    }

    async fn handle_packet(&mut self, p: L5Packet, sockaddr: SocketAddr) -> Result<(), GameError> {
        match p {
            L5Packet::Chunk(chunk_packet) => {
                // print_base!("Receiving packets {}",p.get_packet_type());
                self.event_sx
                    .send(ClientEvent {
                        client_event_type: ClientEventType::ChunkPacketReceived(chunk_packet),
                    })
                    .map_err(|e| custom_error!(CustomErrorKind::SendError, e.to_string()))?;
                // self.chunk_sender.send(chunk_packet).expect("Cannot send the chunk to the map");
                Ok(())
            }
            L5Packet::GetPlayer(player_packet) => {
                let tick_packet = L5Packet::UpdatePlayer(UpdatePlayerPacket::new(self.client_world_data.get_player().read().unwrap().get_camera().get_coords(), 8, player_packet.get_id()));
                self.socket.send_to(tick_packet, &sockaddr, UdpPacketType::Simple).await?;
                Ok(())
            }
            L5Packet::Connect(packet) => {
                print_base!("Spawning on the world with coords : {}", packet.get_pos().deref());
                Ok(())
            }
            L5Packet::Block(packet) => {
                self.event_sx
                    .send(ClientEvent {
                        client_event_type: ClientEventType::BlockInteraction(packet),
                    })
                    .map_err(|e| custom_error!(CustomErrorKind::SendError, e.to_string()))?;
                Ok(())
            }
            _ => Ok(()),
        }
    }
}
