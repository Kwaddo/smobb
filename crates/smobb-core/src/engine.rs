use crate::scene::{Content, Step, Target};
use crate::speaker::{Color, Speaker};

/// Default delay between revealed lines in normal (non-skip) pacing, in ms.
/// Tuned down from the Java original's 4250 for a snappier read; skip-mode and
/// the TUI's hold-to-fast-forward shorten it further.
pub const DEFAULT_INTERVAL_MS: u32 = 1500;
/// Per-dot delay of the dramatic "..." beat in normal pacing, in ms (3 dots).
/// Tuned down from 2000 (6s) to keep the beat without stalling.
const PAUSE_STEP_MS: u32 = 700;

const CHARACTERS: &[&str] = &["ali", "bader", "bevil", "hasan", "josh", "qassim", "smobb"];

const CHARACTER_NAMES: &[&str] = &[
    "ali", "bader", "bevil", "hasan", "josh", "qassim", "qasim", "smobb",
];
const STRUCTURAL: &[&str] = &["guide", "extra", "story", "stories", "names", "settings"];

fn display_name(id: &str) -> String {
    match id {
        "hub" => "Hub",
        "ali" => "Ali",
        "bader" => "Bader",
        "bevil" => "Bevil",
        "hasan" => "Hasan",
        "josh" => "Josh",
        "qassim" | "qasim" => "Qassim",
        "smobb" => "S.M.O.B.B.",
        "mavis" => "M.A.V.I.S.",
        "guide" => "Guide",
        "guide_names" => "Names",
        "extra" => "Stories",
        other => {
            return {
                let mut c = other.chars();
                match c.next() {
                    Some(f) => f.to_uppercase().chain(c).collect(),
                    None => String::new(),
                }
            }
        }
    }
    .to_string()
}

fn structural_label(key: &str) -> String {
    match key {
        "guide" => "Guide",
        "extra" | "story" | "stories" => "Stories",
        "names" => "Names",
        "settings" => "Settings",
        _ => key,
    }
    .to_string()
}

#[derive(Debug, Clone, PartialEq)]
pub enum Output {
    Line { text: String, color: Color, speaker: Option<Speaker> },
    Sleep(u32),
    AwaitInput,
    Recap(String),
    Translation(String),
    End,
    Effect(crate::effect::EffectKind),
}

/// Single-quoted tokens with no whitespace (<=24 chars). Skips contraction apostrophes.
fn extract_quoted(s: &str) -> Vec<String> {
    let b = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'\'' {
            if let Some(rel) = s[i + 1..].find('\'') {
                let content = &s[i + 1..i + 1 + rel];
                if !content.is_empty()
                    && content.len() <= 24
                    && !content.chars().any(|c| c.is_whitespace())
                {
                    out.push(content.to_string());
                    i = i + 1 + rel + 1;
                    continue;
                }
            }
            i += 1;
        } else {
            i += 1;
        }
    }
    out
}

struct Frame {
    steps: Vec<Step>,
    idx: usize,
}

pub struct Engine {
    content: Content,
    loc_stack: Vec<String>,
    frames: Vec<Frame>,
    interval_ms: u32,
    skip: bool,
    pending_prompt: Option<Vec<crate::scene::Branch>>,
    recap: crate::recap::Recap,
    decoded: Vec<(String, String)>,
    discovered: std::collections::BTreeSet<String>,
    visited: std::collections::BTreeSet<String>,
    asked: std::collections::BTreeMap<String, std::collections::BTreeSet<String>>,
    decoding_unlocked: bool,
}

impl Engine {
    pub fn new(content: Content) -> Self {
        let start = content.start.clone();
        Engine {
            content,
            loc_stack: vec![start],
            frames: Vec::new(),
            interval_ms: DEFAULT_INTERVAL_MS,
            skip: false,
            pending_prompt: None,
            recap: crate::recap::Recap::default(),
            decoded: Vec::new(),
            discovered: std::collections::BTreeSet::new(),
            visited: std::collections::BTreeSet::new(),
            asked: std::collections::BTreeMap::new(),
            decoding_unlocked: false,
        }
    }

    pub fn set_skip(&mut self, on: bool) {
        self.skip = on;
    }
    pub fn set_interval(&mut self, ms: u32) {
        self.interval_ms = ms;
    }
    pub fn skip(&self) -> bool {
        self.skip
    }
    pub fn interval(&self) -> u32 {
        self.interval_ms
    }

    /// Export player progress (journal + settings) for persistence.
    pub fn snapshot(&self) -> crate::save::SaveState {
        crate::save::SaveState {
            recap: self.recap.entries().to_vec(),
            discovered: self.discovered.iter().cloned().collect(),
            asked: self
                .asked
                .iter()
                .map(|(k, v)| (k.clone(), v.iter().cloned().collect()))
                .collect(),
            decoded: self.decoded.clone(),
            visited: self.visited.iter().cloned().collect(),
            skip: self.skip,
            interval_ms: self.interval_ms,
            decoding_unlocked: self.decoding_unlocked,
        }
    }

    /// Load a saved snapshot into this engine (call before `start`).
    pub fn restore(&mut self, s: crate::save::SaveState) {
        self.recap = crate::recap::Recap::load(&crate::recap::MemoryStorage { lines: s.recap });
        self.discovered = s.discovered.into_iter().collect();
        self.asked = s
            .asked
            .into_iter()
            .map(|(k, v)| (k, v.into_iter().collect()))
            .collect();
        self.decoded = s.decoded;
        self.visited = s.visited.into_iter().collect();
        self.skip = s.skip;
        self.interval_ms = s.interval_ms;
        self.decoding_unlocked = s.decoding_unlocked;
    }

