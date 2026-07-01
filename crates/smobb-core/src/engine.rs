use crate::scene::{Content, Step, Target};
use crate::speaker::{Color, Speaker};

/// Default delay between revealed lines in normal (non-skip) pacing, in ms.
/// Tuned down from the Java original's 4250 for a snappier read; skip-mode and
/// the TUI's hold-to-fast-forward shorten it further.
pub const DEFAULT_INTERVAL_MS: u32 = 1500;
/// Per-dot delay of the dramatic "..." beat in normal pacing, in ms (3 dots).
/// Tuned down from 2000 (6s) to keep the beat without stalling.
const PAUSE_STEP_MS: u32 = 700;

fn display_name(id: &str) -> String {
    // Accept either a canonical id or any of its aliases.
    let canon = crate::vocab::canonical(id).unwrap_or(id);
    if let Some(l) = crate::vocab::label(canon) {
        return l.to_string();
    }
    // Not a registry id (e.g. a one-off location like "guide" sub-pages):
    // capitalize the first letter of the original id.
    let mut c = id.chars();
    match c.next() {
        Some(f) => f.to_uppercase().chain(c).collect(),
        None => String::new(),
    }
}

fn structural_label(key: &str) -> String {
    let canon = crate::vocab::canonical(key).unwrap_or(key);
    crate::vocab::label(canon)
        .map(|l| l.to_string())
        .unwrap_or_else(|| key.to_string())
}

#[derive(Debug, Clone, PartialEq)]
pub enum Output {
    Line {
        text: String,
        color: Color,
        speaker: Option<Speaker>,
    },
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

/// Does a typed command look like a secret code worth remembering as a
/// "discovered password", versus an ordinary content command?
///
/// A branch key that carries no menu label (see `branch_label`) is only a
/// *secret* if it reads like a code. In this content, real codes are either
/// symbolic / leetspeak (`k1ll`, `!save`, `<---------->`, `end7312`) or long
/// numeric passwords (`7549009`). Plain dialogue commands are all letters
/// (`game`, `hack`, `breach`); menu selectors are 1-2 digit numbers (a story
/// number). So: require at least one non-letter, and reject short all-digit
/// tokens so menu numbers never surface as passwords while long numeric codes
/// still do.
fn is_secret_code(token: &str) -> bool {
    let has_non_letter = token.chars().any(|c| !c.is_ascii_alphabetic());
    let all_digits = !token.is_empty() && token.chars().all(|c| c.is_ascii_digit());
    has_non_letter && !(all_digits && token.len() < 3)
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
    pending_gate: Option<crate::scene::GateSpec>,
    /// Transient: a "close the game?" confirmation is awaiting yes/no. Never
    /// persisted — a reload should never resume mid-confirmation.
    pending_quit: bool,
    recap: crate::recap::Recap,
    decoded: Vec<(String, String)>,
    discovered: std::collections::BTreeSet<String>,
    visited: std::collections::BTreeSet<String>,
    asked: std::collections::BTreeMap<String, std::collections::BTreeSet<String>>,
    decoding_unlocked: bool,
    reduced_effects: bool,
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
            pending_gate: None,
            pending_quit: false,
            recap: crate::recap::Recap::default(),
            decoded: Vec::new(),
            discovered: std::collections::BTreeSet::new(),
            visited: std::collections::BTreeSet::new(),
            asked: std::collections::BTreeMap::new(),
            decoding_unlocked: false,
            reduced_effects: false,
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
    pub fn reduced_effects(&self) -> bool {
        self.reduced_effects
    }
    pub fn set_reduced_effects(&mut self, on: bool) {
        self.reduced_effects = on;
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
            reduced_effects: self.reduced_effects,
        }
    }

