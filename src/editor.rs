//! The level editor, inside the game. F1 opens and closes it.
//! It edits the level being played and the story areas of that level, and writes the same files the game reads.
use crate::*;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Item {
    Solid(usize),
    Thorn(usize),
    Door(usize),
    /// A story scene that starts when a player walks into its box. The number is the scene's place in the story file.
    Area(usize),
    Tomato(usize),
    Start(usize),
}

#[derive(Clone, Copy, PartialEq, Default)]
pub enum Tool {
    #[default]
    Platform,
    Thorns,
    Tomato,
    Door,
    Area,
}

enum Drag {
    Move { grab: Vec2 },
    Size { fixed: Vec2 },
}

#[derive(Resource)]
pub struct Editor {
    pub on: bool,
    pub cam: Vec2,
    pub zoom: f32,
    tool: Tool,
    sel: Option<Item>,
    drag: Option<Drag>,
    touched: bool,
    undo: Vec<(sim::Level, StoryFile)>,
    snap: bool,
    kind: String,
    levels: Vec<String>,
    unsaved: bool,
    note: String,
}

impl Default for Editor {
    fn default() -> Self {
        Editor {
            on: false, cam: Vec2::ZERO, zoom: 1.0, tool: Tool::Platform, sel: None, drag: None, touched: false,
            undo: Vec::new(), snap: true, kind: String::new(), levels: Vec::new(), unsaved: false, note: String::new(),
        }
    }
}

#[derive(Component)]
pub struct EditorText;

pub fn setup(mut commands: Commands) {
    commands.spawn((
        EditorText,
        Text::new(""),
        TextFont { font_size: bevy::text::FontSize::Px(15.0), ..default() },
        TextColor(Color::srgb(1.0, 0.97, 0.85)),
        Node { position_type: PositionType::Absolute, top: Val::Px(96.0), left: Val::Px(14.0), padding: UiRect::all(Val::Px(8.0)), ..default() },
        BackgroundColor(Color::NONE),
    ));
}


fn rect_of<'a>(item: Item, level: &'a mut sim::Level, story: &'a mut StoryFile) -> Option<&'a mut Rect> {
    match item {
        Item::Solid(i) => level.solids.get_mut(i),
        Item::Thorn(i) => level.hazards.get_mut(i),
        Item::Door(i) => level.exits.get_mut(i).map(|e| &mut e.rect),
        Item::Area(i) => match story.scenes.get_mut(i).map(|s| &mut s.trigger) {
            Some(Trigger::Enter(r)) => Some(r),
            _ => None,
        },
        _ => None,
    }
}

fn point_of(item: Item, level: &mut sim::Level) -> Option<(&mut f32, &mut f32)> {
    match item {
        Item::Tomato(i) => level.enemies.get_mut(i).map(|e| (&mut e.x, &mut e.y)),
        Item::Start(i) => level.spawns.get_mut(i).map(|s| (&mut s.0, &mut s.1)),
        _ => None,
    }
}

fn rects(level: &sim::Level, story: &StoryFile) -> Vec<(Item, Rect)> {
    let mut out: Vec<(Item, Rect)> = Vec::new();
    out.extend(level.solids.iter().enumerate().map(|(i, r)| (Item::Solid(i), *r)));
    out.extend(level.hazards.iter().enumerate().map(|(i, r)| (Item::Thorn(i), *r)));
    out.extend(level.exits.iter().enumerate().map(|(i, e)| (Item::Door(i), e.rect)));
    for (i, s) in story.scenes.iter().enumerate() {
        if let Trigger::Enter(r) = s.trigger {
            out.push((Item::Area(i), r));
        }
    }
    out
}

