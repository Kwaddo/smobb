use smobb_core::engine::{Engine, Output};
use smobb_core::speaker::Color;
use std::collections::VecDeque;
use web_time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Options,
    Journal,
}

#[derive(Debug, Clone)]
pub struct Span {
    pub text: String,
    pub color: Color,
    pub dim: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Enter,
    Backspace,
    Tab,
    F1,
    F2,
    Up,
    Down,
    PageUp,
    PageDown,
    Esc,
    CtrlC,
    Shift,
}

/// While the skip key is held, each line's wait is capped to this (ms) — fast
/// but not instant.
const FAST_STEP_MS: u64 = 40;
/// How long a single skip-key press keeps fast mode active (ms). Sized to bridge
/// the terminal's key auto-repeat gap so holding feels continuous; on release,
/// normal pacing resumes after at most this long.
const FAST_WINDOW_MS: u64 = 400;

pub struct App {
    engine: Engine,
    transcript: Vec<Span>,
    queue: VecDeque<Output>,
    reveal_at: Option<Instant>,
    tab: Tab,
    sidebar_visible: bool,
    scroll: u16,
    user_scrolled: bool,
    input: String,
    awaiting: bool,
    quit: bool,
    fast_until: Option<Instant>,
    fast_requested: bool,
    dirty: bool,
}

impl App {
    pub fn new(mut engine: Engine) -> App {
        let outputs = engine.start();
        let mut app = App {
            engine,
            transcript: Vec::new(),
            queue: VecDeque::new(),
            reveal_at: None,
            tab: Tab::Options,
            sidebar_visible: true,
            scroll: 0,
            user_scrolled: false,
            input: String::new(),
            awaiting: false,
            quit: false,
            fast_until: None,
            fast_requested: false,
            dirty: false,
        };
        app.enqueue(outputs);
        app
    }

    /// Snapshot of persistable progress (journal + settings).
    pub fn snapshot(&self) -> smobb_core::save::SaveState {
        self.engine.snapshot()
    }

    /// True if progress changed since the last `clear_dirty` — the frontend
    /// should persist a fresh snapshot.
    pub fn dirty(&self) -> bool {
        self.dirty
    }
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    fn enqueue(&mut self, outputs: Vec<Output>) {
        self.queue.extend(outputs);
        self.awaiting = false;
        self.reveal_at = None;
        self.user_scrolled = false;
        self.fast_until = None;
        self.fast_requested = false;
    }

    pub fn quit(&self) -> bool {
        self.quit
    }
    pub fn transcript(&self) -> &[Span] {
        &self.transcript
    }
    #[cfg(test)]
    pub fn awaiting(&self) -> bool {
        self.awaiting
    }

    pub fn tick(&mut self, now: Instant) {
        if self.awaiting || self.quit {
            return;
        }
        // A skip-key press (held → repeated presses) opens a short fast window.
        if self.fast_requested {
            self.fast_until = Some(now + std::time::Duration::from_millis(FAST_WINDOW_MS));
            self.fast_requested = false;
        }
        let fast = self.fast_until.is_some_and(|u| now < u);
        // While fast, collapse any already-pending wait so a held skip is responsive.
        if fast {
            let cap = now + std::time::Duration::from_millis(FAST_STEP_MS);
            if let Some(at) = self.reveal_at {
                if at > cap {
                    self.reveal_at = Some(cap);
                }
            }
        }
        loop {
            if let Some(at) = self.reveal_at {
                if now < at {
                    return;
                }
            }
            let item = match self.queue.pop_front() {
                Some(i) => i,
                None => {
                    self.reveal_at = None;
                    return;
                }
            };
            match item {
                Output::Line { text, color, .. } => {
                    self.transcript.push(Span {
                        text,
                        color,
                        dim: false,
                    });
                    self.reveal_at = None;
                }
                Output::Translation(t) => {
                    self.transcript.push(Span {
                        text: format!("  \u{21b7} \"{}\"", t),
                        color: Color::White,
                        dim: true,
                    });
                    self.reveal_at = None;
                }
                Output::Sleep(ms) => {
                    let wait = if fast {
                        (ms as u64).min(FAST_STEP_MS)
                    } else {
                        ms as u64
                    };
                    self.reveal_at = Some(now + std::time::Duration::from_millis(wait));
                    return;
                }
                Output::Recap(_) => {
                    self.transcript.push(Span {
                        text: "  \u{270e} noted".to_string(),
                        color: Color::White,
                        dim: true,
                    });
                    self.reveal_at = None;
                }
                Output::AwaitInput => {
                    self.awaiting = true;
                    return;
                }
                Output::End => {
                    self.quit = true;
                    return;
                }
                Output::Effect(_) => {
                    self.reveal_at = None;
                }
            }
        }
    }

