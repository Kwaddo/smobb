use crate::speaker::Speaker;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub enum Step {
    Line(Speaker, String),
    Pause,
    Recap(String),
    Prompt(Vec<Branch>),
    Goto(String),
    Return,
    Enter(String),
    Quit,
    /// Reveals that Jaffar's red lines are hexadecimal. Until a scene plays this,
    /// his hex shows raw (gibberish); afterwards the engine auto-translates it.
    UnlockDecode,
    /// Fire a dramatic effect at this point in the stream.
    Effect(crate::effect::EffectKind),
}

#[derive(Debug, Clone, Deserialize)]
pub struct Branch {
    pub keys: Vec<String>,
    pub target: Target,
}

#[derive(Debug, Clone, Deserialize)]
pub enum Target {
    Scene(String),
    Steps(Vec<Step>),
}

#[derive(Debug, Clone, Deserialize)]
pub struct Scene {
    pub steps: Vec<Step>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Location {
    #[serde(default)]
    pub intro: Vec<Step>,
    #[serde(default)]
    pub prompt: Vec<Step>,
    pub commands: Vec<Branch>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Content {
    pub start: String,
    pub locations: HashMap<String, Location>,
    pub scenes: HashMap<String, Scene>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::speaker::Speaker;

    const SAMPLE: &str = r#"
Scene(steps: [
    Line(Ali, "hello"),
    Recap("clue one"),
    Pause,
    Prompt([
        Branch(keys: ["yes","y"], target: Scene("yes_path")),
        Branch(keys: [], target: Steps([ Line(Smobb, "DEFAULT") ])),
    ]),
    Return,
])
"#;

    #[test]
    fn parses_scene_ron() {
        let s: Scene = ron::from_str(SAMPLE).unwrap();
        assert_eq!(s.steps.len(), 5);
        match &s.steps[0] {
            Step::Line(sp, t) => {
                assert_eq!(*sp, Speaker::Ali);
                assert_eq!(t, "hello");
            }
            _ => panic!("expected Line"),
        }
        match &s.steps[3] {
            Step::Prompt(bs) => {
                assert_eq!(bs[0].keys, vec!["yes", "y"]);
                assert!(bs[1].keys.is_empty());
            }
            _ => panic!("expected Prompt"),
        }
    }

    #[test]
    fn parses_effect_step() {
        let s: Step = ron::from_str(r#"Effect(Banner("BOOM"))"#).unwrap();
        assert!(matches!(s, Step::Effect(crate::effect::EffectKind::Banner(t)) if t == "BOOM"));
    }
}
