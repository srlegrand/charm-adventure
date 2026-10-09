#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
//! Charm Adventure in Tomato Land - movement slice.
mod sim;

use bevy::post_process::bloom::{Bloom, BloomCompositeMode, BloomPrefilter};
use bevy::prelude::*;
use bevy::camera::visibility::RenderLayers;
use bevy::camera::{ClearColorConfig, Hdr, Viewport};
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
    input: [sim::Input; 2],
    players: usize,
    dir: PathBuf,
    stamps: [Option<SystemTime>; 2],
    status: String,
    geo_dirty: bool,
}

/// A piece of a player's corner display: 0 bouquet wrap, 1 flower slot, 2 heart ready to send,
/// 3 life, 5 Simon's face, 6 Charm's face.
#[derive(Component)]
struct HudBit {
    kind: u8,
    i: usize,
    cam: usize,
}
#[derive(Component)]
struct BubblePart;
#[derive(Resource)]
struct StoryUi {
    root: Entity,
    bg: Entity,
    tail: Entity,
    text: Entity,
}
const BUBBLE_TEXT_SCALE: f32 = 0.5;

/// Speech, loaded from assets/story/<level>.ron. This is the data the story editor will write.
#[derive(Deserialize, Default, Clone)]
struct StoryFile {
    scenes: Vec<Scene>,
}
#[derive(Deserialize, Clone)]
struct Scene {
    id: String,
    trigger: Trigger,
    lines: Vec<Line>,
}
#[derive(Deserialize, Clone)]
enum Trigger {
    LevelStart,
    Enter(Rect),
}
#[derive(Deserialize, Clone)]
struct Line {
    who: String,
    text: String,
    #[serde(default)]
    secs: f32,
}
#[derive(Resource, Default)]
struct Story {
    file: StoryFile,
    stamp: Option<SystemTime>,
    poll: f32,
    played: Vec<String>,
    queue: std::collections::VecDeque<Line>,
    current: Option<(Line, f32)>,
    last_tick: u32,
}

/// One of the two game cameras. In split screen each follows its own player.
#[derive(Component)]
struct ViewCam(usize);
/// Where each game camera is looking and how much it sees, for things that stick to the screen.
#[derive(Resource)]
struct Views {
    pos: [Vec2; 2],
    half: [Vec2; 2],
    split: bool,
    /// F2 turns split screen off, back to one shared camera.
    want_split: bool,
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
    cam: usize,
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
    weapon_axis: f32,
}
/// Which character player 1 is shown as (0 Simon, 1 Charm). Tab swaps.
#[derive(Resource)]
struct Skin(usize);
/// Character chooser. While open the game is paused.
#[derive(Resource)]
struct Chooser(bool);
/// Something a player holds: the keyboard and mouse, or one controller.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Device {
    Keyboard,
    Pad(Entity),
}
/// The start menu. Step 0: how many players. Step 1: player 1 picks a character.
/// Whoever confirms step 0 is player 1, on the device they pressed it with.
#[derive(Resource)]
struct Menu {
    step: u8,
    p1: Device,
    p2: Device,
    note: String,
}
#[derive(Component)]
struct ChooserText;
/// The line down the middle in split screen.
#[derive(Component)]
struct Divider;
/// F3 shows the collision shapes.
#[derive(Resource, Default)]
struct ShowShapes(bool);

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
    weapon_axis: f32,
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
    slash: Handle<Image>,
    light: Handle<Image>,
    ring: Handle<Image>,
    wedge: Handle<Image>,
    leafcap: Handle<Image>,
    heart: Handle<Image>,
    props: HashMap<&'static str, Handle<Image>>,
    flowers: Vec<Handle<Image>>,
}

/// Events from the simulation waiting to be dressed up, plus screen shake and the score shown so far.
#[derive(Resource, Default)]
struct Juice {
    events: Vec<sim::Event>,
    shake: f32,
    pulse: f32,
    seed: u32,
}
impl Juice {
    /// Small repeatable random number in 0..1.
    fn rand(&mut self) -> f32 {
        self.seed = self.seed.wrapping_mul(1664525).wrapping_add(1013904223);
        (self.seed >> 8) as f32 / 16777216.0
    }
}

/// A short-lived particle: moves, slows, fades, then goes.
#[derive(Component)]
struct Fx {
    vel: Vec2,
    gravity: f32,
    drag: f32,
    life: f32,
    max: f32,
    spin: f32,
    grow: f32,
}
/// The slash crescent stays on the player who swung.
#[derive(Component)]
struct OnPlayer(usize, Vec2);
/// A flower: bursts out, then flies to the score.
#[derive(Component)]
struct Flower {
    vel: Vec2,
    age: f32,
    delay: f32,
    /// Whose bouquet it flies to.
    player: usize,
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
        world: settled(level, &tuning, 1),
        tuning,
        input: default(),
        players: 1,
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
        .insert_resource(ClearColor(Color::srgb(0.04, 0.027, 0.02)))
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .insert_resource(game)
        .insert_resource(Skin(0))
        .insert_resource(Chooser(true))
        .insert_resource(Menu { step: 0, p1: Device::Keyboard, p2: Device::Keyboard, note: String::new() })
        .init_resource::<Art>()
        .init_resource::<ShowShapes>()
        .init_resource::<Juice>()
        .init_resource::<Story>()
        .insert_resource(Views { pos: [Vec2::ZERO; 2], half: [Vec2::new(432.0, 270.0); 2], split: false, want_split: true })
        .add_systems(Startup, setup)
        .add_systems(Update, (read_input, hot_reload, build_geo, animate, draw_enemies, camera, juice, bouquet, story, hud, hotkeys, shapes).chain())
        .add_systems(FixedUpdate, tick)
        .run();
}

fn setup(mut commands: Commands, assets: Res<AssetServer>, game: Res<Game>, mut art: ResMut<Art>) {
    // Two game cameras (the second only runs in split screen) and one camera that draws the interface over both.
    for i in 0..2usize {
        commands.spawn((
            Camera2d,
            ViewCam(i),
            Camera { order: i as isize, is_active: i == 0, ..default() },
            Hdr,
            Bloom { intensity: 0.22, prefilter: BloomPrefilter { threshold: 1.0, threshold_softness: 0.4 }, composite_mode: BloomCompositeMode::Additive, ..Bloom::NATURAL },
            Projection::Orthographic(OrthographicProjection {
                scaling_mode: bevy::camera::ScalingMode::FixedVertical { viewport_height: VIEW_H },
                ..OrthographicProjection::default_2d()
            }),
            RenderLayers::from_layers(&[0, i + 1]),
        ));
    }
    // It shares the game cameras' HDR setting so all three draw into the same picture.
    commands.spawn((Camera2d, Camera { order: 10, clear_color: ClearColorConfig::None, ..default() }, Hdr, RenderLayers::layer(31), IsDefaultUiCamera));
    commands.spawn((
        ImageNode::new(assets.load("sprites/bg/vignette.png")),
        Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
    ));
    commands.spawn((
        Hud,
        Text::new(""),
        TextColor(Color::srgb(0.91, 0.89, 0.84)),
        Node { position_type: PositionType::Absolute, bottom: Val::Px(10.0), left: Val::Px(14.0), ..default() },
    ));
    commands.spawn((
        Divider,
        Node { position_type: PositionType::Absolute, left: Val::Percent(50.0), margin: UiRect::left(Val::Px(-2.0)), width: Val::Px(4.0), height: Val::Percent(100.0), ..default() },
        BackgroundColor(Color::BLACK),
        Visibility::Hidden,
    ));
    commands.spawn((
        ChooserText,
        Text::new(""),
        TextColor(Color::srgb(0.96, 0.89, 0.77)),
        TextLayout::justify(Justify::Center),
        Node { position_type: PositionType::Absolute, top: Val::Percent(22.0), width: Val::Percent(100.0), ..default() },
    ));
    for (file, factor, z) in [("sprites/bg/far.png", 0.12, -20.0), ("sprites/bg/mid.png", 0.35, -10.0)] {
        for cam in 0..2usize {
            for i in 0..6 {
                let mut sprite = Sprite::from_image(assets.load(file));
                sprite.custom_size = Some(BG_SIZE);
                sprite.flip_x = i % 2 == 1;
                commands.spawn((BgLayer { factor, slot: i as f32 - 2.0, cam }, sprite, Transform::from_xyz(0.0, 0.0, z), RenderLayers::layer(cam + 1)));
            }
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
    art.slash = assets.load("sprites/fx/slash.png");
    art.light = assets.load("sprites/fx/light.png");
    art.ring = assets.load("sprites/fx/ring.png");
    art.wedge = assets.load("sprites/fx/wedge.png");
    art.leafcap = assets.load("sprites/fx/leafcap.png");
    art.heart = assets.load("sprites/fx/heart.png");
    for name in ["pot", "lantern", "vine", "stalactite", "sprout", "arch", "curl", "thorn"] {
        art.props.insert(name, assets.load(format!("sprites/props/{name}.png")));
    }
    // The bouquet in the corner: a wrap, ten flower slots and a row of hearts.
    let bit = |commands: &mut Commands, kind: u8, i: usize, image: Handle<Image>, size: Vec2| {
        for cam in 0..2usize {
            let mut s = Sprite::from_image(image.clone());
            s.custom_size = Some(size);
            commands.spawn((HudBit { kind, i, cam }, s, Transform::from_xyz(0.0, 0.0, 6.0), Visibility::Hidden, RenderLayers::layer(cam + 1)));
        }
    };
    bit(&mut commands, 0, 0, assets.load("sprites/ui/wrap.png"), Vec2::new(24.0, 30.0));
    for i in 0..10 {
        bit(&mut commands, 1, i, assets.load(format!("sprites/fx/flower{}.png", i % 6)), Vec2::splat(17.0));
    }
    for i in 0..8 {
        bit(&mut commands, 2, i, art.heart.clone(), Vec2::splat(15.0));
        bit(&mut commands, 3, i, art.heart.clone(), Vec2::splat(19.0));
    }
    // Faces, cut from each character's head art.
    for (kind, name) in [(5u8, "simon"), (6u8, "charm")] {
        for cam in 0..2usize {
            let mut s = Sprite::from_image(assets.load(format!("sprites/{name}/head.png")));
            s.rect = Some(bevy::math::Rect::new(78.0, 8.0, 162.0, 92.0));
            s.custom_size = Some(Vec2::splat(40.0));
            commands.spawn((HudBit { kind, i: 0, cam }, s, Transform::from_xyz(0.0, 0.0, 6.0), Visibility::Hidden, RenderLayers::layer(cam + 1)));
        }
    }
    // The speech bubble: one, reused for every line.
    let font_px = 26.0;
    let mut bg = Sprite::from_image(assets.load("sprites/ui/bubble.png"));
    bg.image_mode = SpriteImageMode::Sliced(TextureSlicer { border: BorderRect::all(30.0), ..default() });
    let bg = commands.spawn((BubblePart, bg, Transform::from_xyz(0.0, 0.0, 0.0))).id();
    let mut tail = Sprite::from_image(assets.load("sprites/ui/tail.png"));
    tail.custom_size = Some(Vec2::splat(14.0));
    let tail = commands.spawn((BubblePart, tail, Transform::from_xyz(0.0, 0.0, 0.01))).id();
    let text = commands
        .spawn((BubblePart, Text2d::new(""), TextFont { font_size: bevy::text::FontSize::Px(font_px), ..default() }, TextColor(Color::srgb(0.07, 0.05, 0.06)), TextLayout::justify(Justify::Center), Transform::from_xyz(0.0, 0.0, 0.02).with_scale(Vec3::splat(BUBBLE_TEXT_SCALE))))
        .id();
    let root = commands.spawn((BubblePart, Transform::from_xyz(0.0, 0.0, 7.0), Visibility::Hidden)).id();
    commands.entity(root).add_children(&[bg, tail, text]);
    commands.insert_resource(StoryUi { root, bg, tail, text });
    art.flowers = (0..6).map(|i| assets.load(format!("sprites/fx/flower{i}.png"))).collect();
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
        let weapon_axis = rig.weapon_axis;
        let root = commands.spawn((Rig { who, leg_b, leg_f, body, head, weapon, weapon_axis }, Pose { v: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0], grounded: true, squash: 0.0, flip: 0.0, air_jumps: 0 }, Transform::default(), Visibility::Hidden)).id();
        commands.entity(root).add_children(&[leg_b, leg_f, body]);
    }
}

