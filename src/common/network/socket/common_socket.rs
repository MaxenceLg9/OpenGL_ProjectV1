use crate::common::error::err_type::{CustomErrorKind, GameError};
use crate::common::network::l5_packet::L5Packet;
use crate::common::network::packet_type::{L5PacketType, UdpPacketType};
use crate::common::network::reliable_packets::{AckPacket, ReliablePacket, SimplePacket};
use crate::common::network::udp_packet::UdpPacket;
use crate::{custom_error, print_base};
use std::collections::{HashMap, HashSet, VecDeque};
use std::io::{Error, ErrorKind};
use std::net::SocketAddr;
use std::pin::Pin;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time::{Instant, Sleep};

/// Size of the receive buffer: bigger datagrams are truncated by the OS.
pub const BUFFER_SIZE: usize = 1300;
/// Delay before the first resend of an unacknowledged packet. It doubles at each attempt.
const RESEND_DELAY: Duration = Duration::from_millis(500);
/// Upper bound of the resend delay.
const MAX_RESEND_DELAY: Duration = Duration::from_secs(4);
/// Number of sends (first one included) before giving up on a peer (~30 s with the delays above).
const MAX_ATTEMPTS: u32 = 10;
/// Number of reliable acks remembered per peer to drop duplicated packets.
const DEDUP_WINDOW: usize = 512;
/// Deadline of the timer when no packet waits for an ack.
const IDLE: Duration = Duration::from_secs(3600);

pub type Ack = u64;

/// UDP socket with an optional reliability layer:
/// - `Simple` packets are fire and forget,
/// - `Reliable` packets are resent (with backoff) until the peer acks them,
///   and delivered only once to the receiver even if they arrive several times.
pub struct CommonSocket {
    socket: UdpSocket,
    buffer: [u8; BUFFER_SIZE],
    reliability: ReliabilityManager,
}

/// A reliable packet waiting for its ack.
struct Pending {
    /// already encoded datagram, so a resend doesn't encode it again
    bytes: Vec<u8>,
    deadline: Instant,
    attempts: u32,
}

/// The last `DEDUP_WINDOW` acks received from a peer.
/// It is a set rather than a sliding "highest ack" because a reconnecting peer starts from a new random ack.
#[derive(Default)]
struct ReceivedWindow {
    seen: HashSet<Ack>,
    order: VecDeque<Ack>,
}

impl ReceivedWindow {
    /// Returns `true` the first time an ack is seen.
    fn insert(&mut self, ack: Ack) -> bool {
        if !self.seen.insert(ack) {
            return false;
        }
        self.order.push_back(ack);
        if self.order.len() > DEDUP_WINDOW
            && let Some(oldest) = self.order.pop_front()
        {
            self.seen.remove(&oldest);
        }
        true
    }
}

struct ReliabilityManager {
    /// last ack used to send a reliable packet to each peer
    last_ack: HashMap<SocketAddr, Ack>,
    /// sent reliable packets that are not acked yet
    pending: HashMap<(SocketAddr, Ack), Pending>,
    /// reliable acks already received from each peer
    received: HashMap<SocketAddr, ReceivedWindow>,
    /// fires at the earliest deadline of `pending`
    timer: Pin<Box<Sleep>>,
}

fn network_error(e: impl ToString) -> GameError {
    custom_error!(CustomErrorKind::NetworkSocket, e.to_string())
}

fn encode(packet: &UdpPacket) -> Result<Vec<u8>, GameError> {
    Ok(packet.encode()?.into_vec())
}

/// Delay to wait after the `attempts`-th send: 500ms, 1s, 2s, 4s, 4s...
fn resend_delay(attempts: u32) -> Duration {
    RESEND_DELAY.saturating_mul(1 << attempts.saturating_sub(1).min(16)).min(MAX_RESEND_DELAY)
}

impl ReliabilityManager {
    fn new() -> Self {
        Self {
            last_ack: HashMap::new(),
            pending: HashMap::new(),
            received: HashMap::new(),
            timer: Box::pin(tokio::time::sleep(IDLE)),
        }
    }