    pub fn submit(&mut self) {
        if !self.awaiting {
            return;
        }
        let cmd = std::mem::take(&mut self.input);
        let outputs = self.engine.advance(&cmd);
        self.enqueue(outputs);
        self.dirty = true; // a command may have changed journal state
    }

    /// Instant drain of the reveal queue (no pacing). Test-only bulk-fill helper;
    /// the interactive skip key uses the fast-forward path in `tick` instead.
    #[cfg(test)]
    pub fn skip_reveal(&mut self) {
        if self.awaiting || self.quit {
            return;
        }
        self.reveal_at = None;
        while let Some(item) = self.queue.pop_front() {
            match item {
                Output::Line { text, color, .. } => self.transcript.push(Span {
                    text,
                    color,
                    dim: false,
                }),
                Output::Translation(t) => self.transcript.push(Span {
                    text: format!("  \u{21b7} \"{}\"", t),
                    color: Color::White,
                    dim: true,
                }),
                Output::Recap(_) => self.transcript.push(Span {
                    text: "  \u{270e} noted".to_string(),
                    color: Color::White,
                    dim: true,
                }),
                Output::Sleep(_) => {} // ignore timing
                Output::AwaitInput => {
                    self.awaiting = true;
                    return;
                }
                Output::End => {
                    self.quit = true;
                    return;
                }
                Output::Effect(_) => {} // ignored until Task 5
            }
        }
    }

    fn type_char(&mut self, c: char) {
        if self.awaiting {
            self.input.push(c);
        }
    }
    fn backspace(&mut self) {
        if self.awaiting {
            self.input.pop();
        }
    }

    pub fn key(&mut self, key: Key) {
        match key {
            Key::CtrlC | Key::Esc => self.quit = true,
            Key::Enter => self.submit(),
            Key::Backspace => self.backspace(),
            Key::Tab => {
                self.tab = match self.tab {
                    Tab::Options => Tab::Journal,
                    Tab::Journal => Tab::Options,
                }
            }
            Key::F1 => self.sidebar_visible = !self.sidebar_visible,
            Key::F2 => {
                let s = self.engine.skip();
                self.engine.set_skip(!s);
                self.dirty = true; // persist the skip-mode setting
            }
            Key::Up => {
                self.user_scrolled = true;
                self.scroll = self.scroll.saturating_sub(1);
            }
            Key::Down => self.scroll = self.scroll.saturating_add(1),
            Key::PageUp => {
                self.user_scrolled = true;
                self.scroll = self.scroll.saturating_sub(5);
            }
            Key::PageDown => self.scroll = self.scroll.saturating_add(5),
            Key::Shift => self.fast_requested = true, // hold Shift = fast-forward (see tick)
            Key::Char(c) => {
                if self.awaiting {
                    self.type_char(c);
                } else if c == ' ' {
                    // hold Space also fast-forwards (reliable everywhere; Shift is a
                    // bonus on terminals that deliver it)
                    self.fast_requested = true;
                }
            }
        }
    }

    pub fn tab(&self) -> Tab {
        self.tab
    }
    pub fn sidebar_visible(&self) -> bool {
        self.sidebar_visible
    }
    pub fn scroll(&self) -> u16 {
        self.scroll
    }
    pub fn user_scrolled(&self) -> bool {
        self.user_scrolled
    }
    pub fn input(&self) -> &str {
        &self.input
    }
    pub fn options_lines(&self) -> Vec<Output> {
        self.engine.options_view()
    }
    pub fn journal_lines(&self) -> Vec<Output> {
        self.engine.journal_view()
    }