fn read_input(keys: Res<ButtonInput<KeyCode>>, mouse: Res<ButtonInput<MouseButton>>, pads: Query<(Entity, &Gamepad)>, menu: Res<Menu>, mut game: ResMut<Game>) {
    let k = |codes: &[KeyCode]| codes.iter().any(|c| keys.pressed(*c));
    let keyboard = sim::Input {
        x: k(&[KeyCode::ArrowRight, KeyCode::KeyD]) as i8 as f32 - k(&[KeyCode::ArrowLeft, KeyCode::KeyA]) as i8 as f32,
        y: k(&[KeyCode::ArrowUp, KeyCode::KeyW]) as i8 as f32 - k(&[KeyCode::ArrowDown, KeyCode::KeyS]) as i8 as f32,
        jump: k(&[KeyCode::Space, KeyCode::KeyZ]),
        attack: k(&[KeyCode::KeyX, KeyCode::KeyJ]) || mouse.pressed(MouseButton::Left),
        dash: k(&[KeyCode::KeyC, KeyCode::KeyK, KeyCode::ShiftLeft]) || mouse.pressed(MouseButton::Right),
        call_dog: k(&[KeyCode::KeyF]),
        give: k(&[KeyCode::KeyG, KeyCode::KeyQ]),
    };
    let from_pad = |pad: &Gamepad| {
        let stick = pad.left_stick();
        let dpad = Vec2::new(
            pad.pressed(GamepadButton::DPadRight) as i8 as f32 - pad.pressed(GamepadButton::DPadLeft) as i8 as f32,
            pad.pressed(GamepadButton::DPadUp) as i8 as f32 - pad.pressed(GamepadButton::DPadDown) as i8 as f32,
        );
        let v = if dpad != Vec2::ZERO { dpad } else { stick };
        sim::Input {
            x: if v.x.abs() > 0.25 { v.x } else { 0.0 },
            y: if v.y.abs() > 0.25 { v.y } else { 0.0 },
            jump: pad.pressed(GamepadButton::South),
            attack: pad.pressed(GamepadButton::West),
            dash: pad.pressed(GamepadButton::RightTrigger2),
            call_dog: pad.pressed(GamepadButton::North),
            give: pad.pressed(GamepadButton::East),
        }
    };
    let merge = |a: sim::Input, b: sim::Input| sim::Input {
        x: if b.x != 0.0 { b.x } else { a.x },
        y: if b.y != 0.0 { b.y } else { a.y },
        jump: a.jump || b.jump,
        attack: a.attack || b.attack,
        dash: a.dash || b.dash,
        call_dog: a.call_dog || b.call_dog,
        give: a.give || b.give,
    };
    let of = |d: Device| match d {
        Device::Keyboard => keyboard,
        Device::Pad(e) => pads.get(e).map(|(_, p)| from_pad(p)).unwrap_or_default(),
    };
    // One player: every device drives player 1.
    // Two players: each player has exactly the device chosen in the start menu, and nothing else.
    game.input = if game.players < 2 {
        [pads.iter().fold(keyboard, |acc, (_, p)| merge(acc, from_pad(p))), sim::Input::default()]
    } else {
        [of(menu.p1), of(menu.p2)]
    };
}

fn tick(mut game: ResMut<Game>, mut juice: ResMut<Juice>, chooser: Res<Chooser>) {
    if chooser.0 {
        return;
    }
    let g = &mut *game;
    g.world.step(&g.input, &g.tuning);
    juice.events.extend(g.world.events.drain(..));
}

/// A new world, run for a moment so everyone starts standing on the ground.
fn settled(level: sim::Level, tuning: &Tuning, players: usize) -> SimWorld {
    let mut world = SimWorld::new(level, tuning, players);
    for _ in 0..20 {
        world.step(&[sim::Input::default(); 2], tuning);
    }
    world
}

