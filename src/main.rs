//! Charm Adventure in Tomato Land - movement slice.
mod sim;

use bevy::prelude::*;
use bevy::window::{MonitorSelection, PrimaryWindow, WindowMode};
use sim::{AttackDir, Rect, Tuning, World as SimWorld};
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::time::SystemTime;

const VIEW_H: f32 = 540.0;
const BG_SIZE: Vec2 = Vec2::new(1080.0, 675.0);

#[derive(Resource)]
struct Game {
    world: SimWorld,
    tuning: Tuning,
    input: sim::Input,
    dir: PathBuf,
    stamps: [Option<SystemTime>; 2],
    status: String,
    geo_dirty: bool,
}

#[derive(Component)]
struct Geo;
#[derive(Component)]
struct EnemyVis(usize);
#[derive(Component)]
struct Hud;
#[derive(Component)]
struct PartPivot;
#[derive(Component)]
struct BgLayer {
    factor: f32,
    slot: f32,
}
/// A cutout figure: a root at the feet with rotating parts under it.
#[derive(Component)]
struct Rig {
    who: usize,
    leg_b: Entity,
    leg_f: Entity,
    body: Entity,
    head: Entity,
    weapon: Entity,
    swings: [(f32, f32); 3],
}
/// Which character player 1 is shown as (0 Simon, 1 Charm). Tab swaps.
#[derive(Resource)]
struct Skin(usize);

#[derive(Deserialize)]
struct RigFile {
    canvas: (f32, f32),
    origin: (f32, f32),
    units_per_px: f32,
    characters: BTreeMap<String, CharRig>,
    enemies: BTreeMap<String, EnemySprite>,
}
#[derive(Deserialize)]
struct CharRig {
    parts: BTreeMap<String, (f32, f32)>,
    swing_side: (f32, f32),
    swing_up: (f32, f32),
    swing_down: (f32, f32),
}
#[derive(Deserialize)]
struct EnemySprite {
    file: String,
    size: (f32, f32),
    origin: (f32, f32),
    units_per_px: f32,
}
struct EnemyArt {
    image: Handle<Image>,
    size: Vec2,
    lift: f32,
}
#[derive(Resource, Default)]
struct Art {
    enemies: HashMap<String, EnemyArt>,
}

fn asset_dir() -> PathBuf {
    let beside_exe = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.join("assets")));
    match beside_exe {
        Some(p) if p.is_dir() => p,
        _ => PathBuf::from("assets"),
    }
}

fn files(dir: &PathBuf) -> [PathBuf; 2] {
    [dir.join("config/tuning.ron"), dir.join("levels/rootway.ron")]
}

fn load(dir: &PathBuf) -> Result<(Tuning, sim::Level), String> {
    let [t, l] = files(dir);
    let read = |p: &PathBuf| std::fs::read_to_string(p).map_err(|e| format!("{}: {e}", p.display()));
    let tuning = sim::load_tuning(&read(&t)?).map_err(|e| format!("tuning.ron: {e}"))?;
    let level = sim::load_level(&read(&l)?).map_err(|e| format!("rootway.ron: {e}"))?;
    if level.spawns.is_empty() {
        return Err("level has no spawns".into());
    }
    Ok((tuning, level))
}

fn stamps(dir: &PathBuf) -> [Option<SystemTime>; 2] {
    files(dir).map(|p| std::fs::metadata(p).and_then(|m| m.modified()).ok())
}

