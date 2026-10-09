//! The story editor, inside the game. F1 opens and closes it.
//! Every level has a page of scenes. A scene is a card: when it starts, what is said, and the choices that
//! lead on to another scene or another level. Everything is written to assets/story as you go.
use crate::*;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::text::LineBreak;

const CH: f32 = 9.0;
const RH: f32 = 22.0;
const CW: f32 = 440.0;
const GAP_X: f32 = 90.0;
const GAP_Y: f32 = 28.0;
const TOP: f32 = 104.0;
const BOTTOM: f32 = 26.0;

const ACTS: [(&str, &str); 3] = [("", "no action"), ("simon_kneels", "Simon kneels"), ("boss_claps", "Boss claps")];

#[derive(Clone)]
struct Doc {
    file: String,
    name: String,
    story: StoryFile,
    /// Where the door at the end of the level leads. None: the level has no door.
    door: Option<String>,
}

#[derive(Clone, PartialEq)]
enum Field {
    Id(usize),
    Needs(usize),
    Sets(usize),
    Who(usize, usize),
    Text(usize, usize),
    Choice(usize, usize),
    ChoiceSets(usize, usize),
}

#[derive(Clone)]
enum Change {
    Trigger(usize, u8),
    LineAct(usize, usize, String),
    /// Scene, choice, scene to go to, level to go to.
    Target(usize, usize, String, String),
    Door(String),
}

#[derive(Clone)]
enum Act {
    Tab(usize),
    Edit(Field),
    NewScene,
    DelScene(usize),
    Play(usize),
    TriggerMenu(usize),
    AreaHere(usize),
    AddLine(usize),
    DelLine(usize, usize),
    UpLine(usize, usize),
    SwapWho(usize, usize),
    ActMenu(usize, usize),
    AddChoice(usize),
    DelChoice(usize, usize),
    TargetMenu(usize, usize),
    DoorMenu,
    Pick(Change),
}

struct Widget {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    text: String,
    fg: Color,
    bg: Option<Color>,
    act: Option<Act>,
    /// Stays put when the page scrolls.
    fixed: bool,
}

#[derive(Resource, Default)]
pub struct Editor {
    pub on: bool,
    docs: Vec<Doc>,
    tab: usize,
    scroll: Vec2,
    focus: Option<(Field, Vec<char>, usize)>,
    menu: Option<(Vec2, Vec<(String, Change)>)>,
    widgets: Vec<Widget>,
    undo: Vec<Vec<Doc>>,
    arm_delete: Option<usize>,
    note: String,
    dirty: bool,
    size: Vec2,
}

#[derive(Component)]
pub struct EditorRoot;

pub fn setup(mut commands: Commands) {
    commands.spawn((
        EditorRoot,
        Node { position_type: PositionType::Absolute, left: Val::Px(0.0), top: Val::Px(0.0), width: Val::Percent(100.0), height: Val::Percent(100.0), overflow: Overflow::clip(), ..default() },
        BackgroundColor(Color::srgba(0.04, 0.04, 0.07, 0.97)),
        GlobalZIndex(10),
        Visibility::Hidden,
    ));
}

pub fn level_text(level: &sim::Level) -> Result<String, String> {
    let text = ron::ser::to_string_pretty(level, ron::ser::PrettyConfig::default().depth_limit(2)).map_err(|e| e.to_string())?;
    Ok(format!("// Rects are (x, y, width, height) from the bottom-left, Y up.\n{text}\n"))
}

pub fn story_text(story: &StoryFile) -> Result<String, String> {
    let text = ron::ser::to_string_pretty(story, ron::ser::PrettyConfig::default().depth_limit(4)).map_err(|e| e.to_string())?;
    Ok(format!("// Written by the story editor (F1 in the game).\n{text}\n"))
}

/// Every level and its story, in the order the doors lead through them.
fn load_docs(dir: &PathBuf) -> Vec<Doc> {
    let mut names: Vec<String> = std::fs::read_dir(dir.join("levels"))
        .map(|d| d.filter_map(|f| f.ok()).map(|f| f.path()).filter(|p| p.extension().is_some_and(|x| x == "ron")).filter_map(|p| p.file_stem().map(|s| s.to_string_lossy().to_string())).collect())
        .unwrap_or_default();
    names.sort();
    let mut docs: Vec<Doc> = Vec::new();
    for file in names {
        let Some(level) = std::fs::read_to_string(dir.join(format!("levels/{file}.ron"))).ok().and_then(|t| sim::load_level(&t).ok()) else { continue };
        let story = std::fs::read_to_string(dir.join(format!("story/{file}.ron"))).ok().and_then(|t| ron::from_str::<StoryFile>(&t).ok()).unwrap_or_default();
        docs.push(Doc { name: level.name.clone(), door: level.exits.first().map(|e| e.to.clone()), file, story });
    }
    let mut order: Vec<Doc> = Vec::new();
    let mut next = Some(FIRST_LEVEL.to_string());
    while let Some(file) = next.take() {
        let Some(i) = docs.iter().position(|d| d.file == file) else { break };
        let doc = docs.remove(i);
        next = doc.door.clone();
        order.push(doc);
    }
    order.extend(docs);
    order
}

