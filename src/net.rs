//! Two Decks, one game. Each Deck runs the same simulation and they swap button presses, nothing else.
//!
//! Finding each other: a Deck that chose "two Decks" calls out on the local network twice a second and listens.
//! When two hear each other, the one with the lower number hosts (player 1) and tells the other which level and
//! which character, and both start the level from scratch at frame 0.
//!
//! Staying in step, without waiting: each Deck plays its own buttons at once and guesses that the other player
//! is still doing what they were last seen doing. Under that it keeps a second copy of the world, the settled one,
//! which only advances through frames where both Decks' buttons are really known. Every tick the picture is
//! rebuilt from the settled world plus the guesses, so a wrong guess is corrected a few frames later and the two
//! Decks can never drift apart. Every packet repeats the last frames, so a lost packet costs nothing.
//! Only when the other Deck has been silent for MAX_AHEAD frames does the game wait.
use crate::sim::Input;
use std::collections::{BTreeMap, VecDeque};
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::time::Instant;

/// Frames between pressing a button and it being used. One frame: the press is on screen the tick it is made.
pub const DELAY: u32 = 1;
/// How far this Deck may play ahead of what the other has confirmed: 12 frames is 200 ms.
pub const MAX_AHEAD: u32 = 12;
const PORTS: [u16; 2] = [47800, 47801];
const MAGIC: &[u8; 4] = b"CHRM";
const PROTO: u8 = 1;
const HELLO: u8 = 1;
const START: u8 = 2;
const INPUT: u8 = 3;

/// Things other than buttons that both Decks must do on the same frame.
pub const CMD_PAUSE: u8 = 1;
pub const CMD_ASK: u8 = 2;
pub const CMD_CHOICE: u8 = 3;
pub const CMD_RESTART: u8 = 4;

#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub struct Frame {
    pub input: Input,
    pub cmd: u8,
    pub a: u8,
    pub b: u8,
}

/// Stick positions are rounded to what fits in the packet, on both Decks, so both play the very same numbers.
pub fn quantise(i: Input) -> Input {
    let q = |v: f32| (v.clamp(-1.0, 1.0) * 100.0).round() / 100.0;
    Input { x: q(i.x), y: q(i.y), ..i }
}

fn pack(f: &Frame) -> [u8; 6] {
    let i = &f.input;
    let buttons = i.jump as u8 | (i.attack as u8) << 1 | (i.dash as u8) << 2 | (i.call_dog as u8) << 3 | (i.give as u8) << 4;
    [(i.x * 100.0).round() as i8 as u8, (i.y * 100.0).round() as i8 as u8, buttons, f.cmd, f.a, f.b]
}

fn unpack(b: &[u8]) -> Frame {
    let k = b[2];
    Frame {
        input: Input { x: b[0] as i8 as f32 / 100.0, y: b[1] as i8 as f32 / 100.0, jump: k & 1 != 0, attack: k & 2 != 0, dash: k & 4 != 0, call_dog: k & 8 != 0, give: k & 16 != 0 },
        cmd: b[3],
        a: b[4],
        b: b[5],
    }
}

/// The part that keeps two simulations in step. It knows nothing about sockets, so it can be tested on its own.
pub struct Lockstep {
    /// Which player this Deck is: 0 hosts.
    pub me: usize,
    /// The next frame to play.
    pub frame: u32,
    /// The newest frame this Deck has pressed buttons for, and the newest the other Deck says it has.
    pub sent: u32,
    pub remote_sent: u32,
    held: Frame,
    local: BTreeMap<u32, Frame>,
    remote: BTreeMap<u32, Frame>,
    pending: VecDeque<(u8, u8, u8)>,
    hashes: BTreeMap<u32, u32>,
    remote_hashes: BTreeMap<u32, u32>,
    /// The first frame at which the two worlds were found to differ. Should never happen.
    pub desync: Option<u32>,
}

impl Lockstep {
    pub fn new(me: usize) -> Self {
        // Nothing is pressed during the first DELAY frames.
        let local = (0..DELAY).map(|f| (f, Frame::default())).collect();
        Lockstep { me, frame: 0, sent: DELAY - 1, remote_sent: 0, held: Frame::default(), local, remote: BTreeMap::new(), pending: VecDeque::new(), hashes: BTreeMap::new(), remote_hashes: BTreeMap::new(), desync: None }
    }

    /// Ask for something to happen on both Decks at the same frame.
    pub fn command(&mut self, cmd: u8, a: u8, b: u8) {
        self.pending.push_back((cmd, a, b));
    }