fn main() {
    let dir = asset_dir();
    let (tuning, level) = match load(&dir) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Cannot start: {e}");
            std::process::exit(1);
        }
    };
    let fullscreen = std::env::args().any(|a| a == "--fullscreen");
    let game = Game {
        world: SimWorld::new(level, &tuning, 1),
        tuning,
        input: default(),
        stamps: stamps(&dir),
        dir,
        status: String::new(),
        geo_dirty: true,
    };
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Charm Adventure in Tomato Land".into(),
                resolution: (1280u32, 800u32).into(),
                mode: if fullscreen { WindowMode::BorderlessFullscreen(MonitorSelection::Current) } else { WindowMode::Windowed },
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.03, 0.04, 0.09)))
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .insert_resource(game)
        .insert_resource(Skin(0))
        .init_resource::<Art>()
        .add_systems(Startup, setup)
        .add_systems(Update, (read_input, hot_reload, build_geo, animate, draw_enemies, camera, hud, hotkeys).chain())
        .add_systems(FixedUpdate, tick)
        .run();
}

fn setup(mut commands: Commands, assets: Res<AssetServer>, game: Res<Game>, mut art: ResMut<Art>) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode::FixedVertical { viewport_height: VIEW_H },
            ..OrthographicProjection::default_2d()
        }),
    ));
    commands.spawn((
        ImageNode::new(assets.load("sprites/bg/vignette.png")),
        Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
    ));
    commands.spawn((
        Hud,
        Text::new(""),
        TextColor(Color::srgb(0.91, 0.89, 0.84)),
        Node { position_type: PositionType::Absolute, top: Val::Px(10.0), left: Val::Px(14.0), ..default() },
    ));
    for (file, factor, z) in [("sprites/bg/far.png", 0.12, -20.0), ("sprites/bg/mid.png", 0.35, -10.0)] {
        for i in 0..6 {
            let mut sprite = Sprite::from_image(assets.load(file));
            sprite.custom_size = Some(BG_SIZE);
            sprite.flip_x = i % 2 == 1;
            commands.spawn((BgLayer { factor, slot: i as f32 - 2.0 }, sprite, Transform::from_xyz(0.0, 0.0, z)));
        }
    }

    let path = game.dir.join("sprites/rigs.ron");
    let rigs: RigFile = match std::fs::read_to_string(&path).map_err(|e| e.to_string()).and_then(|t| ron::from_str(&t).map_err(|e| e.to_string())) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Cannot load {}: {e}", path.display());
            std::process::exit(1);
        }
    };
    for (kind, e) in &rigs.enemies {
        let size = Vec2::new(e.size.0, e.size.1) * e.units_per_px;
        let lift = (e.origin.1 - e.size.1 / 2.0) * e.units_per_px;
        art.enemies.insert(kind.clone(), EnemyArt { image: assets.load(&e.file), size, lift });
    }

    let upp = rigs.units_per_px;
    let at = |p: (f32, f32)| Vec2::new((p.0 - rigs.origin.0) * upp, (rigs.origin.1 - p.1) * upp);
    let centre = at((rigs.canvas.0 / 2.0, rigs.canvas.1 / 2.0));
    let size = Vec2::new(rigs.canvas.0, rigs.canvas.1) * upp;
    for (who, name) in ["simon", "charm"].into_iter().enumerate() {
        let Some(rig) = rigs.characters.get(name) else { continue };
        let pivot = |role: &str| rig.parts.get(role).map(|p| at(*p));
        let body_at = pivot("body").unwrap_or_default();
        let mut part = |role: &str, parent: Vec2, z: f32| -> Entity {
            let p = pivot(role).unwrap_or(parent);
            let holder = commands.spawn((PartPivot, Transform::from_translation((p - parent).extend(z)), Visibility::Inherited)).id();
            if pivot(role).is_some() {
                let mut sprite = Sprite::from_image(assets.load(format!("sprites/{name}/{role}.png")));
                sprite.custom_size = Some(size);
                let child = commands.spawn((sprite, Transform::from_translation((centre - p).extend(0.0)))).id();
                commands.entity(holder).add_child(child);
            }
            holder
        };
        let leg_b = part("leg_b", Vec2::ZERO, -0.02);
        let leg_f = part("leg_f", Vec2::ZERO, -0.01);
        let body = part("body", Vec2::ZERO, 0.0);
        let back = part("back", body_at, -0.05);
        let head = part("head", body_at, 0.02);
        let weapon = part("weapon", body_at, 0.03);
        commands.entity(body).add_children(&[back, head, weapon]);
        let swings = [rig.swing_side, rig.swing_up, rig.swing_down];
        let root = commands.spawn((Rig { who, leg_b, leg_f, body, head, weapon, swings }, Transform::default(), Visibility::Hidden)).id();
        commands.entity(root).add_children(&[leg_b, leg_f, body]);
    }
}