    /// Returns the next ack for this peer: random for the first packet, then sequential.
    fn next_ack(&mut self, addr: SocketAddr) -> Ack {
        let ack = self.last_ack.entry(addr).and_modify(|ack| *ack = ack.wrapping_add(1)).or_insert_with(rand::random);
        *ack
    }

    #[cfg(test)]
    fn contains(&self, addr: &SocketAddr) -> bool {
        self.last_ack.contains_key(addr)
    }

    /// Wraps the packet in a `Reliable` one, keeps it until acked and returns the bytes to send.
    fn track(&mut self, addr: SocketAddr, packet: L5Packet) -> Result<Vec<u8>, GameError> {
        let ack = self.next_ack(addr);
        let bytes = encode(&UdpPacket::Reliable(ReliablePacket::new(ack, packet)))?;
        let deadline = Instant::now() + resend_delay(1);
        self.pending.insert((addr, ack), Pending { bytes: bytes.clone(), deadline, attempts: 1 });
        if deadline < self.timer.deadline() {
            self.timer.as_mut().reset(deadline);
        }
        Ok(bytes)
    }

    /// The peer received the packet: stop resending it.
    /// The timer is not moved, it recomputes its deadline when it fires.
    fn ack(&mut self, addr: SocketAddr, ack: Ack) {
        self.pending.remove(&(addr, ack));
    }

    /// Returns `true` the first time a reliable packet from this peer is received.
    fn first_reception(&mut self, addr: SocketAddr, ack: Ack) -> bool {
        self.received.entry(addr).or_default().insert(ack)
    }

    /// Forgets everything about the peer. Returns `false` if it was unknown.
    fn remove(&mut self, addr: &SocketAddr) -> bool {
        self.pending.retain(|(a, _), _| a != addr);
        self.received.remove(addr);
        self.last_ack.remove(addr).is_some()
    }

    /// Moves the timer to the earliest pending deadline.
    fn reschedule(&mut self) {
        let next = self.pending.values().map(|p| p.deadline).min().unwrap_or_else(|| Instant::now() + IDLE);
        self.timer.as_mut().reset(next);
    }
}

impl CommonSocket {
    pub async fn new(socket_addr: SocketAddr) -> Result<CommonSocket, GameError> {
        Ok(Self {
            socket: UdpSocket::bind(socket_addr).await?,
            buffer: [0; BUFFER_SIZE],
            reliability: ReliabilityManager::new(),
        })
    }

    pub async fn connect(&self, sockaddr: SocketAddr) -> Result<(), Error> {
        self.socket.connect(sockaddr).await
    }

    pub fn peer_addr(&self) -> Result<SocketAddr, Error> {
        self.socket.peer_addr()
    }

    pub fn local_addr(&self) -> Result<SocketAddr, Error> {
        self.socket.local_addr()
    }

    /// Registers a peer and returns the next ack that will be used for it.
    pub fn init_con(&mut self, sockaddr: SocketAddr) -> Ack {
        self.reliability.next_ack(sockaddr)
    }

    /// Drops every datagram already waiting in the OS buffer (needs a connected socket).
    pub fn flush(&mut self) {
        loop {
            match self.socket.try_recv(&mut self.buffer) {
                Ok(_) => continue,
                Err(e) if e.kind() == ErrorKind::WouldBlock => return,
                Err(e) => {
                    print_base!("Error while flushing the socket: {}", e);
                    return;
                }
            }
        }
    }

    /// Waits for the next data packet (`Simple` or new `Reliable`).
    /// Meanwhile, acks are processed and unacked packets are resent.
    ///
    /// Cancel safe: it can be used in a `tokio::select!` loop.
    pub async fn recv_from(&mut self) -> Result<(L5Packet, SocketAddr), GameError> {
        loop {
            tokio::select! {
                frame = self.socket.recv_from(&mut self.buffer) => {
                    let (size, addr) = frame.map_err(network_error)?;
                    if let Some(packet) = self.handle_datagram(size, addr).await {
                        return Ok((packet, addr));
                    }
                }
                // a packet sent hasn't been acked in time
                _ = &mut self.reliability.timer => self.resend_due().await,
            }
        }
    }

