//! Charm Adventure in Tomato Land - movement slice.
mod sim;

use bevy::prelude::*;
use bevy::window::{MonitorSelection, PrimaryWindow, WindowMode};
use sim::{AttackDir, Tuning, World as SimWorld};
use std::path::PathBuf;
use std::time::SystemTime;

const VIEW_H: f32 = 540.0;

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
struct PlayerVis(usize);
#[derive(Component)]
struct SlashVis(usize);
#[derive(Component)]
struct EnemyVis(usize);
#[derive(Component)]
struct Hud;

fn asset_dir() -> PathBuf {
    let beside_exe = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.join("assets")));
    match beside_exe {
        Some(p) if p.is_dir() => p,
        _ => PathBuf::from("assets"),
    }
}

fn files(dir: &PathBuf) -> [PathBuf; 2] {
    [dir.join("config/tuning.ron"), dir.join("levels/test_room.ron")]
}

fn load(dir: &PathBuf) -> Result<(Tuning, sim::Level), String> {
    let [t, l] = files(dir);
    let read = |p: &PathBuf| std::fs::read_to_string(p).map_err(|e| format!("{}: {e}", p.display()));
    let tuning = sim::load_tuning(&read(&t)?).map_err(|e| format!("tuning.ron: {e}"))?;
    let level = sim::load_level(&read(&l)?).map_err(|e| format!("test_room.ron: {e}"))?;
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
        .insert_resource(ClearColor(Color::srgb(0.55, 0.83, 0.98)))
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .insert_resource(game)
        .add_systems(Startup, setup)
        .add_systems(Update, (read_input, hot_reload, build_geo, draw, hud, hotkeys).chain())
        .add_systems(FixedUpdate, tick)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode::FixedVertical { viewport_height: VIEW_H },
            ..OrthographicProjection::default_2d()
        }),
    ));
    commands.spawn((
        Hud,
        Text::new(""),
        TextColor(Color::srgb(0.1, 0.15, 0.25)),
        Node { position_type: PositionType::Absolute, top: Val::Px(10.0), left: Val::Px(14.0), ..default() },
    ));
    commands.spawn((PlayerVis(0), Sprite::from_color(Color::WHITE, Vec2::ONE), Transform::default()));
    commands.spawn((SlashVis(0), Sprite::from_color(Color::srgba(1.0, 1.0, 1.0, 0.85), Vec2::ONE), Transform::default(), Visibility::Hidden));
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

fn build_geo(mut commands: Commands, mut game: ResMut<Game>, old: Query<Entity, With<Geo>>) {
    if !game.geo_dirty {
        return;
    }
    game.geo_dirty = false;
    for e in &old {
        commands.entity(e).despawn();
    }
    for s in &game.world.level.solids {
        let center = Vec3::new(s.0 + s.2 / 2.0, s.1 + s.3 / 2.0, 0.0);
        commands.spawn((Geo, Sprite::from_color(Color::srgb(0.62, 0.45, 0.3), Vec2::new(s.2, s.3)), Transform::from_translation(center)));
        // grass cap
        let cap = Vec3::new(center.x, s.1 + s.3 - 4.0, 0.1);
        commands.spawn((Geo, Sprite::from_color(Color::srgb(0.4, 0.8, 0.35), Vec2::new(s.2, 8.0)), Transform::from_translation(cap)));
    }
}