fn read_input(keys: Res<ButtonInput<KeyCode>>, pads: Query<&Gamepad>, mut game: ResMut<Game>) {
    let k = |codes: &[KeyCode]| codes.iter().any(|c| keys.pressed(*c));
    let mut i = sim::Input {
        x: k(&[KeyCode::ArrowRight, KeyCode::KeyD]) as i8 as f32 - k(&[KeyCode::ArrowLeft, KeyCode::KeyA]) as i8 as f32,
        y: k(&[KeyCode::ArrowUp, KeyCode::KeyW]) as i8 as f32 - k(&[KeyCode::ArrowDown, KeyCode::KeyS]) as i8 as f32,
        jump: k(&[KeyCode::Space, KeyCode::KeyZ]),
        attack: k(&[KeyCode::KeyX, KeyCode::KeyJ]),
        dash: k(&[KeyCode::KeyC, KeyCode::KeyK, KeyCode::ShiftLeft]),
        call_dog: k(&[KeyCode::KeyF]),
    };
    for pad in &pads {
        let stick = pad.left_stick();
        let dpad = Vec2::new(
            pad.pressed(GamepadButton::DPadRight) as i8 as f32 - pad.pressed(GamepadButton::DPadLeft) as i8 as f32,
            pad.pressed(GamepadButton::DPadUp) as i8 as f32 - pad.pressed(GamepadButton::DPadDown) as i8 as f32,
        );
        let v = if dpad != Vec2::ZERO { dpad } else { stick };
        if v.x.abs() > 0.25 {
            i.x = v.x;
        }
        if v.y.abs() > 0.25 {
            i.y = v.y;
        }
        i.jump |= pad.pressed(GamepadButton::South);
        i.attack |= pad.pressed(GamepadButton::West);
        i.dash |= pad.pressed(GamepadButton::RightTrigger2);
        i.call_dog |= pad.pressed(GamepadButton::North);
    }
    game.input = i;
}

fn tick(mut game: ResMut<Game>) {
    let g = &mut *game;
    g.world.step(&[g.input], &g.tuning);
}

fn restart(game: &mut Game) {
    match load(&game.dir) {
        Ok((tuning, level)) => {
            game.world = SimWorld::new(level, &tuning, 1);
            game.tuning = tuning;
            game.geo_dirty = true;
            game.status.clear();
        }
        Err(e) => game.status = format!("RELOAD FAILED  {e}"),
    }
}

/// Polls the tuning and level files twice a second and applies edits live.
fn hot_reload(time: Res<Time>, mut acc: Local<f32>, mut game: ResMut<Game>) {
    *acc += time.delta_secs();
    if *acc < 0.5 {
        return;
    }
    *acc = 0.0;
    let now = stamps(&game.dir);
    if now == game.stamps {
        return;
    }
    let level_changed = now[1] != game.stamps[1];
    game.stamps = now;
    if level_changed {
        restart(&mut game);
    } else {
        match load(&game.dir) {
            Ok((tuning, _)) => {
                game.tuning = tuning;
                game.status.clear();
            }
            Err(e) => game.status = format!("RELOAD FAILED  {e}"),
        }
    }
}

/// Cheap repeatable noise in 0..1 for decoration placement.
fn noise(n: f32) -> f32 {
    ((n * 12.9898).sin() * 43758.547).fract().abs()
}