    /// Returns the L5 packet of the datagram, if there is a new one to deliver.
    async fn handle_datagram(&mut self, size: usize, addr: SocketAddr) -> Option<L5Packet> {
        let packet = match UdpPacket::decode(&self.buffer[..size]) {
            Ok(packet) => packet,
            Err(e) => {
                print_base!("Ignoring malformed datagram from {}: {}", addr, e);
                return None;
            }
        };
        match packet {
            UdpPacket::Reliable(packet) => {
                let ack = packet.get_ack();
                // acking duplicates too: they mean the previous ack was lost
                match encode(&UdpPacket::Ack(AckPacket::new(ack))) {
                    Ok(bytes) => {
                        if let Err(e) = self.socket.send_to(&bytes, addr).await {
                            print_base!("Cannot ack packet {} of {}: {}", ack, addr, e);
                        }
                    }
                    Err(e) => print_base!("Cannot encode ack {}: {}", ack, e),
                }
                self.reliability.first_reception(addr, ack).then(|| packet.get_l5_packet())
            }
            UdpPacket::Simple(packet) => Some(packet.get_l5_packet()),
            UdpPacket::Ack(packet) => {
                self.reliability.ack(addr, packet.get_ack());
                None
            }
        }
    }

    /// Resends every packet whose deadline passed, and gives up on peers that never answer.
    async fn resend_due(&mut self) {
        let now = Instant::now();
        let due: Vec<(SocketAddr, Ack)> = self.reliability.pending.iter().filter(|(_, p)| p.deadline <= now).map(|(key, _)| *key).collect();

        for key in due {
            // may have been removed by a previous `remove` of this loop
            let Some(pending) = self.reliability.pending.get_mut(&key) else {
                continue;
            };
            if pending.attempts >= MAX_ATTEMPTS {
                print_base!("{} did not ack packet {} after {} attempts, forgetting this peer", key.0, key.1, MAX_ATTEMPTS);
                self.reliability.remove(&key.0);
                continue;
            }
            // if cancelled during this await, the packet keeps its deadline and is resent at the next call
            if let Err(e) = self.socket.send_to(&pending.bytes, key.0).await {
                print_base!("Cannot resend packet {} to {}: {}", key.1, key.0, e);
            }
            pending.attempts += 1;
            pending.deadline = Instant::now() + resend_delay(pending.attempts);
        }
        self.reliability.reschedule();
    }

    /// Sends a Layer 5 packet to an address, wrapped in the given UDP packet type.
    /// Sending a `Quit` packet forgets the peer.
    pub async fn send_to(&mut self, packet: L5Packet, addr: &SocketAddr, udp_packet_type: UdpPacketType) -> Result<usize, GameError> {
        if packet.get_packet_type() == L5PacketType::Quit {
            self.close_connection(addr);
        }
        let bytes = match udp_packet_type {
            UdpPacketType::Reliable => self.reliability.track(*addr, packet)?,
            UdpPacketType::Simple => encode(&UdpPacket::Simple(SimplePacket::new(packet)))?,
            UdpPacketType::Ack => return Err(custom_error!(CustomErrorKind::InvalidData, "Type Ack is not valid to be sent")),
        };
        self.socket.send_to(&bytes, addr).await.map_err(network_error)
    }

