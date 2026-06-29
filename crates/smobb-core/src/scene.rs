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
    /// Run `then` if the condition holds, else `else`. State-gated branching.
    If(Condition, Vec<Step>, Vec<Step>),
    /// Signposted, state-driven code deployment (see `GateSpec`).
    Gate(GateSpec),
}

/// A "you learned a code — deploy it?" interaction. Emits `signpost`, then
/// waits for input: an affirmative (or the literal `code`) routes to
/// `on_deploy` when the player knows `code`, else `on_missing`; a negative
/// routes to `on_decline`.
#[derive(Debug, Clone, Deserialize)]
pub struct GateSpec {
    pub code: String,
    pub signpost: String,
    pub on_deploy: Vec<Step>,
    #[serde(default)]
    pub on_missing: Vec<Step>,
    #[serde(default)]
    pub on_decline: Vec<Step>,
}

/// A predicate over progress the engine already tracks. Used by `Step::If`
/// and `Step::Gate` to branch on what the player has learned/done.
#[derive(Debug, Clone, Deserialize)]
pub enum Condition {
    /// Player has discovered this code/password (case-insensitive).
    Knows(String),
    /// Player has entered this location id.
    ///
    /// Note: only the seven character ids ever populate `visited`
    /// (the engine's `Enter` arm gates `visited.insert` on `vocab::is_character`),
    /// so `Visited` of a non-character id (e.g. `mavis`/`jaffar`) is never true.
    Visited(String),
    /// Player has asked `topic` at location `loc` — `Asked(loc, topic)`.
    Asked(String, String),
    /// Jaffar's hex has been revealed as decodable.
    DecodeUnlocked,
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

    #[test]
    fn parses_gate_step() {
        let s: Step = ron::from_str(
            r#"Gate(GateSpec(
            code: "<---------->",
            signpost: "BREACH DETECTED",
            on_deploy: [ Line(System, "STOPPED") ],
            on_missing: [ Line(System, "NO CODE") ],
            on_decline: [ Line(System, "IGNORED") ],
        ))"#,
        )
        .unwrap();
        match s {
            Step::Gate(g) => {
                assert_eq!(g.code, "<---------->");
                assert_eq!(g.signpost, "BREACH DETECTED");
                assert_eq!(g.on_deploy.len(), 1);
                assert_eq!(g.on_missing.len(), 1);
                assert_eq!(g.on_decline.len(), 1);
            }
            _ => panic!("expected Gate"),
        }
    }

    #[test]
    fn gate_optional_branches_default_empty() {
        let s: Step = ron::from_str(
            r#"Gate(GateSpec(
            code: "x", signpost: "S", on_deploy: [ Line(System, "D") ],
        ))"#,
        )
        .unwrap();
        match s {
            Step::Gate(g) => {
                assert!(g.on_missing.is_empty());
                assert!(g.on_decline.is_empty());
            }
            _ => panic!("expected Gate"),
        }
    }

    #[test]
    fn parses_condition_variants() {
        assert!(matches!(
            ron::from_str::<Condition>(r#"Knows("7549009")"#).unwrap(),
            Condition::Knows(s) if s == "7549009"
        ));
        assert!(matches!(
            ron::from_str::<Condition>(r#"Visited("ali")"#).unwrap(),
            Condition::Visited(s) if s == "ali"
        ));
        assert!(matches!(
            ron::from_str::<Condition>("DecodeUnlocked").unwrap(),
            Condition::DecodeUnlocked
        ));
        assert!(matches!(
            ron::from_str::<Condition>(r#"Asked("ali", "bader")"#).unwrap(),
            Condition::Asked(l, t) if l == "ali" && t == "bader"
        ));
    }
}
