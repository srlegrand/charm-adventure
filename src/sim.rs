//! Deterministic gameplay simulation. No Bevy types in here: the same code
//! will run on both Decks for netplay, driven only by per-tick inputs.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const DT: f32 = 1.0 / 60.0;

#[derive(Clone, Copy, Default, PartialEq, Debug, Serialize, Deserialize)]
pub struct Input {
    pub x: f32,
    pub y: f32,
    pub jump: bool,
    pub attack: bool,
    pub dash: bool,
    pub call_dog: bool,
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
    pub attack_reach: f32,
    pub attack_thickness: f32,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Level {
    pub name: String,
    pub bounds: Rect,
    pub spawns: Vec<(f32, f32)>,
    pub solids: Vec<Rect>,
    pub enemies: Vec<EnemySpawn>,
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
    /// Id of the swing that last hit this enemy, so one swing hits once.
    pub last_hit: u32,
}

#[derive(Clone, Debug)]
pub struct World {
    pub level: Level,
    pub players: Vec<Player>,
    pub enemies: Vec<Enemy>,
    pub tick: u32,
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
    fn new(x: f32, y: f32, t: &PlayerTuning) -> Self {
        Player {
            x, y, px: x, py: y, vx: 0.0, vy: 0.0, facing: 1.0, on_ground: false, wall: 0,
            hp: t.max_hp, coyote: 0.0, jump_buf: 0.0, jumping: false, air_jumps_left: t.air_jumps,
            air_dash_ready: true, dash_t: 0.0, dash_cd: 0.0, sprinting: false, wall_lock: 0.0,
            attack_t: 0.0, attack_cd: 0.0, attack_dir: AttackDir::Side, invuln: 0.0, hitstun: 0.0,
            prev: Input::default(),
        }
    }

    pub fn body(&self, t: &PlayerTuning) -> Rect {
        Rect::centered(self.x, self.y, t.width, t.height)
    }

    /// Hitbox of the current swing, if one is active.
    pub fn attack_box(&self, t: &PlayerTuning) -> Option<Rect> {
        if self.attack_t <= 0.0 {
            return None;
        }
        let (r, th) = (t.attack_reach, t.attack_thickness);
        Some(match self.attack_dir {
            AttackDir::Side => Rect::centered(self.x + self.facing * (t.width / 2.0 + r / 2.0), self.y, r, th),
            AttackDir::Up => Rect::centered(self.x, self.y + t.height / 2.0 + r / 2.0, th, r),
            AttackDir::Down => Rect::centered(self.x, self.y - t.height / 2.0 - r / 2.0, th, r),
        })
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
        let enemies = level
            .enemies
            .iter()
            .filter_map(|s| {
                let t = tuning.enemies.get(&s.kind)?;
                Some(Enemy {
                    kind: s.kind.clone(), x: s.x, y: s.y, px: s.x, py: s.y, vx: 0.0, vy: 0.0, dir: -1.0,
                    hp: t.hp, on_ground: false, timer: 0.0, stun: 0.0, flash: 0.0, last_hit: 0,
                })
            })
            .collect();
        World { level, players, enemies, tick: 0 }
    }

    pub fn step(&mut self, inputs: &[Input], tuning: &Tuning) {
        self.tick += 1;
        let t = &tuning.player;
        let solids = &self.level.solids;
        let bounds = self.level.bounds;

        for (i, p) in self.players.iter_mut().enumerate() {
            let inp = inputs.get(i).copied().unwrap_or_default();
            let pressed = |now: bool, before: bool| now && !before;
            let jump_pressed = pressed(inp.jump, p.prev.jump);
            let dash_pressed = pressed(inp.dash, p.prev.dash);
            let attack_pressed = pressed(inp.attack, p.prev.attack);
            p.px = p.x;
            p.py = p.y;
            for timer in [&mut p.coyote, &mut p.jump_buf, &mut p.dash_cd, &mut p.wall_lock, &mut p.attack_cd, &mut p.attack_t, &mut p.invuln, &mut p.hitstun] {
                *timer = (*timer - DT).max(0.0);
            }
            let control = p.hitstun <= 0.0;
            let sliding = p.wall_sliding(&inp);

            if jump_pressed {
                p.jump_buf = t.jump_buffer;
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
            if control && attack_pressed && p.attack_cd <= 0.0 {
                p.attack_dir = if inp.y > 0.5 {
                    AttackDir::Up
                } else if inp.y < -0.5 && !p.on_ground {
                    AttackDir::Down
                } else {
                    AttackDir::Side
                };
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

            // Fell out of the level: lose a mask and return to the spawn point.
            if p.y < bounds.1 - 200.0 {
                let s = self.level.spawns[i % self.level.spawns.len()];
                (p.x, p.y, p.vx, p.vy) = (s.0, s.1, 0.0, 0.0);
                (p.px, p.py) = (p.x, p.y);
                p.hp -= 1;
                p.invuln = t.invuln_time;
            }
            if p.hp <= 0 {
                let s = self.level.spawns[i % self.level.spawns.len()];
                *p = Player::new(s.0, s.1, t);
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
                                let target = self.players.iter().map(|p| p.x).min_by(|a, b| (a - e.x).abs().total_cmp(&(b - e.x).abs()));
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
            if let Some(hb) = p.attack_box(t) {
                // One id per swing per player: the tick the swing started on.
                let started = self.tick - ((t.attack_time - p.attack_t) / DT).round() as u32;
                let swing = started * 4 + i as u32 + 1;
                let mut connected = false;
                for e in self.enemies.iter_mut() {
                    let Some(et) = tuning.enemies.get(&e.kind) else { continue };
                    if e.last_hit == swing || !hb.overlaps(&Rect::centered(e.x, e.y, et.width, et.height)) {
                        continue;
                    }
                    e.last_hit = swing;
                    e.hp -= t.attack_damage;
                    e.flash = 0.12;
                    e.stun = 0.25;
                    match p.attack_dir {
                        AttackDir::Side => (e.vx, e.vy) = (p.facing * et.knockback, 120.0),
                        AttackDir::Up => (e.vx, e.vy) = (0.0, et.knockback),
                        AttackDir::Down => (e.vx, e.vy) = (0.0, 0.0),
                    }
                    connected = true;
                }
                if connected {
                    match p.attack_dir {
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
            if p.invuln <= 0.0 {
                let body = p.body(t);
                for e in &self.enemies {
                    let Some(et) = tuning.enemies.get(&e.kind) else { continue };
                    if e.hp > 0 && body.overlaps(&Rect::centered(e.x, e.y, et.width, et.height)) {
                        p.hp -= et.contact_damage;
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
            name: "test".into(),
            bounds: Rect(0.0, 0.0, 2000.0, 1000.0),
            spawns: vec![(500.0, 100.0)],
            solids: vec![Rect(0.0, 0.0, 2000.0, 40.0), Rect(900.0, 40.0, 40.0, 600.0)],
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
        load_level(include_str!("../assets/levels/test_room.ron")).unwrap();
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
            kind: "cherry".into(), x: w.players[0].x, y: 40.0 + et.height / 2.0, px: 0.0, py: 0.0, vx: 0.0, vy: 0.0,
            dir: 1.0, hp: 1, on_ground: true, timer: 0.0, stun: 9.0, flash: 0.0, last_hit: 0,
        });
        w.players[0].y += 90.0;
        w.players[0].on_ground = false;
        w.players[0].vy = -100.0;
        w.step(&[Input { y: -1.0, attack: true, ..Default::default() }], &t);
        assert!(w.enemies.is_empty());
        assert!(w.players[0].vy > 0.0);
    }
}
