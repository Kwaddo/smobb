use crate::app::App;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color as RColor, Modifier, Style},
    text::{Line, Span as RSpan},
    widgets::{Block, Paragraph, Wrap},
    Frame,
};
use smobb_core::engine::Output;
use smobb_core::speaker::Color;

const GLITCH_GLYPHS: &[char] = &['#', '%', '@', '&', '░', '▒', '▓', '*'];

/// Body split: story / sidebar column widths in percent. Exposed so frontends
/// (e.g. the desktop wheel hit-test) can map a cursor x to the pane under it
/// without duplicating the split.
pub const STORY_PCT: u16 = 70;
pub const SIDEBAR_PCT: u16 = 30;

/// Border style for a pane, brightened when it currently has scroll focus.
fn pane_border_style(focused: bool) -> Style {
    if focused {
        Style::default()
            .fg(RColor::Rgb(180, 220, 255))
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(RColor::Rgb(90, 90, 90))
    }
}

/// Render the Options/Journal sidebar into `area`, applying the same
/// `line_count`-based scroll clamp the story uses. The sidebar defaults to the
/// top (read top-down) unless the user has scrolled.
fn render_sidebar(f: &mut Frame, app: &App, area: Rect) {
    use crate::app::Focus;
    let (title, outs) = match app.tab() {
        crate::app::Tab::Options => (" OPTIONS ", app.options_lines()),
        crate::app::Tab::Journal => (" JOURNAL ", app.journal_lines()),
    };
    let lines: Vec<Line> = outs.iter().map(sidebar_line).collect();
    let inner_w = area.width.saturating_sub(2);
    let inner_h = area.height.saturating_sub(2);
    let base = Paragraph::new(lines).wrap(Wrap { trim: false });
    let total = base.line_count(inner_w) as u16;
    let max_scroll = total.saturating_sub(inner_h);
    let y = if app.sidebar_user_scrolled() {
        app.sidebar_scroll().min(max_scroll)
    } else {
        0
    };
    let focused = app.scroll_focus() == Focus::Sidebar;
    let block = Block::bordered()
        .title(title)
        .border_style(pane_border_style(focused));
    f.render_widget(base.block(block).scroll((y, 0)), area);
}

/// Deterministically corrupt ~1/3 of glyphs (no RNG — seeded by position).
pub fn scramble(s: &str, seed: usize) -> String {
    s.chars()
        .enumerate()
        .map(|(i, c)| {
            if c.is_whitespace() {
                c
            } else if (i + seed) % 3 == 0 {
                GLITCH_GLYPHS[(i + seed) % GLITCH_GLYPHS.len()]
            } else {
                c
            }
        })
        .collect()
}

fn to_rgb(c: Color) -> RColor {
    let hex = c.hex(); // "#rrggbb"
    let r = hex
        .get(1..3)
        .and_then(|s| u8::from_str_radix(s, 16).ok())
        .unwrap_or(255);
    let g = hex
        .get(3..5)
        .and_then(|s| u8::from_str_radix(s, 16).ok())
        .unwrap_or(255);
    let b = hex
        .get(5..7)
        .and_then(|s| u8::from_str_radix(s, 16).ok())
        .unwrap_or(255);
    RColor::Rgb(r, g, b)
}

/// Render one sidebar/journal line, emphasising section headers.
pub fn sidebar_line(o: &Output) -> Line<'static> {
    match o {
        Output::Line { text, color, .. } => {
            let trimmed = text.trim_start();
            let is_header =
                trimmed.starts_with('[') || trimmed.starts_with("==") || trimmed.ends_with(':');
            let mut style = Style::default().fg(to_rgb(*color));
            if is_header {
                style = style.add_modifier(Modifier::BOLD);
            } else if trimmed.starts_with('-') || trimmed.starts_with('(') {
                style = style.add_modifier(Modifier::DIM);
            }
            Line::from(RSpan::styled(text.clone(), style))
        }
        _ => Line::from(""),
    }
}