    #[cfg(test)]
    pub fn submit_for_test(&mut self, s: &str) {
        self.input = s.to_string();
        self.submit();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use smobb_core::content;
    use smobb_core::engine::Engine;

    fn app_fixture() -> App {
        let c = content::from_str(&[r#"ContentPart(
            locations: { "hub": ( intro: [ Line(Smobb, "HELLO.") ],
                prompt: [ Line(Smobb, "HOW CAN I HELP?") ],
                commands: [ Branch(keys:["go"], target: Steps([ Line(Ali,"hi"), Return ])) ] ) },
            scenes: {},
        )"#]);
        let mut engine = Engine::new(c);
        engine.set_skip(true);
        App::new(engine)
    }

    #[test]
    fn new_seeds_queue_and_is_not_quit() {
        let app = app_fixture();
        assert!(!app.quit());
        // transcript empty until tick reveals; queue has the start() outputs
        assert!(app.transcript().is_empty());
    }

    #[test]
    fn tick_reveals_lines_until_await() {
        use web_time::Instant;
        let mut app = app_fixture();
        let t0 = Instant::now();
        app.tick(t0); // should drain intro + prompt lines up to AwaitInput
                      // "HELLO." and "HOW CAN I HELP?" revealed; now awaiting input
        let texts: Vec<String> = app.transcript().iter().map(|s| s.text.clone()).collect();
        assert!(texts.iter().any(|t| t == "HELLO."));
        assert!(texts.iter().any(|t| t == "HOW CAN I HELP?"));
        assert!(app.awaiting());
    }

    #[test]
    fn submit_runs_command_and_reveals() {
        use smobb_core::content;
        use smobb_core::engine::Engine;
        use web_time::Instant;
        let c = content::from_str(&[r#"ContentPart(
            locations: { "hub": ( prompt:[Line(Smobb,"P")],
                commands: [ Branch(keys:["go"], target: Steps([ Line(Ali,"hi"), Return ])) ] ) },
            scenes: {},
        )"#]);
        let mut app = App::new(Engine::new(c));
        app.tick(Instant::now()); // reach awaiting at hub
        assert!(app.awaiting());
        app.submit_for_test("go"); // run the "go" command
        app.tick(Instant::now()); // reveal its output ("hi") + return to hub prompt
        let texts: Vec<String> = app.transcript().iter().map(|s| s.text.clone()).collect();
        assert!(texts.iter().any(|t| t == "hi"));
    }

    #[test]
    fn skip_reveal_drains_without_waiting() {
        use smobb_core::content;
        use smobb_core::engine::Engine;
        use web_time::Instant;
        let c = content::from_str(&[r#"ContentPart(
            locations: { "hub": ( prompt:[Line(Smobb,"P")],
                commands: [ Branch(keys:["go"], target: Steps([ Line(Ali,"a"), Line(Ali,"b"), Return ])) ] ) },
            scenes: {},
        )"#]);
        let mut app = App::new(Engine::new(c));
        app.tick(Instant::now());
        app.submit_for_test("go");
        app.skip_reveal(); // should reveal a, b and return-prompt instantly
        let texts: Vec<String> = app.transcript().iter().map(|s| s.text.clone()).collect();
        assert!(texts.contains(&"a".to_string()) && texts.contains(&"b".to_string()));
        assert!(app.awaiting());
    }

    #[test]
    fn tick_honors_sleep_timing() {
        use smobb_core::content;
        use smobb_core::engine::Engine;
        use std::time::Duration;
        use web_time::Instant;
        // Content with two lines; skip=off so Sleep is emitted after each Line (default interval 1500ms)
        let c = content::from_str(&[r#"ContentPart(
            locations: { "hub": ( prompt:[Line(Smobb,"P")],
                commands: [ Branch(keys:["go"], target: Steps([ Line(Ali,"line1"), Line(Ali,"line2"), Return ])) ] ) },
            scenes: {},
        )"#]);
        let mut app = App::new(Engine::new(c));
        let t0 = Instant::now();
        app.tick(t0);
        assert!(app.awaiting());

        // Submit the "go" command
        app.submit_for_test("go");

        // First tick(t0) should reveal line1 then hit Sleep(1500)
        app.tick(t0);
        let texts: Vec<String> = app.transcript().iter().map(|s| s.text.clone()).collect();
        assert!(texts.contains(&"line1".to_string()));
        assert!(!texts.contains(&"line2".to_string())); // line2 not revealed yet

        // Second tick(t0) should NOT reveal line2 yet (still within Sleep timeout)
        app.tick(t0);
        let texts: Vec<String> = app.transcript().iter().map(|s| s.text.clone()).collect();
        assert!(!texts.contains(&"line2".to_string()));

        // tick(t0 + 5s) should reveal line2 (Sleep duration was 1500ms, so 5000ms > 1500ms)
        app.tick(t0 + Duration::from_secs(5));
        let texts: Vec<String> = app.transcript().iter().map(|s| s.text.clone()).collect();
        assert!(texts.contains(&"line2".to_string()));

        // tick(t0 + 14s) should reveal the hub prompt and return to awaiting
        // (1500ms after line1, 1500ms after line2, 1500ms after hub prompt line)
        app.tick(t0 + Duration::from_secs(14));
        assert!(app.awaiting()); // should be back at hub waiting for input
    }

    #[test]
    fn holding_shift_fast_forwards_not_instant() {
        use smobb_core::content;
        use smobb_core::engine::Engine;
        use std::time::Duration;
        use web_time::Instant;
        // skip=off so the engine emits Sleep(1500) between lines.
        let c = content::from_str(&[r#"ContentPart(
            locations: { "hub": ( prompt:[Line(Smobb,"P")],
                commands: [ Branch(keys:["go"], target: Steps([ Line(Ali,"line1"), Line(Ali,"line2"), Return ])) ] ) },
            scenes: {},
        )"#]);
        let mut app = App::new(Engine::new(c));
        let t0 = Instant::now();
        app.tick(t0);
        app.submit_for_test("go");
        app.tick(t0); // line1 revealed, then Sleep -> reveal_at = t0 + 1500ms
        let texts: Vec<String> = app.transcript().iter().map(|s| s.text.clone()).collect();
        assert!(texts.contains(&"line1".to_string()) && !texts.contains(&"line2".to_string()));

        // Press the skip key (Shift, as when held). It must NOT instantly dump line2.
        app.key(Key::Shift);
        app.tick(t0); // promotes fast mode + caps the pending wait, but now == t0 < cap
        let texts: Vec<String> = app.transcript().iter().map(|s| s.text.clone()).collect();
        assert!(
            !texts.contains(&"line2".to_string()),
            "fast skip must not be instant on the keypress"
        );

        // Far sooner than the normal 1500ms, line2 reveals (fast-forward, ~40ms step).
        app.tick(t0 + Duration::from_millis(100));
        let texts: Vec<String> = app.transcript().iter().map(|s| s.text.clone()).collect();
        assert!(
            texts.contains(&"line2".to_string()),
            "fast skip should reveal the next line within ~40ms, not 1500ms"
        );
    }

    #[test]
    fn holding_space_also_fast_forwards() {
        use smobb_core::content;
        use smobb_core::engine::Engine;
        use std::time::Duration;
        use web_time::Instant;
        let c = content::from_str(&[r#"ContentPart(
            locations: { "hub": ( prompt:[Line(Smobb,"P")],
                commands: [ Branch(keys:["go"], target: Steps([ Line(Ali,"line1"), Line(Ali,"line2"), Return ])) ] ) },
            scenes: {},
        )"#]);
        let mut app = App::new(Engine::new(c));
        let t0 = Instant::now();
        app.tick(t0);
        app.submit_for_test("go");
        app.tick(t0); // line1 + Sleep -> reveal_at = t0 + 1500ms
        assert!(!app.transcript().iter().any(|s| s.text == "line2"));
        app.key(Key::Char(' ')); // hold Space while revealing -> fast
        app.tick(t0); // promote fast + cap the pending wait to ~40ms
        app.tick(t0 + Duration::from_millis(100));
        assert!(
            app.transcript().iter().any(|s| s.text == "line2"),
            "hold-Space should fast-forward like Shift"
        );
    }

    #[test]
    fn keys_toggle_tab_sidebar_and_quit() {
        let mut app = app_fixture();
        assert_eq!(app.tab(), Tab::Options);
        app.key(Key::Tab);
        assert_eq!(app.tab(), Tab::Journal);
        assert!(app.sidebar_visible());
        app.key(Key::F1);
        assert!(!app.sidebar_visible());
        app.key(Key::Esc);
        assert!(app.quit());
    }

    #[test]
    fn scrolling_up_sets_user_scrolled_and_enqueue_resets() {
        let mut app = app_fixture();
        app.tick(web_time::Instant::now()); // reveal intro + await
                                            // After initial state, user_scrolled starts false
        assert!(!app.user_scrolled());
        // submit_for_test -> enqueue -> user_scrolled reset to false
        app.submit_for_test("go");
        assert!(!app.user_scrolled());
        // drain output back to awaiting
        app.skip_reveal();
        // press Up -> user_scrolled becomes true
        app.key(Key::Up);
        assert!(app.user_scrolled());
        // another submit -> enqueue -> user_scrolled reset to false
        app.submit_for_test("go");
        assert!(!app.user_scrolled());
    }

    #[test]
    fn typing_and_enter_submits() {
        use web_time::Instant;
        let mut app = app_fixture();
        app.tick(Instant::now()); // awaiting
        for c in "go".chars() {
            app.key(Key::Char(c));
        }
        assert_eq!(app.input(), "go");
        app.key(Key::Enter);
        app.tick(Instant::now());
        assert!(app.transcript().iter().any(|s| s.text == "hi"));
        assert_eq!(app.input(), ""); // cleared on submit
    }
}