/// What is under the mouse. Tomatoes and start points first, then the smallest box.
/// Doors and story areas cover a lot of ground, so unless their tool is chosen they are only picked by their edge.
fn pick(c: Vec2, unit: f32, tool: Tool, level: &sim::Level, story: &StoryFile) -> Option<Item> {
    for (i, s) in level.spawns.iter().enumerate() {
        if Vec2::new(s.0, s.1).distance(c) < 24.0 {
            return Some(Item::Start(i));
        }
    }
    for (i, e) in level.enemies.iter().enumerate().rev() {
        if Vec2::new(e.x, e.y).distance(c) < 26.0 {
            return Some(Item::Tomato(i));
        }
    }
    let edge = 8.0 * unit;
    rects(level, story)
        .into_iter()
        .filter(|(item, r)| {
            let wide = Rect(r.0 - edge, r.1 - edge, r.2 + edge * 2.0, r.3 + edge * 2.0);
            let inner = Rect(r.0 + edge, r.1 + edge, r.2 - edge * 2.0, r.3 - edge * 2.0);
            match item {
                Item::Door(_) if tool != Tool::Door => wide.contains(c.x, c.y) && !inner.contains(c.x, c.y),
                Item::Area(_) if tool != Tool::Area => wide.contains(c.x, c.y) && !inner.contains(c.x, c.y),
                _ => r.contains(c.x, c.y),
            }
        })
        .min_by(|a, b| (a.1 .2 * a.1 .3).total_cmp(&(b.1 .2 * b.1 .3)))
        .map(|(item, _)| item)
}

pub fn level_text(level: &sim::Level) -> Result<String, String> {
    let text = ron::ser::to_string_pretty(level, ron::ser::PrettyConfig::default().depth_limit(2)).map_err(|e| e.to_string())?;
    Ok(format!("// Saved by the level editor (F1 in the game). Rects are (x, y, width, height) from the bottom-left, Y up.\n{text}\n"))
}

pub fn story_text(story: &StoryFile) -> Result<String, String> {
    let text = ron::ser::to_string_pretty(story, ron::ser::PrettyConfig::default().depth_limit(4)).map_err(|e| e.to_string())?;
    Ok(format!("// Saved by the editor (F1 in the game).\n{text}\n"))
}