/// Build the transcript as ratatui lines, inserting a dim speaker header line
/// (and a blank spacer) whenever the speaker changes. `System`/`None` lines get
/// no header. This breaks the wall of same-color text into readable turns.
/// When `glitch` is true, the last transcript line is scrambled.
pub fn story_lines(transcript: &[crate::app::Span], glitch: bool) -> Vec<Line<'static>> {
    use smobb_core::speaker::Speaker;
    let last = transcript.len().saturating_sub(1);
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut prev: Option<Speaker> = None;
    for (idx, s) in transcript.iter().enumerate() {
        let label = s.speaker.map(|sp| sp.label()).unwrap_or("");
        let changed = s.speaker != prev && !label.is_empty();
        if changed {
            if !lines.is_empty() {
                lines.push(Line::from(""));
            }
            lines.push(Line::from(RSpan::styled(
                label,
                Style::default()
                    .fg(RColor::Rgb(120, 120, 120))
                    .add_modifier(Modifier::DIM | Modifier::BOLD),
            )));
        } else if s.speaker == prev && s.speaker.is_some_and(|sp| sp.spaced_turns()) {
            // Same speaker continuing a turn. Caps-heavy voices (SMOBB) get a
            // blank spacer between each line so the turn reads as separate
            // sentences rather than one wall of capitals.
            lines.push(Line::from(""));
        }
        let mut style = Style::default().fg(to_rgb(s.color));
        if s.dim {
            style = style
                .add_modifier(Modifier::DIM)
                .fg(RColor::Rgb(160, 160, 160));
        }
        let text = if glitch && idx == last {
            scramble(&s.text, idx)
        } else {
            s.text.clone()
        };
        lines.push(Line::from(RSpan::styled(text, style)));
        if !label.is_empty() {
            prev = s.speaker;
        }
    }
    lines
}

/// Build a dramatic full-area banner: the text upper-cased and letter-spaced,
/// vertically centered. `height` is the available story-area inner height.
pub fn banner_lines(text: &str, height: u16) -> Vec<Line<'static>> {
    if height == 0 {
        return Vec::new();
    }
    let spaced: String = text
        .to_uppercase()
        .chars()
        .collect::<Vec<_>>()
        .chunks(1)
        .map(|c| c.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join(" ");
    let mut lines: Vec<Line<'static>> = Vec::new();
    let h = height as usize;
    let pad = h.saturating_sub(1) / 2;
    for _ in 0..pad {
        lines.push(Line::from(""));
    }
    lines.push(
        Line::from(RSpan::styled(
            spaced,
            Style::default().add_modifier(Modifier::BOLD),
        ))
        .alignment(ratatui::layout::Alignment::Center),
    );
    while (lines.len() as u16) < height {
        lines.push(Line::from(""));
    }
    lines
}