fn save(doc: &Doc, g: &mut Game, story: &mut Story, door_too: bool) -> Result<(), String> {
    let path = g.dir.join(format!("story/{}.ron", doc.file));
    if !doc.story.scenes.is_empty() || path.exists() {
        std::fs::create_dir_all(g.dir.join("story")).ok();
        std::fs::write(&path, story_text(&doc.story)?).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    if doc.file == g.level {
        story.file = doc.story.clone();
        story.stamp = std::fs::metadata(&path).and_then(|m| m.modified()).ok();
    }
    if door_too {
        if let Some(to) = &doc.door {
            let path = g.dir.join(format!("levels/{}.ron", doc.file));
            let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            let mut level = sim::load_level(&text)?;
            if let Some(e) = level.exits.first_mut() {
                e.to = to.clone();
            }
            std::fs::write(&path, level_text(&level)?).map_err(|e| format!("{}: {e}", path.display()))?;
            if doc.file == g.level {
                if let Some(e) = g.world.level.exits.first_mut() {
                    e.to = to.clone();
                }
                g.stamps = stamps(&g.dir, &g.level);
            }
        }
    }
    Ok(())
}

fn flags(text: &str) -> Vec<String> {
    text.split(',').map(|f| f.trim().to_string()).filter(|f| !f.is_empty()).collect()
}

fn field_value(doc: &Doc, f: &Field) -> String {
    let s = &doc.story.scenes;
    match *f {
        Field::Id(i) => s.get(i).map(|s| s.id.clone()),
        Field::Needs(i) => s.get(i).map(|s| s.requires.join(", ")),
        Field::Sets(i) => s.get(i).map(|s| s.set.join(", ")),
        Field::Who(i, l) => s.get(i).and_then(|s| s.lines.get(l)).map(|l| l.who.clone()),
        Field::Text(i, l) => s.get(i).and_then(|s| s.lines.get(l)).map(|l| l.text.clone()),
        Field::Choice(i, c) => s.get(i).and_then(|s| s.choices.get(c)).map(|c| c.text.clone()),
        Field::ChoiceSets(i, c) => s.get(i).and_then(|s| s.choices.get(c)).map(|c| c.set.join(", ")),
    }
    .unwrap_or_default()
}

fn set_field(doc: &mut Doc, f: &Field, text: String) -> Result<(), String> {
    let text = text.trim().to_string();
    let scenes = &mut doc.story.scenes;
    match *f {
        Field::Id(i) => {
            if text.is_empty() {
                return Err("A scene needs a name.".into());
            }
            if scenes.iter().enumerate().any(|(k, s)| k != i && s.id == text) {
                return Err(format!("There is already a scene called \"{text}\" here."));
            }
            let Some(old) = scenes.get(i).map(|s| s.id.clone()) else { return Ok(()) };
            for s in scenes.iter_mut() {
                for c in &mut s.choices {
                    if c.goto == old {
                        c.goto = text.clone();
                    }
                }
            }
            scenes[i].id = text;
        }
        Field::Needs(i) => {
            if let Some(s) = scenes.get_mut(i) {
                s.requires = flags(&text);
            }
        }
        Field::Sets(i) => {
            if let Some(s) = scenes.get_mut(i) {
                s.set = flags(&text);
            }
        }
        Field::Who(i, l) => {
            if let Some(l) = scenes.get_mut(i).and_then(|s| s.lines.get_mut(l)) {
                l.who = text;
            }
        }
        Field::Text(i, l) => {
            if let Some(l) = scenes.get_mut(i).and_then(|s| s.lines.get_mut(l)) {
                l.text = text;
            }
        }
        Field::Choice(i, c) => {
            if let Some(c) = scenes.get_mut(i).and_then(|s| s.choices.get_mut(c)) {
                c.text = text;
            }
        }
        Field::ChoiceSets(i, c) => {
            if let Some(c) = scenes.get_mut(i).and_then(|s| s.choices.get_mut(c)) {
                c.set = flags(&text);
            }
        }
    }
    Ok(())
}

const INK: Color = Color::srgb(0.93, 0.91, 0.86);
const DIM: Color = Color::srgb(0.55, 0.56, 0.62);
const CARD: Color = Color::srgb(0.11, 0.12, 0.17);
const HEAD: Color = Color::srgb(0.2, 0.23, 0.36);
const BOX: Color = Color::srgb(0.05, 0.05, 0.08);
const LIT: Color = Color::srgb(0.32, 0.25, 0.04);
const BTN: Color = Color::srgb(0.17, 0.27, 0.42);
const RED: Color = Color::srgb(0.45, 0.14, 0.14);
const GOLD: Color = Color::srgb(1.0, 0.76, 0.2);
const GREEN: Color = Color::srgb(0.16, 0.4, 0.26);

/// Which column each scene sits in: scenes nothing leads to on the left, what they lead to further right.
fn columns(story: &StoryFile) -> Vec<usize> {
    let n = story.scenes.len();
    let index = |id: &str| story.scenes.iter().position(|s| s.id == id);
    let mut col = vec![0usize; n];
    for _ in 0..n {
        for i in 0..n {
            for c in &story.scenes[i].choices {
                if let Some(t) = index(&c.goto) {
                    if t != i && col[t] < col[i] + 1 && col[i] + 1 < n {
                        col[t] = col[i] + 1;
                    }
                }
            }
        }
    }
    col
}

fn layout(ed: &Editor, playing: &str) -> Vec<Widget> {
    let mut out: Vec<Widget> = Vec::new();
    let Some(doc) = ed.docs.get(ed.tab) else { return out };
    let name_of = |file: &str| ed.docs.iter().find(|d| d.file == file).map(|d| d.name.clone()).unwrap_or_else(|| file.to_string());
    let shown = |f: Field, empty: &str| -> (String, Color, Color) {
        match &ed.focus {
            Some((at, buf, caret)) if *at == f => {
                let mut s: String = buf[..*caret].iter().collect();
                s.push('|');
                s.extend(buf[*caret..].iter());
                (s, INK, LIT)
            }
            _ => {
                let v = field_value(doc, &f);
                if v.is_empty() { (empty.to_string(), DIM, BOX) } else { (v, INK, BOX) }
            }
        }
    };

    // The page: one card per scene.
    let col = columns(&doc.story);
    let mut col_y: Vec<f32> = vec![TOP + 20.0; col.iter().max().map(|m| m + 1).unwrap_or(1)];
    let mut heads: Vec<(f32, f32)> = Vec::new();
    let mut links: Vec<(f32, f32, usize)> = Vec::new();
    for (i, scene) in doc.story.scenes.iter().enumerate() {
        let x = 30.0 + col[i] as f32 * (CW + GAP_X);
        let top = col_y[col[i]];
        let mut y = top;
        let back = out.len();
        out.push(Widget { x, y, w: CW, h: 0.0, text: String::new(), fg: INK, bg: Some(CARD), act: None, fixed: false });
        let put = |out: &mut Vec<Widget>, dx: f32, y: f32, w: f32, h: f32, text: String, fg: Color, bg: Option<Color>, act: Option<Act>| {
            out.push(Widget { x: x + dx, y, w, h, text, fg, bg, act, fixed: false });
        };
        // Name, play, delete.
        put(&mut out, 0.0, y, CW, RH + 8.0, String::new(), INK, Some(HEAD), None);
        let (t, fg, bg) = shown(Field::Id(i), "name");
        put(&mut out, 6.0, y + 4.0, 250.0, RH, t, fg, Some(bg), Some(Act::Edit(Field::Id(i))));
        put(&mut out, CW - 150.0, y + 4.0, 62.0, RH, " PLAY".into(), INK, Some(GREEN), Some(Act::Play(i)));
        let armed = ed.arm_delete == Some(i);
        put(&mut out, CW - 82.0, y + 4.0, 76.0, RH, if armed { " SURE?" } else { " DELETE" }.into(), INK, Some(RED), Some(Act::DelScene(i)));
        heads.push((x, y + RH / 2.0 + 4.0));
        y += RH + 14.0;
        // When it starts.
        let when = match scene.trigger {
            Trigger::LevelStart => "when the level starts".to_string(),
            Trigger::Enter(_) => "when a player walks into its area".to_string(),
            Trigger::Manual => "only from a choice".to_string(),
        };
        put(&mut out, 8.0, y, 70.0, RH, "Starts".into(), DIM, None, None);
        put(&mut out, 78.0, y, CW - 86.0, RH, when, INK, Some(BTN), Some(Act::TriggerMenu(i)));
        y += RH + 4.0;
        if let Trigger::Enter(r) = scene.trigger {
            put(&mut out, 8.0, y, 190.0, RH, format!("area at {:.0}, {:.0}", r.0, r.1), DIM, None, None);
            if doc.file == playing {
                put(&mut out, 200.0, y, CW - 208.0, RH, "put it where player 1 is".into(), INK, Some(BTN), Some(Act::AreaHere(i)));
            } else {
                put(&mut out, 200.0, y, CW - 208.0, RH, "(play this level to move it)".into(), DIM, None, None);
            }
            y += RH + 4.0;
        }
        for (label, field) in [("Only if", Field::Needs(i)), ("Sets", Field::Sets(i))] {
            put(&mut out, 8.0, y, 70.0, RH, label.into(), DIM, None, None);
            let (t, fg, bg) = shown(field.clone(), "flags, separated by commas");
            put(&mut out, 78.0, y, CW - 86.0, RH, t, fg, Some(bg), Some(Act::Edit(field)));
            y += RH + 4.0;
        }
        // What is said.
        y += 6.0;
        for (l, line) in scene.lines.iter().enumerate() {
            let (t, fg, bg) = shown(Field::Who(i, l), "who");
            put(&mut out, 8.0, y, 110.0, RH, t, fg, Some(bg), Some(Act::Edit(Field::Who(i, l))));
            put(&mut out, 122.0, y, 52.0, RH, "swap".into(), INK, Some(BTN), Some(Act::SwapWho(i, l)));
            let act = ACTS.iter().find(|a| a.0 == line.act).map(|a| a.1.to_string()).unwrap_or_else(|| line.act.clone());
            put(&mut out, 178.0, y, 150.0, RH, act, if line.act.is_empty() { DIM } else { GOLD }, Some(BTN), Some(Act::ActMenu(i, l)));
            put(&mut out, CW - 106.0, y, 36.0, RH, "up".into(), INK, Some(BTN), Some(Act::UpLine(i, l)));
            put(&mut out, CW - 66.0, y, 58.0, RH, "remove".into(), INK, Some(RED), Some(Act::DelLine(i, l)));
            y += RH + 2.0;
            let (t, fg, bg) = shown(Field::Text(i, l), "(says nothing: a silent moment)");
            let rows = wrap(&t, ((CW - 24.0) / CH) as usize);
            let h = rows.len().max(1) as f32 * 19.0 + 6.0;
            put(&mut out, 8.0, y, CW - 16.0, h, rows.join("\n"), fg, Some(bg), Some(Act::Edit(Field::Text(i, l))));
            y += h + 8.0;
        }
        put(&mut out, 8.0, y, 90.0, RH, "+ line".into(), INK, Some(BTN), Some(Act::AddLine(i)));
        y += RH + 12.0;
        // Where it can go.
        for (c, choice) in scene.choices.iter().enumerate() {
            let (t, fg, bg) = shown(Field::Choice(i, c), "what the player sees");
            put(&mut out, 8.0, y, 70.0, RH, "Choice".into(), GOLD, None, None);
            put(&mut out, 78.0, y, CW - 152.0, RH, t, fg, Some(bg), Some(Act::Edit(Field::Choice(i, c))));
            put(&mut out, CW - 66.0, y, 58.0, RH, "remove".into(), INK, Some(RED), Some(Act::DelChoice(i, c)));
            y += RH + 2.0;
            let target = if !choice.level.is_empty() {
                format!("to level: {}", name_of(&choice.level))
            } else if !choice.goto.is_empty() {
                format!("to scene: {}", choice.goto)
            } else {
                "carries on playing".to_string()
            };
            put(&mut out, 8.0, y, 70.0, RH, "  goes".into(), DIM, None, None);
            put(&mut out, 78.0, y, CW - 86.0, RH, target, INK, Some(if choice.level.is_empty() { BTN } else { GREEN }), Some(Act::TargetMenu(i, c)));
            if let Some(t) = doc.story.scenes.iter().position(|s| s.id == choice.goto) {
                if choice.level.is_empty() {
                    links.push((x + CW, y + RH / 2.0, t));
                }
            }
            y += RH + 2.0;
            let (t, fg, bg) = shown(Field::ChoiceSets(i, c), "flags, separated by commas");
            put(&mut out, 8.0, y, 70.0, RH, "  sets".into(), DIM, None, None);
            put(&mut out, 78.0, y, CW - 86.0, RH, t, fg, Some(bg), Some(Act::Edit(Field::ChoiceSets(i, c))));
            y += RH + 8.0;
        }
        put(&mut out, 8.0, y, 100.0, RH, "+ choice".into(), INK, Some(BTN), Some(Act::AddChoice(i)));
        y += RH + 10.0;
        out[back].h = y - top;
        col_y[col[i]] = y + GAP_Y;
    }
    if doc.story.scenes.is_empty() {
        out.push(Widget { x: 30.0, y: TOP + 30.0, w: 700.0, h: RH, text: "No scenes in this level yet. Press \"+ new scene\".".into(), fg: DIM, bg: None, act: None, fixed: false });
    }
    // Lines from a choice to the scene it leads to.
    for (k, (x1, y1, t)) in links.into_iter().enumerate() {
        let Some(&(tx, ty)) = heads.get(t) else { continue };
        let gx = x1 + 16.0 + (k % 6) as f32 * 9.0;
        let x2 = if tx > gx { tx } else { tx + CW };
        let mut seg = |x: f32, y: f32, w: f32, h: f32| out.push(Widget { x, y, w, h, text: String::new(), fg: INK, bg: Some(GOLD), act: None, fixed: false });
        seg(x1, y1 - 1.5, gx - x1 + 3.0, 3.0);
        seg(gx, y1.min(ty) - 1.5, 3.0, (y1 - ty).abs() + 3.0);
        seg(gx.min(x2), ty - 1.5, (gx - x2).abs() + 3.0, 3.0);
        seg(x2 - if x2 == tx { 9.0 } else { 0.0 }, ty - 5.0, 9.0, 10.0);
    }

    // The bars that stay put.
    let w = ed.size.x;
    let mut bar = |x: f32, y: f32, w: f32, h: f32, text: String, fg: Color, bg: Option<Color>, act: Option<Act>| out.push(Widget { x, y, w, h, text, fg, bg, act, fixed: true });
    bar(0.0, 0.0, w, TOP, String::new(), INK, Some(Color::srgb(0.07, 0.07, 0.11)), None);
    bar(12.0, 9.0, 70.0, RH, "STORY".into(), GOLD, None, None);
    let mut x = 84.0;
    for (i, d) in ed.docs.iter().enumerate() {
        let label = format!(" {} ({})", d.name, d.story.scenes.len());
        let tw = label.chars().count() as f32 * CH + 14.0;
        bar(x, 8.0, tw, RH + 2.0, label, INK, Some(if i == ed.tab { HEAD } else { BOX }), Some(Act::Tab(i)));
        x += tw + 6.0;
    }
    bar(12.0, 40.0, 130.0, RH, " + new scene".into(), INK, Some(BTN), Some(Act::NewScene));
    match &doc.door {
        Some(to) => {
            bar(160.0, 40.0, 264.0, RH, "The door at the end leads to".into(), DIM, None, None);
            bar(428.0, 40.0, 220.0, RH, if to.is_empty() { "nowhere".to_string() } else { name_of(to) }, INK, Some(GREEN), Some(Act::DoorMenu));
        }
        None => bar(160.0, 40.0, 500.0, RH, "This level has no door at the end.".into(), DIM, None, None),
    }
    let mut ways: Vec<String> = Vec::new();
    if let Some(to) = doc.door.as_ref().filter(|t| !t.is_empty()) {
        ways.push(format!("{} (door)", name_of(to)));
    }
    for s in &doc.story.scenes {
        for c in s.choices.iter().filter(|c| !c.level.is_empty()) {
            ways.push(format!("{} (choice \"{}\" in {})", name_of(&c.level), c.text, s.id));
        }
    }
    let mut from: Vec<String> = Vec::new();
    for d in ed.docs.iter().filter(|d| d.file != doc.file) {
        if d.door.as_deref() == Some(doc.file.as_str()) || d.story.scenes.iter().any(|s| s.choices.iter().any(|c| c.level == doc.file)) {
            from.push(d.name.clone());
        }
    }
    let line = format!(
        "From here the story goes to: {}.    You arrive here from: {}.",
        if ways.is_empty() { "nowhere, this is the end".to_string() } else { ways.join(", ") },
        if from.is_empty() { "nowhere, this is the start".to_string() } else { from.join(", ") }
    );
    bar(12.0, 72.0, w - 24.0, RH, line, INK, None, None);
    let h = ed.size.y;
    bar(0.0, h - BOTTOM, w, BOTTOM, String::new(), INK, Some(Color::srgb(0.07, 0.07, 0.11)), None);
    let help = if ed.note.is_empty() {
        "Click any text to change it. Enter: done. Esc: cancel.   Wheel or arrows: scroll. Shift+wheel: sideways.   Ctrl+Z: undo.   Saved as you go.   F1: back to the game".to_string()
    } else {
        ed.note.clone()
    };
    bar(12.0, h - BOTTOM + 3.0, w - 24.0, RH, help, if ed.note.is_empty() { DIM } else { GOLD }, None, None);
    // A list to pick from.
    if let Some((at, options)) = &ed.menu {
        let mw = options.iter().map(|o| o.0.chars().count()).max().unwrap_or(10) as f32 * CH + 20.0;
        let mh = options.len() as f32 * (RH + 2.0) + 8.0;
        let mx = at.x.min(w - mw - 6.0).max(0.0);
        let my = at.y.min(h - mh - 6.0).max(0.0);
        bar(mx - 3.0, my - 3.0, mw + 6.0, mh + 6.0, String::new(), INK, Some(GOLD), None);
        bar(mx, my, mw, mh, String::new(), INK, Some(Color::srgb(0.1, 0.12, 0.2)), None);
        for (k, (label, change)) in options.iter().enumerate() {
            bar(mx + 4.0, my + 4.0 + k as f32 * (RH + 2.0), mw - 8.0, RH, label.clone(), INK, Some(BTN), Some(Act::Pick(change.clone())));
        }
    }
    out
}

#[allow(clippy::too_many_arguments)]
pub fn edit(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    scroll: Res<AccumulatedMouseScroll>,
    time: Res<Time>,
    mut typed: MessageReader<KeyboardInput>,
    window: Query<&Window, With<PrimaryWindow>>,
    chooser: Res<Chooser>,
    mut ed: ResMut<Editor>,
    mut game: ResMut<Game>,
    mut story: ResMut<Story>,
    mut root: Query<(Entity, &mut Visibility), With<EditorRoot>>,
) {
    let ed = &mut *ed;
    let g = &mut *game;
    let Ok((root, mut vis)) = root.single_mut() else { return };
    let Ok(win) = window.single() else { return };
    let events: Vec<KeyboardInput> = typed.read().filter(|e| e.state.is_pressed()).cloned().collect();
    if keys.just_pressed(KeyCode::F1) && !chooser.0 {
        if ed.on {
            commit(ed, g, &mut story);
            ed.on = false;
            ed.menu = None;
            *vis = Visibility::Hidden;
            commands.entity(root).despawn_children();
            return;
        }
        ed.on = true;
        ed.docs = load_docs(&g.dir);
        ed.tab = ed.docs.iter().position(|d| d.file == g.level).unwrap_or(0);
        ed.scroll = Vec2::ZERO;
        ed.focus = None;
        ed.menu = None;
        ed.arm_delete = None;
        ed.undo.clear();
        ed.note.clear();
        ed.dirty = true;
        *vis = Visibility::Visible;
    }
    if !ed.on {
        return;
    }
    let size = Vec2::new(win.width(), win.height());
    if size != ed.size {
        ed.size = size;
        ed.dirty = true;
    }
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    let mut play: Option<usize> = None;

    // Typing.
    for ev in &events {
        ed.dirty = true;
        if ed.focus.is_some() {
            let mut done = false;
            let mut cancel = false;
            if let Some((_, buf, caret)) = ed.focus.as_mut() {
                match &ev.logical_key {
                    Key::Enter | Key::Tab => done = true,
                    Key::Escape => cancel = true,
                    Key::Backspace => {
                        if *caret > 0 {
                            *caret -= 1;
                            buf.remove(*caret);
                        }
                    }
                    Key::Delete => {
                        if *caret < buf.len() {
                            buf.remove(*caret);
                        }
                    }
                    Key::ArrowLeft => *caret = caret.saturating_sub(1),
                    Key::ArrowRight => *caret = (*caret + 1).min(buf.len()),
                    Key::Home => *caret = 0,
                    Key::End => *caret = buf.len(),
                    _ => {
                        if !ctrl {
                            for ch in ev.text.as_ref().map(|t| t.chars().collect::<Vec<char>>()).unwrap_or_default() {
                                if !ch.is_control() {
                                    buf.insert(*caret, ch);
                                    *caret += 1;
                                }
                            }
                        }
                    }
                }
            }
            if done {
                commit(ed, g, &mut story);
            }
            if cancel {
                ed.focus = None;
            }
        } else {
            match ev.key_code {
                KeyCode::Escape => ed.menu = None,
                KeyCode::KeyZ if ctrl => {
                    if let Some(docs) = ed.undo.pop() {
                        ed.docs = docs;
                        ed.tab = ed.tab.min(ed.docs.len().saturating_sub(1));
                        let mut note = "Undone.".to_string();
                        for d in &ed.docs {
                            if let Err(e) = save(d, g, &mut story, true) {
                                note = format!("SAVE FAILED  {e}");
                            }
                        }
                        ed.note = note;
                    }
                }
                _ => {}
            }
        }
    }
    // Scrolling.
    if ed.focus.is_none() {
        let k = |a: KeyCode| keys.pressed(a) as i8 as f32;
        let dir = Vec2::new(k(KeyCode::ArrowRight) - k(KeyCode::ArrowLeft), k(KeyCode::ArrowDown) - k(KeyCode::ArrowUp));
        if dir != Vec2::ZERO {
            ed.scroll += dir * 900.0 * time.delta_secs();
            ed.dirty = true;
        }
    }
    if scroll.delta != Vec2::ZERO {
        let step = -scroll.delta.y.clamp(-1.0, 1.0) * 70.0;
        if shift {
            ed.scroll.x += step;
        } else {
            ed.scroll.y += step;
        }
        ed.scroll.x -= scroll.delta.x.clamp(-1.0, 1.0) * 70.0;
        ed.dirty = true;
    }
    ed.scroll = ed.scroll.max(Vec2::ZERO);

    // Clicking.
    if mouse.just_pressed(MouseButton::Left) {
        ed.dirty = true;
        if let Some(p) = win.cursor_position() {
            let hit = ed.widgets.iter().rev().find_map(|w| {
                let (x, y) = if w.fixed { (w.x, w.y) } else { (w.x - ed.scroll.x, w.y - ed.scroll.y) };
                let inside = p.x >= x && p.x < x + w.w && p.y >= y && p.y < y + w.h;
                let covered = !w.fixed && (p.y < TOP || p.y > ed.size.y - BOTTOM);
                if inside && !covered { w.act.clone() } else { None }
            });
            let had_menu = ed.menu.is_some();
            if !matches!(hit, Some(Act::Edit(_))) || had_menu {
                commit(ed, g, &mut story);
            }
            if !matches!(hit, Some(Act::DelScene(_))) {
                ed.arm_delete = None;
            }
            match hit {
                Some(Act::Pick(change)) => {
                    ed.menu = None;
                    change_doc(ed, g, &mut story, |doc| match &change {
                        Change::Trigger(i, kind) => {
                            if let Some(s) = doc.story.scenes.get_mut(*i) {
                                s.trigger = match (kind, &s.trigger) {
                                    (0, _) => Trigger::LevelStart,
                                    (1, Trigger::Enter(r)) => Trigger::Enter(*r),
                                    (1, _) => Trigger::Enter(Rect(0.0, 0.0, 300.0, 300.0)),
                                    _ => Trigger::Manual,
                                };
                            }
                        }
                        Change::LineAct(i, l, act) => {
                            if let Some(l) = doc.story.scenes.get_mut(*i).and_then(|s| s.lines.get_mut(*l)) {
                                l.act = act.clone();
                            }
                        }
                        Change::Target(i, c, goto, level) => {
                            if let Some(c) = doc.story.scenes.get_mut(*i).and_then(|s| s.choices.get_mut(*c)) {
                                c.goto = goto.clone();
                                c.level = level.clone();
                            }
                        }
                        Change::Door(to) => doc.door = Some(to.clone()),
                    });
                }
                _ if had_menu => ed.menu = None,
                Some(Act::Tab(i)) => {
                    ed.tab = i;
                    ed.scroll = Vec2::ZERO;
                }
                Some(Act::Edit(f)) => {
                    if ed.focus.as_ref().map(|x| &x.0) != Some(&f) {
                        commit(ed, g, &mut story);
                        if let Some(doc) = ed.docs.get(ed.tab) {
                            let buf: Vec<char> = field_value(doc, &f).chars().collect();
                            let n = buf.len();
                            ed.focus = Some((f, buf, n));
                        }
                    }
                }
                Some(Act::NewScene) => change_doc(ed, g, &mut story, |doc| {
                    let mut n = doc.story.scenes.len() + 1;
                    while doc.story.scenes.iter().any(|s| s.id == format!("scene {n}")) {
                        n += 1;
                    }
                    let trigger = if doc.story.scenes.is_empty() { Trigger::LevelStart } else { Trigger::Manual };
                    doc.story.scenes.push(Scene { id: format!("scene {n}"), trigger, requires: vec![], set: vec![], lines: vec![], choices: vec![] });
                }),
                Some(Act::DelScene(i)) => {
                    if ed.arm_delete == Some(i) {
                        ed.arm_delete = None;
                        change_doc(ed, g, &mut story, |doc| {
                            if i < doc.story.scenes.len() {
                                let id = doc.story.scenes.remove(i).id;
                                for s in &mut doc.story.scenes {
                                    for c in s.choices.iter_mut().filter(|c| c.goto == id) {
                                        c.goto.clear();
                                    }
                                }
                            }
                        });
                    } else {
                        ed.arm_delete = Some(i);
                    }
                }
                Some(Act::Play(i)) => play = Some(i),
                Some(Act::TriggerMenu(i)) => {
                    ed.menu = Some((p, vec![
                        ("when the level starts".into(), Change::Trigger(i, 0)),
                        ("when a player walks into its area".into(), Change::Trigger(i, 1)),
                        ("only from a choice".into(), Change::Trigger(i, 2)),
                    ]));
                }
                Some(Act::AreaHere(i)) => {
                    if let Some(pl) = g.world.players.first() {
                        let r = Rect(((pl.x - 150.0) / 10.0).round() * 10.0, ((pl.y - g.tuning.player.height / 2.0) / 10.0).round() * 10.0, 300.0, 300.0);
                        change_doc(ed, g, &mut story, |doc| {
                            if let Some(s) = doc.story.scenes.get_mut(i) {
                                s.trigger = Trigger::Enter(r);
                            }
                        });
                    }
                }
                Some(Act::AddLine(i)) => {
                    change_doc(ed, g, &mut story, |doc| {
                        if let Some(s) = doc.story.scenes.get_mut(i) {
                            let who = match s.lines.last().map(|l| l.who.to_lowercase()) {
                                Some(w) if w == "simon" => "charm",
                                _ => "simon",
                            };
                            s.lines.push(Line { who: who.into(), text: String::new(), secs: 0.0, act: String::new() });
                        }
                    });
                    if let Some(n) = ed.docs.get(ed.tab).and_then(|d| d.story.scenes.get(i)).map(|s| s.lines.len()) {
                        ed.focus = Some((Field::Text(i, n.saturating_sub(1)), Vec::new(), 0));
                    }
                }
                Some(Act::DelLine(i, l)) => change_doc(ed, g, &mut story, |doc| {
                    if let Some(s) = doc.story.scenes.get_mut(i) {
                        if l < s.lines.len() {
                            s.lines.remove(l);
                        }
                    }
                }),
                Some(Act::UpLine(i, l)) => change_doc(ed, g, &mut story, |doc| {
                    if let Some(s) = doc.story.scenes.get_mut(i) {
                        if l > 0 && l < s.lines.len() {
                            s.lines.swap(l, l - 1);
                        }
                    }
                }),
                Some(Act::SwapWho(i, l)) => change_doc(ed, g, &mut story, |doc| {
                    if let Some(line) = doc.story.scenes.get_mut(i).and_then(|s| s.lines.get_mut(l)) {
                        line.who = if line.who.to_lowercase() == "simon" { "charm" } else { "simon" }.into();
                    }
                }),
                Some(Act::ActMenu(i, l)) => {
                    ed.menu = Some((p, ACTS.iter().map(|a| (a.1.to_string(), Change::LineAct(i, l, a.0.to_string()))).collect()));
                }
                Some(Act::AddChoice(i)) => {
                    change_doc(ed, g, &mut story, |doc| {
                        if let Some(s) = doc.story.scenes.get_mut(i) {
                            s.choices.push(Choice { text: String::new(), set: vec![], goto: String::new(), level: String::new() });
                        }
                    });
                    if let Some(n) = ed.docs.get(ed.tab).and_then(|d| d.story.scenes.get(i)).map(|s| s.choices.len()) {
                        ed.focus = Some((Field::Choice(i, n.saturating_sub(1)), Vec::new(), 0));
                    }
                }
                Some(Act::DelChoice(i, c)) => change_doc(ed, g, &mut story, |doc| {
                    if let Some(s) = doc.story.scenes.get_mut(i) {
                        if c < s.choices.len() {
                            s.choices.remove(c);
                        }
                    }
                }),
                Some(Act::TargetMenu(i, c)) => {
                    let mut options = vec![("carries on playing".to_string(), Change::Target(i, c, String::new(), String::new()))];
                    if let Some(doc) = ed.docs.get(ed.tab) {
                        for (k, s) in doc.story.scenes.iter().enumerate() {
                            if k != i {
                                options.push((format!("to scene: {}", s.id), Change::Target(i, c, s.id.clone(), String::new())));
                            }
                        }
                    }
                    for d in &ed.docs {
                        options.push((format!("to level: {}", d.name), Change::Target(i, c, String::new(), d.file.clone())));
                    }
                    ed.menu = Some((p, options));
                }
                Some(Act::DoorMenu) => {
                    let here = ed.docs.get(ed.tab).map(|d| d.file.clone()).unwrap_or_default();
                    let mut options = vec![("nowhere".to_string(), Change::Door(String::new()))];
                    options.extend(ed.docs.iter().filter(|d| d.file != here).map(|d| (d.name.clone(), Change::Door(d.file.clone()))));
                    ed.menu = Some((p, options));
                }
                None => {}
            }
        }
    }

    // Play from a scene: go to its level, stand in its area if it has one, and start it.
    if let Some(i) = play {
        commit(ed, g, &mut story);
        if let Some(doc) = ed.docs.get(ed.tab) {
            if let Some(scene) = doc.story.scenes.get(i) {
                g.level = doc.file.clone();
                restart(g);
                g.stamps = stamps(&g.dir, &g.level);
                if let Trigger::Enter(r) = scene.trigger {
                    let mut level = g.world.level.clone();
                    let starts = level.spawns.clone();
                    let at = (r.0 + r.2 / 2.0, r.1 + g.tuning.player.height / 2.0 + 4.0);
                    level.spawns = vec![at, (at.0 + 50.0, at.1)];
                    g.world = settled(level, &g.tuning, g.players);
                    g.world.level.spawns = starts;
                }
                let path = g.dir.join(format!("story/{}.ron", doc.file));
                *story = Story { file: doc.story.clone(), stamp: std::fs::metadata(&path).and_then(|m| m.modified()).ok(), level: doc.file.clone(), flags: std::mem::take(&mut story.flags), start_at: Some(scene.id.clone()), ..default() };
                ed.on = false;
                ed.menu = None;
                *vis = Visibility::Hidden;
                commands.entity(root).despawn_children();
                return;
            }
        }
    }

    if !ed.dirty {
        return;
    }
    ed.dirty = false;
    ed.widgets = layout(ed, &g.level);
    commands.entity(root).despawn_children();
    let (scroll, size) = (ed.scroll, ed.size);
    commands.entity(root).with_children(|c| {
        for w in &ed.widgets {
            let (x, y) = if w.fixed { (w.x, w.y) } else { (w.x - scroll.x, w.y - scroll.y) };
            if x > size.x || y > size.y || x + w.w < 0.0 || y + w.h < 0.0 {
                continue;
            }
            let mut e = c.spawn(Node {
                position_type: PositionType::Absolute,
                left: Val::Px(x),
                top: Val::Px(y),
                width: Val::Px(w.w),
                height: Val::Px(w.h),
                padding: UiRect::axes(Val::Px(4.0), Val::Px(2.0)),
                overflow: Overflow::clip(),
                ..default()
            });
            if let Some(bg) = w.bg {
                e.insert(BackgroundColor(bg));
            }
            if !w.text.is_empty() {
                e.insert((Text::new(w.text.clone()), TextFont { font_size: bevy::text::FontSize::Px(15.0), ..default() }, TextColor(w.fg), TextLayout::new(Justify::Left, LineBreak::NoWrap)));
            }
        }
    });
}

/// Finishes the text being typed.
fn commit(ed: &mut Editor, g: &mut Game, story: &mut Story) {
    let Some((field, buf, _)) = ed.focus.take() else { return };
    let text: String = buf.into_iter().collect();
    let Some(doc) = ed.docs.get(ed.tab) else { return };
    if field_value(doc, &field) == text.trim() {
        return;
    }
    let before = ed.docs.clone();
    let doc = &mut ed.docs[ed.tab];
    match set_field(doc, &field, text) {
        Ok(()) => {
            ed.undo.push(before);
            ed.note = match save(doc, g, story, false) {
                Ok(()) => String::new(),
                Err(e) => format!("SAVE FAILED  {e}"),
            };
        }
        Err(e) => ed.note = e,
    }
}

/// Makes one change to the open level's story and saves it.
fn change_doc(ed: &mut Editor, g: &mut Game, story: &mut Story, change: impl FnOnce(&mut Doc)) {
    commit(ed, g, story);
    let before = ed.docs.clone();
    let Some(doc) = ed.docs.get_mut(ed.tab) else { return };
    let door = doc.door.clone();
    change(doc);
    let door_changed = doc.door != door;
    ed.undo.push(before);
    if ed.undo.len() > 100 {
        ed.undo.remove(0);
    }
    ed.note = match save(doc, g, story, door_changed) {
        Ok(()) => String::new(),
        Err(e) => format!("SAVE FAILED  {e}"),
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saved_files_load_again() {
        let docs = load_docs(&PathBuf::from("assets"));
        let order: Vec<&str> = docs.iter().map(|d| d.file.as_str()).collect();
        assert_eq!(order, ["rootway", "australia", "newzealand", "france", "uk"]);
        for doc in &docs {
            let level = sim::load_level(&std::fs::read_to_string(format!("assets/levels/{}.ron", doc.file)).unwrap()).unwrap();
            let again = sim::load_level(&level_text(&level).unwrap()).unwrap();
            assert_eq!(format!("{level:?}"), format!("{again:?}"));
            let back: StoryFile = ron::from_str(&story_text(&doc.story).unwrap()).unwrap();
            assert_eq!(story_text(&doc.story).unwrap(), story_text(&back).unwrap());
        }
        let uk = docs.iter().find(|d| d.file == "uk").unwrap();
        assert_eq!(uk.story.scenes[0].lines[1].act, "boss_claps");
    }

    #[test]
    fn renaming_a_scene_keeps_choices_pointing_at_it() {
        let scene = |id: &str, goto: &str| Scene { id: id.into(), trigger: Trigger::Manual, requires: vec![], set: vec![], lines: vec![], choices: vec![Choice { text: "go".into(), set: vec![], goto: goto.into(), level: String::new() }] };
        let mut doc = Doc { file: "x".into(), name: "X".into(), story: StoryFile { scenes: vec![scene("a", "b"), scene("b", "")] }, door: None };
        set_field(&mut doc, &Field::Id(1), "ending".into()).unwrap();
        assert_eq!(doc.story.scenes[0].choices[0].goto, "ending");
        assert!(set_field(&mut doc, &Field::Id(1), "a".into()).is_err());
        assert_eq!(columns(&doc.story), vec![0, 1]);
    }
}