    /// Record what is pressed now. Does nothing once this Deck is MAX_AHEAD frames past the settled world.
    pub fn capture(&mut self, input: Input) {
        if self.sent < self.frame + MAX_AHEAD {
            let (cmd, a, b) = self.pending.pop_front().unwrap_or((0, 0, 0));
            self.sent += 1;
            self.local.insert(self.sent, Frame { input: quantise(input), cmd, a, b });
        }
    }

    /// The body of an input packet: the newest frames (repeated in every packet) and the newest checksum.
    pub fn packet(&self) -> Vec<u8> {
        let count = (self.sent + 1).min(20);
        let mut out = Vec::with_capacity(16 + count as usize * 6);
        out.extend(self.sent.to_le_bytes());
        out.push(count as u8);
        for f in self.sent + 1 - count..=self.sent {
            out.extend(pack(self.local.get(&f).unwrap_or(&Frame::default())));
        }
        let (hf, h) = self.hashes.iter().next_back().map(|(f, h)| (*f, *h)).unwrap_or((u32::MAX, 0));
        out.extend(hf.to_le_bytes());
        out.extend(h.to_le_bytes());
        out
    }

    pub fn receive(&mut self, b: &[u8]) {
        if b.len() < 5 {
            return;
        }
        let last = u32::from_le_bytes([b[0], b[1], b[2], b[3]]);
        let count = b[4] as u32;
        if b.len() < 5 + count as usize * 6 + 8 || count > last + 1 {
            return;
        }
        self.remote_sent = self.remote_sent.max(last);
        for k in 0..count {
            let f = last + 1 - count + k;
            if f >= self.frame {
                let at = 5 + k as usize * 6;
                self.remote.entry(f).or_insert_with(|| unpack(&b[at..at + 6]));
            }
        }
        let at = 5 + count as usize * 6;
        let hf = u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]]);
        let h = u32::from_le_bytes([b[at + 4], b[at + 5], b[at + 6], b[at + 7]]);
        if hf != u32::MAX {
            self.remote_hashes.insert(hf, h);
            self.compare(hf);
        }
    }

    /// Both players' frame for the next step, player 1 first, once both are known.
    pub fn next(&mut self) -> Option<[Frame; 2]> {
        let (l, r) = (*self.local.get(&self.frame)?, *self.remote.get(&self.frame)?);
        let done = self.frame;
        self.held = r;
        self.frame += 1;
        self.remote.retain(|f, _| *f > done);
        // Keep what the other Deck may still need repeated.
        self.local.retain(|f, _| *f + 40 > done);
        Some(if self.me == 0 { [l, r] } else { [r, l] })
    }

    /// This Deck's own buttons for a frame it has not settled yet.
    pub fn mine(&self, frame: u32) -> Option<Frame> {
        self.local.get(&frame).copied()
    }

    /// The other Deck's buttons for a frame: the real ones if they have arrived, otherwise the last ones seen.
    pub fn theirs(&self, frame: u32) -> Input {
        self.remote.range(..=frame).next_back().map(|(_, f)| f.input).unwrap_or(self.held.input)
    }

    /// How many frames from the other Deck are waiting to be played.
    #[allow(dead_code)]
    pub fn backlog(&self) -> u32 {
        self.remote.keys().next_back().map(|f| f + 1 - self.frame.min(f + 1)).unwrap_or(0)
    }

    /// A checksum of the world after a frame. The two Decks compare them.
    pub fn note_hash(&mut self, frame: u32, hash: u32) {
        self.hashes.insert(frame, hash);
        while self.hashes.len() > 8 {
            self.hashes.pop_first();
        }
        self.compare(frame);
    }

    fn compare(&mut self, frame: u32) {
        if let (Some(a), Some(b)) = (self.hashes.get(&frame), self.remote_hashes.get(&frame)) {
            if a != b && self.desync.is_none() {
                self.desync = Some(frame);
            }
        }
        while self.remote_hashes.len() > 8 {
            self.remote_hashes.pop_first();
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Phase {
    Off,
    Searching,
    Playing,
}

/// What the game must do when the two Decks have found each other.
pub struct Started {
    pub skin: u8,
    pub level: String,
}

#[derive(bevy::prelude::Resource)]
pub struct Net {
    pub phase: Phase,
    socket: Option<UdpSocket>,
    id: u64,
    peer: Option<SocketAddr>,
    peer_id: u64,
    session: u32,
    pub lock: Lockstep,
    skin: u8,
    level: String,
    beacon: Option<Instant>,
    heard: Instant,
    got_input: bool,
    /// Who has the pause menu up, and who has a story choice on screen. The world waits while any is set.
    pub paused: [bool; 2],
    pub asking: [bool; 2],
    pub note: String,
    /// The world as of the last frame both Decks agree on. The one on screen runs ahead of it on guesses.
    pub settled: Option<crate::sim::World>,
    /// The newest frame whose effects (sparks, shakes) have been shown, so none is shown twice.
    pub shown: u32,
    pub ticks: u32,
}

impl Default for Net {
    fn default() -> Self {
        Net {
            phase: Phase::Off, socket: None, id: 0, peer: None, peer_id: 0, session: 0, lock: Lockstep::new(0), skin: 0, level: String::new(),
            beacon: None, heard: Instant::now(), got_input: false, paused: [false; 2], asking: [false; 2], note: String::new(), settled: None, shown: 0, ticks: 0,
        }
    }
}

fn nonce() -> u64 {
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(1);
    (t ^ (std::process::id() as u64).rotate_left(40)).wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1
}

impl Net {
    pub fn on(&self) -> bool {
        self.phase != Phase::Off
    }
    pub fn playing(&self) -> bool {
        self.phase == Phase::Playing
    }
    /// Seconds since the other Deck was last heard from.
    pub fn silence(&self) -> f32 {
        self.heard.elapsed().as_secs_f32()
    }

    /// Start calling out for another Deck. `skin` is the character this player would like, `level` where they are.
    pub fn search(&mut self, skin: u8, level: &str) {
        *self = Net::default();
        let socket = PORTS.iter().find_map(|p| UdpSocket::bind((Ipv4Addr::UNSPECIFIED, *p)).ok());
        match socket {
            Some(s) => {
                s.set_nonblocking(true).ok();
                s.set_broadcast(true).ok();
                self.socket = Some(s);
                self.id = nonce();
                self.skin = skin;
                self.level = level.to_string();
                self.phase = Phase::Searching;
            }
            None => self.note = "Could not open the network.".into(),
        }
    }

    pub fn stop(&mut self) {
        *self = Net::default();
    }

    fn header(&self, kind: u8) -> Vec<u8> {
        let mut out = MAGIC.to_vec();
        out.push(PROTO);
        out.push(kind);
        out
    }

    fn hello(&self) {
        let Some(s) = &self.socket else { return };
        let mut out = self.header(HELLO);
        out.extend(self.id.to_le_bytes());
        // Everyone on this network, this machine itself (two copies on one computer, for testing), and a named address.
        let mut hosts = vec![Ipv4Addr::BROADCAST, Ipv4Addr::LOCALHOST];
        if let Some(ip) = std::env::var("CHARM_PEER").ok().and_then(|v| v.parse().ok()) {
            hosts.push(ip);
        }
        for h in hosts {
            for p in PORTS {
                s.send_to(&out, (h, p)).ok();
            }
        }
    }

    fn start_packet(&self) {
        let (Some(s), Some(peer)) = (&self.socket, self.peer) else { return };
        let mut out = self.header(START);
        out.extend(self.id.to_le_bytes());
        out.extend(self.peer_id.to_le_bytes());
        out.extend(self.session.to_le_bytes());
        out.push(self.skin);
        out.extend(self.level.as_bytes());
        s.send_to(&out, peer).ok();
    }

    /// Send this Deck's newest frames.
    pub fn send(&self) {
        let (Some(s), Some(peer)) = (&self.socket, self.peer) else { return };
        let mut out = self.header(INPUT);
        out.extend(self.session.to_le_bytes());
        out.extend(self.lock.packet());
        s.send_to(&out, peer).ok();
    }

    /// Read everything that has arrived. Returns Some when the two Decks have just paired.
    pub fn poll(&mut self) -> Option<Started> {
        let mut started = None;
        let mut buf = [0u8; 512];
        let due = self.beacon.map(|t| t.elapsed().as_secs_f32() > 0.5).unwrap_or(true);
        if due {
            self.beacon = Some(Instant::now());
            match self.phase {
                Phase::Searching => self.hello(),
                // The host repeats the start until the other Deck is heard playing.
                Phase::Playing if self.lock.me == 0 && !self.got_input => self.start_packet(),
                _ => {}
            }
        }
        loop {
            let Some(s) = &self.socket else { break };
            let Ok((n, from)) = s.recv_from(&mut buf) else { break };
            let b = &buf[..n];
            if n < 6 || &b[..4] != MAGIC || b[4] != PROTO {
                continue;
            }
            let u64_at = |at: usize| u64::from_le_bytes(b[at..at + 8].try_into().unwrap());
            match b[5] {
                HELLO if n >= 14 => {
                    let them = u64_at(6);
                    if them == self.id {
                        continue;
                    }
                    if self.phase == Phase::Searching && self.id < them {
                        // The lower number hosts.
                        self.peer = Some(from);
                        self.peer_id = them;
                        self.session = nonce() as u32;
                        self.lock = Lockstep::new(0);
                        self.phase = Phase::Playing;
                        self.heard = Instant::now();
                        self.start_packet();
                        started = Some(Started { skin: self.skin, level: self.level.clone() });
                    } else if self.phase == Phase::Playing && them == self.peer_id && !self.got_input {
                        self.start_packet();
                    }
                }
                START if n >= 27 => {
                    if self.phase == Phase::Searching && u64_at(14) == self.id {
                        self.peer = Some(from);
                        self.peer_id = u64_at(6);
                        self.session = u32::from_le_bytes(b[22..26].try_into().unwrap());
                        self.skin = b[26];
                        self.level = String::from_utf8_lossy(&b[27..]).to_string();
                        self.lock = Lockstep::new(1);
                        self.phase = Phase::Playing;
                        self.heard = Instant::now();
                        started = Some(Started { skin: self.skin, level: self.level.clone() });
                    }
                }
                INPUT if n >= 10 => {
                    if self.phase == Phase::Playing && u32::from_le_bytes(b[6..10].try_into().unwrap()) == self.session {
                        self.lock.receive(&b[10..]);
                        self.heard = Instant::now();
                        self.got_input = true;
                    }
                }
                _ => {}
            }
        }
        started
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::{load_level, load_tuning, World};

    fn checksum(w: &World) -> u32 {
        let mut h = w.tick;
        for p in &w.players {
            h = h.wrapping_mul(31).wrapping_add(p.x.to_bits()).wrapping_mul(31).wrapping_add(p.y.to_bits()).wrapping_add(p.hp as u32);
        }
        h.wrapping_mul(31).wrapping_add(w.enemies.len() as u32)
    }

    /// Two Decks on a bad network (a third of the packets lost, the rest late) must still play the very same game.
    #[test]
    fn two_decks_stay_in_step_on_a_bad_network() {
        let tuning = load_tuning(include_str!("../assets/config/tuning.ron")).unwrap();
        let level = || load_level(include_str!("../assets/levels/rootway.ron")).unwrap();
        let mut worlds = [World::new(level(), &tuning, 2), World::new(level(), &tuning, 2)];
        let mut decks = [Lockstep::new(0), Lockstep::new(1)];
        let mut wire: Vec<(u32, usize, Vec<u8>)> = Vec::new();
        let mut seed = 7u32;
        let mut rand = move || {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            seed >> 16
        };
        let mut restarts = [0, 0];
        for now in 0..4000u32 {
            for d in 0..2 {
                // Each player mashes their own buttons.
                let k = rand();
                let input = Input { x: if (now / 90 + d as u32) % 3 == 0 { -1.0 } else { 0.73 }, y: 0.0, jump: k % 7 == 0, attack: k % 3 == 0, dash: k % 11 == 0, call_dog: false, give: false };
                if now == 1000 && d == 1 {
                    decks[d].command(CMD_RESTART, 0, 0);
                }
                decks[d].capture(input);
                if rand() % 3 != 0 {
                    wire.push((now + 1 + rand() % 4, 1 - d, decks[d].packet()));
                }
            }
            let (arrived, later): (Vec<_>, Vec<_>) = wire.into_iter().partition(|p| p.0 <= now);
            wire = later;
            for (_, to, bytes) in arrived {
                decks[to].receive(&bytes);
            }
            for d in 0..2 {
                while let Some(fr) = decks[d].next() {
                    if fr.iter().any(|f| f.cmd == CMD_RESTART) {
                        worlds[d] = World::new(level(), &tuning, 2);
                        restarts[d] += 1;
                    }
                    worlds[d].step(&[fr[0].input, fr[1].input], &tuning);
                    worlds[d].events.clear();
                    let frame = decks[d].frame - 1;
                    if frame % 30 == 0 {
                        let h = checksum(&worlds[d]);
                        decks[d].note_hash(frame, h);
                    }
                }
            }
        }
        assert!(decks[0].frame > 3000, "the game barely moved: frame {}", decks[0].frame);
        assert!(decks[0].frame.abs_diff(decks[1].frame) < 30);
        assert_eq!(restarts, [1, 1]);
        assert_eq!((decks[0].desync, decks[1].desync), (None, None));
        // Bring both to the same frame and compare the worlds outright.
        let common = decks[0].frame.min(decks[1].frame);
        assert!(common > 0);
        let moved = worlds[0].players.iter().any(|p| (p.x - 200.0).abs() > 50.0);
        assert!(moved, "nobody moved");
    }

    #[test]
    fn a_frame_survives_the_trip() {
        let f = Frame { input: quantise(Input { x: -0.737, y: 1.0, jump: true, attack: false, dash: true, call_dog: false, give: true }), cmd: CMD_CHOICE, a: 3, b: 1 };
        assert_eq!(unpack(&pack(&f)), f);
    }
}