fn build_geo(mut commands: Commands, mut game: ResMut<Game>, old: Query<Entity, With<Geo>>) {
    if !game.geo_dirty {
        return;
    }
    game.geo_dirty = false;
    for e in &old {
        commands.entity(e).despawn();
    }
    let rock = Color::srgb(0.025, 0.035, 0.065);
    let moss = Color::srgb(0.31, 0.82, 0.69);
    let blade = Color::srgb(0.18, 0.6, 0.52);
    let solids = &game.world.level.solids;
    let mut bar = |c: Color, x: f32, y: f32, w: f32, h: f32, z: f32, rot: f32| {
        commands.spawn((Geo, Sprite::from_color(c, Vec2::new(w, h)), Transform::from_xyz(x, y, z).with_rotation(Quat::from_rotation_z(rot))));
    };
    for (i, s) in solids.iter().enumerate() {
        bar(rock, s.0 + s.2 / 2.0, s.1 + s.3 / 2.0, s.2, s.3, 0.0, 0.0);
        let top = s.1 + s.3;
        let mut x = s.0;
        while x < s.0 + s.2 {
            let w = (s.0 + s.2 - x).min(20.0);
            let mid = x + w / 2.0;
            let buried = solids.iter().enumerate().any(|(j, o)| j != i && o.contains(mid, top + 3.0));
            if !buried {
                bar(moss, mid, top - 2.5, w, 5.0, 0.2, 0.0);
                bar(Color::srgba(0.56, 0.94, 0.85, 0.5), mid, top - 6.5, w, 1.5, 0.2, 0.0);
                for k in 0..2 {
                    let n = mid * 0.37 + k as f32 * 7.1 + top;
                    let h = 6.0 + noise(n) * 14.0;
                    bar(blade, x + noise(n + 1.0) * w, top + h / 2.0 - 1.0, 2.0, h, 0.15, (noise(n + 2.0) - 0.5) * 0.7);
                }
            } else {
                // hanging roots under an overhang are drawn from the rock above, nothing here
            }
            x += 20.0;
        }
        // roots under free-floating slabs
        if s.3 < 60.0 {
            for k in 0..(s.2 / 60.0) as i32 {
                let n = s.0 + k as f32 * 13.3 + s.1;
                let len = 20.0 + noise(n) * 50.0;
                bar(rock, s.0 + 20.0 + noise(n + 3.0) * (s.2 - 40.0), s.1 - len / 2.0 + 2.0, 3.0, len, 0.1, (noise(n + 5.0) - 0.5) * 0.3);
            }
        }
    }
    for h in &game.world.level.hazards {
        bar(Color::srgb(0.12, 0.02, 0.05), h.0 + h.2 / 2.0, h.1 + 4.0, h.2, 8.0, 0.3, 0.0);
        let mut x = h.0 + 6.0;
        while x < h.0 + h.2 - 4.0 {
            let len = h.3 + 6.0 + noise(x) * 16.0;
            bar(Color::srgb(0.6, 0.12, 0.14), x, h.1 + len / 2.0, 5.0, len, 0.3, (noise(x + 9.0) - 0.5) * 0.6);
            x += 11.0;
        }
    }
}