fn restart(game: &mut Game) {
    match load(&game.dir) {
        Ok((tuning, level)) => {
            game.world = settled(level, &tuning, game.players);
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

fn build_geo(mut commands: Commands, mut game: ResMut<Game>, art: Res<Art>, old: Query<Entity, With<Geo>>) {
    if !game.geo_dirty {
        return;
    }
    game.geo_dirty = false;
    for e in &old {
        commands.entity(e).despawn();
    }
    let rock = Color::srgb(0.045, 0.03, 0.022);
    let brick = Color::srgb(0.085, 0.056, 0.04);
    let trim = Color::srgb(0.72, 0.45, 0.17);
    let side = Color::srgb(0.24, 0.15, 0.08);
    let blade = Color::srgb(0.3, 0.46, 0.2);
    let level = &game.world.level;
    let solids = &level.solids;
    let inside = |x: f32, y: f32, skip: usize| solids.iter().enumerate().any(|(j, o)| j != skip && o.contains(x, y));
    let near_thorns = |x: f32, y: f32| level.hazards.iter().any(|h| h.overlaps(&Rect::centered(x, y, 40.0, 80.0)));
    let mut bar = |c: Color, x: f32, y: f32, w: f32, h: f32, z: f32, rot: f32| {
        commands.spawn((Geo, Sprite::from_color(c, Vec2::new(w, h)), Transform::from_xyz(x, y, z).with_rotation(Quat::from_rotation_z(rot))));
    };
    let mut props: Vec<(&str, f32, f32, Vec2, f32, Color)> = Vec::new();
    for (i, s) in solids.iter().enumerate() {
        bar(rock, s.0 + s.2 / 2.0, s.1 + s.3 / 2.0, s.2, s.3, 0.0, 0.0);
        // Stonework: a scatter of slightly lighter blocks inside the rock.
        for k in 0..((s.2 * s.3 / 5000.0) as i32).min(160) {
            let n = s.0 * 0.013 + s.1 * 0.021 + k as f32 * 3.7;
            let (w, h) = (16.0 + noise(n) * 26.0, 7.0 + noise(n + 1.0) * 8.0);
            if s.2 > w + 14.0 && s.3 > h + 14.0 {
                bar(brick, s.0 + 7.0 + w / 2.0 + noise(n + 2.0) * (s.2 - w - 14.0), s.1 + 7.0 + h / 2.0 + noise(n + 3.0) * (s.3 - h - 14.0), w, h, 0.05, 0.0);
            }
        }
        let (top, bottom) = (s.1 + s.3, s.1);
        let mut x = s.0;
        while x < s.0 + s.2 {
            let w = (s.0 + s.2 - x).min(20.0);
            let mid = x + w / 2.0;
            let n = mid * 0.37 + top;
            if !inside(mid, top + 3.0, i) {
                // Walkable top: an amber lip, a shadow line under it, grass, and now and then a sprout or a jar.
                bar(trim, mid, top - 1.5, w, 3.0, 0.2, 0.0);
                bar(Color::srgba(0.0, 0.0, 0.0, 0.5), mid, top - 5.0, w, 4.0, 0.2, 0.0);
                for k in 0..2 {
                    let nk = n + k as f32 * 7.1;
                    let h = 4.0 + noise(nk) * 11.0;
                    bar(blade, x + noise(nk + 1.0) * w, top + h / 2.0 - 1.0, 1.6, h, 0.15, (noise(nk + 2.0) - 0.5) * 0.8);
                }
                let roll = noise(n + 11.0);
                if !near_thorns(mid, top + 20.0) && !inside(mid, top + 40.0, i) {
                    if roll < 0.07 {
                        props.push(("sprout", mid, top + 9.0, Vec2::new(16.0, 20.0), 0.45, Color::WHITE));
                    } else if roll < 0.12 && s.3 >= 60.0 {
                        let k = 0.8 + noise(n + 12.0) * 0.5;
                        props.push(("pot", mid, top + 11.0 * k, Vec2::new(22.0, 25.0) * k, 0.4, Color::WHITE));
                    }
                }
            }
            if !inside(mid, bottom - 3.0, i) && bottom > level.bounds.1 + 30.0 {
                // Underside: a dim edge, and things that hang.
                bar(side, mid, bottom + 1.0, w, 2.0, 0.2, 0.0);
                let roll = noise(n + 21.0);
                let room = !inside(mid, bottom - 70.0, i);
                if roll < 0.2 && s.3 >= 40.0 {
                    let k = 0.6 + noise(n + 22.0) * 0.9;
                    props.push(("stalactite", mid, bottom - 19.0 * k + 2.0, Vec2::new(20.0, 38.0) * k, 0.3, Color::WHITE));
                } else if roll < 0.3 && room {
                    let k = 0.7 + noise(n + 23.0) * 0.7;
                    props.push(("vine", mid, bottom - 32.0 * k + 2.0, Vec2::new(22.0, 64.0) * k, 0.35, Color::WHITE));
                } else if roll < 0.335 && room && s.3 >= 40.0 {
                    // Lantern fruit: brighter than white so it blooms, with a pool of light around it.
                    props.push(("lantern", mid, bottom - 27.0 + 2.0, Vec2::new(26.0, 54.0), 0.36, Color::srgb(1.7, 1.45, 1.2)));
                    props.push(("light", mid, bottom - 40.0, Vec2::splat(230.0), 0.12, Color::srgba(1.0, 0.8, 0.5, 0.16)));
                }
            }
            x += 20.0;
        }
        // Exposed side faces get a dim edge.
        let mut y = s.1;
        while y < s.1 + s.3 {
            let h = (s.1 + s.3 - y).min(20.0);
            let mid = y + h / 2.0;
            if !inside(s.0 - 3.0, mid, i) && s.0 > level.bounds.0 + 1.0 {
                bar(side, s.0 + 1.0, mid, 2.0, h, 0.2, 0.0);
            }
            if !inside(s.0 + s.2 + 3.0, mid, i) && s.0 + s.2 < level.bounds.0 + level.bounds.2 - 1.0 {
                bar(side, s.0 + s.2 - 1.0, mid, 2.0, h, 0.2, 0.0);
            }
            y += 20.0;
        }
        // Roots under free-floating slabs.
        if s.3 < 60.0 {
            for k in 0..(s.2 / 50.0) as i32 {
                let n = s.0 + k as f32 * 13.3 + s.1;
                let len = 16.0 + noise(n) * 44.0;
                bar(rock, s.0 + 14.0 + noise(n + 3.0) * (s.2 - 28.0), s.1 - len / 2.0 + 2.0, 2.5, len, 0.1, (noise(n + 5.0) - 0.5) * 0.4);
            }
        }
    }
    // Thorns: tall crimson spikes with pale tips, brighter than white so they glow, over a red pool of light.
    for h in &level.hazards {
        bar(Color::srgb(0.1, 0.0, 0.02), h.0 + h.2 / 2.0, h.1 + 3.0, h.2 + 8.0, 8.0, 0.55, 0.0);
        props.push(("light", h.0 + h.2 / 2.0, h.1 + 10.0, Vec2::new(h.2 * 1.9 + 120.0, 170.0), 0.5, Color::srgba(1.0, 0.12, 0.1, 0.5)));
        let mut x = h.0 + 8.0;
        let mut k = 0.0;
        while x < h.0 + h.2 - 6.0 {
            let tall = h.3 + 16.0 + noise(x) * 22.0;
            let wide = 20.0 + noise(x + 4.0) * 8.0;
            props.push(("thorn", x, h.1 + tall / 2.0 - 2.0, Vec2::new(wide, tall), 0.6 + k * 0.001, Color::srgb(1.7, 1.25, 1.2)));
            x += 13.0 + noise(x + 7.0) * 5.0;
            k += 1.0;
        }
    }
    // Far scenery standing behind the play space: arches and curling vines, dimmed.
    let b = level.bounds;
    let mut x = b.0 + 320.0;
    let mut k = 0.0;
    while x < b.0 + b.2 - 200.0 {
        for row in [100.0, 1150.0] {
            let n = x * 0.011 + row;
            if noise(n) < 0.75 {
                let scale = 0.8 + noise(n + 1.0) * 0.6;
                let (name, size) = if noise(n + 2.0) < 0.5 { ("arch", Vec2::new(200.0, 260.0)) } else { ("curl", Vec2::new(150.0, 300.0)) };
                props.push((name, x + noise(n + 3.0) * 200.0, row + size.y * scale / 2.0 - 4.0, size * scale, -5.0 - k * 0.001, Color::srgba(1.0, 1.0, 1.0, 0.85)));
            }
        }
        x += 560.0;
        k += 1.0;
    }
    for (name, x, y, size, z, color) in props {
        let image = if name == "light" { art.light.clone() } else { art.props.get(name).cloned().unwrap_or_default() };
        let mut sprite = Sprite::from_image(image);
        sprite.custom_size = Some(size);
        sprite.color = color;
        sprite.flip_x = name != "light" && noise(x * 0.7 + y) < 0.5;
        commands.spawn((Geo, sprite, Transform::from_xyz(x, y, z)));
    }
}

/// Smoothed pose values for one rig, so poses snap in fast but never pop.
/// Order: back leg, front leg, body, head, weapon, root tilt, stretch x, stretch y.
#[derive(Component)]
struct Pose {
    v: [f32; 8],
    grounded: bool,
    squash: f32,
    /// Time left on the double-jump flip, and the air jumps seen last frame (to spot a new one).
    flip: f32,
    air_jumps: u8,
}

fn animate(
    game: Res<Game>,
    skin: Res<Skin>,
    fixed: Res<Time<Fixed>>,
    time: Res<Time>,
    mut rigs: Query<(&Rig, &mut Pose, &mut Transform, &mut Visibility), Without<PartPivot>>,
    mut parts: Query<&mut Transform, With<PartPivot>>,
) {
    let a = fixed.overstep_fraction();
    let t = &game.tuning.player;
    let w = &game.world;
    let now = time.elapsed_secs();
    let dt = time.delta_secs();
    for (rig, mut pose, mut tf, mut vis) in &mut rigs {
        // Player 1 is the chosen character, player 2 the other one.
        let index = (rig.who + 2 - skin.0) % 2;
        let Some(p) = w.players.get(index) else {
            *vis = Visibility::Hidden;
            continue;
        };
        let blink = p.invuln > 0.0 && (w.tick / 4) % 2 == 0;
        *vis = if blink { Visibility::Hidden } else { Visibility::Inherited };
        let (x, y) = (p.px + (p.x - p.px) * a, p.py + (p.y - p.py) * a);
        let running = p.on_ground && p.vx.abs() > 30.0;
        let stride = (now * if p.sprinting { 22.0 } else { 16.0 }).sin();
        let fall = (p.vy.abs() / t.max_fall_speed).min(1.0);
        let attacking = p.attack_t > 0.0;

        // Target pose. Angles in radians for a figure facing right: negative leans forward.
        // [back leg, front leg, body, head, weapon, root tilt, stretch x, stretch y]
        let breathe = (now * 2.0).sin();
        let mut g = [0.14, -0.14, breathe * 0.03, breathe * 0.03, breathe * 0.05, 0.0, 1.0, 1.0 + breathe * 0.012];
        let mut bob = 0.0;
        let mut snap = 26.0;
        if p.dash_t > 0.0 {
            g = [-0.55, -0.2, -0.25, 0.75, -0.9, -1.05, 0.8, 1.25];
            snap = 70.0;
        } else if p.hitstun > 0.0 {
            g = [1.1, 1.35, 0.9, 0.6, 1.4, 0.45, 1.1, 0.9];
            snap = 70.0;
        } else if !p.on_ground && p.wall != 0 && p.vy < 0.0 {
            g = [0.85, -0.55, 0.32, -0.45, 0.9, 0.12, 0.95, 1.05];
        } else if !p.on_ground && p.vy > 0.0 {
            g = [-1.0, 1.1, -0.3, -0.25, 0.7, -0.12, 0.86, 1.0 + 0.22 * fall];
        } else if !p.on_ground {
            g = [0.85, -0.9, 0.3, 0.4, -1.3, 0.1, 0.9, 1.0 + 0.18 * fall];
        } else if running {
            let lean = if p.sprinting { -0.62 } else { -0.4 };
            g = [stride * 1.05, -stride * 1.05, lean, -lean * 0.6, -0.5 + stride * 0.35, if p.sprinting { -0.12 } else { -0.05 }, 1.0, 1.0];
            bob = stride.abs() * 4.0;
            snap = 40.0;
        }
        // Double jump: a tight tucked flip, one full turn.
        const FLIP_TIME: f32 = 0.38;
        if p.air_jumps_left < pose.air_jumps && !p.on_ground {
            pose.flip = FLIP_TIME;
        }
        pose.air_jumps = p.air_jumps_left;
        if p.on_ground || p.dash_t > 0.0 || p.hitstun > 0.0 {
            pose.flip = 0.0;
        }
        pose.flip = (pose.flip - dt).max(0.0);
        if pose.flip > 0.0 {
            g = [1.5, 1.7, -0.7, 0.5, -1.2, 0.0, 0.92, 0.92];
            snap = 90.0;
        }
        if p.down {
            // Out of lives: flat on the ground until the partner passes one over.
            g = [0.2, -0.1, 0.1, 0.3, 0.6, 1.5, 1.0, 1.0];
        }
        if attacking {
            let (lb, lf, body, tilt, sy) = match p.attack_dir {
                AttackDir::Side => (-0.75, 0.85, -0.55, -0.1, 1.0),
                AttackDir::Up => (0.25, -0.25, 0.3, 0.08, 1.14),
                AttackDir::Down => (1.15, 1.3, -0.75, -0.35, 0.92),
            };
            if p.on_ground || p.attack_dir != AttackDir::Side {
                (g[0], g[1]) = (lb, lf);
            }
            (g[2], g[3], g[5], g[7]) = (body, -body * 0.5, tilt, sy);
            snap = 80.0;
        }

        // Landing squash.
        if p.on_ground && !pose.grounded {
            pose.squash = 1.0;
        }
        pose.grounded = p.on_ground;
        pose.squash = (pose.squash - dt * 7.0).max(0.0);

        let k = 1.0 - (-snap * dt).exp();
        for i in 0..8 {
            pose.v[i] += (g[i] - pose.v[i]) * k;
        }
        // The swing comes straight from the simulation, so the drawn weapon is the line that hits.
        if let Some((_, _, angle)) = p.weapon(t) {
            pose.v[4] = (angle - rig.weapon_axis).to_radians() - pose.v[2] - pose.v[5];
        }
        let v = pose.v;
        let sq = pose.squash;
        // Lean about the body centre, not the feet, so the figure stays over its collision box.
        // The flip spins the whole figure forward once, easing out.
        let turn = if pose.flip > 0.0 { let u = 1.0 - pose.flip / FLIP_TIME; -(1.0 - (1.0 - u).powi(2)) * std::f32::consts::TAU } else { 0.0 };
        let lean = (v[5] + turn) * p.facing;
        let half = t.height / 2.0;
        tf.translation = Vec3::new(x + lean.sin() * half, y - lean.cos() * half + bob, 2.0);
        tf.rotation = Quat::from_rotation_z(lean);
        tf.scale = Vec3::new(p.facing * v[6] * (1.0 + 0.22 * sq), v[7] * (1.0 - 0.28 * sq), 1.0);
        for (e, angle) in [(rig.leg_b, v[0] - v[5] * 0.5), (rig.leg_f, v[1] - v[5] * 0.5), (rig.body, v[2]), (rig.head, v[3]), (rig.weapon, v[4])] {
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
                sprite.flip_x = e.dir < 0.0;
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

/// Each camera eases toward what it follows, a little ahead of where they face, kept inside the level.
/// One player or shared view: one camera on the middle of the players. Split screen: one camera each, side by side.
/// Background layers follow their camera at a fraction of its speed.
fn camera(
    game: Res<Game>,
    fixed: Res<Time<Fixed>>,
    time: Res<Time>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut cams: Query<(&ViewCam, &mut Transform, &mut Camera)>,
    mut layers: Query<(&BgLayer, &mut Transform, &mut Visibility), Without<ViewCam>>,
    mut juice: ResMut<Juice>,
    mut views: ResMut<Views>,
    mut base: Local<[Option<Vec2>; 2]>,
) {
    let Ok(win) = window.single() else { return };
    let w = &game.world;
    let a = fixed.overstep_fraction();
    let b: Rect = w.level.bounds;
    let split = w.players.len() == 2 && views.want_split;
    views.split = split;
    let (pw, ph) = (win.physical_width().max(2), win.physical_height().max(2));
    juice.shake = (juice.shake - time.delta_secs() * 40.0).max(0.0);
    let at = |p: &sim::Player| Vec2::new(p.px + (p.x - p.px) * a, p.py + (p.y - p.py) * a);
    let up: Vec<&sim::Player> = w.players.iter().filter(|p| !p.down).collect();
    for (vc, mut tf, mut cam) in &mut cams {
        let i = vc.0;
        if i == 1 {
            cam.is_active = split;
            if !split {
                base[1] = None;
                continue;
            }
        }
        cam.viewport = if split { Some(Viewport { physical_position: UVec2::new(i as u32 * pw / 2, 0), physical_size: UVec2::new(pw / 2, ph), ..default() }) } else { None };
        let aspect = if split { (pw / 2) as f32 / ph as f32 } else { pw as f32 / ph as f32 };
        let half = Vec2::new(VIEW_H / 2.0 * aspect, VIEW_H / 2.0);
        // What this camera follows: its own player in split screen (their partner if they are down), otherwise everyone standing.
        let (focus, facing) = if split {
            let p = w.players.get(i).filter(|p| !p.down).or(up.first().copied()).or(w.players.get(i));
            p.map(|p| (at(p), p.facing)).unwrap_or((Vec2::ZERO, 0.0))
        } else if up.is_empty() {
            w.players.first().map(|p| (at(p), 0.0)).unwrap_or((Vec2::ZERO, 0.0))
        } else {
            let n = up.len() as f32;
            (up.iter().map(|p| at(p)).sum::<Vec2>() / n, if up.len() == 1 { up[0].facing } else { 0.0 })
        };
        let clamp = |v: f32, lo: f32, hi: f32| if lo > hi { (lo + hi) / 2.0 } else { v.clamp(lo, hi) };
        let target = Vec2::new(
            clamp(focus.x + facing * if split { 40.0 } else { 60.0 }, b.0 + half.x, b.0 + b.2 - half.x),
            clamp(focus.y + 50.0, b.1 + half.y, b.1 + b.3 - half.y),
        );
        let k = 1.0 - (-8.0 * time.delta_secs()).exp();
        let pos = base[i].unwrap_or(target).lerp(target, k);
        base[i] = Some(pos);
        // Shake starts hard and dies away fast.
        let jolt = Vec2::new(juice.rand() - 0.5, juice.rand() - 0.5) * 2.0 * juice.shake;
        tf.translation = (pos + jolt).extend(tf.translation.z);
        views.pos[i] = pos;
        views.half[i] = half;
    }
    let mid_y = b.1 + b.3 / 2.0;
    for (layer, mut tf, mut vis) in &mut layers {
        if layer.cam == 1 && !split {
            *vis = Visibility::Hidden;
            continue;
        }
        *vis = Visibility::Visible;
        let pos = views.pos[layer.cam];
        let shift = (pos.x * layer.factor).rem_euclid(BG_SIZE.x * 2.0);
        tf.translation.x = pos.x - shift + layer.slot * BG_SIZE.x;
        tf.translation.y = pos.y + (mid_y - pos.y) * layer.factor * 0.3;
    }
}

fn hud(
    game: Res<Game>,
    skin: Res<Skin>,
    chooser: Res<Chooser>,
    menu: Res<Menu>,
    show: Res<ShowShapes>,
    pads: Query<(Entity, &Gamepad)>,
    mut text: Query<&mut Text, (With<Hud>, Without<ChooserText>)>,
    mut pick: Query<&mut Text, (With<ChooserText>, Without<Hud>)>,
) {
    let (Ok(mut text), Some(_)) = (text.single_mut(), game.world.players.first()) else { return };
    let pad_ids: Vec<Entity> = pads.iter().map(|(e, _)| e).collect();
    let pad = match pads.iter().count() {
        0 => "no controller".to_string(),
        n => format!("{n} controller{}", if n == 1 { "" } else { "s" }),
    };
    // The top line is only for trouble (a file that failed to load) and, with F3, a line of details.
    let line = if show.0 {
        format!("Tomatoes {}   {}   {}\n{}", game.world.enemies.len(), game.world.level.name, pad, game.status)
    } else {
        game.status.clone()
    };
    if text.0 != line {
        text.0 = line;
    }
    if let Ok(mut pick) = pick.single_mut() {
        let want = if chooser.0 {
            let name = |d: Device| match d {
                Device::Keyboard => "keyboard".to_string(),
                Device::Pad(e) => format!("controller {}", pad_ids.iter().position(|p| *p == e).map(|i| i + 1).unwrap_or(0)),
            };
            if menu.step == 0 {
                let n = if game.players == 1 { "[ 1 PLAYER ]     2 PLAYERS  " } else { "  1 PLAYER     [ 2 PLAYERS ]" };
                format!("HOW MANY PLAYERS?\n\n{n}\n\nLeft / Right to choose\nA, Space or Enter to confirm\nWhoever confirms is player 1\n\n{}", menu.note)
            } else {
                let (mine, theirs) = if skin.0 == 0 { ("SIMON", "CHARM") } else { ("CHARM", "SIMON") };
                let (a, b) = if skin.0 == 0 { ("[ SIMON ]", "  CHARM  ") } else { ("  SIMON  ", "[ CHARM ]") };
                if game.players == 1 {
                    format!("CHOOSE YOUR CHARACTER\n\n{a}      {b}\n\nLeft / Right to choose\nA, Space or Enter to start\nB or Backspace to go back")
                } else {
                    format!(
                        "PLAYER 1, CHOOSE YOUR CHARACTER\n\n{a}      {b}\n\nPlayer 1 ({}) is {mine}\nPlayer 2 ({}) is {theirs}\n\nLeft / Right to choose\nA, Space or Enter to start\nB or Backspace to go back\n\nSend 10 flowers to your partner as a heart: B, or G on the keyboard\nF2 switches split screen on and off",
                        name(menu.p1),
                        name(menu.p2)
                    )
                }
            }
        } else if game.world.players.iter().any(|q| q.combo >= 2) {
            format!("x{} COMBO", game.world.players.iter().map(|q| q.combo).max().unwrap_or(0))
        } else {
            String::new()
        };
        if pick.0 != want {
            pick.0 = want;
        }
    }
}

fn hotkeys(
    keys: Res<ButtonInput<KeyCode>>,
    pads: Query<(Entity, &Gamepad)>,
    mut game: ResMut<Game>,
    mut skin: ResMut<Skin>,
    mut chooser: ResMut<Chooser>,
    mut menu: ResMut<Menu>,
    mut show: ResMut<ShowShapes>,
    mut views: ResMut<Views>,
    mut held: Local<Vec<Entity>>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
    mut exit: MessageWriter<AppExit>,
) {
    let pad = |b: GamepadButton| pads.iter().any(|(_, p)| p.just_pressed(b));
    // Tab or the controller's Select / View button opens the start menu.
    if keys.just_pressed(KeyCode::Tab) || pad(GamepadButton::Select) {
        chooser.0 = !chooser.0;
        menu.step = 0;
        menu.note.clear();
    }
    if chooser.0 {
        // What each device is asking for this frame: (device, left, right, confirm, back).
        let kp = |c: KeyCode| keys.just_pressed(c);
        let mut asks = vec![(
            Device::Keyboard,
            kp(KeyCode::ArrowLeft) || kp(KeyCode::KeyA) || kp(KeyCode::ArrowUp) || kp(KeyCode::KeyW),
            kp(KeyCode::ArrowRight) || kp(KeyCode::KeyD) || kp(KeyCode::ArrowDown) || kp(KeyCode::KeyS),
            kp(KeyCode::Enter) || kp(KeyCode::Space),
            kp(KeyCode::Backspace),
        )];
        for (e, p) in &pads {
            let x = p.left_stick().x;
            let was = held.contains(&e);
            let flick = x.abs() > 0.6 && !was;
            if x.abs() > 0.4 {
                if !was {
                    held.push(e);
                }
            } else {
                held.retain(|h| *h != e);
            }
            asks.push((
                Device::Pad(e),
                p.just_pressed(GamepadButton::DPadLeft) || p.just_pressed(GamepadButton::DPadUp) || (flick && x < 0.0),
                p.just_pressed(GamepadButton::DPadRight) || p.just_pressed(GamepadButton::DPadDown) || (flick && x > 0.0),
                p.just_pressed(GamepadButton::South) || p.just_pressed(GamepadButton::Start),
                p.just_pressed(GamepadButton::East),
            ));
        }
        let pad_list: Vec<Entity> = pads.iter().map(|(e, _)| e).collect();
        for (device, left, right, ok, back) in asks {
            if menu.step == 0 {
                // Step 0: anyone may choose the number of players. Whoever confirms becomes player 1.
                if left || right {
                    game.players = 3 - game.players;
                    menu.note.clear();
                }
                if ok {
                    // Player 2 gets a different device: another controller if there is one, otherwise the keyboard.
                    let other = match device {
                        Device::Keyboard => pad_list.first().map(|e| Device::Pad(*e)),
                        Device::Pad(mine) => Some(pad_list.iter().find(|e| **e != mine).map(|e| Device::Pad(*e)).unwrap_or(Device::Keyboard)),
                    };
                    match (game.players, other) {
                        (2, None) => menu.note = "Two players need a controller as well as the keyboard, or two controllers.".into(),
                        (_, other) => {
                            menu.p1 = device;
                            menu.p2 = other.unwrap_or(Device::Keyboard);
                            menu.note.clear();
                            menu.step = 1;
                            if game.world.players.len() != game.players {
                                restart(&mut game);
                            }
                        }
                    }
                    break;
                }
            } else if device == menu.p1 || game.players == 1 {
                // Step 1: only player 1 chooses. Player 2 is given the other character.
                if left {
                    skin.0 = 0;
                }
                if right {
                    skin.0 = 1;
                }
                if back {
                    menu.step = 0;
                }
                if ok {
                    chooser.0 = false;
                    break;
                }
            }
        }
    }
    if keys.just_pressed(KeyCode::F2) {
        views.want_split = !views.want_split;
    }
    if keys.just_pressed(KeyCode::F3) {
        show.0 = !show.0;
    }
    if keys.just_pressed(KeyCode::KeyR) {
        restart(&mut game);
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

/// F3: draw what the simulation actually collides with.
fn shapes(show: Res<ShowShapes>, game: Res<Game>, mut gizmos: Gizmos) {
    if !show.0 {
        return;
    }
    let t = &game.tuning.player;
    let w = &game.world;
    let v = |p: (f32, f32)| Vec2::new(p.0, p.1);
    for p in &w.players {
        let (a, b, r) = p.capsule(t);
        let green = Color::srgb(0.3, 1.0, 0.4);
        gizmos.circle_2d(v(a), r, green);
        gizmos.circle_2d(v(b), r, green);
        gizmos.line_2d(v(a) - Vec2::X * r, v(b) - Vec2::X * r, green);
        gizmos.line_2d(v(a) + Vec2::X * r, v(b) + Vec2::X * r, green);
        gizmos.rect_2d(Vec2::new(p.x, p.y), Vec2::new(t.width, t.height), Color::srgba(1.0, 1.0, 1.0, 0.35));
        if let Some((a, b, _)) = p.weapon(t) {
            let n = (v(b) - v(a)).normalize_or_zero().perp() * t.weapon_thickness;
            let yellow = Color::srgb(1.0, 0.9, 0.2);
            gizmos.line_2d(v(a) + n, v(b) + n, yellow);
            gizmos.line_2d(v(a) - n, v(b) - n, yellow);
            gizmos.circle_2d(v(b), t.weapon_thickness, yellow);
        }
    }
    for e in &w.enemies {
        let Some(et) = game.tuning.enemies.get(&e.kind) else { continue };
        let o = e.body(et);
        gizmos.ellipse_2d(Vec2::new(o.x, o.y), Vec2::new(o.rx, o.ry), Color::srgb(1.0, 0.3, 0.3));
        gizmos.rect_2d(Vec2::new(e.x, e.y), Vec2::new(et.width, et.height), Color::srgba(1.0, 1.0, 1.0, 0.35));
    }
    for h in &w.level.hazards {
        gizmos.rect_2d(Vec2::new(h.0 + h.2 / 2.0, h.1 + h.3 / 2.0), Vec2::new(h.2, h.3), Color::srgb(1.0, 0.4, 1.0));
    }
}

/// Turns simulation events into slash trails, speed lines, sparks, bursts and flowers, and animates them.
#[allow(clippy::too_many_arguments)]
fn juice(
    mut commands: Commands,
    game: Res<Game>,
    art: Res<Art>,
    time: Res<Time>,
    fixed: Res<Time<Fixed>>,
    views: Res<Views>,
    mut juice: ResMut<Juice>,
    mut fx: Query<(Entity, &mut Fx, &mut Transform, &mut Sprite, Option<&OnPlayer>), Without<Flower>>,
    mut flowers: Query<(Entity, &mut Flower, &mut Transform), Without<Fx>>,
) {
    let dt = time.delta_secs();
    let a = fixed.overstep_fraction();
    let t = &game.tuning.player;
    let w = &game.world;
    // Brighter than white: with the HDR camera these bloom, so effects read as giving off light.
    let white = Color::srgb(2.6, 2.4, 2.0);
    let light = |commands: &mut Commands, pos: Vec2, size: f32, strength: f32, life: f32| {
        let mut s = Sprite::from_image(art.light.clone());
        s.custom_size = Some(Vec2::splat(size));
        s.color = Color::srgba(1.0, 0.93, 0.78, strength);
        commands.spawn((s, Transform::from_translation(pos.extend(0.6)), Fx { vel: Vec2::ZERO, gravity: 0.0, drag: 0.0, life, max: life, spin: 0.0, grow: 1.5 }));
    };
    let at = |i: usize| w.players.get(i).map(|p| Vec2::new(p.px + (p.x - p.px) * a, p.py + (p.y - p.py) * a));
    let spawn_fx = |commands: &mut Commands, sprite: Sprite, pos: Vec2, z: f32, rot: f32, fx: Fx| {
        commands.spawn((sprite, Transform::from_translation(pos.extend(z)).with_rotation(Quat::from_rotation_z(rot)), fx)).id()
    };
    // Lines flying out from a point.
    let burst = |commands: &mut Commands, juice: &mut Juice, pos: Vec2, n: usize, speed: f32, len: f32, col: Color, toward: f32| {
        for _ in 0..n {
            let ang = juice.rand() * std::f32::consts::TAU;
            let dir = (Vec2::from_angle(ang) + Vec2::X * toward * 0.8).normalize_or_zero();
            let sp = speed * (0.5 + juice.rand());
            let life = 0.1 + juice.rand() * 0.12;
            spawn_fx(commands, Sprite::from_color(col, Vec2::new(len * (0.6 + juice.rand()), 2.5)), pos + dir * 8.0, 4.0, dir.to_angle(), Fx { vel: dir * sp, gravity: 0.0, drag: 6.0, life, max: life, spin: 0.0, grow: -2.0 });
        }
    };

    for ev in std::mem::take(&mut juice.events) {
        match ev {
            sim::Event::Swing { player } => {
                let Some(p) = w.players.get(player) else { continue };
                let (from, to) = p.arc(t);
                let lift = if p.attack_dir == AttackDir::Down { -t.shoulder_height * 0.5 } else { t.shoulder_height };
                let mid = ((from + to) / 2.0).to_radians();
                let mut sprite = Sprite::from_image(art.slash.clone());
                sprite.color = Color::srgb(1.9, 1.8, 1.6);
                light(&mut commands, Vec2::new(p.x + p.facing * 30.0, p.y + lift), 240.0, 0.12, 0.14);
                sprite.custom_size = Some(Vec2::splat((t.weapon_length + 10.0) / 117.0 * 256.0));
                // The crescent art thickens toward its leading end; flip it so that end leads the swing.
                sprite.flip_y = (to < from) != (p.facing < 0.0);
                let rot = if p.facing < 0.0 { std::f32::consts::PI - mid } else { mid };
                let offset = Vec2::new(0.0, lift);
                let id = spawn_fx(&mut commands, sprite, Vec2::new(p.x, p.y) + offset, 3.5, rot, Fx { vel: Vec2::ZERO, gravity: 0.0, drag: 0.0, life: 0.16, max: 0.16, spin: 0.0, grow: 0.5 });
                commands.entity(id).insert(OnPlayer(player, offset));
            }
            sim::Event::AirJump { player } => {
                // The flip is mostly speed lines: two crescents whirling round the body, a ring, and a kick of light.
                if let Some(p) = w.players.get(player) {
                    let pos = Vec2::new(p.x, p.y);
                    for k in 0..2 {
                        let mut sprite = Sprite::from_image(art.slash.clone());
                        sprite.custom_size = Some(Vec2::splat(t.height * 1.5));
                        sprite.color = Color::srgb(1.9, 1.8, 1.6);
                        sprite.flip_y = p.facing > 0.0;
                        let id = spawn_fx(&mut commands, sprite, pos, 3.5, k as f32 * std::f32::consts::PI, Fx { vel: Vec2::ZERO, gravity: 0.0, drag: 0.0, life: 0.34, max: 0.34, spin: -p.facing * 22.0, grow: 0.6 });
                        commands.entity(id).insert(OnPlayer(player, Vec2::ZERO));
                    }
                    let mut ring = Sprite::from_image(art.ring.clone());
                    ring.custom_size = Some(Vec2::new(t.width * 2.2, 10.0));
                    ring.color = Color::srgb(1.8, 1.7, 1.5);
                    spawn_fx(&mut commands, ring, pos - Vec2::Y * t.height * 0.5, 1.9, 0.0, Fx { vel: Vec2::ZERO, gravity: 0.0, drag: 0.0, life: 0.22, max: 0.22, spin: 0.0, grow: 5.0 });
                    light(&mut commands, pos, 220.0, 0.14, 0.2);
                    burst(&mut commands, &mut juice, pos - Vec2::Y * t.height * 0.4, 8, 220.0, 16.0, white, 0.0);
                }
            }
            sim::Event::Dash { player } => {
                if let Some(p) = w.players.get(player) {
                    burst(&mut commands, &mut juice, Vec2::new(p.x, p.y - t.height * 0.3), 6, 160.0, 14.0, white, -p.facing);
                }
            }
            sim::Event::Hit { x, y, dir } => {
                juice.shake = juice.shake.max(3.0);
                light(&mut commands, Vec2::new(x, y), 200.0, 0.2, 0.12);
                burst(&mut commands, &mut juice, Vec2::new(x, y), 8, 420.0, 22.0, white, dir);
                burst(&mut commands, &mut juice, Vec2::new(x, y), 5, 260.0, 8.0, Color::srgb(0.85, 0.2, 0.18), dir);
            }
            sim::Event::Give { from, to } => {
                if let (Some(a0), Some(b0)) = (at(from), at(to)) {
                    // Ten flowers stream across and burst into a heart on the partner.
                    let pink = Color::srgb(3.0, 1.2, 1.8);
                    light(&mut commands, b0, 300.0, 0.3, 0.45);
                    for i in 0..10 {
                        let life = 0.25 + i as f32 * 0.03;
                        let mut s = Sprite::from_image(art.flowers.get(i % art.flowers.len().max(1)).cloned().unwrap_or_default());
                        s.custom_size = Some(Vec2::splat(15.0));
                        let arc = 60.0 + juice.rand() * 80.0;
                        spawn_fx(&mut commands, s, a0, 4.5, 0.0, Fx { vel: (b0 - a0) / life + Vec2::Y * arc, gravity: 2.0 * arc / life, drag: 0.0, life, max: life, spin: 8.0, grow: 0.0 });
                    }
                    let mut heart = Sprite::from_image(art.heart.clone());
                    heart.custom_size = Some(Vec2::splat(26.0));
                    spawn_fx(&mut commands, heart, b0 + Vec2::Y * 40.0, 4.6, 0.0, Fx { vel: Vec2::Y * 60.0, gravity: 0.0, drag: 1.0, life: 0.9, max: 0.9, spin: 0.0, grow: 0.6 });
                    burst(&mut commands, &mut juice, b0, 16, 300.0, 16.0, pink, 0.0);
                }
            }
            sim::Event::Down { player } => {
                juice.shake = juice.shake.max(10.0);
                if let Some(p0) = at(player) {
                    burst(&mut commands, &mut juice, p0, 16, 420.0, 24.0, white, 0.0);
                }
            }
            sim::Event::Hurt { x, y } => {
                juice.shake = juice.shake.max(9.0);
                burst(&mut commands, &mut juice, Vec2::new(x, y), 10, 380.0, 18.0, white, 0.0);
            }
            sim::Event::Kill { x, y, kind, flowers, dir, combo, player } => {
                juice.shake = juice.shake.max(6.0 + combo.min(6) as f32);
                let pos = Vec2::new(x, y);
                light(&mut commands, pos, 380.0, 0.28, 0.2);
                let size = game.tuning.enemies.get(&kind).map(|e| e.width).unwrap_or(30.0);
                // The tomato itself swells and whites out,
                if let Some(look) = art.enemies.get(&kind) {
                    let mut ghost = Sprite::from_image(look.image.clone());
                    ghost.custom_size = Some(look.size);
                    ghost.color = Color::srgb(4.0, 4.0, 4.0);
                    spawn_fx(&mut commands, ghost, pos + Vec2::new(0.0, look.lift - size * 0.45), 1.5, 0.0, Fx { vel: Vec2::ZERO, gravity: 0.0, drag: 0.0, life: 0.14, max: 0.14, spin: 0.0, grow: 5.0 });
                }
                // a white flash and impact lines,
                spawn_fx(&mut commands, Sprite::from_color(white, Vec2::splat(size * 1.2)), pos, 3.8, 0.785, Fx { vel: Vec2::ZERO, gravity: 0.0, drag: 0.0, life: 0.09, max: 0.09, spin: 6.0, grow: 14.0 });
                burst(&mut commands, &mut juice, pos, 14, 560.0, 30.0, white, dir * 0.5);
                // a shockwave ring, wedges of tomato, its leaf cap spinning off,
                let mut ring = Sprite::from_image(art.ring.clone());
                ring.custom_size = Some(Vec2::splat(size * 0.8));
                ring.color = Color::srgb(2.2, 2.0, 1.7);
                spawn_fx(&mut commands, ring, pos, 3.6, 0.0, Fx { vel: Vec2::ZERO, gravity: 0.0, drag: 0.0, life: 0.28, max: 0.28, spin: 0.0, grow: 9.0 });
                for _ in 0..(4 + size as usize / 12) {
                    let ang = juice.rand() * std::f32::consts::TAU;
                    let v = Vec2::from_angle(ang) * (220.0 + juice.rand() * 300.0) + Vec2::new(dir * 220.0, 240.0);
                    let mut wedge = Sprite::from_image(art.wedge.clone());
                    wedge.custom_size = Some(Vec2::new(1.0, 0.75) * size * (0.4 + juice.rand() * 0.25));
                    let life = 0.7 + juice.rand() * 0.4;
                    spawn_fx(&mut commands, wedge, pos, 3.2, ang, Fx { vel: v, gravity: 1400.0, drag: 0.8, life, max: life, spin: (juice.rand() - 0.5) * 24.0, grow: -0.3 });
                }
                let mut cap = Sprite::from_image(art.leafcap.clone());
                cap.custom_size = Some(Vec2::new(1.0, 0.66) * size * 0.8);
                spawn_fx(&mut commands, cap, pos + Vec2::Y * size * 0.4, 3.3, 0.0, Fx { vel: Vec2::new(dir * 120.0 + (juice.rand() - 0.5) * 160.0, 520.0), gravity: 1300.0, drag: 0.6, life: 1.0, max: 1.0, spin: 14.0, grow: 0.0 });
                // seeds,
                for _ in 0..10 {
                    let ang = juice.rand() * std::f32::consts::TAU;
                    let v = Vec2::from_angle(ang) * (200.0 + juice.rand() * 420.0) + Vec2::Y * 200.0;
                    let life = 0.5 + juice.rand() * 0.4;
                    spawn_fx(&mut commands, Sprite::from_color(Color::srgb(1.6, 1.5, 1.2), Vec2::new(4.0, 2.2)), pos, 3.1, ang, Fx { vel: v, gravity: 1200.0, drag: 1.0, life, max: life, spin: 20.0, grow: 0.0 });
                }
                // smaller chunks,
                for i in 0..(10 + size as usize / 4) {
                    let ang = juice.rand() * std::f32::consts::TAU;
                    let v = Vec2::from_angle(ang) * (140.0 + juice.rand() * 320.0) + Vec2::new(dir * 180.0, 160.0);
                    let s = size * (0.1 + juice.rand() * 0.16);
                    let col = if i % 3 == 0 { Color::srgb(0.96, 0.54, 0.42) } else { Color::srgb(0.85, 0.2, 0.18) };
                    let life = 0.45 + juice.rand() * 0.35;
                    spawn_fx(&mut commands, Sprite::from_color(col, Vec2::splat(s)), pos, 3.0, ang, Fx { vel: v, gravity: 1500.0, drag: 1.2, life, max: life, spin: (juice.rand() - 0.5) * 20.0, grow: -0.6 });
                }
                // and the flowers.
                for i in 0..flowers {
                    let ang = juice.rand() * std::f32::consts::TAU;
                    let v = Vec2::from_angle(ang) * (120.0 + juice.rand() * 240.0) + Vec2::Y * 140.0;
                    let pick = (juice.rand() * art.flowers.len() as f32) as usize % art.flowers.len().max(1);
                    let mut sprite = Sprite::from_image(art.flowers.get(pick).cloned().unwrap_or_default());
                    sprite.custom_size = Some(Vec2::splat(13.0 + juice.rand() * 5.0));
                    commands.spawn((sprite, Transform::from_translation(pos.extend(5.0)), Flower { vel: v, age: 0.0, delay: 0.3 + i as f32 * 0.045, player }));
                }
            }
        }
    }

    // Speed lines trail the whole dash.
    for (i, p) in w.players.iter().enumerate() {
        if p.dash_t > 0.0 && w.hitstop == 0 && !p.down {
            let Some(pos) = at(i) else { continue };
            light(&mut commands, pos - Vec2::X * p.facing * 30.0, 240.0, 0.10, 0.12);
            for _ in 0..2 {
                let y = (juice.rand() - 0.5) * t.height * 0.9;
                let len = 40.0 + juice.rand() * 70.0;
                let life = 0.12 + juice.rand() * 0.1;
                spawn_fx(&mut commands, Sprite::from_color(white, Vec2::new(len, 2.0 + juice.rand() * 2.0)), pos + Vec2::new(-p.facing * (len * 0.5 + 6.0), y), 1.8, 0.0, Fx { vel: Vec2::new(-p.facing * 120.0, 0.0), gravity: 0.0, drag: 3.0, life, max: life, spin: 0.0, grow: -1.5 });
            }
        }
    }

    for (e, mut f, mut tf, mut sprite, follow) in &mut fx {
        f.life -= dt;
        if f.life <= 0.0 {
            commands.entity(e).despawn();
            continue;
        }
        let drag = (-f.drag * dt).exp();
        f.vel *= drag;
        f.vel.y -= f.gravity * dt;
        match follow.and_then(|o| at(o.0).map(|p| p + o.1)) {
            Some(p) => tf.translation = p.extend(tf.translation.z),
            None => tf.translation += (f.vel * dt).extend(0.0),
        }
        tf.rotate_z(f.spin * dt);
        tf.scale *= 1.0 + f.grow * dt;
        // Fade by the fraction of life lost this frame, so each sprite keeps its own starting strength.
        let before = ((f.life + dt) / f.max).clamp(0.001, 1.0);
        let k = (f.life / f.max).clamp(0.0, 1.0);
        let alpha = sprite.color.alpha() * (k / before).sqrt();
        sprite.color = sprite.color.with_alpha(alpha);
    }

    // Flowers burst out, hang for a moment, then fly to the score in the corner.
    juice.pulse = (juice.pulse - dt).max(0.0);
    for (e, mut f, mut tf) in &mut flowers {
        f.age += dt;
        let pos = tf.translation.truncate();
        if f.age < f.delay {
            f.vel *= (-4.0 * dt).exp();
            f.vel.y -= 300.0 * dt;
        } else {
            let goal = bouquet_base(&views, f.player) + Vec2::Y * 30.0;
            let to = goal - pos;
            if to.length() < 14.0 {
                commands.entity(e).despawn();
                juice.pulse = 0.15;
                continue;
            }
            let speed = (300.0 + (f.age - f.delay) * 2600.0).min(1800.0);
            f.vel = f.vel.lerp(to.normalize() * speed, (12.0 * dt).min(1.0));
        }
        tf.translation += (f.vel * dt).extend(0.0);
        tf.rotate_z(5.0 * dt);
    }
}

/// Top-left corner of a player's display. Split screen: the corner of their own half.
/// One shared view: player 1 on the left, player 2 on the right.
fn hud_corner(views: &Views, player: usize) -> Vec2 {
    if views.split {
        let c = player.min(1);
        views.pos[c] + Vec2::new(-views.half[c].x, views.half[c].y)
    } else if player == 0 {
        views.pos[0] + Vec2::new(-views.half[0].x, views.half[0].y)
    } else {
        views.pos[0] + Vec2::new(views.half[0].x - 230.0, views.half[0].y)
    }
}

/// Where a player's bouquet sits: under their face and lives.
fn bouquet_base(views: &Views, player: usize) -> Vec2 {
    hud_corner(views, player) + Vec2::new(34.0, -98.0)
}

/// Each player's corner: their face, a row of hearts for lives, and their bouquet.
/// The bouquet holds one flower per flower in hand; every ten shows as a small heart ready to send.
#[allow(clippy::too_many_arguments)]
fn bouquet(
    mut commands: Commands,
    game: Res<Game>,
    skin: Res<Skin>,
    juice: Res<Juice>,
    views: Res<Views>,
    time: Res<Time>,
    flying: Query<&Flower>,
    mut bits: Query<(Entity, &HudBit, &mut Transform, &mut Visibility, &mut Sprite), Without<Divider>>,
    mut divider: Query<&mut Visibility, With<Divider>>,
    mut was_split: Local<Option<bool>>,
) {
    if let Ok(mut v) = divider.single_mut() {
        *v = if views.split { Visibility::Visible } else { Visibility::Hidden };
    }
    // In one shared view both players' corners are drawn by the first camera.
    let relayer = *was_split != Some(views.split);
    *was_split = Some(views.split);
    let players = &game.world.players;
    let max = game.tuning.player.max_hp.max(0) as usize;
    let beat = 1.0 + (time.elapsed_secs() * 6.0).sin().max(0.0) * 0.18;
    for (e, bit, mut tf, mut vis, mut sprite) in &mut bits {
        if relayer {
            commands.entity(e).insert(RenderLayers::layer(if views.split { bit.cam + 1 } else { 1 }));
        }
        let Some(p) = players.get(bit.cam) else {
            *vis = Visibility::Hidden;
            continue;
        };
        // Flowers still in the air are not in the bunch yet.
        let in_air = flying.iter().filter(|f| f.player == bit.cam).count() as u32;
        let have = p.flowers.saturating_sub(in_air);
        let held = (have % sim::FLOWERS_PER_HEART) as usize;
        let ready = (have / sim::FLOWERS_PER_HEART) as usize;
        let corner = hud_corner(&views, bit.cam);
        let base = bouquet_base(&views, bit.cam);
        let character = (skin.0 + bit.cam) % 2;
        let mut scale = 1.0;
        let (show, pos) = match bit.kind {
            0 => (true, base),
            1 => {
                // Slots fill from the middle outward, so the bunch grows evenly.
                let step = ((bit.i + 1) / 2) as f32 * if bit.i % 2 == 0 { 1.0 } else { -1.0 };
                let a = (step * 13.0f32).to_radians();
                let r = 12.0 + (bit.i % 3) as f32 * 7.0;
                if bit.i + 1 == held && juice.pulse > 0.0 {
                    scale = 1.0 + juice.pulse * 3.0;
                }
                (bit.i < held, base + Vec2::new(a.sin() * r, 14.0 + a.cos() * r))
            }
            2 => {
                // Hearts ready to send sit beside the bouquet and beat, so they read as "press to give".
                scale = beat;
                (bit.i < ready, base + Vec2::new(40.0 + bit.i as f32 * 17.0, 2.0))
            }
            3 => {
                // Lives: bright hearts for lives left, dark ones for lives lost.
                let alive = (bit.i as i32) < p.hp;
                sprite.color = if alive { Color::WHITE } else { Color::srgba(0.12, 0.08, 0.1, 0.75) };
                (bit.i < max, corner + Vec2::new(64.0 + bit.i as f32 * 20.0, -30.0))
            }
            k => {
                sprite.color = if p.down { Color::srgb(0.3, 0.3, 0.35) } else { Color::WHITE };
                (k as usize == 5 + character, corner + Vec2::new(30.0, -30.0))
            }
        };
        *vis = if show { Visibility::Visible } else { Visibility::Hidden };
        tf.translation = pos.extend(6.0 + bit.kind as f32 * 0.01);
        tf.scale = Vec3::splat(scale);
    }
}

/// Breaks a line of speech into rows short enough for a bubble.
fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut rows = vec![String::new()];
    for word in text.split_whitespace() {
        let last = rows.last_mut().unwrap();
        if !last.is_empty() && last.chars().count() + 1 + word.chars().count() > width {
            rows.push(word.to_string());
        } else {
            if !last.is_empty() {
                last.push(' ');
            }
            last.push_str(word);
        }
    }
    rows
}

/// Plays scenes from the story file as speech bubbles over whoever is talking.
#[allow(clippy::too_many_arguments)]
fn story(
    game: Res<Game>,
    skin: Res<Skin>,
    chooser: Res<Chooser>,
    time: Res<Time>,
    fixed: Res<Time<Fixed>>,
    ui: Res<StoryUi>,
    views: Res<Views>,
    mut story: ResMut<Story>,
    mut parts: Query<(&mut Transform, Option<&mut Sprite>, Option<&mut Text2d>, Option<&mut Visibility>), With<BubblePart>>,
) {
    let dt = time.delta_secs();
    // Load the story, and reload it when the file is saved.
    story.poll -= dt;
    if story.poll <= 0.0 {
        story.poll = 0.5;
        let path = game.dir.join("story/rootway.ron");
        let stamp = std::fs::metadata(&path).and_then(|m| m.modified()).ok();
        if stamp != story.stamp {
            story.stamp = stamp;
            match std::fs::read_to_string(&path).map_err(|e| e.to_string()).and_then(|t| ron::from_str::<StoryFile>(&t).map_err(|e| e.to_string())) {
                Ok(file) => story.file = file,
                Err(e) => eprintln!("story/rootway.ron: {e}"),
            }
        }
    }
    let w = &game.world;
    // A restarted level tells its story again.
    if w.tick < story.last_tick {
        story.played.clear();
        story.queue.clear();
        story.current = None;
    }
    story.last_tick = w.tick;
    if !chooser.0 {
        let scenes = story.file.scenes.clone();
        for scene in scenes {
            if story.played.contains(&scene.id) {
                continue;
            }
            let fire = match scene.trigger {
                Trigger::LevelStart => true,
                Trigger::Enter(r) => w.players.iter().any(|p| !p.down && r.contains(p.x, p.y)),
            };
            if fire {
                story.played.push(scene.id.clone());
                story.queue.extend(scene.lines.iter().cloned());
            }
        }
        if story.current.is_none() {
            if let Some(line) = story.queue.pop_front() {
                story.current = Some((line, 0.0));
            }
        }
    }
    let mut done = false;
    let mut show = None;
    if let Some((line, age)) = story.current.as_mut() {
        if !chooser.0 {
            *age += dt;
        }
        let secs = if line.secs > 0.0 { line.secs } else { 1.4 + line.text.chars().count() as f32 * 0.06 };
        done = *age > secs;
        show = Some((line.clone(), *age));
    }
    if done {
        story.current = None;
    }
    let Some((line, age)) = show.filter(|_| !done) else {
        if let Ok((_, _, _, Some(mut vis))) = parts.get_mut(ui.root) {
            *vis = Visibility::Hidden;
        }
        return;
    };
    // Who is talking, and are they in the game?
    let who = match line.who.to_lowercase().as_str() {
        "simon" => Some(0),
        "charm" => Some(1),
        _ => None,
    };
    let a = fixed.overstep_fraction();
    let speaker = who.map(|c| (c + 2 - skin.0) % 2).and_then(|i| w.players.get(i)).filter(|p| !p.down).map(|p| Vec2::new(p.px + (p.x - p.px) * a, p.py + (p.y - p.py) * a));
    let full = if speaker.is_some() { line.text.clone() } else { format!("{}: {}", line.who.to_uppercase(), line.text) };
    let rows = wrap(&full, 20);
    let widest = rows.iter().map(|r| r.chars().count()).max().unwrap_or(1) as f32;
    let font_px = 26.0;
    let size = Vec2::new(widest * font_px * 0.6 * BUBBLE_TEXT_SCALE + 18.0, rows.len() as f32 * font_px * 1.2 * BUBBLE_TEXT_SCALE + 13.0);
    // Letters appear quickly, one after another.
    let mut left = (age * 45.0) as usize;
    let typed: Vec<String> = rows
        .iter()
        .map(|r| {
            let take = left.min(r.chars().count());
            left -= take;
            let mut s: String = r.chars().take(take).collect();
            s.extend(std::iter::repeat(' ').take(r.chars().count() - take));
            s
        })
        .collect();
    let half = views.half[0];
    let centre = views.pos[0];
    let head = game.tuning.player.height / 2.0 + 24.0;
    let mut pos = match speaker {
        Some(p) => p + Vec2::new(0.0, head + 8.0 + size.y / 2.0),
        None => centre + Vec2::new(0.0, half.y - 70.0 - size.y / 2.0),
    };
    let anchor_x = speaker.map(|p| p.x).unwrap_or(pos.x);
    // With one view the bubble is kept on screen. In split screen it simply stays over the speaker.
    if !views.split || speaker.is_none() {
        pos.x = pos.x.clamp(centre.x - half.x + size.x / 2.0 + 6.0, centre.x + half.x - size.x / 2.0 - 6.0);
        pos.y = pos.y.min(centre.y + half.y - size.y / 2.0 - 6.0);
    }
    let pop = (age * 9.0).min(1.0);
    if let Ok((mut tf, _, _, Some(mut vis))) = parts.get_mut(ui.root) {
        tf.translation = pos.extend(7.0);
        tf.scale = Vec3::splat(0.6 + 0.4 * pop * (2.0 - pop));
        *vis = Visibility::Visible;
    }
    if let Ok((_, Some(mut sprite), _, _)) = parts.get_mut(ui.bg) {
        sprite.custom_size = Some(size);
    }
    if let Ok((mut tf, _, _, vis)) = parts.get_mut(ui.tail) {
        let dx = (anchor_x - pos.x).clamp(-size.x / 2.0 + 12.0, size.x / 2.0 - 12.0);
        tf.translation = Vec3::new(dx, -size.y / 2.0 - 4.0, 0.01);
        tf.scale = Vec3::splat(if speaker.is_some() { 1.0 } else { 0.0 });
        let _ = vis;
    }
    if let Ok((_, _, Some(mut text), _)) = parts.get_mut(ui.text) {
        let want = typed.join("\n");
        if text.0 != want {
            text.0 = want;
        }
    }
}