    /// Load a saved snapshot into this engine (call before `start`).
    pub fn restore(&mut self, s: crate::save::SaveState) {
        self.recap = crate::recap::Recap::load(&crate::recap::MemoryStorage { lines: s.recap });
        // Drop any non-code entries an older build wrongly recorded as passwords
        // (e.g. plain commands like "game"), so a stale save self-heals on load.
        self.discovered = s
            .discovered
            .into_iter()
            .filter(|d| is_secret_code(d))
            .collect();
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
        self.reduced_effects = s.reduced_effects;
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

    /// The "close the game?" confirmation line + an input await. Emitting this
    /// is always paired with setting `pending_quit`.
    fn quit_prompt(&self) -> Vec<Output> {
        vec![
            Output::Line {
                text: "CLOSE S.M.O.B.B.? YOUR PROGRESS IS SAVED. (yes / no)".into(),
                color: crate::speaker::Speaker::System.color(),
                speaker: Some(crate::speaker::Speaker::System),
            },
            Output::AwaitInput,
        ]
    }

    /// Begin the close-the-game confirmation (e.g. from an Esc keypress in a
    /// frontend). Returns the prompt to enqueue; the next `advance` resolves it.
    pub fn request_quit(&mut self) -> Vec<Output> {
        self.pending_quit = true;
        self.quit_prompt()
    }

    /// Whether a close-the-game confirmation is currently awaiting yes/no.
    pub fn quit_pending(&self) -> bool {
        self.pending_quit
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
        // A pending "close the game?" confirmation owns the next input.
        if self.pending_quit {
            self.pending_quit = false;
            return match crate::vocab::canonical(&input) {
                Some("yes") => vec![Output::End],
                Some("no") => {
                    // Declined: drop any in-progress dialogue, back to the menu.
                    self.pending_prompt = None;
                    self.pending_gate = None;
                    self.frames.clear();
                    self.menu_prompt()
                }
                // Anything else: keep waiting on the confirmation.
                _ => {
                    self.pending_quit = true;
                    self.quit_prompt()
                }
            };
        }
        // Navigation: step out of the current dialogue/location toward the hub.
        // Works from anywhere and NEVER closes the game.
        if matches!(input.as_str(), "leave" | "back") {
            if self.pending_prompt.is_some()
                || self.pending_gate.is_some()
                || !self.frames.is_empty()
            {
                self.pending_prompt = None;
                self.pending_gate = None;
                self.frames.clear();
                return self.menu_prompt();
            }
            self.frames.clear();
            if self.loc_stack.len() > 1 {
                self.loc_stack.pop();
                return self.menu_prompt();
            }
            // Already at the hub root — there is nothing to leave.
            return vec![
                Output::Line {
                    text: "YOU'RE AT THE START. TYPE 'QUIT' TO CLOSE THE GAME.".into(),
                    color: crate::speaker::Speaker::System.color(),
                    speaker: Some(crate::speaker::Speaker::System),
                },
                Output::AwaitInput,
            ];
        }
        // Close the game — from anywhere — behind a confirmation.
        if matches!(input.as_str(), "quit" | "exit" | "close" | "close game") {
            self.pending_quit = true;
            return self.quit_prompt();
        }
        if let Some(gate) = self.pending_gate.take() {
            let aff = crate::vocab::canonical(&input) == Some("yes");
            let neg = crate::vocab::canonical(&input) == Some("no");
            let typed_code = crate::vocab::key_matches(&gate.code, &input);
            let knows = self.eval_condition(&crate::scene::Condition::Knows(gate.code.clone()));
            let steps = if typed_code || (aff && knows) {
                Some(gate.on_deploy.clone())
            } else if aff || neg {
                // No prevention code in hand (or an explicit "no"): there is no
                // separate warning. Route straight to the breach-not-deployed
                // (decline) path. `on_missing` is intentionally never reached.
                Some(gate.on_decline.clone())
            } else {
                None
            };
            match steps {
                Some(s) => {
                    self.frames.push(Frame { steps: s, idx: 0 });
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
                    // unrecognized: re-emit the signpost and keep waiting.
                    let signpost = gate.signpost.clone();
                    self.pending_gate = Some(gate);
                    return vec![
                        Output::Line {
                            text: signpost,
                            color: crate::speaker::Speaker::System.color(),
                            speaker: Some(crate::speaker::Speaker::System),
                        },
                        Output::AwaitInput,
                    ];
                }
            }
        }
        if let Some(branches) = self.pending_prompt.take() {
            let chosen = branches
                .iter()
                .find(|b| b.keys.iter().any(|k| crate::vocab::key_matches(k, &input)))
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
            if let Some(c) = crate::vocab::canonical(k) {
                if crate::vocab::is_character(c) {
                    return Some(format!("Ask about {}", display_name(c)));
                }
            }
        }
        for k in &b.keys {
            if let Some(c) = crate::vocab::canonical(k) {
                if matches!(c, "guide" | "extra" | "guide_names" | "settings") {
                    return Some(structural_label(k));
                }
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
            "guide",
            "help",
            "play",
            "pl",
            "shortcut",
            "shortcuts",
            "info",
            "information",
            "options",
            "choices",
            "choice",
        ];
        for b in &loc.commands {
            if b.keys.iter().any(|k| crate::vocab::key_matches(k, input)) {
                let recorded = crate::vocab::canonical(input).unwrap_or(input).to_string();
                let is_meta = NONRECORD.contains(&input);
                if !is_meta && self.branch_label(&loc_id, b).is_none() && is_secret_code(&recorded)
                {
                    self.discovered.insert(recorded.clone());
                }
                let is_nav = matches!(&b.target, crate::scene::Target::Steps(steps)
                    if steps.iter().any(|s| matches!(s, crate::scene::Step::Enter(_))));
                if !is_nav && !is_meta {
                    self.asked
                        .entry(loc_id.clone())
                        .or_default()
                        .insert(recorded.clone());
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
        // unknown command: tell the player it wasn't understood, then re-prompt.
        let mut out = vec![Output::Line {
            text: format!(
                "I don't understand '{}'. Type 'options' to see what you can do.",
                input
            ),
            color: crate::speaker::Color::White,
            speaker: None,
        }];
        out.extend(self.menu_prompt());
        out
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
                    if crate::vocab::is_character(&id) {
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
                    // The hub "Quit" option asks before actually closing.
                    self.pending_quit = true;
                    out.extend(self.quit_prompt());
                    return out;
                }
                Step::UnlockDecode => {
                    self.decoding_unlocked = true;
                }
                Step::Effect(k) => {
                    out.push(Output::Effect(k));
                }
                Step::If(cond, then_steps, else_steps) => {
                    let chosen = if self.eval_condition(&cond) {
                        then_steps
                    } else {
                        else_steps
                    };
                    self.frames.push(Frame {
                        steps: chosen,
                        idx: 0,
                    });
                }
                Step::Gate(spec) => {
                    out.push(Output::Line {
                        text: spec.signpost.clone(),
                        color: crate::speaker::Speaker::System.color(),
                        speaker: Some(crate::speaker::Speaker::System),
                    });
                    self.pending_gate = Some(spec);
                    out.push(Output::AwaitInput);
                    return out;
                }
            }
        }
        out
    }

    fn eval_condition(&self, c: &crate::scene::Condition) -> bool {
        use crate::scene::Condition;
        match c {
            Condition::Knows(t) => {
                let key = crate::vocab::canonical(t)
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| t.to_lowercase());
                self.discovered.contains(&key)
            }
            Condition::Visited(id) => {
                let id = crate::vocab::canonical(id)
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| id.to_lowercase());
                self.visited.contains(&id)
            }
            Condition::Asked(loc, topic) => {
                let loc = crate::vocab::canonical(loc)
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| loc.to_lowercase());
                let topic = crate::vocab::canonical(topic)
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| topic.to_lowercase());
                self.asked.get(&loc).is_some_and(|s| s.contains(&topic))
            }
            Condition::DecodeUnlocked => self.decoding_unlocked,
        }
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
    fn unknown_command_gives_feedback_then_reprompts() {
        use crate::content;
        let c = content::from_str(&[r#"ContentPart(
            locations: { "hub": ( prompt:[Line(Smobb,"MENU")], commands: [] ) },
            scenes: {},
        )"#]);
        let mut e = Engine::new(c);
        let _ = e.start();
        let out = e.advance("zzzz");
        assert!(out.iter().any(|o| matches!(o, Output::Line{text,..}
            if text.contains("zzzz") && text.to_lowercase().contains("options"))));
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
    fn prompt_matches_affirmative_aliases() {
        use crate::content;
        let c = content::from_str(&[r#"ContentPart(
            locations: { "hub": ( commands: [ Branch(keys:["go"], target: Steps([
                Prompt([
                    Branch(keys:["yes"], target: Steps([ Line(Smobb,"YESPATH"), Return ])),
                    Branch(keys:[], target: Steps([ Line(Smobb,"DEFAULT"), Return ])),
                ]),
            ])) ] ) },
            scenes: {},
        )"#]);
        let mut e = Engine::new(c);
        e.set_skip(true);
        let _ = e.start();
        let _ = e.advance("go");
        let out = e.advance("yeah"); // alias of "yes" — must take YESPATH, not DEFAULT
        assert!(out
            .iter()
            .any(|o| matches!(o, Output::Line{text,..} if text == "YESPATH")));
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
        let out = e.advance("end"); // hub "Quit" option asks first
        assert!(!out.contains(&Output::End));
        assert!(out
            .iter()
            .any(|o| matches!(o, Output::Line{text,..} if text.contains("CLOSE S.M.O.B.B.?"))));
        let out = e.advance("yes"); // confirm
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
    fn back_pops_location_and_quit_confirms_at_hub() {
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
        let out = e.advance("back"); // navigation: leave ali -> hub, never closes
        assert!(out
            .iter()
            .any(|o| matches!(o, Output::Line{text,..} if text=="HUB")));
        let out = e.advance("quit"); // closing the game asks first
        assert!(!out.contains(&Output::End));
        let out = e.advance("yes"); // confirm
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
    fn plain_content_commands_and_menu_numbers_are_not_passwords() {
        use crate::content;
        // A dialogue command ("game") and a story-menu number ("1") are unlabeled
        // and non-meta, but must NOT be recorded as discovered passwords. A real
        // code ("k1ll") on an identical branch shape still is.
        let c = content::from_str(&[r#"ContentPart(
            locations: { "hub": ( commands: [
                Branch(keys:["game"], target: Steps([ Line(System, "ha") ])),
                Branch(keys:["1"], target: Steps([ Line(System, "story") ])),
                Branch(keys:["k1ll"], target: Steps([ Line(System, "secret") ])),
            ] ) },
            scenes: {},
        )"#]);
        let mut e = Engine::new(c);
        e.set_skip(true);
        let _ = e.start();
        let _ = e.advance("game");
        let _ = e.advance("1");
        let _ = e.advance("k1ll");
        assert!(!e.discovered().contains("game"), "'game' is not a password");
        assert!(
            !e.discovered().contains("1"),
            "menu number is not a password"
        );
        assert!(e.discovered().contains("k1ll"), "'k1ll' is a secret code");
    }

    #[test]
    fn restore_drops_stale_non_code_passwords() {
        use crate::content;
        // A save written by an older build may hold junk ("game") in discovered;
        // restoring it must keep real codes and drop the junk.
        let mut e = Engine::new(content::load());
        let save = crate::save::SaveState {
            discovered: vec!["game".into(), "7549009".into(), "k1ll".into()],
            ..Default::default()
        };
        e.restore(save);
        assert!(!e.discovered().contains("game"));
        assert!(e.discovered().contains("7549009"));
        assert!(e.discovered().contains("k1ll"));
    }

    #[test]
    fn is_secret_code_classifies_codes_and_words() {
        assert!(is_secret_code("7549009")); // long numeric password
        assert!(is_secret_code("k1ll")); // leetspeak
        assert!(is_secret_code("!save")); // symbolic
        assert!(is_secret_code("<---------->")); // symbolic
        assert!(is_secret_code("end7312")); // alnum mix
        assert!(!is_secret_code("game")); // plain word
        assert!(!is_secret_code("mapping")); // plain word
        assert!(!is_secret_code("1")); // menu selector
        assert!(!is_secret_code("12")); // menu selector
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
        assert!(out.iter().any(|o| matches!(
            o,
            Output::Line {
                speaker: Some(Speaker::Ali),
                ..
            }
        )));
        assert!(e
            .journal_view()
            .iter()
            .all(|o| matches!(o, Output::Line { speaker: None, .. })));
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
        assert!(out
            .iter()
            .any(|o| matches!(o, Output::Effect(EffectKind::Flash(_)))));
    }

    #[test]
    fn reduced_effects_round_trips_through_snapshot() {
        use crate::content;
        let mut e = Engine::new(content::from_str(&[
            r#"ContentPart(locations:{"hub":(commands:[])},scenes:{})"#,
        ]));
        e.set_reduced_effects(true);
        let snap = e.snapshot();
        assert!(snap.reduced_effects);
        let snap2 = crate::save::SaveState::from_json(&snap.to_json()).unwrap();
        let mut e2 = Engine::new(content::from_str(&[
            r#"ContentPart(locations:{"hub":(commands:[])},scenes:{})"#,
        ]));
        e2.restore(snap2);
        assert!(e2.reduced_effects());
    }

    #[test]
    fn label_word_enters_branch_keyed_by_sibling_alias() {
        use crate::content;
        // Branch is keyed only "extra"; typing the displayed label "stories" must enter it.
        let c = content::from_str(&[r#"ContentPart(
            locations: {
                "hub": ( prompt:[Line(Smobb,"HUB")], commands: [ Branch(keys:["extra"], target: Steps([ Enter("extra") ])) ] ),
                "extra": ( prompt:[Line(Smobb,"STORIES")], commands: [] ),
            },
            scenes: {},
        )"#]);
        let mut e = Engine::new(c);
        e.set_skip(true);
        let _ = e.start();
        let out = e.advance("stories");
        assert!(out
            .iter()
            .any(|o| matches!(o, Output::Line{text,..} if text == "STORIES")));
    }

    #[test]
    fn password_branch_still_requires_exact_match() {
        use crate::content;
        let c = content::from_str(&[r#"ContentPart(
            locations: { "hub": ( commands: [ Branch(keys:["7549009"], target: Steps([ Line(System,"ok") ])) ] ) },
            scenes: {},
        )"#]);
        let mut e = Engine::new(c);
        e.set_skip(true);
        let _ = e.start();
        let out = e.advance("754");
        assert!(!out
            .iter()
            .any(|o| matches!(o, Output::Line{text,..} if text == "ok")));
        let out = e.advance("7549009");
        assert!(out
            .iter()
            .any(|o| matches!(o, Output::Line{text,..} if text == "ok")));
    }

    #[test]
    fn display_name_comes_from_registry_with_capitalize_fallback() {
        assert_eq!(display_name("extra"), "Stories");
        assert_eq!(display_name("qassim"), "Qassim");
        assert_eq!(display_name("smobb"), "S.M.O.B.B.");
        assert_eq!(display_name("guide_names"), "Names");
        assert_eq!(display_name("qasim"), "Qassim"); // alias resolves to canonical label
                                                     // unknown id still capitalizes
        assert_eq!(display_name("tranava"), "Tranava");
    }

    #[test]
    fn entering_via_alias_records_canonical_visit() {
        use crate::content;
        // Branch keyed "qasim" enters location "qassim"; visited must store canonical "qassim".
        let c = content::from_str(&[r#"ContentPart(
            locations: {
                "hub": ( commands: [ Branch(keys:["qasim"], target: Steps([ Enter("qassim") ])) ] ),
                "qassim": ( commands: [ Branch(keys:["end"], target: Steps([ Return ])) ] ),
            },
            scenes: {},
        )"#]);
        let mut e = Engine::new(c);
        e.set_skip(true);
        let _ = e.start();
        let _ = e.advance("qasim");
        assert!(e.visited().contains("qassim"));
    }

    #[test]
    fn if_step_branches_on_knows() {
        use crate::content;
        // Branch "withpw" learns the password (Recap) then If(Knows) -> KNOWN.
        // Branch "nopw" has not learned it -> UNKNOWN.
        let c = content::from_str(&[r#"ContentPart(
            locations: { "hub": ( commands: [
                Branch(keys:["withpw"], target: Steps([
                    Recap("The password for the Tranava files is '7549009'."),
                    If(Knows("7549009"), [ Line(System,"KNOWN") ], [ Line(System,"UNKNOWN") ]),
                    Return,
                ])),
                Branch(keys:["nopw"], target: Steps([
                    If(Knows("7549009"), [ Line(System,"KNOWN") ], [ Line(System,"UNKNOWN") ]),
                    Return,
                ])),
            ] ) },
            scenes: {},
        )"#]);
        let mut e = Engine::new(c);
        e.set_skip(true);
        let _ = e.start();
        let out = e.advance("nopw");
        assert!(out
            .iter()
            .any(|o| matches!(o, Output::Line{text,..} if text == "UNKNOWN")));
        let out = e.advance("withpw");
        assert!(out
            .iter()
            .any(|o| matches!(o, Output::Line{text,..} if text == "KNOWN")));
    }

    #[test]
    fn gate_emits_signpost_then_awaits() {
        use crate::content;
        let c = content::from_str(&[r#"ContentPart(
        locations: { "hub": ( commands: [ Branch(keys:["go"], target: Steps([
            Gate(GateSpec(
                code: "<---------->",
                signpost: "BREACH DETECTED — deploy prevention? (yes)",
                on_deploy: [ Line(System, "STOPPED"), Return ],
                on_missing: [ Line(System, "NO CODE"), Return ],
                on_decline: [ Line(System, "IGNORED"), Return ],
            )),
        ])) ] ) },
        scenes: {},
    )"#]);
        let mut e = Engine::new(c);
        e.set_skip(true);
        let _ = e.start();
        let out = e.advance("go");
        assert!(out
            .iter()
            .any(|o| matches!(o, Output::Line{text,..} if text.contains("BREACH DETECTED"))));
        assert!(matches!(out.last().unwrap(), Output::AwaitInput));
        // not yet resolved: none of the outcome lines emitted
        assert!(!out.iter().any(|o| matches!(o, Output::Line{text,..}
            if text == "STOPPED" || text == "NO CODE" || text == "IGNORED")));
    }

    #[test]
    fn branch_label_identifies_character_and_structural_via_registry() {
        use crate::content;
        let c = content::from_str(&[r#"ContentPart(
            locations: { "hub": ( commands: [
                Branch(keys:["bader"], target: Steps([ Line(Ali,"about bader") ])),
                Branch(keys:["stories"], target: Steps([ Enter("extra") ])),
            ] ) },
            scenes: {},
        )"#]);
        let mut e = Engine::new(c);
        let _ = e.start();
        let joined: String = e
            .options_view()
            .iter()
            .filter_map(|o| match o {
                Output::Line { text, .. } => Some(text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert!(joined.contains("Ask about Bader"));
        assert!(joined.contains("Stories"));
    }

    fn gate_fixture() -> crate::scene::Content {
        crate::content::from_str(&[r#"ContentPart(
            locations: { "hub": ( prompt:[Line(Smobb,"MENU")], commands: [
                Branch(keys:["learn"], target: Steps([
                    Recap("The breach prevention code is '<---------->'."),
                ])),
                Branch(keys:["breach"], target: Steps([
                    Gate(GateSpec(
                        code: "<---------->",
                        signpost: "BREACH — deploy prevention? (yes)",
                        on_deploy: [ Line(System, "STOPPED"), Return ],
                        on_missing: [ Line(System, "NO CODE"), Return ],
                        on_decline: [ Line(System, "IGNORED"), Return ],
                    )),
                ])),
            ] ) },
            scenes: {},
        )"#])
    }

    #[test]
    fn gate_deploys_when_known_and_affirmative() {
        let mut e = Engine::new(gate_fixture());
        e.set_skip(true);
        let _ = e.start();
        let _ = e.advance("learn"); // discover the code
        let _ = e.advance("breach"); // signpost + await
        let out = e.advance("yes");
        assert!(out
            .iter()
            .any(|o| matches!(o, Output::Line{text,..} if text == "STOPPED")));
    }

    #[test]
    fn gate_missing_routes_to_decline_no_warning() {
        // Affirmative without the prevention code: no warning path. Go straight
        // to the breach-not-deployed (decline) outcome.
        let mut e = Engine::new(gate_fixture());
        e.set_skip(true);
        let _ = e.start();
        let _ = e.advance("breach"); // never learned the code
        let out = e.advance("yes");
        assert!(out
            .iter()
            .any(|o| matches!(o, Output::Line{text,..} if text == "IGNORED")));
        // the warning path must NOT fire
        assert!(!out
            .iter()
            .any(|o| matches!(o, Output::Line{text,..} if text == "NO CODE")));
    }

    #[test]
    fn gate_declines_on_negative() {
        let mut e = Engine::new(gate_fixture());
        e.set_skip(true);
        let _ = e.start();
        let _ = e.advance("learn");
        let _ = e.advance("breach");
        let out = e.advance("no");
        assert!(out
            .iter()
            .any(|o| matches!(o, Output::Line{text,..} if text == "IGNORED")));
    }

    #[test]
    fn gate_literal_code_deploys() {
        let mut e = Engine::new(gate_fixture());
        e.set_skip(true);
        let _ = e.start();
        let _ = e.advance("breach"); // even without "learn", typing the code proves knowledge
        let out = e.advance("<---------->");
        assert!(out
            .iter()
            .any(|o| matches!(o, Output::Line{text,..} if text == "STOPPED")));
    }

    #[test]
    fn gate_reawaits_on_unrecognized_input() {
        let mut e = Engine::new(gate_fixture());
        e.set_skip(true);
        let _ = e.start();
        let _ = e.advance("breach");
        let out = e.advance("banana");
        // re-emits signpost, still awaiting, no outcome line
        assert!(out
            .iter()
            .any(|o| matches!(o, Output::Line{text,..} if text.contains("BREACH"))));
        assert!(matches!(out.last().unwrap(), Output::AwaitInput));
        assert!(!out.iter().any(|o| matches!(o, Output::Line{text,..}
            if text == "STOPPED" || text == "NO CODE" || text == "IGNORED")));
    }

    #[test]
    fn gate_universal_escape_aborts_to_menu() {
        let mut e = Engine::new(gate_fixture());
        e.set_skip(true);
        let _ = e.start();
        let _ = e.advance("breach");
        let out = e.advance("leave");
        assert!(out
            .iter()
            .any(|o| matches!(o, Output::Line{text,..} if text == "MENU")));
        assert!(matches!(out.last().unwrap(), Output::AwaitInput));
    }

    #[test]
    fn snapshot_works_with_open_gate() {
        let mut e = Engine::new(gate_fixture());
        e.set_skip(true);
        let _ = e.start();
        let _ = e.advance("learn");
        let _ = e.advance("breach"); // gate now open
        let snap = e.snapshot();
        assert!(snap.discovered.contains(&"<---------->".to_string()));
        let snap2 = crate::save::SaveState::from_json(&snap.to_json()).unwrap();
        assert_eq!(snap, snap2);
    }

    #[test]
    fn if_step_visited_matches_via_alias() {
        use crate::content;
        let c = content::from_str(&[r#"ContentPart(
            locations: {
                "hub": ( commands: [
                    Branch(keys:["go"], target: Steps([ Enter("qassim") ])),
                    Branch(keys:["check"], target: Steps([
                        If(Visited("qasim"), [ Line(System,"SEEN") ], [ Line(System,"UNSEEN") ]),
                        Return,
                    ])),
                ] ),
                "qassim": ( commands: [ Branch(keys:["end"], target: Steps([ Return ])) ] ),
            },
            scenes: {},
        )"#]);
        let mut e = Engine::new(c);
        e.set_skip(true);
        let _ = e.start();
        let out = e.advance("check"); // not visited yet
        assert!(out
            .iter()
            .any(|o| matches!(o, Output::Line{text,..} if text == "UNSEEN")));
        let _ = e.advance("go"); // enter qassim (canonical recorded in visited)
        let _ = e.advance("end"); // back to hub
        let out = e.advance("check"); // alias "qasim" must canonicalize to "qassim"
        assert!(out
            .iter()
            .any(|o| matches!(o, Output::Line{text,..} if text == "SEEN")));
    }

    #[test]
    fn if_decode_unlocked_branches_correctly() {
        use crate::content;
        let c = content::from_str(&[r#"ContentPart(
            locations: { "hub": ( commands: [
                Branch(keys:["unlock"], target: Steps([
                    UnlockDecode,
                    If(DecodeUnlocked, [ Line(System,"DEC") ], [ Line(System,"RAW") ]),
                    Return,
                ])),
                Branch(keys:["check"], target: Steps([
                    If(DecodeUnlocked, [ Line(System,"DEC") ], [ Line(System,"RAW") ]),
                    Return,
                ])),
            ] ) },
            scenes: {},
        )"#]);
        let mut e = Engine::new(c);
        e.set_skip(true);
        let _ = e.start();
        let out = e.advance("check"); // not yet unlocked
        assert!(out
            .iter()
            .any(|o| matches!(o, Output::Line{text,..} if text == "RAW")));
        let out = e.advance("unlock"); // unlock then check
        assert!(out
            .iter()
            .any(|o| matches!(o, Output::Line{text,..} if text == "DEC")));
    }

    #[test]
    fn if_step_asked_matches_via_alias() {
        use crate::content;
        // Ask about "qasim" (alias) at "ali" — the engine stores canonical "qassim".
        // Then If(Asked("ali","qasim"), ...) must canonicalize to "qassim" and be true.
        // Note: "check" has no Return so we stay in "ali" between calls.
        let c = content::from_str(&[r#"ContentPart(
            locations: {
                "hub": ( commands: [
                    Branch(keys:["go"], target: Steps([ Enter("ali") ])),
                ] ),
                "ali": ( commands: [
                    Branch(keys:["qasim"], target: Steps([ Line(Ali,"about qassim") ])),
                    Branch(keys:["check"], target: Steps([
                        If(Asked("ali","qasim"), [ Line(System,"WAS ASKED") ], [ Line(System,"NOT ASKED") ]),
                    ])),
                ] ),
            },
            scenes: {},
        )"#]);
        let mut e = Engine::new(c);
        e.set_skip(true);
        let _ = e.start();
        let _ = e.advance("go"); // enter ali
        let out = e.advance("check"); // not yet asked
        assert!(out
            .iter()
            .any(|o| matches!(o, Output::Line{text,..} if text == "NOT ASKED")));
        let _ = e.advance("qasim"); // ask about qasim (alias -> canonical "qassim" stored)
        let out = e.advance("check"); // alias in condition must match
        assert!(out
            .iter()
            .any(|o| matches!(o, Output::Line{text,..} if text == "WAS ASKED")));
    }

    #[test]
    fn gate_deploys_when_known_and_affirmative_alias() {
        // "yeah" is an alias of "yes"; after learning the code, saying "yeah" must deploy.
        let mut e = Engine::new(gate_fixture());
        e.set_skip(true);
        let _ = e.start();
        let _ = e.advance("learn"); // discover the code
        let _ = e.advance("breach"); // signpost + await
        let out = e.advance("yeah"); // affirmative alias -> STOPPED
        assert!(out
            .iter()
            .any(|o| matches!(o, Output::Line{text,..} if text == "STOPPED")));
    }

    #[test]
    fn options_command_not_recorded_in_asked() {
        use crate::content;
        let c = content::from_str(&[r#"ContentPart(
            locations: {
                "hub": ( commands: [ Branch(keys:["go"], target: Steps([ Enter("g") ])) ] ),
                "g": ( prompt:[Line(System,"?")], commands: [
                    Branch(keys:["options","choices","choice"], target: Steps([ Line(System,"OPTS") ])),
                ] ),
            },
            scenes: {},
        )"#]);
        let mut e = Engine::new(c);
        e.set_skip(true);
        let _ = e.start();
        let _ = e.advance("go");
        let _ = e.advance("options");
        let jrnl: String = e
            .journal_view()
            .iter()
            .filter_map(|o| match o {
                Output::Line { text, .. } => Some(text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            !jrnl.contains("options"),
            "meta-command 'options' must not appear in [Asked]"
        );
    }

    #[test]
    fn quit_label_is_typeable_at_hub() {
        use crate::content;
        let c = content::from_str(&[r#"ContentPart(
            locations: { "hub": ( prompt:[Line(Smobb,"HUB")], commands: [
                Branch(keys:["end"], target: Steps([ Quit ])),
            ] ) },
            scenes: {},
        )"#]);
        let mut e = Engine::new(c);
        e.set_skip(true);
        let _ = e.start();
        // "quit" asks for confirmation; only "yes" actually closes.
        let out = e.advance("quit");
        assert!(!out.contains(&Output::End));
        assert!(out
            .iter()
            .any(|o| matches!(o, Output::Line{text,..} if text.contains("CLOSE S.M.O.B.B.?"))));
        let out = e.advance("yes");
        assert!(out.contains(&Output::End));
    }

    #[test]
    fn quit_confirmation_declines_to_menu() {
        use crate::content;
        let c = content::from_str(&[r#"ContentPart(
            locations: { "hub": ( prompt:[Line(Smobb,"HUB")], commands: [
                Branch(keys:["end"], target: Steps([ Quit ])),
            ] ) },
            scenes: {},
        )"#]);
        let mut e = Engine::new(c);
        e.set_skip(true);
        let _ = e.start();
        let _ = e.advance("quit"); // ask
        let out = e.advance("no"); // decline -> back to the hub menu, game stays open
        assert!(!out.contains(&Output::End));
        assert!(out
            .iter()
            .any(|o| matches!(o, Output::Line{text,..} if text=="HUB")));
    }

    #[test]
    fn leave_at_hub_root_hints_instead_of_quitting() {
        use crate::content;
        let c = content::from_str(&[r#"ContentPart(
            locations: { "hub": ( prompt:[Line(Smobb,"HUB")], commands: [] ) },
            scenes: {},
        )"#]);
        let mut e = Engine::new(c);
        e.set_skip(true);
        let _ = e.start();
        let out = e.advance("leave"); // nothing to leave at the root
        assert!(!out.contains(&Output::End));
        assert!(out
            .iter()
            .any(|o| matches!(o, Output::Line{text,..} if text.contains("TYPE 'QUIT'"))));
        assert!(matches!(out.last().unwrap(), Output::AwaitInput));
    }
}