    pub fn start(&mut self) -> Vec<Output> {
        let mut out = Vec::new();
        let loc = self
            .content
            .locations
            .get(self.loc_stack.last().unwrap())
            .expect("start location missing")
            .clone();
        self.emit_intro(&loc.intro, &mut out);
        out.extend(self.menu_prompt());
        out
    }

    fn menu_prompt(&self) -> Vec<Output> {
        let mut out = Vec::new();
        let loc = self
            .content
            .locations
            .get(self.loc_stack.last().unwrap())
            .unwrap();
        for s in &loc.prompt {
            self.emit_step_lines(s, &mut out);
        }
        out.push(Output::AwaitInput);
        out
    }

    /// Emit a location intro: Lines (paced by interval unless skip) and Pause beats.
    fn emit_intro(&self, steps: &[crate::scene::Step], out: &mut Vec<Output>) {
        use crate::scene::Step;
        for s in steps {
            match s {
                Step::Line(sp, text) => {
                    out.push(Output::Line {
                        text: text.clone(),
                        color: sp.color(),
                        speaker: Some(*sp),
                    });
                    if !self.skip {
                        out.push(Output::Sleep(self.interval_ms));
                    }
                }
                Step::Pause => self.emit_pause(out),
                Step::Effect(k) => out.push(Output::Effect(k.clone())),
                _ => {}
            }
        }
    }

    fn emit_step_lines(&self, step: &Step, out: &mut Vec<Output>) {
        match step {
            Step::Line(sp, text) => out.push(Output::Line {
                text: text.clone(),
                color: sp.color(),
                speaker: Some(*sp),
            }),
            Step::Effect(k) => out.push(Output::Effect(k.clone())),
            _ => {}
        }
    }

    pub fn advance(&mut self, input: &str) -> Vec<Output> {
        let input = input.trim().to_lowercase();
        // Universal escape: works the same from anywhere. Inside a dialogue/scene
        // it aborts back to the current location's menu; at a menu it leaves the
        // location (the hub quits the game).
        if matches!(input.as_str(), "leave" | "exit" | "back" | "end") {
            if self.pending_prompt.is_some() || !self.frames.is_empty() {
                self.pending_prompt = None;
                self.frames.clear();
                return self.menu_prompt();
            }
            self.frames.clear();
            if self.loc_stack.len() > 1 {
                self.loc_stack.pop();
                return self.menu_prompt();
            }
            return vec![Output::End];
        }
        if let Some(branches) = self.pending_prompt.take() {
            let chosen = branches
                .iter()
                .find(|b| b.keys.iter().any(|k| k.to_lowercase() == input))
                .or_else(|| branches.iter().find(|b| b.keys.is_empty()));
            match chosen {
                Some(b) => {
                    let t = b.target.clone();
                    self.push_target(&t);
                    let mut out = self.run();
                    if self.frames.is_empty()
                        && !out.contains(&Output::End)
                        && !matches!(out.last(), Some(Output::AwaitInput))
                    {
                        out.extend(self.menu_prompt());
                    }
                    return out;
                }
                None => {
                    // no match, no default: re-await same prompt
                    self.pending_prompt = Some(branches);
                    return vec![Output::AwaitInput];
                }
            }
        }
        if self.frames.is_empty() {
            return self.handle_command(&input);
        }
        self.run()
    }

    fn branch_label(&self, loc_id: &str, b: &crate::scene::Branch) -> Option<String> {
        use crate::scene::{Step, Target};
        if let Target::Steps(steps) = &b.target {
            for s in steps {
                if let Step::Enter(x) = s {
                    return Some(display_name(x));
                }
            }
        }
        if b.keys.iter().any(|k| k == "end" || k == "quit") {
            return Some(if loc_id == "hub" {
                "Quit".into()
            } else {
                "Leave".into()
            });
        }
        for k in &b.keys {
            if CHARACTER_NAMES.contains(&k.as_str()) {
                return Some(format!("Ask about {}", display_name(k)));
            }
        }
        for k in &b.keys {
            if STRUCTURAL.contains(&k.as_str()) {
                return Some(structural_label(k));
            }
        }
        None
    }