#[allow(clippy::type_complexity)]
fn draw(
    mut commands: Commands,
    game: Res<Game>,
    fixed: Res<Time<Fixed>>,
    time: Res<Time>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut players: Query<(&PlayerVis, &mut Transform, &mut Sprite), (Without<SlashVis>, Without<EnemyVis>, Without<Camera2d>)>,
    mut slashes: Query<(&SlashVis, &mut Transform, &mut Sprite, &mut Visibility), (Without<PlayerVis>, Without<EnemyVis>, Without<Camera2d>)>,
    mut enemies: Query<(&EnemyVis, &mut Transform, &mut Sprite, &mut Visibility), (Without<PlayerVis>, Without<SlashVis>, Without<Camera2d>)>,
    mut camera: Query<&mut Transform, With<Camera2d>>,
) {
    let a = fixed.overstep_fraction();
    let lerp = |p: f32, c: f32| p + (c - p) * a;
    let t = &game.tuning.player;
    let w = &game.world;

    for (vis, mut tf, mut sprite) in &mut players {
        let Some(p) = w.players.get(vis.0) else { continue };
        tf.translation = Vec3::new(lerp(p.px, p.x), lerp(p.py, p.y), 2.0);
        sprite.custom_size = Some(Vec2::new(t.width, t.height));
        let blink = p.invuln > 0.0 && (w.tick / 4) % 2 == 0;
        sprite.color = if p.dash_t > 0.0 {
            Color::srgb(0.6, 0.9, 1.0)
        } else if blink {
            Color::srgba(1.0, 1.0, 1.0, 0.35)
        } else {
            Color::srgb(0.85, 0.2, 0.45)
        };
    }
    for (vis, mut tf, mut sprite, mut visible) in &mut slashes {
        let hb = w.players.get(vis.0).and_then(|p| p.attack_box(t).map(|hb| (p, hb)));
        match hb {
            Some((p, hb)) => {
                let off = Vec2::new(lerp(p.px, p.x) - p.x, lerp(p.py, p.y) - p.y);
                tf.translation = Vec3::new(hb.0 + hb.2 / 2.0 + off.x, hb.1 + hb.3 / 2.0 + off.y, 3.0);
                sprite.custom_size = Some(match p.attack_dir {
                    AttackDir::Side => Vec2::new(hb.2, hb.3 * 0.5),
                    _ => Vec2::new(hb.2 * 0.5, hb.3),
                });
                *visible = Visibility::Visible;
            }
            None => *visible = Visibility::Hidden,
        }
    }

    let mut pooled = 0;
    for (vis, mut tf, mut sprite, mut visible) in &mut enemies {
        pooled += 1;
        let Some((e, et)) = w.enemies.get(vis.0).and_then(|e| Some((e, game.tuning.enemies.get(&e.kind)?))) else {
            *visible = Visibility::Hidden;
            continue;
        };
        *visible = Visibility::Visible;
        tf.translation = Vec3::new(lerp(e.px, e.x), lerp(e.py, e.y), 1.0);
        sprite.custom_size = Some(Vec2::new(et.width, et.height));
        sprite.color = if e.flash > 0.0 { Color::WHITE } else { Color::srgb(et.color.0, et.color.1, et.color.2) };
    }
    for i in pooled..w.enemies.len() {
        commands.spawn((EnemyVis(i), Sprite::from_color(Color::WHITE, Vec2::ONE), Transform::default(), Visibility::Hidden));
    }

    // Camera: ease toward the player, a little ahead of where they face, kept inside the level.
    if let (Ok(mut cam), Some(p), Ok(win)) = (camera.single_mut(), w.players.first(), window.single()) {
        let b = w.level.bounds;
        let half_h = VIEW_H / 2.0;
        let half_w = half_h * win.width() / win.height().max(1.0);
        let clamp = |v: f32, lo: f32, hi: f32| if lo > hi { (lo + hi) / 2.0 } else { v.clamp(lo, hi) };
        let target = Vec2::new(
            clamp(lerp(p.px, p.x) + p.facing * 60.0, b.0 + half_w, b.0 + b.2 - half_w),
            clamp(lerp(p.py, p.y) + 50.0, b.1 + half_h, b.1 + b.3 - half_h),
        );
        let k = 1.0 - (-8.0 * time.delta_secs()).exp();
        let pos = cam.translation.truncate().lerp(target, k);
        cam.translation = pos.extend(cam.translation.z);
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

fn hotkeys(keys: Res<ButtonInput<KeyCode>>, mut game: ResMut<Game>, mut window: Query<&mut Window, With<PrimaryWindow>>, mut exit: MessageWriter<AppExit>) {
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
