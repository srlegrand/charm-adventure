//! The story as it is kept on disk, shared by the game and the story editor.
//! The editor (charm_story) writes assets/story/<level>.ron; the game reads it and plays it.
use crate::sim::{self, Rect};
use serde::{Deserialize, Serialize};

pub const FIRST_LEVEL: &str = "rootway";

/// Speech, loaded from assets/story/<level>.ron. This is the data the story editor will write.
#[derive(Deserialize, Serialize, Default, Clone)]
pub struct StoryFile {
    pub scenes: Vec<Scene>,
}
#[derive(Deserialize, Serialize, Clone)]
pub struct Scene {
    pub id: String,
    pub trigger: Trigger,
    /// Flags that must all be set for this scene to play.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requires: Vec<String>,
    /// Flags set when it plays.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub set: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lines: Vec<Line>,
    /// Offered after the lines; the game waits for an answer.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub choices: Vec<Choice>,
}
/// One answer to a choice: it can set flags, jump to another scene, or leave for another level.
#[derive(Deserialize, Serialize, Clone)]
pub struct Choice {
    pub text: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub set: Vec<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub goto: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub level: String,
}
#[derive(Deserialize, Serialize, Clone)]
pub enum Trigger {
    LevelStart,
    Enter(Rect),
    /// Only reached from a choice.
    Manual,
}
#[derive(Deserialize, Serialize, Clone)]
pub struct Line {
    pub who: String,
    pub text: String,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub secs: f32,
    /// Something that happens when this line comes up: "simon_kneels", "boss_claps". It lasts until the level is left.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub act: String,
}
pub fn is_zero(v: &f32) -> bool {
    *v == 0.0
}

/// Breaks a line of speech into rows short enough for a bubble.
pub fn wrap(text: &str, width: usize) -> Vec<String> {
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


pub fn level_text(level: &sim::Level) -> Result<String, String> {
    let text = ron::ser::to_string_pretty(level, ron::ser::PrettyConfig::default().depth_limit(2)).map_err(|e| e.to_string())?;
    Ok(format!("// Rects are (x, y, width, height) from the bottom-left, Y up.\n{text}\n"))
}

pub fn story_text(story: &StoryFile) -> Result<String, String> {
    let text = ron::ser::to_string_pretty(story, ron::ser::PrettyConfig::default().depth_limit(4)).map_err(|e| e.to_string())?;
    Ok(format!("// Written by the story editor (charm_story).\n{text}\n"))
}

/// The editor asks the running game to jump to a scene by writing this to assets/story/play_request.ron.
#[derive(Deserialize, Serialize, Clone)]
pub struct PlayRequest {
    pub level: String,
    pub scene: String,
}