    pub fn options_view(&self) -> Vec<Output> {
        let c = crate::speaker::Color::White;
        let loc_id = self.loc_stack.last().cloned().unwrap_or_default();
        let loc = match self.content.locations.get(&loc_id) {
            Some(l) => l,
            None => {
                return vec![Output::Line {
                    text: "(no options)".into(),
                    color: c,
                    speaker: None,
                }]
            }
        };
        let mut out = vec![Output::Line {
            text: "You can:".into(),
            color: c,
            speaker: None,
        }];
        let mut seen = std::collections::BTreeSet::new();
        for b in &loc.commands {
            if let Some(label) = self.branch_label(&loc_id, b) {
                if seen.insert(label.clone()) {
                    out.push(Output::Line {
                        text: format!("  - {}", label),
                        color: c,
                        speaker: None,
                    });
                }
            }
        }
        let known = if self.discovered.is_empty() {
            "  (none discovered yet)".to_string()
        } else {
            format!(
                "  {}",
                self.discovered
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        out.push(Output::Line {
            text: "Known passwords (try these):".into(),
            color: c,
            speaker: None,
        });
        out.push(Output::Line {
            text: known,
            color: c,
            speaker: None,
        });
        out
    }

    fn handle_command(&mut self, input: &str) -> Vec<Output> {
        let loc_id = self.loc_stack.last().cloned().unwrap_or_default();
        let loc = self.content.locations.get(&loc_id).unwrap().clone();
        // commands that are navigation or meta/builtins aren't "things asked about"
        const NONRECORD: &[&str] = &[
            "recap",
            "rp",
            "skipmode",
            "sm",
            "interval",
            "in",
            "options",
            "journal",
            "j",
            "guide",
            "help",
            "play",
            "pl",
            "shortcut",
            "shortcuts",
            "look",
            "?",
            "info",
            "information",
        ];
        for b in &loc.commands {
            if b.keys.iter().any(|k| k.to_lowercase() == input) {
                if self.branch_label(&loc_id, b).is_none() {
                    self.discovered.insert(input.to_string());
                }
                let is_nav = matches!(&b.target, crate::scene::Target::Steps(steps)
                    if steps.iter().any(|s| matches!(s, crate::scene::Step::Enter(_))));
                if !is_nav && !NONRECORD.contains(&input) {
                    self.asked
                        .entry(loc_id.clone())
                        .or_default()
                        .insert(input.to_string());
                }
                self.push_target(&b.target);
                let mut out = self.run();
                if self.frames.is_empty()
                    && !out.contains(&Output::End)
                    && !matches!(out.last(), Some(Output::AwaitInput))
                {
                    out.extend(self.menu_prompt());
                }
                return out;
            }
        }
        // unknown command: just re-prompt
        self.menu_prompt()
    }

    fn push_target(&mut self, target: &Target) {
        match target {
            Target::Steps(steps) => self.frames.push(Frame {
                steps: steps.clone(),
                idx: 0,
            }),
            Target::Scene(id) => {
                let sc = self
                    .content
                    .scenes
                    .get(id)
                    .unwrap_or_else(|| panic!("missing scene: {id}"))
                    .clone();
                self.frames.push(Frame {
                    steps: sc.steps,
                    idx: 0,
                });
            }
        }
    }

    /// Executes frames until a Prompt (AwaitInput, Task 8) or all frames end.
    fn run(&mut self) -> Vec<Output> {
        let mut out = Vec::new();
        while let Some(frame) = self.frames.last_mut() {
            if frame.idx >= frame.steps.len() {
                self.frames.pop();
                continue;
            }
            let step = frame.steps[frame.idx].clone();
            frame.idx += 1;
            match step {
                Step::Line(sp, text) => {
                    out.push(Output::Line {
                        text: text.clone(),
                        color: sp.color(),
                        speaker: Some(sp),
                    });
                    // Only auto-translate Jaffar's hex once the player has learned
                    // it's encoded (see Step::UnlockDecode). Until then it reads as
                    // raw gibberish and nothing lands in the Decoded journal.
                    if self.decoding_unlocked && sp == crate::speaker::Speaker::Jaffar {
                        if let Some(plain) = crate::decode::decode_hex(&text) {
                            out.push(Output::Translation(plain.clone()));
                            if !self.decoded.iter().any(|(h, _)| h == &text) {
                                self.decoded.push((text.clone(), plain));
                            }
                        }
                    }
                    if !self.skip {
                        out.push(Output::Sleep(self.interval_ms));
                    }
                }
                Step::Recap(t) => {
                    self.recap.add(&t);
                    for tok in extract_quoted(&t) {
                        // store lowercased so it matches the use-hook (advance lowercases
                        // all input) — avoids K1LL/k1ll duplicates in options/journal.
                        self.discovered.insert(tok.to_lowercase());
                    }
                    out.push(Output::Recap(t));
                }
                Step::Pause => self.emit_pause(&mut out), // Task 7
                Step::Return => {
                    self.frames.clear();
                    if self.loc_stack.len() > 1 {
                        self.loc_stack.pop();
                        out.extend(self.menu_prompt());
                        return out;
                    }
                    // at root: clear frames and exit loop; caller adds menu_prompt
                    break;
                }
                Step::Goto(id) => {
                    let sc = self
                        .content
                        .scenes
                        .get(&id)
                        .unwrap_or_else(|| panic!("missing scene: {id}"))
                        .clone();
                    self.frames.push(Frame {
                        steps: sc.steps,
                        idx: 0,
                    });
                }
                Step::Prompt(branches) => {
                    self.pending_prompt = Some(branches);
                    out.push(Output::AwaitInput);
                    return out;
                }
                Step::Enter(id) => {
                    self.frames.clear();
                    self.loc_stack.push(id.clone());
                    if CHARACTERS.contains(&id.as_str()) {
                        self.visited.insert(id.clone());
                    }
                    let loc = self
                        .content
                        .locations
                        .get(&id)
                        .unwrap_or_else(|| panic!("missing location: {id}"))
                        .clone();
                    self.emit_intro(&loc.intro, &mut out);
                    out.extend(self.menu_prompt());
                    return out;
                }
                Step::Quit => {
                    out.push(Output::End);
                    return out;
                }
                Step::UnlockDecode => {
                    self.decoding_unlocked = true;
                }
                Step::Effect(k) => {
                    out.push(Output::Effect(k));
                }
            }
        }
        out
    }

    fn emit_pause(&self, out: &mut Vec<Output>) {
        for _ in 0..3 {
            out.push(Output::Line {
                text: ".".into(),
                color: crate::speaker::Color::White,
                speaker: None,
            });
            // The dramatic beat honors skip-mode: with skip on, the dots still
            // print but without the wait, so skipping never stalls on a pause.
            if !self.skip {
                out.push(Output::Sleep(PAUSE_STEP_MS));
            }
        }
    }

    pub fn recap_entries(&self) -> &[String] {
        self.recap.entries()
    }

    #[cfg(test)]
    pub(crate) fn discovered(&self) -> &std::collections::BTreeSet<String> {
        &self.discovered
    }

    #[cfg(test)]
    pub(crate) fn visited(&self) -> &std::collections::BTreeSet<String> {
        &self.visited
    }

    pub fn journal_view(&self) -> Vec<Output> {
        let c = crate::speaker::Color::White;
        let mut out = Vec::new();
        out.push(Output::Line {
            text: "== JOURNAL ==".into(),
            color: c,
            speaker: None,
        });

        out.push(Output::Line {
            text: "[Clues]".into(),
            color: c,
            speaker: None,
        });
        if self.recap.entries().is_empty() {
            out.push(Output::Line {
                text: "  (none yet)".into(),
                color: c,
                speaker: None,
            });
        } else {
            for e in self.recap.entries() {
                out.push(Output::Line {
                    text: format!("  - {}", e),
                    color: c,
                    speaker: None,
                });
            }
        }

        out.push(Output::Line {
            text: "[Passwords]".into(),
            color: c,
            speaker: None,
        });
        if self.discovered.is_empty() {
            out.push(Output::Line {
                text: "  (none yet)".into(),
                color: c,
                speaker: None,
            });
        } else {
            for p in &self.discovered {
                out.push(Output::Line {
                    text: format!("  - {}", p),
                    color: c,
                    speaker: None,
                });
            }
        }

        out.push(Output::Line {
            text: "[People]".into(),
            color: c,
            speaker: None,
        });
        if self.visited.is_empty() {
            out.push(Output::Line {
                text: "  (none yet)".into(),
                color: c,
                speaker: None,
            });
        } else {
            for v in &self.visited {
                out.push(Output::Line {
                    text: format!("  - {}", display_name(v)),
                    color: c,
                    speaker: None,
                });
            }
        }

        out.push(Output::Line {
            text: "[Asked]".into(),
            color: c,
            speaker: None,
        });
        if self.asked.is_empty() {
            out.push(Output::Line {
                text: "  (none yet)".into(),
                color: c,
                speaker: None,
            });
        } else {
            for (loc, topics) in &self.asked {
                let list = topics.iter().cloned().collect::<Vec<_>>().join(", ");
                out.push(Output::Line {
                    text: format!("  {}: {}", display_name(loc), list),
                    color: c,
                    speaker: None,
                });
            }
        }

        out.push(Output::Line {
            text: "[Decoded]".into(),
            color: c,
            speaker: None,
        });
        if self.decoded.is_empty() {
            out.push(Output::Line {
                text: "  (none yet)".into(),
                color: c,
                speaker: None,
            });
        } else {
            for (h, p) in &self.decoded {
                out.push(Output::Line {
                    text: format!("  - {} \u{2192} {}", h, p),
                    color: c,
                    speaker: None,
                });
            }
        }

        out
    }

    pub fn recap_view(&self) -> Vec<Output> {
        let c = Color::White;
        if self.recap.entries().is_empty() {
            return vec![Output::Line {
                text: "(empty)".into(),
                color: c,
                speaker: None,
            }];
        }
        self.recap
            .entries()
            .iter()
            .map(|t| Output::Line {
                text: t.clone(),
                color: c,
                speaker: None,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content;

    fn fx() -> crate::scene::Content {
        content::from_str(&[r#"ContentPart(
            locations: { "hub": (
                intro: [ Line(Smobb, "HELLO.") ],
                prompt: [ Line(Smobb, "HOW CAN I HELP YOU?") ],
                commands: [],
            ) },
            scenes: {},
        )"#])
    }

    #[test]
    fn start_emits_intro_then_prompt_then_await() {
        let mut e = Engine::new(fx());
        let out = e.start();
        // first line is intro, then prompt line, then AwaitInput
        assert!(matches!(out[0], Output::Line { .. }));
        assert!(matches!(out.last().unwrap(), Output::AwaitInput));
        let texts: Vec<_> = out
            .iter()
            .filter_map(|o| match o {
                Output::Line { text, .. } => Some(text.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(texts, vec!["HELLO.", "HOW CAN I HELP YOU?"]);
    }

    #[test]
    fn runs_a_command_scene_then_returns_to_menu() {
        use crate::content;
        let c = content::from_str(&[r#"ContentPart(
            locations: { "hub": (
                prompt: [ Line(Smobb, "MENU") ],
                commands: [ Branch(keys: ["go"], target: Steps([
                    Line(Ali, "line a"),
                    Recap("note"),
                    Return,
                ])) ],
            ) },
            scenes: {},
        )"#]);
        let mut e = Engine::new(c);
        let _ = e.start();
        let out = e.advance("go");
        // Expect: Line "line a", Sleep(interval), Recap("note"), then menu "MENU", AwaitInput
        let kinds: Vec<&str> = out
            .iter()
            .map(|o| match o {
                Output::Line { .. } => "L",
                Output::Sleep(_) => "S",
                Output::Recap(_) => "R",
                Output::AwaitInput => "A",
                Output::Translation(_) => "T",
                Output::End => "E",
                Output::Effect(_) => "X",
            })
            .collect();
        assert_eq!(kinds, vec!["L", "S", "R", "L", "A"]);
        assert!(out.contains(&Output::Recap("note".into())));
    }

    #[test]
    fn unknown_command_reprompts() {
        use crate::content;
        let c = content::from_str(&[r#"ContentPart(
            locations: { "hub": ( prompt: [ Line(Smobb, "MENU") ], commands: [] ) },
            scenes: {},
        )"#]);
        let mut e = Engine::new(c);
        let _ = e.start();
        let out = e.advance("nope");
        assert!(matches!(out.last().unwrap(), Output::AwaitInput));
    }

    #[test]
    fn pause_emits_three_dotted_sleeps() {
        use crate::content;
        let c = content::from_str(&[r#"ContentPart(
            locations: { "hub": ( commands: [ Branch(keys:["go"], target: Steps([ Pause, Return ])) ] ) },
            scenes: {},
        )"#]);
        let mut e = Engine::new(c);
        let _ = e.start();
        let out = e.advance("go");
        let sleeps: Vec<u32> = out
            .iter()
            .filter_map(|o| match o {
                Output::Sleep(ms) => Some(*ms),
                _ => None,
            })
            .collect();
        assert_eq!(sleeps, vec![700, 700, 700]);
    }

    #[test]
    fn pause_skips_sleeps_when_skipmode() {
        use crate::content;
        let c = content::from_str(&[r#"ContentPart(
            locations: { "hub": ( commands: [ Branch(keys:["go"], target: Steps([ Pause, Return ])) ] ) },
            scenes: {},
        )"#]);
        let mut e = Engine::new(c);
        e.set_skip(true);
        let _ = e.start();
        let out = e.advance("go");
        // dots still print, but NO sleeps (the dramatic beat collapses under skip-mode)
        let dots = out
            .iter()
            .filter(|o| matches!(o, Output::Line{text,..} if text == "."))
            .count();
        let sleeps = out.iter().filter(|o| matches!(o, Output::Sleep(_))).count();
        assert_eq!(dots, 3);
        assert_eq!(sleeps, 0);
    }

    #[test]
    fn prompt_branches_match_and_default() {
        use crate::content;
        let c = content::from_str(&[r#"ContentPart(
            locations: { "hub": ( commands: [ Branch(keys:["go"], target: Steps([
                Prompt([
                    Branch(keys:["yes","y"], target: Steps([ Line(Smobb,"YESPATH"), Return ])),
                    Branch(keys:[], target: Steps([ Line(Smobb,"DEFAULT"), Return ])),
                ]),
            ])) ] ) },
            scenes: {},
        )"#]);
        let mut e = Engine::new(c);
        let _ = e.start();
        let out = e.advance("go");
        assert!(matches!(out.last().unwrap(), Output::AwaitInput));
        let out = e.advance("y");
        assert!(out
            .iter()
            .any(|o| matches!(o, Output::Line{text,..} if text=="YESPATH")));

        // re-run, take default
        let mut e2 = Engine::new(content::from_str(&[r#"ContentPart(
            locations: { "hub": ( commands: [ Branch(keys:["go"], target: Steps([
                Prompt([ Branch(keys:["yes"], target: Steps([ Line(Smobb,"Y"), Return ])),
                         Branch(keys:[], target: Steps([ Line(Smobb,"DEFAULT"), Return ])) ]),
            ])) ] ) }, scenes: {},
        )"#]));
        let _ = e2.start();
        let _ = e2.advance("go");
        let out = e2.advance("whatever");
        assert!(out
            .iter()
            .any(|o| matches!(o, Output::Line{text,..} if text=="DEFAULT")));
    }

    #[test]
    fn enter_and_exit_locations() {
        use crate::content;
        let c = content::from_str(&[r#"ContentPart(
            locations: {
                "hub": ( prompt:[Line(Smobb,"HUB")], commands: [
                    Branch(keys:["ali"], target: Steps([ Enter("ali") ])),
                    Branch(keys:["end"], target: Steps([ Quit ])),
                ]),
                "ali": ( intro:[Line(Ali,"hi")], prompt:[Line(Ali,"ALIMENU")], commands: [
                    Branch(keys:["end"], target: Steps([ Return ])),
                ]),
            },
            scenes: {},
        )"#]);
        let mut e = Engine::new(c);
        let _ = e.start();
        let out = e.advance("ali");
        assert!(out
            .iter()
            .any(|o| matches!(o, Output::Line{text,..} if text=="hi")));
        assert!(out
            .iter()
            .any(|o| matches!(o, Output::Line{text,..} if text=="ALIMENU")));
        let out = e.advance("end"); // pop back to hub
        assert!(out
            .iter()
            .any(|o| matches!(o, Output::Line{text,..} if text=="HUB")));
        let out = e.advance("end"); // quit at hub
        assert!(out.contains(&Output::End));
    }

    #[test]
    fn intro_renders_pause() {
        use crate::content;
        let c = content::from_str(&[r#"ContentPart(
            locations: { "hub": ( intro: [ Pause, Line(Smobb, "HI") ], prompt: [ Line(Smobb, "MENU") ], commands: [] ) },
            scenes: {},
        )"#]);
        let mut e = Engine::new(c);
        let out = e.start();
        // Pause emits Sleep(700) beats; assert at least one and the HI line present
        assert!(out.iter().any(|o| matches!(o, Output::Sleep(700))));
        assert!(out
            .iter()
            .any(|o| matches!(o, Output::Line{text,..} if text=="HI")));
        assert!(matches!(out.last().unwrap(), Output::AwaitInput));
    }

    #[test]
    fn goto_runs_target_scene() {
        use crate::content;
        let c = content::from_str(&[r#"ContentPart(
            locations: { "hub": ( commands: [ Branch(keys:["go"], target: Steps([ Goto("dest") ])) ] ) },
            scenes: { "dest": ( steps: [ Line(Smobb, "ARRIVED"), Return ] ) },
        )"#]);
        let mut e = Engine::new(c);
        let _ = e.start();
        let out = e.advance("go");
        assert!(out
            .iter()
            .any(|o| matches!(o, Output::Line{text,..} if text=="ARRIVED")));
    }

    #[test]
    fn skip_suppresses_interval_sleeps() {
        use crate::content;
        let c = content::from_str(&[r#"ContentPart(
            locations: { "hub": ( commands: [ Branch(keys:["go"], target: Steps([ Line(Ali,"a"), Return ])) ] ) },
            scenes: {},
        )"#]);
        let mut e = Engine::new(c);
        e.set_skip(true);
        let _ = e.start();
        let out = e.advance("go");
        assert!(!out.iter().any(|o| matches!(o, Output::Sleep(_))));
    }

    #[test]
    fn interval_value_used() {
        use crate::content;
        let c = content::from_str(&[r#"ContentPart(
            locations: { "hub": ( commands: [ Branch(keys:["go"], target: Steps([ Line(Ali,"a"), Return ])) ] ) },
            scenes: {},
        )"#]);
        let mut e = Engine::new(c);
        e.set_interval(100);
        let _ = e.start();
        let out = e.advance("go");
        assert!(out.contains(&Output::Sleep(100)));
    }

    #[test]
    fn engine_collects_recap_dedup() {
        use crate::content;
        let c = content::from_str(&[r#"ContentPart(
            locations: { "hub": ( commands: [ Branch(keys:["go"], target: Steps([ Recap("clue"), Recap("clue"), Return ])) ] ) },
            scenes: {},
        )"#]);
        let mut e = Engine::new(c);
        let _ = e.start();
        let _ = e.advance("go");
        assert_eq!(e.recap_entries(), &["clue".to_string()]);
    }

    #[test]
    fn recap_view_lists_entries() {
        use crate::content;
        let c = content::from_str(&[r#"ContentPart(
            locations: { "hub": ( commands: [ Branch(keys:["go"], target: Steps([ Recap("one"), Return ])) ] ) },
            scenes: {},
        )"#]);
        let mut e = Engine::new(c);
        let _ = e.start();
        let _ = e.advance("go");
        let view = e.recap_view();
        assert!(view
            .iter()
            .any(|o| matches!(o, Output::Line{text,..} if text=="one")));
    }

    #[test]
    fn recap_view_empty() {
        use crate::content;
        let c = content::from_str(&[r#"ContentPart(locations:{"hub":(commands:[])},scenes:{})"#]);
        let e = Engine::new(c);
        let view = e.recap_view();
        assert!(view
            .iter()
            .any(|o| matches!(o, Output::Line{text,..} if text=="(empty)")));
    }

    #[test]
    fn jaffar_hex_translation_gated_by_unlock() {
        use crate::content;
        let c = content::from_str(&[r#"ContentPart(
            locations: { "hub": ( commands: [
                Branch(keys:["pre"], target: Steps([ Line(Jaffar, "48656C6C6F") ])),
                Branch(keys:["learn"], target: Steps([ UnlockDecode, Line(Jaffar, "48656C6C6F") ])),
            ] ) },
            scenes: {},
        )"#]);
        let mut e = Engine::new(c);
        e.set_skip(true); // drop Sleeps for a clean sequence
        let _ = e.start();
        let translations = |out: &[Output]| -> Vec<String> {
            out.iter()
                .filter_map(|o| match o {
                    Output::Translation(t) => Some(t.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>()
        };
        // before the reveal: raw hex, no translation (gibberish)
        let out = e.advance("pre");
        assert!(translations(&out).is_empty());
        assert!(out
            .iter()
            .any(|o| matches!(o, Output::Line{text,..} if text=="48656C6C6F")));
        // after UnlockDecode: auto-translated
        let out = e.advance("learn");
        assert_eq!(translations(&out), vec!["Hello".to_string()]);
    }

    #[test]
    fn extract_quoted_handles_contractions() {
        // contraction apostrophe in "S.M.O.B.B.'s" must NOT break token capture
        assert_eq!(
            extract_quoted("The password for S.M.O.B.B.'s files is '9285005'."),
            vec!["9285005".to_string()]
        );
        assert_eq!(
            extract_quoted("code is '<---------->'."),
            vec!["<---------->".to_string()]
        );
        assert_eq!(
            extract_quoted("It is '!SAVE'. Done."),
            vec!["!SAVE".to_string()]
        );
        assert!(extract_quoted("no quotes here").is_empty());
    }

    #[test]
    fn recap_populates_discovered() {
        use crate::content;
        let c = content::from_str(&[r#"ContentPart(
            locations: { "hub": ( commands: [ Branch(keys:["go"], target: Steps([
                Recap("The password for the Tranava files is '7549009'."),
            ])) ] ) },
            scenes: {},
        )"#]);
        let mut e = Engine::new(c);
        let _ = e.start();
        let _ = e.advance("go");
        assert!(e.discovered().contains("7549009"));
    }

    #[test]
    fn entering_a_character_records_visit() {
        use crate::content;
        let c = content::from_str(&[r#"ContentPart(
            locations: {
                "hub": ( commands: [ Branch(keys:["ali"], target: Steps([ Enter("ali") ])) ] ),
                "ali": ( commands: [ Branch(keys:["end"], target: Steps([ Return ])) ] ),
            },
            scenes: {},
        )"#]);
        let mut e = Engine::new(c);
        let _ = e.start();
        let _ = e.advance("ali");
        assert!(e.visited().contains("ali"));
    }

    #[test]
    fn options_view_lists_public_and_known() {
        use crate::content;
        let c = content::from_str(&[r#"ContentPart(
            locations: { "hub": ( commands: [
                Branch(keys:["ali"], target: Steps([ Enter("ali") ])),
                Branch(keys:["7549009"], target: Steps([ Line(System, "secret") ])),
                Branch(keys:["end"], target: Steps([ Quit ])),
            ] ) },
            scenes: {},
        )"#]);
        let mut e = Engine::new(c);
        let _ = e.start();
        let text: Vec<String> = e
            .options_view()
            .iter()
            .filter_map(|o| match o {
                Output::Line { text, .. } => Some(text.clone()),
                _ => None,
            })
            .collect();
        let joined = text.join("\n");
        assert!(joined.contains("Ali")); // nav label
        assert!(joined.contains("Quit")); // exit label
        assert!(!joined.contains("7549009")); // secret hidden (not yet discovered)
        assert!(joined.to_lowercase().contains("known passwords"));
    }

    #[test]
    fn universal_exit_from_dialogue_returns_to_menu() {
        use crate::content;
        let c = content::from_str(&[r#"ContentPart(
            locations: { "hub": ( prompt:[Line(Smobb,"MENU")], commands: [
                Branch(keys:["talk"], target: Steps([
                    Line(Smobb,"A"),
                    Prompt([ Branch(keys:["yes"], target: Steps([ Line(Smobb,"YES"), Return ])) ]),
                ])),
            ] ) },
            scenes: {},
        )"#]);
        let mut e = Engine::new(c);
        e.set_skip(true);
        let _ = e.start();
        let out = e.advance("talk"); // emit A, suspend at the prompt
        assert!(matches!(out.last().unwrap(), Output::AwaitInput));
        let out = e.advance("leave"); // bail out of the dialogue
        assert!(matches!(out.last().unwrap(), Output::AwaitInput));
        assert!(out
            .iter()
            .any(|o| matches!(o, Output::Line{text,..} if text=="MENU")));
    }

    #[test]
    fn universal_exit_pops_location_and_quits_at_hub() {
        use crate::content;
        let c = content::from_str(&[r#"ContentPart(
            locations: {
                "hub": ( prompt:[Line(Smobb,"HUB")], commands: [ Branch(keys:["ali"], target: Steps([ Enter("ali") ])) ] ),
                "ali": ( prompt:[Line(Ali,"ALI")], commands: [] ),
            },
            scenes: {},
        )"#]);
        let mut e = Engine::new(c);
        e.set_skip(true);
        let _ = e.start();
        let _ = e.advance("ali");
        let out = e.advance("exit"); // leave ali -> hub
        assert!(out
            .iter()
            .any(|o| matches!(o, Output::Line{text,..} if text=="HUB")));
        let out = e.advance("end"); // hub -> quit
        assert!(out.contains(&Output::End));
    }

    #[test]
    fn asked_topics_recorded_in_journal() {
        use crate::content;
        let c = content::from_str(&[r#"ContentPart(
            locations: {
                "hub": ( commands: [ Branch(keys:["go"], target: Steps([ Enter("ali") ])) ] ),
                "ali": ( prompt:[Line(Ali,"?")], commands: [
                    Branch(keys:["bader"], target: Steps([ Line(Ali, "about bader") ])),
                ] ),
            },
            scenes: {},
        )"#]);
        let mut e = Engine::new(c);
        e.set_skip(true);
        let _ = e.start();
        let _ = e.advance("go"); // navigation -> NOT recorded
        let _ = e.advance("bader"); // topic -> recorded under ali
        let jrnl: String = e
            .journal_view()
            .iter()
            .filter_map(|o| match o {
                Output::Line { text, .. } => Some(text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert!(jrnl.contains("[Asked]"));
        assert!(jrnl.contains("bader"));
        assert!(!jrnl.contains("- go") && !jrnl.contains(": go"));
    }

    #[test]
    fn snapshot_restore_round_trips_progress() {
        use crate::content;
        let src = r#"ContentPart(
            locations: {
                "hub": ( commands: [ Branch(keys:["go"], target: Steps([ Enter("ali") ])) ] ),
                "ali": ( prompt:[Line(Ali,"?")], commands: [
                    Branch(keys:["bader"], target: Steps([ UnlockDecode, Line(Ali,"about bader") ])),
                    Branch(keys:["7549009"], target: Steps([ Line(System,"secret") ])),
                ] ),
            },
            scenes: {},
        )"#;
        let mut e = Engine::new(content::from_str(&[src]));
        e.set_skip(true);
        let _ = e.start();
        let _ = e.advance("go");
        let _ = e.advance("bader"); // asked
        let _ = e.advance("7549009"); // discovered
        let snap = e.snapshot();
        assert!(snap.discovered.contains(&"7549009".to_string()));
        assert!(snap
            .asked
            .get("ali")
            .is_some_and(|v| v.contains(&"bader".to_string())));
        assert!(snap.skip);
        assert!(snap.decoding_unlocked); // unlocked via the bader scene's UnlockDecode

        // round-trip through JSON
        let snap2 = crate::save::SaveState::from_json(&snap.to_json()).unwrap();
        assert_eq!(snap, snap2);

        // restore into a fresh engine -> journal + settings reflect it.
        // Mirror the real frontend path: restore THEN start() (App::new calls
        // start), and confirm start doesn't clobber the restored progress.
        let mut e2 = Engine::new(content::from_str(&[src]));
        e2.restore(snap2);
        let _ = e2.start();
        let jrnl: String = e2
            .journal_view()
            .iter()
            .filter_map(|o| match o {
                Output::Line { text, .. } => Some(text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert!(jrnl.contains("7549009")); // password persisted
        assert!(jrnl.contains("bader")); // asked persisted
        assert!(e2.skip()); // setting persisted
        assert!(e2.snapshot().decoding_unlocked); // decode-unlock persisted through restore
    }

    #[test]
    fn journal_view_has_four_sections_and_data() {
        use crate::content;
        let c = content::from_str(&[r#"ContentPart(
            locations: { "hub": ( commands: [ Branch(keys:["go"], target: Steps([
                Recap("clue: the password is '7549009'."),
                UnlockDecode,
                Line(Jaffar, "48656C6C6F"),
            ])) ] ) },
            scenes: {},
        )"#]);
        let mut e = Engine::new(c);
        e.set_skip(true);
        let _ = e.start();
        let _ = e.advance("go");
        let joined: String = e
            .journal_view()
            .iter()
            .filter_map(|o| match o {
                Output::Line { text, .. } => Some(text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert!(joined.contains("[Clues]"));
        assert!(joined.contains("[Passwords]"));
        assert!(joined.contains("[People]"));
        assert!(joined.contains("[Decoded]"));
        assert!(joined.contains("7549009")); // password discovered + clue text
        assert!(joined.contains("Hello")); // decoded jaffar line
    }

    #[test]
    fn using_a_public_key_does_not_pollute_discovered() {
        use crate::content;
        let c = content::from_str(&[r#"ContentPart(
            locations: {
                "hub": ( commands: [ Branch(keys:["ali"], target: Steps([ Enter("ali") ])) ] ),
                "ali": ( commands: [ Branch(keys:["end"], target: Steps([ Return ])) ] ),
            },
            scenes: {},
        )"#]);
        let mut e = Engine::new(c);
        let _ = e.start();
        let _ = e.advance("ali");
        assert!(
            !e.discovered().contains("ali"),
            "navigation key 'ali' must not appear in discovered"
        );
        assert!(
            e.discovered().is_empty(),
            "discovered should be empty after a public navigation"
        );
    }

    #[test]
    fn using_a_secret_key_discovers_it() {
        use crate::content;
        let c = content::from_str(&[r#"ContentPart(
            locations: { "hub": ( commands: [
                Branch(keys:["7549009"], target: Steps([ Line(System, "ok") ])),
            ] ) },
            scenes: {},
        )"#]);
        let mut e = Engine::new(c);
        let _ = e.start();
        let _ = e.advance("7549009");
        assert!(e.discovered().contains("7549009"));
    }

    #[test]
    fn dialogue_line_carries_speaker_journal_line_does_not() {
        use crate::content;
        use crate::speaker::Speaker;
        let c = content::from_str(&[r#"ContentPart(
            locations: { "hub": ( prompt:[Line(Smobb,"P")],
                commands: [ Branch(keys:["go"], target: Steps([ Line(Ali,"hi"), Return ])) ] ) },
            scenes: {},
        )"#]);
        let mut e = Engine::new(c);
        e.set_skip(true);
        let _ = e.start();
        let out = e.advance("go");
        assert!(out.iter().any(|o| matches!(o, Output::Line { speaker: Some(Speaker::Ali), .. })));
        assert!(e.journal_view().iter().all(|o| matches!(o, Output::Line { speaker: None, .. })));
    }

    #[test]
    fn effect_step_emits_output_effect_even_when_skipping() {
        use crate::content;
        use crate::effect::EffectKind;
        let c = content::from_str(&[r#"ContentPart(
            locations: { "hub": ( commands: [ Branch(keys:["go"], target: Steps([
                Effect(Flash(Red)), Return ])) ] ) },
            scenes: {},
        )"#]);
        let mut e = Engine::new(c);
        e.set_skip(true);
        let _ = e.start();
        let out = e.advance("go");
        assert!(out.iter().any(|o| matches!(o, Output::Effect(EffectKind::Flash(_)))));
    }
}