fn save(g: &mut Game, story: &mut Story) -> Result<(), String> {
    let path = g.dir.join(format!("levels/{}.ron", g.level));
    std::fs::write(&path, level_text(&g.world.level)?).map_err(|e| format!("{}: {e}", path.display()))?;
    g.stamps = stamps(&g.dir, &g.level);
    let path = g.dir.join(format!("story/{}.ron", g.level));
    if !story.file.scenes.is_empty() || path.exists() {
        std::fs::write(&path, story_text(&story.file)?).map_err(|e| format!("{}: {e}", path.display()))?;
        story.stamp = std::fs::metadata(&path).and_then(|m| m.modified()).ok();
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn edit(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    time: Res<Time>,
    window: Query<&Window, With<PrimaryWindow>>,
    chooser: Res<Chooser>,
    views: Res<Views>,
    mut ed: ResMut<Editor>,
    mut game: ResMut<Game>,
    mut story: ResMut<Story>,
    mut cams: Query<(&ViewCam, &mut Projection)>,
    mut text: Query<(&mut Text, &mut BackgroundColor), With<EditorText>>,
    mut gizmos: Gizmos,
) {
    let ed = &mut *ed;
    let g = &mut *game;
    let mut leave = false;
    let mut play_at = None;
    if keys.just_pressed(KeyCode::F1) && !chooser.0 {
        if ed.on {
            leave = true;
        } else {
            ed.on = true;
            ed.cam = views.pos[0];
            ed.zoom = 1.0;
            ed.sel = None;
            ed.drag = None;
            ed.note.clear();
            let mut names: Vec<String> = std::fs::read_dir(g.dir.join("levels"))
                .map(|d| d.filter_map(|f| f.ok()).map(|f| f.path()).filter(|p| p.extension().is_some_and(|x| x == "ron")).filter_map(|p| p.file_stem().map(|s| s.to_string_lossy().to_string())).collect())
                .unwrap_or_default();
            names.sort();
            ed.levels = names;
            if !g.tuning.enemies.contains_key(&ed.kind) {
                ed.kind = g.tuning.enemies.keys().next().cloned().unwrap_or_default();
            }
        }
    }
    let Ok((mut text, mut panel)) = text.single_mut() else { return };
    let shade = if ed.on { Color::srgba(0.0, 0.0, 0.0, 0.72) } else { Color::NONE };
    if panel.0 != shade {
        panel.0 = shade;
    }
    if !ed.on {
        if !text.0.is_empty() {
            text.0.clear();
        }
        return;
    }
    let Ok(win) = window.single() else { return };
    let half = views.half[0];
    // World units per screen pixel.
    let unit = 2.0 * half.y / win.height().max(1.0);
    let cursor = win.cursor_position().map(|p| ed.cam + Vec2::new(p.x - win.width() / 2.0, win.height() / 2.0 - p.y) * unit);
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let snap_on = ed.snap;
    let snap = move |v: Vec2| if snap_on { (v / 10.0).round() * 10.0 } else { v };
    let mut rebuild = false;

    // The view.
    if mouse.pressed(MouseButton::Right) || mouse.pressed(MouseButton::Middle) {
        ed.cam -= Vec2::new(motion.delta.x, -motion.delta.y) * unit;
    }
    if !ctrl {
        let k = |a: KeyCode, b: KeyCode| (keys.pressed(a) || keys.pressed(b)) as i8 as f32;
        let dir = Vec2::new(k(KeyCode::ArrowRight, KeyCode::KeyD) - k(KeyCode::ArrowLeft, KeyCode::KeyA), k(KeyCode::ArrowUp, KeyCode::KeyW) - k(KeyCode::ArrowDown, KeyCode::KeyS));
        ed.cam += dir * 700.0 * ed.zoom * time.delta_secs();
    }
    if scroll.delta.y != 0.0 {
        ed.zoom = (ed.zoom * 0.88f32.powf(scroll.delta.y.clamp(-1.0, 1.0))).clamp(0.4, 6.0);
    }

    // Tools and keys.
    for (key, tool) in [(KeyCode::Digit1, Tool::Platform), (KeyCode::Digit2, Tool::Thorns), (KeyCode::Digit3, Tool::Tomato), (KeyCode::Digit4, Tool::Door), (KeyCode::Digit5, Tool::Area)] {
        if keys.just_pressed(key) {
            ed.tool = tool;
        }
    }
    if keys.just_pressed(KeyCode::KeyG) {
        ed.snap = !ed.snap;
    }
    if keys.just_pressed(KeyCode::Escape) {
        ed.sel = None;
    }
    if keys.just_pressed(KeyCode::Tab) {
        let next = |list: &[String], now: &str| -> Option<String> {
            if list.is_empty() {
                return None;
            }
            let at = list.iter().position(|n| n == now).map(|i| i + 1).unwrap_or(0);
            Some(list[at % list.len()].clone())
        };
        let kinds: Vec<String> = g.tuning.enemies.keys().cloned().collect();
        match ed.sel {
            Some(Item::Tomato(i)) if i < g.world.level.enemies.len() => {
                if let Some(k) = next(&kinds, &g.world.level.enemies[i].kind) {
                    ed.undo.push((g.world.level.clone(), story.file.clone()));
                    g.world.level.enemies[i].kind = k.clone();
                    ed.kind = k;
                    rebuild = true;
                }
            }
            Some(Item::Door(i)) if i < g.world.level.exits.len() => {
                let others: Vec<String> = ed.levels.iter().filter(|n| **n != g.level).cloned().collect();
                if let Some(to) = next(&others, &g.world.level.exits[i].to) {
                    ed.undo.push((g.world.level.clone(), story.file.clone()));
                    g.world.level.exits[i].to = to;
                    rebuild = true;
                }
            }
            _ => {
                if let Some(k) = next(&kinds, &ed.kind) {
                    ed.kind = k;
                }
            }
        }
    }
    if keys.just_pressed(KeyCode::Delete) || keys.just_pressed(KeyCode::Backspace) {
        if let Some(sel) = ed.sel {
            let before = (g.world.level.clone(), story.file.clone());
            let level = &mut g.world.level;
            let gone = match sel {
                Item::Solid(i) if i < level.solids.len() => {
                    level.solids.remove(i);
                    true
                }
                Item::Thorn(i) if i < level.hazards.len() => {
                    level.hazards.remove(i);
                    true
                }
                Item::Door(i) if i < level.exits.len() => {
                    level.exits.remove(i);
                    true
                }
                Item::Tomato(i) if i < level.enemies.len() => {
                    level.enemies.remove(i);
                    true
                }
                Item::Area(i) if i < story.file.scenes.len() => {
                    let s = &story.file.scenes[i];
                    if s.lines.is_empty() && s.choices.is_empty() {
                        story.file.scenes.remove(i);
                        true
                    } else {
                        ed.note = format!("Scene \"{}\" has lines. Remove it in story/{}.ron.", s.id, g.level);
                        false
                    }
                }
                Item::Start(_) => {
                    ed.note = "Start points cannot be removed.".into();
                    false
                }
                _ => false,
            };
            if gone {
                ed.undo.push(before);
                ed.sel = None;
                rebuild = true;
            }
        }
    }
    if ctrl && keys.just_pressed(KeyCode::KeyZ) {
        if let Some((level, file)) = ed.undo.pop() {
            g.world.level = level;
            story.file = file;
            ed.sel = None;
            ed.drag = None;
            rebuild = true;
        }
    }
    if ctrl && keys.just_pressed(KeyCode::KeyS) {
        match save(g, &mut story) {
            Ok(()) => {
                ed.unsaved = false;
                ed.note = "Saved.".into();
            }
            Err(e) => ed.note = format!("SAVE FAILED  {e}"),
        }
    }
    if keys.just_pressed(KeyCode::KeyP) {
        play_at = cursor;
        leave = play_at.is_some();
    }

    // The mouse.
    if let Some(c) = cursor {
        if mouse.just_pressed(MouseButton::Left) {
            ed.undo.push((g.world.level.clone(), story.file.clone()));
            ed.touched = false;
            ed.note.clear();
            let level = &mut g.world.level;
            let mut started = false;
            if let Some(r) = ed.sel.and_then(|s| rect_of(s, level, &mut story.file).map(|r| *r)) {
                for (cx, cy) in [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0), (1.0, 1.0)] {
                    if Vec2::new(r.0 + r.2 * cx, r.1 + r.3 * cy).distance(c) < 10.0 * unit {
                        ed.drag = Some(Drag::Size { fixed: Vec2::new(r.0 + r.2 * (1.0 - cx), r.1 + r.3 * (1.0 - cy)) });
                        started = true;
                    }
                }
            }
            if !started {
                match pick(c, unit, ed.tool, level, &story.file) {
                    Some(item) => {
                        let at = match rect_of(item, level, &mut story.file) {
                            Some(r) => Vec2::new(r.0, r.1),
                            None => point_of(item, level).map(|(x, y)| Vec2::new(*x, *y)).unwrap_or(c),
                        };
                        ed.sel = Some(item);
                        ed.drag = Some(Drag::Move { grab: c - at });
                    }
                    None => {
                        let p = snap(c);
                        let r = Rect(p.x, p.y, 0.0, 0.0);
                        ed.sel = Some(match ed.tool {
                            Tool::Platform => {
                                level.solids.push(r);
                                Item::Solid(level.solids.len() - 1)
                            }
                            Tool::Thorns => {
                                level.hazards.push(r);
                                Item::Thorn(level.hazards.len() - 1)
                            }
                            Tool::Door => {
                                let to = ed.levels.iter().find(|n| **n != g.level).cloned().unwrap_or_default();
                                level.exits.push(sim::Exit { rect: r, to });
                                Item::Door(level.exits.len() - 1)
                            }
                            Tool::Area => {
                                let mut n = story.file.scenes.len() + 1;
                                while story.file.scenes.iter().any(|s| s.id == format!("area_{n}")) {
                                    n += 1;
                                }
                                story.file.scenes.push(Scene { id: format!("area_{n}"), trigger: Trigger::Enter(r), requires: vec![], set: vec![], lines: vec![], choices: vec![] });
                                Item::Area(story.file.scenes.len() - 1)
                            }
                            Tool::Tomato => {
                                level.enemies.push(sim::EnemySpawn { kind: ed.kind.clone(), x: p.x, y: p.y });
                                ed.touched = true;
                                Item::Tomato(level.enemies.len() - 1)
                            }
                        });
                        ed.drag = Some(if ed.tool == Tool::Tomato { Drag::Move { grab: Vec2::ZERO } } else { Drag::Size { fixed: p } });
                    }
                }
            }
        }
        if mouse.pressed(MouseButton::Left) {
            if let (Some(sel), Some(drag)) = (ed.sel, &ed.drag) {
                let level = &mut g.world.level;
                match *drag {
                    Drag::Move { grab } => {
                        let to = snap(c - grab);
                        if let Some(r) = rect_of(sel, level, &mut story.file) {
                            if r.0 != to.x || r.1 != to.y {
                                (r.0, r.1) = (to.x, to.y);
                                ed.touched = true;
                            }
                        } else if let Some((x, y)) = point_of(sel, level) {
                            if *x != to.x || *y != to.y {
                                (*x, *y) = (to.x, to.y);
                                ed.touched = true;
                            }
                        }
                    }
                    Drag::Size { fixed } => {
                        let p = snap(c);
                        if let Some(r) = rect_of(sel, level, &mut story.file) {
                            let new = Rect(fixed.x.min(p.x), fixed.y.min(p.y), (fixed.x - p.x).abs(), (fixed.y - p.y).abs());
                            if (new.0, new.1, new.2, new.3) != (r.0, r.1, r.2, r.3) {
                                *r = new;
                                ed.touched = true;
                            }
                        }
                    }
                }
            }
        }
    }
    if mouse.just_released(MouseButton::Left) && ed.drag.is_some() {
        let sized = matches!(ed.drag, Some(Drag::Size { .. }));
        ed.drag = None;
        let tiny = sized && ed.sel.and_then(|s| rect_of(s, &mut g.world.level, &mut story.file).map(|r| r.2 < 10.0 || r.3 < 10.0)).unwrap_or(false);
        if tiny || !ed.touched {
            // Nothing useful happened: put things back as they were.
            if let Some((level, file)) = ed.undo.pop() {
                if tiny {
                    g.world.level = level;
                    story.file = file;
                    ed.sel = None;
                }
            }
        } else {
            rebuild = true;
        }
    }
    if ed.undo.len() > 200 {
        ed.undo.remove(0);
    }
    if rebuild {
        g.world = SimWorld::new(g.world.level.clone(), &g.tuning, g.players);
        g.geo_dirty = true;
        ed.unsaved = true;
    }
    if leave {
        ed.on = false;
        ed.drag = None;
        let mut level = g.world.level.clone();
        let starts = level.spawns.clone();
        if let Some(p) = play_at {
            level.spawns = vec![(p.x, p.y), (p.x + 50.0, p.y)];
        }
        g.world = settled(level, &g.tuning, g.players);
        g.world.level.spawns = starts;
        g.geo_dirty = true;
    }
    for (vc, mut proj) in &mut cams {
        let want = if ed.on && vc.0 == 0 { ed.zoom } else { 1.0 };
        if let Projection::Orthographic(o) = &mut *proj {
            if o.scale != want {
                o.scale = want;
            }
        }
    }
    if !ed.on {
        text.0.clear();
        panel.0 = Color::NONE;
        return;
    }

    // Outlines over the level art.
    let level = &g.world.level;
    let b = level.bounds;
    gizmos.rect_2d(Vec2::new(b.0 + b.2 / 2.0, b.1 + b.3 / 2.0), Vec2::new(b.2, b.3), Color::srgba(1.0, 1.0, 1.0, 0.25));
    for (item, r) in rects(level, &story.file) {
        let picked = ed.sel == Some(item);
        let color = match item {
            Item::Solid(_) => Color::srgb(0.75, 0.8, 0.9),
            Item::Thorn(_) => Color::srgb(1.0, 0.25, 0.25),
            Item::Door(_) => Color::srgb(1.0, 0.85, 0.2),
            _ => Color::srgb(0.3, 0.9, 1.0),
        };
        let color = if picked { Color::WHITE } else { color };
        gizmos.rect_2d(Vec2::new(r.0 + r.2 / 2.0, r.1 + r.3 / 2.0), Vec2::new(r.2, r.3), color);
        if matches!(item, Item::Door(_) | Item::Area(_)) {
            gizmos.line_2d(Vec2::new(r.0, r.1), Vec2::new(r.0 + r.2, r.1 + r.3), color.with_alpha(0.35));
        }
        if picked {
            for (cx, cy) in [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0), (1.0, 1.0)] {
                gizmos.rect_2d(Vec2::new(r.0 + r.2 * cx, r.1 + r.3 * cy), Vec2::splat(9.0 * unit), Color::WHITE);
            }
        }
    }
    for (i, e) in level.enemies.iter().enumerate() {
        let color = if ed.sel == Some(Item::Tomato(i)) { Color::WHITE } else { Color::srgb(1.0, 0.55, 0.15) };
        gizmos.circle_2d(Vec2::new(e.x, e.y), 26.0, color);
    }
    for (i, s) in level.spawns.iter().enumerate() {
        let color = if ed.sel == Some(Item::Start(i)) { Color::WHITE } else { Color::srgb(0.3, 1.0, 0.4) };
        let p = Vec2::new(s.0, s.1);
        gizmos.circle_2d(p, 24.0, color);
        gizmos.line_2d(p - Vec2::Y * 34.0, p + Vec2::Y * 34.0, color);
    }

    let tool = match ed.tool {
        Tool::Platform => "Platform",
        Tool::Thorns => "Thorns",
        Tool::Tomato => "Tomato",
        Tool::Door => "Door",
        Tool::Area => "Story area",
    };
    let size = |item: Item| rects(level, &story.file).into_iter().find(|(i, _)| *i == item).map(|(_, r)| format!("{} x {} at {}, {}", r.2, r.3, r.0, r.1)).unwrap_or_default();
    let sel = match ed.sel {
        None => "nothing".to_string(),
        Some(Item::Solid(i)) => format!("platform  {}", size(Item::Solid(i))),
        Some(Item::Thorn(i)) => format!("thorns  {}", size(Item::Thorn(i))),
        Some(Item::Door(i)) => format!("door to \"{}\"  {}   (Tab changes where it leads)", level.exits.get(i).map(|e| e.to.as_str()).unwrap_or(""), size(Item::Door(i))),
        Some(Item::Area(i)) => format!("story area, scene \"{}\"  {}", story.file.scenes.get(i).map(|s| s.id.as_str()).unwrap_or(""), size(Item::Area(i))),
        Some(Item::Tomato(i)) => format!("tomato \"{}\"   (Tab changes the kind)", level.enemies.get(i).map(|e| e.kind.as_str()).unwrap_or("")),
        Some(Item::Start(i)) => format!("start point of player {}", i + 1),
    };
    let want = format!(
        "LEVEL EDITOR   {}{}\nTool: {tool}      1 Platform   2 Thorns   3 Tomato ({})   4 Door   5 Story area\nSelected: {sel}\nDrag empty space: new    Drag a thing: move    Drag a corner: resize    Delete: remove    Tab: tomato kind or door target\nRight mouse or arrows: move the view    Wheel: zoom    G: snap {}    Ctrl+Z: undo    Ctrl+S: save\nP: play from the mouse    F1: play from the start\n{}",
        g.level,
        if ed.unsaved { "   NOT SAVED" } else { "" },
        ed.kind,
        if ed.snap { "on" } else { "off" },
        ed.note,
    );
    if text.0 != want {
        text.0 = want;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saved_files_load_again() {
        for name in ["rootway", "glasshouse", "cannery"] {
            let level = sim::load_level(&std::fs::read_to_string(format!("assets/levels/{name}.ron")).unwrap()).unwrap();
            let again = sim::load_level(&level_text(&level).unwrap()).unwrap();
            assert_eq!(format!("{level:?}"), format!("{again:?}"));
            let story: StoryFile = ron::from_str(&std::fs::read_to_string(format!("assets/story/{name}.ron")).unwrap()).unwrap();
            let back: StoryFile = ron::from_str(&story_text(&story).unwrap()).unwrap();
            assert_eq!(story_text(&story).unwrap(), story_text(&back).unwrap());
            assert_eq!(story.scenes.len(), back.scenes.len());
        }
    }
}