fn animate(
    game: Res<Game>,
    skin: Res<Skin>,
    fixed: Res<Time<Fixed>>,
    time: Res<Time>,
    mut rigs: Query<(&Rig, &mut Transform, &mut Visibility), Without<PartPivot>>,
    mut parts: Query<&mut Transform, With<PartPivot>>,
) {
    let a = fixed.overstep_fraction();
    let t = &game.tuning.player;
    let w = &game.world;
    let now = time.elapsed_secs();
    for (rig, mut tf, mut vis) in &mut rigs {
        let Some(p) = w.players.first().filter(|_| rig.who == skin.0) else {
            *vis = Visibility::Hidden;
            continue;
        };
        let blink = p.invuln > 0.0 && (w.tick / 4) % 2 == 0;
        *vis = if blink { Visibility::Hidden } else { Visibility::Inherited };
        let (x, y) = (p.px + (p.x - p.px) * a, p.py + (p.y - p.py) * a);
        let running = p.on_ground && p.vx.abs() > 30.0;
        let stride = (now * if p.sprinting { 20.0 } else { 15.0 }).sin();
        // (back leg, front leg, body lean, head) in radians, figure facing right
        let (mut lb, mut lf, mut body, mut head) = (0.0, 0.0, (now * 2.0).sin() * 0.015, (now * 2.0 + 1.0).sin() * 0.02);
        let mut bob = 0.0;
        if p.dash_t > 0.0 {
            (lb, lf, body, head) = (-1.0, -0.8, -0.55, 0.25);
        } else if p.hitstun > 0.0 {
            (lb, lf, body, head) = (0.5, 0.7, 0.4, 0.2);
        } else if !p.on_ground && p.wall != 0 && p.vy < 0.0 {
            (lb, lf, body, head) = (0.35, -0.25, 0.12, -0.1);
        } else if !p.on_ground && p.vy > 0.0 {
            (lb, lf, body, head) = (-0.45, 0.55, -0.1, -0.08);
        } else if !p.on_ground {
            (lb, lf, body, head) = (0.4, -0.35, 0.08, 0.1);
        } else if running {
            (lb, lf, body, head) = (stride * 0.6, -stride * 0.6, if p.sprinting { -0.3 } else { -0.14 }, 0.06);
            bob = stride.abs() * 1.5;
        }
        let mut weapon = (now * 2.0).sin() * 0.03 + if running { -0.25 * p.facing.abs() } else { 0.0 };
        if p.attack_t > 0.0 {
            let u = 1.0 - p.attack_t / t.attack_time.max(0.001);
            let (from, to) = rig.swings[match p.attack_dir {
                AttackDir::Side => 0,
                AttackDir::Up => 1,
                AttackDir::Down => 2,
            }];
            weapon = (from + (to - from) * u).to_radians() - body;
        }
        tf.translation = Vec3::new(x, y - t.height / 2.0 + bob, 2.0);
        tf.scale = Vec3::new(p.facing, 1.0, 1.0);
        for (e, angle) in [(rig.leg_b, lb), (rig.leg_f, lf), (rig.body, body), (rig.head, head - body * 0.5), (rig.weapon, weapon)] {
            if let Ok(mut part) = parts.get_mut(e) {
                part.rotation = Quat::from_rotation_z(angle);
            }
        }
    }
}

fn draw_enemies(
    mut commands: Commands,
    game: Res<Game>,
    art: Res<Art>,
    fixed: Res<Time<Fixed>>,
    time: Res<Time>,
    mut enemies: Query<(&EnemyVis, &mut Transform, &mut Sprite, &mut Visibility)>,
) {
    let a = fixed.overstep_fraction();
    let w = &game.world;
    let mut pooled = 0;
    for (vis, mut tf, mut sprite, mut visible) in &mut enemies {
        pooled += 1;
        let Some((e, et)) = w.enemies.get(vis.0).and_then(|e| Some((e, game.tuning.enemies.get(&e.kind)?))) else {
            *visible = Visibility::Hidden;
            continue;
        };
        *visible = Visibility::Visible;
        let (x, y) = (e.px + (e.x - e.px) * a, e.py + (e.y - e.py) * a);
        let punch = if e.flash > 0.0 { 1.25 } else { 1.0 };
        match art.enemies.get(&e.kind) {
            Some(look) => {
                if sprite.image != look.image {
                    sprite.image = look.image.clone();
                }
                sprite.color = if e.flash > 0.0 { Color::srgb(1.0, 0.75, 0.75) } else { Color::WHITE };
                sprite.custom_size = Some(look.size);
                sprite.flip_x = e.dir > 0.0;
                let wobble = if e.on_ground && e.vx.abs() > 1.0 { (time.elapsed_secs() * 12.0 + vis.0 as f32).sin() * 0.1 } else { 0.0 };
                let feet = y - et.height / 2.0;
                tf.translation = Vec3::new(x, feet + look.lift * punch, 1.0);
                tf.rotation = Quat::from_rotation_z(wobble);
            }
            None => {
                sprite.image = Handle::default();
                sprite.custom_size = Some(Vec2::new(et.width, et.height));
                sprite.color = Color::srgb(et.color.0, et.color.1, et.color.2);
                tf.translation = Vec3::new(x, y, 1.0);
            }
        }
        tf.scale = Vec3::splat(punch);
    }
    for i in pooled..w.enemies.len() {
        commands.spawn((EnemyVis(i), Sprite::from_color(Color::WHITE, Vec2::ONE), Transform::default(), Visibility::Hidden));
    }
}

