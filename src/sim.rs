//! Deterministic gameplay simulation. No Bevy types in here: the same code
//! will run on both Decks for netplay, driven only by per-tick inputs.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const DT: f32 = 1.0 / 60.0;
/// Flowers that make one heart when sent to the partner.
pub const FLOWERS_PER_HEART: u32 = 10;

#[derive(Clone, Copy, Default, PartialEq, Debug, Serialize, Deserialize)]
pub struct Input {
    pub x: f32,
    pub y: f32,
    pub jump: bool,
    pub attack: bool,
    pub dash: bool,
    pub call_dog: bool,
    /// Pass one life to the other player.
    pub give: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerTuning {
    pub width: f32,
    pub height: f32,
    pub max_hp: i32,
    pub run_speed: f32,
    pub sprint_speed: f32,
    pub ground_accel: f32,
    pub air_accel: f32,
    pub gravity: f32,
    pub max_fall_speed: f32,
    pub jump_speed: f32,
    /// Upward speed is multiplied by this when jump is released early.
    pub jump_cut: f32,
    pub coyote_time: f32,
    pub jump_buffer: f32,
    pub air_jumps: u8,
    pub air_jump_speed: f32,
    pub dash_speed: f32,
    pub dash_time: f32,
    pub dash_cooldown: f32,
    pub wall_slide_speed: f32,
    pub wall_jump_x: f32,
    pub wall_jump_y: f32,
    pub wall_jump_lock: f32,
    pub attack_time: f32,
    pub attack_cooldown: f32,
    /// A press this long before the attack is ready still fires it on the first possible tick.
    pub attack_buffer: f32,
    /// Ticks the whole world freezes on a hit, a kill, and when the player is hurt.
    /// Kills this close together chain into a combo.
    pub combo_window: f32,
    /// After a kill the next attack is ready this soon.
    pub kill_refund: f32,
    pub hitstop_hit: u32,
    pub hitstop_kill: u32,
    pub hitstop_hurt: u32,
    /// The weapon is a line from the shoulder, swept through an arc. Hits follow it exactly.
    pub weapon_length: f32,
    /// Half-thickness of the weapon line.
    pub weapon_thickness: f32,
    /// Height of the swing pivot above the body centre.
    pub shoulder_height: f32,
    /// Swing arcs in degrees (start, end), facing right: 0 is forward, 90 is up.
    pub swing_side: (f32, f32),
    pub swing_up: (f32, f32),
    pub swing_down: (f32, f32),
    pub attack_damage: i32,
    pub attack_recoil: f32,
    pub pogo_speed: f32,
    pub invuln_time: f32,
    pub hitstun_time: f32,
    pub knockback_x: f32,
    pub knockback_y: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Behaviour {
    /// Patrols, turns at walls and ledges.
    Walker,
    /// Jumps toward the nearest player every `interval` seconds.
    Hopper { jump_speed: f32, interval: f32 },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnemyTuning {
    pub width: f32,
    pub height: f32,
    pub hp: i32,
    pub speed: f32,
    pub contact_damage: i32,
    pub knockback: f32,
    /// Leg length under the round body. The body above the legs is what can be hit.
    #[serde(default)]
    pub leg: f32,
    /// Flowers released when it dies.
    #[serde(default)]
    pub flowers: u32,
    pub behaviour: Behaviour,
    /// Placeholder colour (r, g, b) until sprites exist.
    pub color: (f32, f32, f32),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tuning {
    pub player: PlayerTuning,
    pub enemies: BTreeMap<String, EnemyTuning>,
}

/// Axis-aligned box: bottom-left corner plus size. Y is up.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Rect(pub f32, pub f32, pub f32, pub f32);

impl Rect {
    pub fn centered(cx: f32, cy: f32, w: f32, h: f32) -> Self {
        Rect(cx - w / 2.0, cy - h / 2.0, w, h)
    }
    pub fn overlaps(&self, o: &Rect) -> bool {
        self.0 < o.0 + o.2 && self.0 + self.2 > o.0 && self.1 < o.1 + o.3 && self.1 + self.3 > o.1
    }
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x > self.0 && x < self.0 + self.2 && y > self.1 && y < self.1 + self.3
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnemySpawn {
    pub kind: String,
    pub x: f32,
    pub y: f32,
}

/// A doorway: walking into the box leaves for another level.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Exit {
    pub rect: Rect,
    pub to: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Level {
    pub name: String,
    /// Which look to draw it with: a file name in assets/themes.
    #[serde(default)]
    pub theme: String,
    #[serde(default)]
    pub exits: Vec<Exit>,
    pub bounds: Rect,
    pub spawns: Vec<(f32, f32)>,
    pub solids: Vec<Rect>,
    /// Thorns: touching one costs a mask and returns the player to the last safe ground.
    #[serde(default)]
    pub hazards: Vec<Rect>,
    pub enemies: Vec<EnemySpawn>,
    /// People who stand in the level and take no part in the fighting: "boss", and "partner"
    /// (whichever of Simon and Charm nobody is playing).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actors: Vec<EnemySpawn>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum AttackDir {
    Side,
    Up,
    Down,
}

#[derive(Clone, Debug)]
pub struct Player {
    pub x: f32,
    pub y: f32,
    pub px: f32,
    pub py: f32,
    pub vx: f32,
    pub vy: f32,
    pub facing: f32,
    pub on_ground: bool,
    pub wall: i8,
    pub hp: i32,
    pub coyote: f32,
    pub jump_buf: f32,
    pub jumping: bool,
    pub air_jumps_left: u8,
    pub air_dash_ready: bool,
    pub dash_t: f32,
    pub dash_cd: f32,
    pub sprinting: bool,
    pub wall_lock: f32,
    pub attack_t: f32,
    pub attack_cd: f32,
    pub attack_dir: AttackDir,
    pub invuln: f32,
    pub hitstun: f32,
    pub attack_buf: f32,
    /// A swing started this tick and has not been resolved yet.
    pub swing_fresh: bool,
    /// Flowers collected.
    pub score: u32,
    /// Flowers in hand. Ten of them can be sent to the partner, where they become a heart.
    pub flowers: u32,
    /// Kills chained so far, and the time left to add another.
    pub combo: u32,
    pub combo_t: f32,
    /// Side swings alternate direction: down-stroke, then up-stroke.
    pub swing_alt: bool,
    /// Out of lives in a two-player game: lying where they fell until the partner passes a life.
    pub down: bool,
    pub safe_x: f32,
    pub safe_y: f32,
    pub prev: Input,
}

#[derive(Clone, Debug)]
pub struct Enemy {
    pub kind: String,
    pub x: f32,
    pub y: f32,
    pub px: f32,
    pub py: f32,
    pub vx: f32,
    pub vy: f32,
    pub dir: f32,
    pub hp: i32,
    pub on_ground: bool,
    pub timer: f32,
    pub stun: f32,
    pub flash: f32,
}

/// Things that happened this tick, for the front end to dress up. They never feed back into the simulation.
#[derive(Clone, Debug)]
pub enum Event {
    Swing { player: usize },
    Dash { player: usize },
    /// The second jump, in mid-air.
    AirJump { player: usize },
    Hit { x: f32, y: f32, dir: f32 },
    Kill { x: f32, y: f32, kind: String, flowers: u32, dir: f32, combo: u32, player: usize },
    Hurt { x: f32, y: f32 },
    /// A life passed from one player to the other.
    /// Ten flowers sent across, becoming a heart for the partner.
    Give { from: usize, to: usize },
    Down { player: usize },
}

#[derive(Clone, Debug)]
pub struct World {
    pub level: Level,
    pub players: Vec<Player>,
    pub enemies: Vec<Enemy>,
    pub tick: u32,
    /// Ticks left of the freeze that follows a hit.
    pub hitstop: u32,
    /// Set when a player walks through an exit: the level to go to next.
    pub exit: Option<String>,
    pub events: Vec<Event>,
}

/// Distance from point (px, py) to the segment a-b.
fn seg_dist(a: (f32, f32), b: (f32, f32), px: f32, py: f32) -> f32 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len2 = dx * dx + dy * dy;
    let t = if len2 > 0.0 { (((px - a.0) * dx + (py - a.1) * dy) / len2).clamp(0.0, 1.0) } else { 0.0 };
    ((a.0 + dx * t - px).powi(2) + (a.1 + dy * t - py).powi(2)).sqrt()
}

/// An ellipse: centre and radii.
#[derive(Clone, Copy, Debug)]
pub struct Oval {
    pub x: f32,
    pub y: f32,
    pub rx: f32,
    pub ry: f32,
}

impl Oval {
    /// True if a line of the given half-thickness touches the ellipse.
    pub fn touches_line(&self, a: (f32, f32), b: (f32, f32), thickness: f32) -> bool {
        // Squash space vertically so the ellipse becomes a circle of radius rx.
        let k = self.rx / self.ry.max(0.001);
        let f = |p: (f32, f32)| (p.0, self.y + (p.1 - self.y) * k);
        seg_dist(f(a), f(b), self.x, self.y) <= self.rx + thickness
    }
}

impl Enemy {
    /// The round body that hits and gets hit: the box minus the legs.
    pub fn body(&self, et: &EnemyTuning) -> Oval {
        Oval { x: self.x, y: self.y + et.leg / 2.0, rx: et.width / 2.0, ry: ((et.height - et.leg) / 2.0).max(1.0) }
    }
}

fn approach(v: f32, target: f32, step: f32) -> f32 {
    if v < target {
        (v + step).min(target)
    } else {
        (v - step).max(target)
    }
}

fn hits(solids: &[Rect], r: &Rect) -> bool {
    solids.iter().any(|s| s.overlaps(r))
}

/// Moves a box through the solids, one axis at a time. Returns (on_ground, wall).
fn move_body(x: &mut f32, y: &mut f32, vx: &mut f32, vy: &mut f32, w: f32, h: f32, solids: &[Rect]) -> (bool, i8) {
    *x += *vx * DT;
    for s in solids {
        if s.overlaps(&Rect::centered(*x, *y, w, h)) {
            *x = if *vx > 0.0 { s.0 - w / 2.0 } else { s.0 + s.2 + w / 2.0 };
            *vx = 0.0;
        }
    }
    *y += *vy * DT;
    for s in solids {
        if s.overlaps(&Rect::centered(*x, *y, w, h)) {
            *y = if *vy > 0.0 { s.1 - h / 2.0 } else { s.1 + s.3 + h / 2.0 };
            *vy = 0.0;
        }
    }
    let ground = *vy <= 0.0 && hits(solids, &Rect::centered(*x, *y - 1.0, w, h));
    let wall = if hits(solids, &Rect::centered(*x + 1.0, *y, w, h - 4.0)) {
        1
    } else if hits(solids, &Rect::centered(*x - 1.0, *y, w, h - 4.0)) {
        -1
    } else {
        0
    };
    (ground, wall)
}

impl Player {
    pub fn new(x: f32, y: f32, t: &PlayerTuning) -> Self {
        Player {
            x, y, px: x, py: y, vx: 0.0, vy: 0.0, facing: 1.0, on_ground: false, wall: 0,
            hp: t.max_hp, coyote: 0.0, jump_buf: 0.0, jumping: false, air_jumps_left: t.air_jumps,
            air_dash_ready: true, dash_t: 0.0, dash_cd: 0.0, sprinting: false, wall_lock: 0.0,
            attack_t: 0.0, attack_cd: 0.0, attack_dir: AttackDir::Side, invuln: 0.0, hitstun: 0.0, attack_buf: 0.0, swing_fresh: false, score: 0, flowers: 0, combo: 0, combo_t: 0.0, swing_alt: false, down: false, safe_x: x, safe_y: y,
            prev: Input::default(),
        }
    }

    pub fn body(&self, t: &PlayerTuning) -> Rect {
        Rect::centered(self.x, self.y, t.width, t.height)
    }

    /// The current swing's arc in degrees (start, end), facing right.
    pub fn arc(&self, t: &PlayerTuning) -> (f32, f32) {
        match self.attack_dir {
            AttackDir::Side if self.swing_alt => (t.swing_side.1, t.swing_side.0),
            AttackDir::Side => t.swing_side,
            AttackDir::Up => t.swing_up,
            AttackDir::Down => t.swing_down,
        }
    }

    /// Swing angle in degrees, facing right, at a given time left on the swing.
    fn swing_angle(&self, t: &PlayerTuning, time_left: f32) -> f32 {
        let u = (1.0 - time_left / t.attack_time.max(0.001)).clamp(0.0, 1.0);
        let eased = 1.0 - (1.0 - u).powi(3);
        let (from, to) = self.arc(t);
        from + (to - from) * eased
    }

    fn weapon_at(&self, t: &PlayerTuning, angle: f32) -> ((f32, f32), (f32, f32)) {
        let lift = if self.attack_dir == AttackDir::Down { -t.shoulder_height * 0.5 } else { t.shoulder_height };
        let a = angle.to_radians();
        let pivot = (self.x, self.y + lift);
        (pivot, (pivot.0 + a.cos() * t.weapon_length * self.facing, pivot.1 + a.sin() * t.weapon_length))
    }

    /// The weapon line right now (pivot, tip) and its swing angle, if a swing is active.
    pub fn weapon(&self, t: &PlayerTuning) -> Option<((f32, f32), (f32, f32), f32)> {
        if self.attack_t <= 0.0 {
            return None;
        }
        let angle = self.swing_angle(t, self.attack_t);
        let (p, q) = self.weapon_at(t, angle);
        Some((p, q, angle))
    }

    /// The weapon line at steps across the whole arc. The hit lands on the first tick of the swing,
    /// everywhere the arc will pass; the drawn swing then catches up within a few frames.
    fn weapon_arc(&self, t: &PlayerTuning) -> [((f32, f32), (f32, f32)); 9] {
        let (from, to) = self.arc(t);
        std::array::from_fn(|i| self.weapon_at(t, from + (to - from) * i as f32 / 8.0))
    }

    /// The body as an upright capsule: a centre line and a radius.
    pub fn capsule(&self, t: &PlayerTuning) -> ((f32, f32), (f32, f32), f32) {
        let r = t.width / 2.0;
        let half = (t.height / 2.0 - r).max(0.0);
        ((self.x, self.y - half), (self.x, self.y + half), r)
    }

    pub fn wall_sliding(&self, input: &Input) -> bool {
        !self.on_ground && self.wall != 0 && self.vy < 0.0 && input.x * self.wall as f32 > 0.3
    }
}

impl World {
    pub fn new(level: Level, tuning: &Tuning, player_count: usize) -> Self {
        let players = (0..player_count)
            .map(|i| {
                let s = level.spawns[i % level.spawns.len()];
                Player::new(s.0, s.1, &tuning.player)
            })
            .collect();
        let enemies = Self::spawn_enemies(&level, tuning);
        World { level, players, enemies, tick: 0, hitstop: 0, exit: None, events: Vec::new() }
    }

    fn spawn_enemies(level: &Level, tuning: &Tuning) -> Vec<Enemy> {
        level
            .enemies
            .iter()
            .filter_map(|s| {
                let t = tuning.enemies.get(&s.kind)?;
                // A spawn point set a little into the floor is lifted clear, so nothing starts stuck in rock.
                let mut y = s.y;
                for _ in 0..80 {
                    if !hits(&level.solids, &Rect::centered(s.x, y, t.width, t.height)) {
                        break;
                    }
                    y += 1.0;
                }
                Some(Enemy {
                    kind: s.kind.clone(), x: s.x, y, px: s.x, py: y, vx: 0.0, vy: 0.0, dir: -1.0,
                    hp: t.hp, on_ground: false, timer: 0.0, stun: 0.0, flash: 0.0,
                })
            })
            .collect()
    }

    /// Moves everyone to another level. Lives, flowers and score come along.
    pub fn travel(&mut self, level: Level, tuning: &Tuning) {
        self.enemies = Self::spawn_enemies(&level, tuning);
        for (i, p) in self.players.iter_mut().enumerate() {
            let s = level.spawns[i % level.spawns.len()];
            let (hp, score, flowers, down) = (p.hp, p.score, p.flowers, p.down);
            *p = Player::new(s.0, s.1, &tuning.player);
            (p.hp, p.score, p.flowers, p.down) = (hp, score, flowers, down);
        }
        self.level = level;
        self.hitstop = 0;
        self.exit = None;
        self.events.clear();
    }

    pub fn step(&mut self, inputs: &[Input], tuning: &Tuning) {
        self.events.clear();
        let t = &tuning.player;
        // Hitstop: everything holds still, but presses made during the freeze are kept.
        if self.hitstop > 0 {
            self.hitstop -= 1;
            for (i, p) in self.players.iter_mut().enumerate() {
                let inp = inputs.get(i).copied().unwrap_or_default();
                if inp.jump && !p.prev.jump {
                    p.jump_buf = t.jump_buffer;
                }
                if inp.attack && !p.prev.attack {
                    p.attack_buf = t.attack_buffer;
                }
                (p.px, p.py) = (p.x, p.y);
                p.prev = inp;
            }
            for e in self.enemies.iter_mut() {
                (e.px, e.py) = (e.x, e.y);
            }
            return;
        }
        self.tick += 1;
        // The give button: send ten of your flowers to your partner, where they become a heart (one life).
        // You cannot make hearts for yourself.
        let count = self.players.len();
        if count == 2 {
            for from in 0..2 {
                let to = 1 - from;
                let inp = inputs.get(from).copied().unwrap_or_default();
                let pressed = inp.give && !self.players[from].prev.give;
                let (giver, taker) = (&self.players[from], &self.players[to]);
                if !pressed || giver.down || giver.flowers < FLOWERS_PER_HEART || !(taker.down || taker.hp < t.max_hp) {
                    continue;
                }
                let (gx, gy, face) = (giver.x, giver.y, giver.facing);
                self.players[from].flowers -= FLOWERS_PER_HEART;
                let p = &mut self.players[to];
                if p.down {
                    let (score, flowers) = (p.score, p.flowers);
                    *p = Player::new(gx, gy, t);
                    (p.score, p.flowers) = (score, flowers);
                    p.hp = 1;
                    p.facing = face;
                    p.invuln = t.invuln_time * 2.0;
                } else {
                    p.hp += 1;
                }
                self.events.push(Event::Give { from, to });
            }
        }
        // Both down: everyone gets back up at the start.
        if count == 2 && self.players.iter().all(|p| p.down) {
            for (i, p) in self.players.iter_mut().enumerate() {
                let s = self.level.spawns[i % self.level.spawns.len()];
                let (score, flowers) = (p.score, p.flowers);
                *p = Player::new(s.0, s.1, t);
                (p.score, p.flowers) = (score, flowers);
            }
        }
        let solo = self.players.len() < 2;
        let solids = &self.level.solids;
        let bounds = self.level.bounds;

        for (i, p) in self.players.iter_mut().enumerate() {
            let inp = inputs.get(i).copied().unwrap_or_default();
            if p.down {
                (p.px, p.py) = (p.x, p.y);
                p.prev = inp;
                continue;
            }
            let pressed = |now: bool, before: bool| now && !before;
            let jump_pressed = pressed(inp.jump, p.prev.jump);
            let dash_pressed = pressed(inp.dash, p.prev.dash);
            let attack_pressed = pressed(inp.attack, p.prev.attack);
            p.px = p.x;
            p.py = p.y;
            for timer in [&mut p.coyote, &mut p.jump_buf, &mut p.dash_cd, &mut p.wall_lock, &mut p.attack_cd, &mut p.attack_t, &mut p.invuln, &mut p.hitstun, &mut p.attack_buf, &mut p.combo_t] {
                *timer = (*timer - DT).max(0.0);
            }
            if p.combo_t <= 0.0 {
                p.combo = 0;
            }
            let control = p.hitstun <= 0.0;
            let sliding = p.wall_sliding(&inp);

            if jump_pressed {
                p.jump_buf = t.jump_buffer;
            }
            if attack_pressed {
                p.attack_buf = t.attack_buffer;
            }

            // Dash
            if control && dash_pressed && p.dash_cd <= 0.0 && p.dash_t <= 0.0 && (p.on_ground || p.air_dash_ready) {
                if sliding {
                    p.facing = -(p.wall as f32);
                } else if inp.x.abs() > 0.3 {
                    p.facing = inp.x.signum();
                }
                if !p.on_ground {
                    p.air_dash_ready = false;
                }
                p.dash_t = t.dash_time;
                p.dash_cd = t.dash_time + t.dash_cooldown;
                self.events.push(Event::Dash { player: i });
                p.jumping = false;
            }

            if p.dash_t > 0.0 {
                p.dash_t -= DT;
                p.vx = p.facing * t.dash_speed;
                p.vy = 0.0;
                if p.dash_t <= 0.0 {
                    p.sprinting = p.on_ground && inp.dash;
                    p.vx = p.facing * if p.sprinting { t.sprint_speed } else { t.run_speed };
                }
            } else {
                if !(inp.dash && p.on_ground && inp.x.abs() > 0.3) && p.on_ground {
                    p.sprinting = false;
                }
                if control && p.wall_lock <= 0.0 {
                    let speed = if p.sprinting { t.sprint_speed } else { t.run_speed };
                    let dir = if inp.x.abs() > 0.3 { inp.x.signum() } else { 0.0 };
                    let accel = if p.on_ground { t.ground_accel } else { t.air_accel };
                    p.vx = approach(p.vx, dir * speed, accel * DT);
                    if dir != 0.0 && p.attack_t <= 0.0 {
                        p.facing = dir;
                    }
                }
                p.vy = (p.vy - t.gravity * DT).max(-t.max_fall_speed);
                if sliding {
                    p.vy = p.vy.max(-t.wall_slide_speed);
                    p.facing = -(p.wall as f32);
                }
            }

            // Jump, wall jump, air jump (a jump also cancels a dash)
            if control && p.jump_buf > 0.0 {
                let mut jumped = true;
                if p.on_ground || p.coyote > 0.0 {
                    p.vy = t.jump_speed;
                } else if p.wall != 0 {
                    p.vx = -(p.wall as f32) * t.wall_jump_x;
                    p.vy = t.wall_jump_y;
                    p.facing = -(p.wall as f32);
                    p.wall_lock = t.wall_jump_lock;
                } else if p.air_jumps_left > 0 {
                    p.air_jumps_left -= 1;
                    p.vy = t.air_jump_speed;
                    self.events.push(Event::AirJump { player: i });
                } else {
                    jumped = false;
                }
                if jumped {
                    p.jump_buf = 0.0;
                    p.coyote = 0.0;
                    p.dash_t = 0.0;
                    p.jumping = true;
                }
            }
            if p.jumping && !inp.jump && p.vy > 0.0 {
                p.vy *= t.jump_cut;
                p.jumping = false;
            }
            if p.vy <= 0.0 {
                p.jumping = false;
            }

            // Attack
            if control && p.attack_buf > 0.0 && p.attack_cd <= 0.0 {
                p.attack_buf = 0.0;
                p.swing_fresh = true;
                self.events.push(Event::Swing { player: i });
                p.attack_dir = if inp.y > 0.5 {
                    AttackDir::Up
                } else if inp.y < -0.5 && !p.on_ground {
                    AttackDir::Down
                } else {
                    AttackDir::Side
                };
                if p.attack_dir == AttackDir::Side {
                    p.swing_alt = !p.swing_alt;
                }
                p.attack_t = t.attack_time;
                p.attack_cd = t.attack_cooldown;
            }

            let (ground, wall) = move_body(&mut p.x, &mut p.y, &mut p.vx, &mut p.vy, t.width, t.height, solids);
            p.on_ground = ground;
            p.wall = wall;
            if ground {
                p.coyote = t.coyote_time;
                p.air_jumps_left = t.air_jumps;
                p.air_dash_ready = true;
            } else {
                p.sprinting = p.sprinting && p.dash_t > 0.0;
            }

            // Remember the last spot where both feet were on solid ground, away from thorns.
            let feet = p.y - t.height / 2.0 - 2.0;
            let danger = Rect::centered(p.x, p.y, t.width + 80.0, t.height + 20.0);
            if ground
                && solids.iter().any(|s| s.contains(p.x - t.width / 2.0, feet))
                && solids.iter().any(|s| s.contains(p.x + t.width / 2.0, feet))
                && !self.level.hazards.iter().any(|h| h.overlaps(&danger))
            {
                (p.safe_x, p.safe_y) = (p.x, p.y);
            }
            // Thorns or a fall out of the level: lose a mask and return to safe ground.
            let body = p.body(t);
            if p.y < bounds.1 - 200.0 || self.level.hazards.iter().any(|h| h.overlaps(&body)) {
                let s = (p.safe_x, p.safe_y);
                p.dash_t = 0.0;
                p.attack_t = 0.0;
                (p.x, p.y, p.vx, p.vy) = (s.0, s.1, 0.0, 0.0);
                (p.px, p.py) = (p.x, p.y);
                p.hp -= 1;
                p.invuln = t.invuln_time;
                self.events.push(Event::Hurt { x: p.x, y: p.y });
            }
            if p.hp <= 0 && solo {
                let s = self.level.spawns[i % self.level.spawns.len()];
                let (score, flowers) = (p.score, p.flowers);
                *p = Player::new(s.0, s.1, t);
                (p.score, p.flowers) = (score, flowers);
            } else if p.hp <= 0 {
                p.hp = 0;
                p.down = true;
                (p.vx, p.vy, p.dash_t, p.attack_t) = (0.0, 0.0, 0.0, 0.0);
                self.events.push(Event::Down { player: i });
            }
            p.prev = inp;
        }

        // Enemies
        for e in self.enemies.iter_mut() {
            let Some(et) = tuning.enemies.get(&e.kind) else { continue };
            e.px = e.x;
            e.py = e.y;
            e.flash = (e.flash - DT).max(0.0);
            e.stun = (e.stun - DT).max(0.0);
            if e.stun <= 0.0 {
                match et.behaviour {
                    Behaviour::Walker => {
                        if e.on_ground {
                            let ahead = e.x + e.dir * (et.width / 2.0 + 2.0);
                            let floor = solids.iter().any(|s| s.contains(ahead, e.y - et.height / 2.0 - 3.0));
                            if !floor {
                                e.dir = -e.dir;
                            }
                        }
                        e.vx = e.dir * et.speed;
                    }
                    Behaviour::Hopper { jump_speed, interval } => {
                        if e.on_ground {
                            e.vx = 0.0;
                            e.timer += DT;
                            if e.timer >= interval {
                                e.timer = 0.0;
                                let target = self.players.iter().filter(|p| !p.down).map(|p| p.x).min_by(|a, b| (a - e.x).abs().total_cmp(&(b - e.x).abs()));
                                if let Some(tx) = target {
                                    e.dir = if tx < e.x { -1.0 } else { 1.0 };
                                }
                                e.vx = e.dir * et.speed;
                                e.vy = jump_speed;
                            }
                        }
                    }
                }
            }
            e.vy = (e.vy - t.gravity * DT).max(-t.max_fall_speed);
            let want = e.vx;
            let (ground, wall) = move_body(&mut e.x, &mut e.y, &mut e.vx, &mut e.vy, et.width, et.height, solids);
            e.on_ground = ground;
            if wall != 0 && want * wall as f32 > 0.0 {
                e.dir = -(wall as f32);
            }
            if e.y < bounds.1 - 200.0 {
                e.hp = 0;
            }
        }

        // Player swings against enemies, then enemy contact against players.
        for (i, p) in self.players.iter_mut().enumerate() {
            if p.swing_fresh {
                p.swing_fresh = false;
                let sweep = p.weapon_arc(t);
                let mut connected = false;
                let mut killed = false;
                for e in self.enemies.iter_mut() {
                    let Some(et) = tuning.enemies.get(&e.kind) else { continue };
                    let body = e.body(et);
                    if !sweep.iter().any(|(a, b)| body.touches_line(*a, *b, t.weapon_thickness)) {
                        continue;
                    }
                    e.hp -= t.attack_damage;
                    let dir = if p.attack_dir == AttackDir::Side { p.facing } else { 0.0 };
                    if e.hp <= 0 {
                        killed = true;
                        p.combo += 1;
                        p.combo_t = t.combo_window;
                        // Each link in the chain is worth one more flower, up to five extra.
                        let flowers = et.flowers + (p.combo - 1).min(5);
                        p.score += flowers;
                        p.flowers += flowers;
                        self.hitstop = self.hitstop.max(t.hitstop_kill);
                        self.events.push(Event::Kill { x: body.x, y: body.y, kind: e.kind.clone(), flowers, dir, combo: p.combo, player: i });
                    } else {
                        self.hitstop = self.hitstop.max(t.hitstop_hit);
                        self.events.push(Event::Hit { x: body.x, y: body.y, dir });
                    }
                    e.flash = 0.12;
                    e.stun = 0.25;
                    match p.attack_dir {
                        AttackDir::Side => (e.vx, e.vy) = (p.facing * et.knockback, 120.0),
                        AttackDir::Up => (e.vx, e.vy) = (0.0, et.knockback),
                        AttackDir::Down => (e.vx, e.vy) = (0.0, 0.0),
                    }
                    connected = true;
                }
                if killed {
                    // A kill keeps the flow going: no recoil, next attack almost at once, air dash back.
                    p.attack_cd = p.attack_cd.min(t.kill_refund);
                    p.air_dash_ready = true;
                }
                if connected {
                    match p.attack_dir {
                        AttackDir::Side if killed => {}
                        AttackDir::Side => p.vx = -p.facing * t.attack_recoil,
                        AttackDir::Up => {}
                        AttackDir::Down => {
                            p.vy = t.pogo_speed;
                            p.jumping = false;
                            p.air_dash_ready = true;
                            p.air_jumps_left = t.air_jumps;
                        }
                    }
                }
            }
            if p.invuln <= 0.0 && !p.down {
                let (a, b, r) = p.capsule(t);
                for e in &self.enemies {
                    let Some(et) = tuning.enemies.get(&e.kind) else { continue };
                    if e.hp > 0 && e.body(et).touches_line(a, b, r) {
                        p.hp -= et.contact_damage;
                        self.hitstop = self.hitstop.max(t.hitstop_hurt);
                        self.events.push(Event::Hurt { x: p.x, y: p.y });
                        p.invuln = t.invuln_time;
                        p.hitstun = t.hitstun_time;
                        p.dash_t = 0.0;
                        p.vx = if p.x < e.x { -t.knockback_x } else { t.knockback_x };
                        p.vy = t.knockback_y;
                        break;
                    }
                }
            }
        }
        self.enemies.retain(|e| e.hp > 0);
        if self.exit.is_none() {
            for p in self.players.iter().filter(|p| !p.down) {
                if let Some(e) = self.level.exits.iter().find(|e| e.rect.contains(p.x, p.y)) {
                    self.exit = Some(e.to.clone());
                }
            }
        }
    }
}

pub fn load_tuning(text: &str) -> Result<Tuning, String> {
    ron::from_str(text).map_err(|e| e.to_string())
}

pub fn load_level(text: &str) -> Result<Level, String> {
    ron::from_str(text).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (World, Tuning) {
        let tuning = load_tuning(include_str!("../assets/config/tuning.ron")).unwrap();
        let level = Level {
            actors: vec![],
            name: "test".into(),
            bounds: Rect(0.0, 0.0, 2000.0, 1000.0),
            spawns: vec![(500.0, 100.0)],
            solids: vec![Rect(0.0, 0.0, 2000.0, 40.0), Rect(900.0, 40.0, 40.0, 600.0)],
            hazards: vec![],
            theme: String::new(),
            exits: vec![],
            enemies: vec![],
        };
        let mut w = World::new(level, &tuning, 1);
        for _ in 0..60 {
            w.step(&[Input::default()], &tuning);
        }
        (w, tuning)
    }

    #[test]
    fn shipped_assets_parse() {
        let tuning = load_tuning(include_str!("../assets/config/tuning.ron")).unwrap();
        for text in [include_str!("../assets/levels/rootway.ron"), include_str!("../assets/levels/australia.ron"), include_str!("../assets/levels/newzealand.ron"), include_str!("../assets/levels/france.ron"), include_str!("../assets/levels/uk.ron")] {
            let level = load_level(text).unwrap();
            assert!(!level.spawns.is_empty());
            // Nobody starts inside rock or on thorns, and every tomato is a known kind.
            for s in &level.spawns {
                let body = Rect::centered(s.0, s.1, tuning.player.width, tuning.player.height);
                assert!(!hits(&level.solids, &body) && !level.hazards.iter().any(|h| h.overlaps(&body)), "{}: bad spawn", level.name);
            }
            for e in &level.enemies {
                assert!(tuning.enemies.contains_key(&e.kind), "{}: unknown tomato {}", level.name, e.kind);
            }
        }
    }

    /// Walk the whole game by its doors: every door must open onto a level that exists, and the last place is the UK.
    #[test]
    fn the_doors_lead_from_the_caves_to_the_uk() {
        let tuning = load_tuning(include_str!("../assets/config/tuning.ron")).unwrap();
        let open = |name: &str| load_level(&std::fs::read_to_string(format!("assets/levels/{name}.ron")).unwrap_or_else(|_| panic!("no level {name}"))).unwrap();
        let mut name = "rootway".to_string();
        let mut w = World::new(open(&name), &tuning, 2);
        let mut route = vec![name.clone()];
        while let Some(door) = w.level.exits.first().cloned() {
            for p in &mut w.players {
                p.x = door.rect.0 + door.rect.2 / 2.0;
                p.y = door.rect.1 + tuning.player.height / 2.0;
            }
            w.step(&[Input::default(); 2], &tuning);
            name = w.exit.clone().unwrap_or_else(|| panic!("the door in {name} did not open"));
            w.travel(open(&name), &tuning);
            for _ in 0..30 {
                w.step(&[Input::default(); 2], &tuning);
            }
            assert!(w.players.iter().all(|p| p.on_ground && p.hp == tuning.player.max_hp), "{name}: bad arrival");
            route.push(name.clone());
            assert!(route.len() < 20, "the doors go round in a circle");
        }
        assert_eq!(route, ["rootway", "australia", "newzealand", "france", "uk"]);
        assert!(w.level.actors.iter().any(|a| a.kind == "boss") && w.level.actors.iter().any(|a| a.kind == "partner"));
    }

    /// The simulation must stay far inside its 16.6 ms tick, with room for a handheld's slower processor.
    #[test]
    fn a_tick_is_cheap() {
        let tuning = load_tuning(include_str!("../assets/config/tuning.ron")).unwrap();
        let mut w = World::new(load_level(include_str!("../assets/levels/rootway.ron")).unwrap(), &tuning, 2);
        let input = [Input { x: 1.0, attack: true, ..Default::default() }; 2];
        let t = std::time::Instant::now();
        for _ in 0..6000 {
            w.step(&input, &tuning);
            w.events.clear();
        }
        let per = t.elapsed().as_secs_f64() * 1000.0 / 6000.0;
        println!("one tick: {per:.4} ms");
        assert!(per < 1.0, "one tick took {per:.3} ms");
    }

    #[test]
    fn exits_carry_lives_and_flowers_to_the_next_level() {
        let tuning = load_tuning(include_str!("../assets/config/tuning.ron")).unwrap();
        let mut a = load_level(include_str!("../assets/levels/rootway.ron")).unwrap();
        a.exits = vec![Exit { rect: Rect(0.0, 0.0, 400.0, 400.0), to: "australia".into() }];
        let b = load_level(include_str!("../assets/levels/australia.ron")).unwrap();
        let mut w = World::new(a, &tuning, 1);
        w.players[0].hp = 3;
        w.players[0].flowers = 7;
        w.step(&[Input::default()], &tuning);
        assert_eq!(w.exit.as_deref(), Some("australia"));
        w.travel(b, &tuning);
        assert_eq!((w.players[0].hp, w.players[0].flowers, w.exit.is_none()), (3, 7, true));
        assert_eq!(w.level.name, "Australia");
    }

    #[test]
    fn lands_and_jump_height_depends_on_hold() {
        let (w0, t) = setup();
        assert!(w0.players[0].on_ground);
        let floor = w0.players[0].y;
        let apex = |hold: usize| {
            let mut w = w0.clone();
            let mut top = floor;
            for i in 0..120 {
                w.step(&[Input { jump: i < hold, ..Default::default() }], &t);
                top = top.max(w.players[0].y);
            }
            assert!(w.players[0].on_ground);
            top - floor
        };
        let (full, tap) = (apex(60), apex(3));
        let ideal = t.player.jump_speed.powi(2) / (2.0 * t.player.gravity);
        assert!((full - ideal).abs() < 12.0, "full {full} ideal {ideal}");
        assert!(tap < full * 0.6, "tap {tap} full {full}");
    }

    #[test]
    fn dash_distance_and_air_dash_once() {
        let (mut w, t) = setup();
        let x0 = w.players[0].x;
        w.step(&[Input { dash: true, ..Default::default() }], &t);
        for _ in 0..40 {
            w.step(&[Input::default()], &t);
        }
        let d = w.players[0].x - x0;
        assert!(d > t.player.dash_speed * t.player.dash_time * 0.9, "dash {d}");
        // In the air: second dash must not fire.
        for i in 0..20 {
            w.step(&[Input { jump: i < 10, ..Default::default() }], &t);
        }
        w.step(&[Input { dash: true, ..Default::default() }], &t);
        assert!(w.players[0].dash_t > 0.0);
        for _ in 0..25 {
            w.step(&[Input::default()], &t);
        }
        if !w.players[0].on_ground {
            w.step(&[Input { dash: true, ..Default::default() }], &t);
            assert!(w.players[0].dash_t <= 0.0);
        }
    }

    #[test]
    fn wall_slide_and_wall_jump() {
        let (mut w, t) = setup();
        let right = |jump| Input { x: 1.0, jump, ..Default::default() };
        for _ in 0..90 {
            w.step(&[right(false)], &t);
        }
        assert_eq!(w.players[0].wall, 1);
        for i in 0..40 {
            w.step(&[right(i < 15)], &t);
        }
        let p = &w.players[0];
        assert!(!p.on_ground && p.vy >= -t.player.wall_slide_speed - 0.1, "vy {}", p.vy);
        w.step(&[right(false)], &t);
        w.step(&[right(true)], &t);
        let p = &w.players[0];
        assert!(p.vx < 0.0 && p.vy > 0.0, "vx {} vy {}", p.vx, p.vy);
    }

    #[test]
    fn pogo_bounces_and_kills() {
        let (mut w, t) = setup();
        let et = &t.enemies["cherry"];
        w.enemies.push(Enemy {
            kind: "cherry".into(), x: w.players[0].x + 20.0, y: 40.0 + et.height / 2.0, px: 0.0, py: 0.0, vx: 0.0, vy: 0.0,
            dir: 1.0, hp: 1, on_ground: true, timer: 0.0, stun: 9.0, flash: 0.0,
        });
        w.players[0].y += 60.0;
        w.players[0].on_ground = false;
        w.players[0].vy = -100.0;
        for _ in 0..8 {
            w.step(&[Input { y: -1.0, attack: true, ..Default::default() }], &t);
            if w.enemies.is_empty() {
                break;
            }
        }
        assert!(w.enemies.is_empty());
        assert!(w.players[0].vy > 0.0);
    }

    #[test]
    fn sending_flowers_makes_a_heart_for_the_partner() {
        let tuning = load_tuning(include_str!("../assets/config/tuning.ron")).unwrap();
        let level = Level {
            actors: vec![],
            name: "test".into(),
            bounds: Rect(0.0, 0.0, 2000.0, 1000.0),
            spawns: vec![(500.0, 100.0), (700.0, 100.0)],
            solids: vec![Rect(0.0, 0.0, 2000.0, 40.0)],
            hazards: vec![],
            theme: String::new(),
            exits: vec![],
            enemies: vec![],
        };
        let mut w = World::new(level, &tuning, 2);
        let none = Input::default();
        let give = Input { give: true, ..Default::default() };
        let max = tuning.player.max_hp;
        // Not enough flowers: nothing happens, and your own lives are never spent.
        w.players[0].flowers = 9;
        w.players[1].hp = 2;
        w.step(&[give, none], &tuning);
        assert_eq!((w.players[0].hp, w.players[1].hp, w.players[0].flowers), (max, 2, 9));
        w.step(&[none, none], &tuning);
        // Ten flowers become one heart for the partner.
        w.players[0].flowers = 23;
        w.step(&[give, none], &tuning);
        assert_eq!((w.players[0].hp, w.players[1].hp, w.players[0].flowers), (max, 3, 13));
        // Holding the button sends once.
        w.step(&[give, none], &tuning);
        assert_eq!(w.players[0].flowers, 13);
        w.step(&[none, none], &tuning);
        // A downed partner is brought back beside the sender.
        w.players[1].hp = 0;
        w.step(&[none, none], &tuning);
        assert!(w.players[1].down);
        w.step(&[give, none], &tuning);
        assert!(!w.players[1].down);
        assert_eq!((w.players[1].hp, w.players[0].flowers), (1, 3));
        assert!((w.players[1].x - w.players[0].x).abs() < 5.0);
        // A partner on full lives is not sent anything.
        w.players[0].flowers = 10;
        w.players[1].hp = max;
        w.step(&[none, none], &tuning);
        w.step(&[give, none], &tuning);
        assert_eq!(w.players[0].flowers, 10);
    }
}