    /// Forgets the peer: its unacked packets are not resent anymore.
    pub fn close_connection(&mut self, socket_addr: &SocketAddr) {
        self.reliability.remove(socket_addr);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::network::server::quit_packet::QuitPacket;
    use crate::common::network::server::tick_packet::GetPlayerPacket;

    const WAIT: Duration = Duration::from_secs(2);

    fn addr(port: u16) -> SocketAddr {
        SocketAddr::from(([127, 0, 0, 1], port))
    }

    fn get_player(id: u16) -> L5Packet {
        L5Packet::GetPlayer(GetPlayerPacket::new(id))
    }

    fn get_player_id(packet: L5Packet) -> u16 {
        match packet {
            L5Packet::GetPlayer(p) => p.get_id(),
            _ => panic!("expected a GetPlayer packet"),
        }
    }

    /// Pending `(address, ack)` sorted by ack.
    fn pending(manager: &ReliabilityManager) -> Vec<(SocketAddr, Ack)> {
        let mut keys: Vec<_> = manager.pending.keys().copied().collect();
        keys.sort_by_key(|k| k.1);
        keys
    }

    /// Two sockets bound on random localhost ports.
    async fn pair() -> (CommonSocket, SocketAddr, CommonSocket, SocketAddr) {
        let a = CommonSocket::new(addr(0)).await.unwrap();
        let b = CommonSocket::new(addr(0)).await.unwrap();
        let (a_addr, b_addr) = (a.local_addr().unwrap(), b.local_addr().unwrap());
        (a, a_addr, b, b_addr)
    }

    async fn recv(socket: &mut CommonSocket) -> (L5Packet, SocketAddr) {
        tokio::time::timeout(WAIT, socket.recv_from()).await.expect("nothing received").unwrap()
    }

    /// `recv_from` only returns on data packets; run it a little so it processes acks/resends.
    async fn pump(socket: &mut CommonSocket, duration: Duration) {
        assert!(tokio::time::timeout(duration, socket.recv_from()).await.is_err(), "unexpected data packet");
    }

    // ------------------------------------------------------ ReliabilityManager

    #[test]
    fn resend_delay_doubles_up_to_the_maximum() {
        let delays: Vec<_> = (1..=6).map(resend_delay).collect();
        let ms = |ms| Duration::from_millis(ms);
        assert_eq!(delays, vec![ms(500), ms(1000), ms(2000), ms(4000), ms(4000), ms(4000)]);
        assert_eq!(resend_delay(u32::MAX), MAX_RESEND_DELAY);
    }

    #[test]
    fn received_window_drops_duplicates_and_forgets_old_acks() {
        let mut window = ReceivedWindow::default();
        assert!(window.insert(u64::MAX));
        assert!(!window.insert(u64::MAX));
        for ack in 0..DEDUP_WINDOW as Ack {
            assert!(window.insert(ack));
        }
        assert_eq!(window.seen.len(), DEDUP_WINDOW);
        assert!(window.insert(u64::MAX), "the oldest ack was forgotten");
    }

    #[tokio::test]
    async fn acks_are_sequential_per_address() {
        let mut manager = ReliabilityManager::new();
        let first = manager.next_ack(addr(1));
        assert_eq!(manager.next_ack(addr(1)), first.wrapping_add(1));
        assert!(manager.contains(&addr(1)));
        assert!(!manager.contains(&addr(2)));
    }

    #[tokio::test]
    async fn ack_counter_wraps_instead_of_overflowing() {
        let mut manager = ReliabilityManager::new();
        manager.last_ack.insert(addr(1), Ack::MAX);
        assert_eq!(manager.next_ack(addr(1)), 0);
    }

    #[tokio::test]
    async fn ack_only_removes_the_matching_packet_of_the_matching_address() {
        let mut manager = ReliabilityManager::new();
        manager.last_ack.insert(addr(1), 10);
        manager.last_ack.insert(addr(2), 20);
        manager.track(addr(1), get_player(0)).unwrap();
        manager.track(addr(1), get_player(1)).unwrap();
        manager.track(addr(2), get_player(2)).unwrap();
        assert_eq!(pending(&manager), vec![(addr(1), 11), (addr(1), 12), (addr(2), 21)]);

        manager.ack(addr(1), 11);
        assert_eq!(pending(&manager), vec![(addr(1), 12), (addr(2), 21)]);

        // addr(2)'s ack number coming from addr(1) must not remove addr(2)'s packet
        manager.ack(addr(1), 21);
        manager.ack(addr(3), 21);
        assert_eq!(pending(&manager).len(), 2);
    }

    #[tokio::test]
    async fn tracking_a_packet_arms_the_timer() {
        let mut manager = ReliabilityManager::new();
        let before = Instant::now();
        manager.track(addr(1), get_player(0)).unwrap();
        assert!(manager.timer.deadline() <= before + RESEND_DELAY + Duration::from_millis(50));
        assert!(manager.timer.deadline() >= before + RESEND_DELAY);
    }

    #[tokio::test]
    async fn removing_an_address_forgets_everything_about_it() {
        let mut manager = ReliabilityManager::new();
        manager.track(addr(1), get_player(0)).unwrap();
        manager.track(addr(2), get_player(1)).unwrap();
        assert!(manager.first_reception(addr(1), 5));

        assert!(manager.remove(&addr(1)));
        assert!(!manager.contains(&addr(1)));
        assert!(pending(&manager).iter().all(|k| k.0 == addr(2)));
        assert!(manager.first_reception(addr(1), 5), "received acks are forgotten too");
        assert!(!manager.remove(&addr(1)), "already removed");
    }

    // ------------------------------------------------------------ CommonSocket

    #[tokio::test]
    async fn simple_packets_are_delivered_and_not_tracked() {
        let (mut a, a_addr, mut b, b_addr) = pair().await;
        a.send_to(get_player(5), &b_addr, UdpPacketType::Simple).await.unwrap();
        assert!(a.reliability.pending.is_empty());

        let (packet, from) = recv(&mut b).await;
        assert_eq!(get_player_id(packet), 5);
        assert_eq!(from, a_addr);
    }

    #[tokio::test]
    async fn reliable_packet_is_removed_from_the_queue_once_acked() {
        let (mut a, a_addr, mut b, b_addr) = pair().await;
        a.send_to(get_player(7), &b_addr, UdpPacketType::Reliable).await.unwrap();
        assert_eq!(a.reliability.pending.len(), 1);

        let (packet, from) = recv(&mut b).await; // b answers with an Ack
        assert_eq!(get_player_id(packet), 7);
        assert_eq!(from, a_addr);

        pump(&mut a, Duration::from_millis(200)).await;
        assert!(a.reliability.pending.is_empty(), "the ack was not processed");
    }

    #[tokio::test]
    async fn unacked_reliable_packet_is_resent_but_delivered_once() {
        let (mut a, _, mut b, b_addr) = pair().await;
        a.send_to(get_player(9), &b_addr, UdpPacketType::Reliable).await.unwrap();

        // b doesn't read (so doesn't ack) while a's 500ms resend timer expires
        pump(&mut a, Duration::from_millis(700)).await;
        assert_eq!(a.reliability.pending.values().next().unwrap().attempts, 2, "the packet was not resent");

        assert_eq!(get_player_id(recv(&mut b).await.0), 9);
        // the resent copy is acked but not delivered a second time
        pump(&mut b, Duration::from_millis(200)).await;
        pump(&mut a, Duration::from_millis(200)).await;
        assert!(a.reliability.pending.is_empty());
    }

    #[tokio::test]
    async fn peer_is_forgotten_after_too_many_attempts() {
        let (mut a, _, _b, b_addr) = pair().await;
        a.send_to(get_player(1), &b_addr, UdpPacketType::Reliable).await.unwrap();
        let entry = a.reliability.pending.values_mut().next().unwrap();
        entry.attempts = MAX_ATTEMPTS;
        entry.deadline = Instant::now();

        a.resend_due().await;
        assert!(a.reliability.pending.is_empty());
        assert!(!a.reliability.contains(&b_addr));
        assert!(a.reliability.timer.deadline() > Instant::now() + IDLE / 2, "timer parked");
    }

    #[tokio::test]
    async fn malformed_datagrams_are_ignored() {
        let (mut a, a_addr, b, _) = pair().await;
        b.socket.send_to(&[255, 1, 2, 3], a_addr).await.unwrap();
        pump(&mut a, Duration::from_millis(200)).await; // no error returned
    }

    #[tokio::test]
    async fn sending_quit_drops_the_pending_packets() {
        let (mut a, _, _b, b_addr) = pair().await;
        a.send_to(get_player(1), &b_addr, UdpPacketType::Reliable).await.unwrap();
        a.send_to(L5Packet::Quit(QuitPacket::new()), &b_addr, UdpPacketType::Simple).await.unwrap();
        assert!(a.reliability.pending.is_empty());
        assert!(!a.reliability.contains(&b_addr));
    }

    #[tokio::test]
    async fn ack_is_not_a_sendable_packet_type() {
        let (mut a, _, _b, b_addr) = pair().await;
        assert!(a.send_to(get_player(1), &b_addr, UdpPacketType::Ack).await.is_err());
    }
}