/// Camera eases toward the player, a little ahead of where they face, kept inside the level.
/// Background layers follow it at a fraction of its speed.
fn camera(
    game: Res<Game>,
    fixed: Res<Time<Fixed>>,
    time: Res<Time>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut cam: Query<&mut Transform, With<Camera2d>>,
    mut layers: Query<(&BgLayer, &mut Transform), Without<Camera2d>>,
) {
    let (Ok(mut cam), Some(p), Ok(win)) = (cam.single_mut(), game.world.players.first(), window.single()) else { return };
    let a = fixed.overstep_fraction();
    let b: Rect = game.world.level.bounds;
    let half_h = VIEW_H / 2.0;
    let half_w = half_h * win.width() / win.height().max(1.0);
    let clamp = |v: f32, lo: f32, hi: f32| if lo > hi { (lo + hi) / 2.0 } else { v.clamp(lo, hi) };
    let target = Vec2::new(
        clamp(p.px + (p.x - p.px) * a + p.facing * 60.0, b.0 + half_w, b.0 + b.2 - half_w),
        clamp(p.py + (p.y - p.py) * a + 50.0, b.1 + half_h, b.1 + b.3 - half_h),
    );
    let k = 1.0 - (-8.0 * time.delta_secs()).exp();
    let pos = cam.translation.truncate().lerp(target, k);
    cam.translation = pos.extend(cam.translation.z);
    let mid_y = b.1 + b.3 / 2.0;
    for (layer, mut tf) in &mut layers {
        let shift = (pos.x * layer.factor).rem_euclid(BG_SIZE.x * 2.0);
        tf.translation.x = pos.x - shift + layer.slot * BG_SIZE.x;
        tf.translation.y = pos.y + (mid_y - pos.y) * layer.factor * 0.3;
    }
}

fn hud(game: Res<Game>, mut text: Query<&mut Text, With<Hud>>) {
    let (Ok(mut text), Some(p)) = (text.single_mut(), game.world.players.first()) else { return };
    let line = format!(
        "HP {}/{}   Tomatoes {}   {}\n{}",
        p.hp, game.tuning.player.max_hp, game.world.enemies.len(), game.world.level.name, game.status
    );
    if text.0 != line {
        text.0 = line;
    }
}

fn hotkeys(keys: Res<ButtonInput<KeyCode>>, mut game: ResMut<Game>, mut skin: ResMut<Skin>, mut window: Query<&mut Window, With<PrimaryWindow>>, mut exit: MessageWriter<AppExit>) {
    if keys.just_pressed(KeyCode::KeyR) {
        restart(&mut game);
    }
    if keys.just_pressed(KeyCode::Tab) {
        skin.0 = 1 - skin.0;
    }
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
    if keys.just_pressed(KeyCode::F11) {
        if let Ok(mut w) = window.single_mut() {
            w.mode = match w.mode {
                WindowMode::Windowed => WindowMode::BorderlessFullscreen(MonitorSelection::Current),
                _ => WindowMode::Windowed,
            };
        }
    }
}