pub fn render(f: &mut Frame, app: &App) {
    let outer = Block::bordered().title(" S.M.O.B.B. ");
    let area = f.area();
    let inner = outer.inner(area);
    f.render_widget(outer, area);

    // rows: body (fill) + input (1)
    let rows = Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).split(inner);
    let body = rows[0];
    let input_area = rows[1];

    // body columns: story (70%) + sidebar (30%) when visible
    let (story_area, sidebar_area): (Rect, Option<Rect>) = if app.sidebar_visible() {
        let cols = Layout::horizontal([
            Constraint::Percentage(STORY_PCT),
            Constraint::Percentage(SIDEBAR_PCT),
        ])
        .split(body);
        (cols[0], Some(cols[1]))
    } else {
        (body, None)
    };

    if let Some(text) = app.banner() {
        let tone = match app.mood() {
            smobb_core::effect::Mood::RedAlert => RColor::Rgb(255, 60, 60),
            smobb_core::effect::Mood::ColdCyan => RColor::Rgb(120, 220, 255),
            smobb_core::effect::Mood::Corruption => RColor::Rgb(120, 255, 120),
            smobb_core::effect::Mood::Normal => RColor::Rgb(255, 255, 255),
        };
        let inner_h = story_area.height.saturating_sub(2);
        let banner = Paragraph::new(banner_lines(text, inner_h))
            .style(Style::default().fg(tone))
            .block(Block::bordered())
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(banner, story_area);
        // still render the sidebar so the layout doesn't jump
        if let Some(sa) = sidebar_area {
            render_sidebar(f, app, sa);
        }
        // input line
        let input = Paragraph::new(Line::from(format!("> {}", app.input())));
        f.render_widget(input, input_area);
        return;
    }

    // story transcript
    let story_lines: Vec<Line> = story_lines(app.transcript(), app.glitch());
    let bg = if let Some(tone) = app.flash() {
        to_rgb(tone)
    } else {
        match app.mood() {
            smobb_core::effect::Mood::RedAlert => RColor::Rgb(24, 0, 0),
            smobb_core::effect::Mood::ColdCyan => RColor::Rgb(0, 14, 22),
            smobb_core::effect::Mood::Corruption => RColor::Rgb(0, 18, 0),
            smobb_core::effect::Mood::Normal => RColor::Reset,
        }
    };

    // Build paragraph without block first so line_count reflects only content lines.
    let story_inner_w = story_area.width.saturating_sub(2); // minus borders
    let story_inner_h = story_area.height.saturating_sub(2);
    let story_base = Paragraph::new(story_lines)
        .wrap(Wrap { trim: false })
        .style(Style::default().bg(bg));
    let total = story_base.line_count(story_inner_w) as u16;
    let max_scroll = total.saturating_sub(story_inner_h);
    let y = if app.user_scrolled() {
        app.scroll().min(max_scroll)
    } else {
        max_scroll
    };
    let story_focused = app.scroll_focus() == crate::app::Focus::Story;
    let story = story_base
        .block(Block::bordered().border_style(pane_border_style(story_focused)))
        .scroll((y, 0));
    f.render_widget(story, story_area);

    // sidebar
    if let Some(sa) = sidebar_area {
        render_sidebar(f, app, sa);
    }

    // input line
    let fx_label = if app.reduced_effects() {
        "[F3] effects:OFF"
    } else {
        "[F3] effects"
    };
    let hint = format!(
        "[Tab] sidebar  [<-/-> focus]  [hold Space] fast  [F2] instant  {}  ['leave' exits]  [Esc] quit",
        fx_label
    );
    let input = Paragraph::new(Line::from(vec![
        RSpan::styled(
            format!("> {}", app.input()),
            Style::default().fg(RColor::Rgb(220, 220, 220)),
        ),
        RSpan::styled(
            format!("    {}", hint),
            Style::default()
                .fg(RColor::Rgb(110, 110, 110))
                .add_modifier(Modifier::DIM),
        ),
    ]));
    f.render_widget(input, input_area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{App, Key};
    use ratatui::{backend::TestBackend, Terminal};
    use smobb_core::content;
    use smobb_core::engine::Engine;

    #[test]
    fn speaker_change_inserts_one_header_per_turn() {
        use crate::app::Span;
        use smobb_core::speaker::{Color, Speaker};
        let t = vec![
            Span {
                text: "hi".into(),
                color: Color::Aqua,
                dim: false,
                speaker: Some(Speaker::Ali),
            },
            Span {
                text: "there".into(),
                color: Color::Aqua,
                dim: false,
                speaker: Some(Speaker::Ali),
            },
            Span {
                text: "YO".into(),
                color: Color::Salmon,
                dim: false,
                speaker: Some(Speaker::Bader),
            },
        ];
        let lines = super::story_lines(&t, false);
        let flat: Vec<String> = lines
            .iter()
            .map(|l| {
                l.spans
                    .iter()
                    .map(|s| s.content.to_string())
                    .collect::<String>()
            })
            .collect();
        // exact layout: header, two same-speaker lines, blank spacer, header, line
        assert_eq!(flat, vec!["ALI", "hi", "there", "", "BADER", "YO"]);
    }

    #[test]
    fn system_interjection_keeps_one_header() {
        use crate::app::Span;
        use smobb_core::speaker::{Color, Speaker};
        let t = vec![
            Span {
                text: "hi".into(),
                color: Color::Aqua,
                dim: false,
                speaker: Some(Speaker::Ali),
            },
            Span {
                text: "SYSTEM MSG".into(),
                color: Color::White,
                dim: false,
                speaker: Some(Speaker::System),
            },
            Span {
                text: "again".into(),
                color: Color::Aqua,
                dim: false,
                speaker: Some(Speaker::Ali),
            },
        ];
        let lines = super::story_lines(&t, false);
        let flat: Vec<String> = lines
            .iter()
            .map(|l| {
                l.spans
                    .iter()
                    .map(|s| s.content.to_string())
                    .collect::<String>()
            })
            .collect();
        // System has an empty label, so it renders as a plain line and does NOT
        // reset grouping: only one "ALI" header, no spacer before the second Ali line.
        assert_eq!(flat, vec!["ALI", "hi", "SYSTEM MSG", "again"]);
        assert_eq!(flat.iter().filter(|s| s.trim() == "ALI").count(), 1);
    }

    #[test]
    fn smobb_turn_spaces_consecutive_lines() {
        use crate::app::Span;
        use smobb_core::speaker::{Color, Speaker};
        let smobb = |t: &str| Span {
            text: t.into(),
            color: Color::White,
            dim: false,
            speaker: Some(Speaker::Smobb),
        };
        let t = vec![smobb("INCOMPETENT."), smobb("A LIAR."), smobb("A FAKE.")];
        let lines = super::story_lines(&t, false);
        let flat: Vec<String> = lines
            .iter()
            .map(|l| {
                l.spans
                    .iter()
                    .map(|s| s.content.to_string())
                    .collect::<String>()
            })
            .collect();
        // one header, then each caps line separated by a blank spacer.
        assert_eq!(
            flat,
            vec!["SMOBB", "INCOMPETENT.", "", "A LIAR.", "", "A FAKE."]
        );
    }

    #[test]
    fn auto_scrolls_to_latest_line() {
        // Build content with 40 output lines so transcript is much taller than the viewport.
        let many_lines: String = (1..=40)
            .map(|i| format!(r#"Line(Smobb, "LINE{:02}")"#, i))
            .collect::<Vec<_>>()
            .join(", ");
        let content_str = format!(
            r#"ContentPart(
                locations: {{ "hub": ( prompt:[Line(Smobb,"PROMPT")],
                    commands: [ Branch(keys:["go"], target: Steps([ {}, Return ])) ] ) }},
                scenes: {{}},
            )"#,
            many_lines
        );
        let c = content::from_str(&[content_str.as_str()]);
        let mut app = App::new(Engine::new(c));
        app.tick(web_time::Instant::now()); // reveal PROMPT + await
        app.submit_for_test("go");
        app.skip_reveal(); // drain all 40 lines + return to hub + await

        // Render to a small terminal (40w x 12h) — story viewport is roughly 7 rows tall.
        let backend = TestBackend::new(40, 12);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, &app)).unwrap();
        let buf = terminal.backend().buffer().clone();
        let dump: String = buf.content().iter().map(|cell| cell.symbol()).collect();

        assert!(
            dump.contains("LINE40"),
            "LINE40 should be visible (auto-scroll to bottom)"
        );
        assert!(
            !dump.contains("LINE01"),
            "LINE01 should not be visible (auto-scrolled past top)"
        );
    }

    #[test]
    fn hidden_sidebar_omits_options() {
        let c = content::from_str(&[r#"ContentPart(
            locations: { "hub": ( prompt:[Line(Smobb,"MENU")],
                commands: [ Branch(keys:["end"], target: Steps([ Quit ])) ] ) },
            scenes: {},
        )"#]);
        let mut app = App::new(Engine::new(c));
        app.tick(web_time::Instant::now());
        // Hide sidebar with F1
        app.key(Key::F1);
        assert!(!app.sidebar_visible());

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, &app)).unwrap();
        let buf = terminal.backend().buffer().clone();
        let dump: String = buf.content().iter().map(|cell| cell.symbol()).collect();
        assert!(
            !dump.contains("OPTIONS"),
            "OPTIONS should be absent when sidebar is hidden"
        );
    }

    #[test]
    fn sidebar_headers_are_bold() {
        use smobb_core::engine::Output;
        use smobb_core::speaker::Color;
        let header = Output::Line {
            text: "[Clues]".into(),
            color: Color::White,
            speaker: None,
        };
        let item = Output::Line {
            text: "  - a clue".into(),
            color: Color::White,
            speaker: None,
        };
        let h = super::sidebar_line(&header);
        let i = super::sidebar_line(&item);
        let bold = |l: &ratatui::text::Line| {
            l.spans.iter().any(|s| {
                s.style
                    .add_modifier
                    .contains(ratatui::style::Modifier::BOLD)
            })
        };
        assert!(bold(&h), "section header should be bold");
        assert!(!bold(&i), "list item should not be bold");
    }

    #[test]
    fn sidebar_scrolls_past_top_when_focused() {
        use crate::app::{Focus, Key};
        // 20 distinct option labels (via Enter steps) -> tall Options sidebar.
        let branches: String = (1..=20)
            .map(|i| {
                format!(
                    r#"Branch(keys:["k{0:02}"], target: Steps([ Enter("p{0:02}") ]))"#,
                    i
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        let locs: String = (1..=20)
            .map(|i| format!(r#""p{0:02}": ( commands: [] )"#, i))
            .collect::<Vec<_>>()
            .join(", ");
        let content_str = format!(
            r#"ContentPart(
                locations: {{ "hub": ( prompt:[Line(Smobb,"PROMPT")], commands: [ {} ] ), {} }},
                scenes: {{}},
            )"#,
            branches, locs
        );
        let c = content::from_str(&[content_str.as_str()]);
        let mut app = App::new(Engine::new(c));
        app.tick(web_time::Instant::now()); // reveal prompt + await; default tab = Options

        // Small terminal so the sidebar (30% width) overflows its viewport.
        let backend = TestBackend::new(70, 12);
        let mut terminal = Terminal::new(backend).unwrap();

        // Not scrolled: the very first options line "You can:" is visible.
        terminal.draw(|f| render(f, &app)).unwrap();
        let dump: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect();
        assert!(
            dump.contains("You can:"),
            "top of sidebar visible before scrolling"
        );

        // Focus the sidebar and scroll well past the top.
        app.key(Key::Right);
        assert_eq!(app.scroll_focus(), Focus::Sidebar);
        for _ in 0..6 {
            app.key(Key::PageDown);
        }
        terminal.draw(|f| render(f, &app)).unwrap();
        let dump: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect();
        assert!(!dump.contains("You can:"), "scrolled past the top header");
        assert!(
            dump.contains("- "),
            "still showing clamped option items, not blank"
        );
    }

    #[test]
    fn banner_is_letterspaced_and_centered() {
        let lines = super::banner_lines("SELF", 11);
        let flat: Vec<String> = lines
            .iter()
            .map(|l| {
                l.spans
                    .iter()
                    .map(|s| s.content.to_string())
                    .collect::<String>()
            })
            .collect();
        // letter-spaced text present
        assert!(flat.iter().any(|s| s.contains("S E L F")));
        // exactly `height` lines, text centered with top/bottom padding
        assert_eq!(lines.len(), 11);
    }

    #[test]
    fn banner_lines_zero_height_is_empty() {
        assert!(super::banner_lines("SELF", 0).is_empty());
    }

    #[test]
    fn scramble_preserves_length_and_corrupts() {
        let s = "hello world";
        let g = super::scramble(s, 3);
        assert_eq!(g.chars().count(), s.chars().count());
        assert_ne!(g, s);
    }

    #[test]
    fn renders_title_story_and_sidebar() {
        let c = content::from_str(&[r#"ContentPart(
            locations: { "hub": ( prompt:[Line(Smobb,"MENU")],
                commands: [ Branch(keys:["ali"], target: Steps([ Enter("ali") ])),
                            Branch(keys:["end"], target: Steps([ Quit ])) ] ),
                "ali": ( commands: [ Branch(keys:["end"], target: Steps([ Return ])) ] ) },
            scenes: {},
        )"#]);
        let mut app = App::new(Engine::new(c));
        app.tick(web_time::Instant::now()); // reveal MENU + await
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, &app)).unwrap();
        let buf = terminal.backend().buffer().clone();
        let dump: String = buf.content().iter().map(|cell| cell.symbol()).collect();
        assert!(dump.contains("S.M.O.B.B.")); // title
        assert!(dump.contains("MENU")); // story line
        assert!(dump.contains("OPTIONS")); // sidebar header (default tab)
    }
}
