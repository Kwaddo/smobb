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
            let is_header = trimmed.starts_with('[')
                || trimmed.starts_with("==")
                || trimmed.ends_with(':');
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
pub fn story_lines(transcript: &[crate::app::Span]) -> Vec<Line<'static>> {
    use smobb_core::speaker::Speaker;
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut prev: Option<Speaker> = None;
    for s in transcript {
        let label = s.speaker.map(|sp| sp.label()).unwrap_or("");
        let changed = s.speaker.is_some() && s.speaker != prev && !label.is_empty();
        if changed {
            if !lines.is_empty() {
                lines.push(Line::from(""));
            }
            lines.push(Line::from(RSpan::styled(
                label.to_string(),
                Style::default().fg(RColor::Rgb(120, 120, 120)).add_modifier(Modifier::DIM | Modifier::BOLD),
            )));
        }
        let mut style = Style::default().fg(to_rgb(s.color));
        if s.dim {
            style = style.add_modifier(Modifier::DIM).fg(RColor::Rgb(160, 160, 160));
        }
        lines.push(Line::from(RSpan::styled(s.text.clone(), style)));
        if s.speaker.is_some() && !label.is_empty() {
            prev = s.speaker;
        }
    }
    lines
}

/// Build a dramatic full-area banner: the text upper-cased and letter-spaced,
/// vertically centered. `height` is the available story-area inner height.
pub fn banner_lines(text: &str, height: u16) -> Vec<Line<'static>> {
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
        let cols = Layout::horizontal([Constraint::Percentage(70), Constraint::Percentage(30)])
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
            let (title, outs) = match app.tab() {
                crate::app::Tab::Options => (" OPTIONS ", app.options_lines()),
                crate::app::Tab::Journal => (" JOURNAL ", app.journal_lines()),
            };
            let lines: Vec<Line> = outs.iter().map(sidebar_line).collect();
            f.render_widget(
                Paragraph::new(lines).wrap(Wrap { trim: false }).block(Block::bordered().title(title)),
                sa,
            );
        }
        // input line
        let input = Paragraph::new(Line::from(format!("> {}", app.input())));
        f.render_widget(input, input_area);
        return;
    }

    // story transcript
    let story_lines: Vec<Line> = story_lines(app.transcript());

    // Build paragraph without block first so line_count reflects only content lines.
    let story_inner_w = story_area.width.saturating_sub(2); // minus borders
    let story_inner_h = story_area.height.saturating_sub(2);
    let story_base = Paragraph::new(story_lines).wrap(Wrap { trim: false });
    let total = story_base.line_count(story_inner_w) as u16;
    let max_scroll = total.saturating_sub(story_inner_h);
    let y = if app.user_scrolled() {
        app.scroll().min(max_scroll)
    } else {
        max_scroll
    };
    let story = story_base.block(Block::bordered()).scroll((y, 0));
    f.render_widget(story, story_area);

    // sidebar
    if let Some(sa) = sidebar_area {
        let (title, outs) = match app.tab() {
            crate::app::Tab::Options => (" OPTIONS ", app.options_lines()),
            crate::app::Tab::Journal => (" JOURNAL ", app.journal_lines()),
        };
        let lines: Vec<Line> = outs.iter().map(sidebar_line).collect();
        let side = Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .block(Block::bordered().title(title));
        f.render_widget(side, sa);
    }

    // input line
    let hint = "[Tab] sidebar  [hold Space] fast  [F2] instant  [F3] effects  ['leave' exits]  [Esc] quit";
    let input = Paragraph::new(Line::from(vec![
        RSpan::styled(format!("> {}", app.input()), Style::default().fg(RColor::Rgb(220, 220, 220))),
        RSpan::styled(format!("    {}", hint), Style::default().fg(RColor::Rgb(110, 110, 110)).add_modifier(Modifier::DIM)),
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
            Span { text: "hi".into(), color: Color::Aqua, dim: false, speaker: Some(Speaker::Ali) },
            Span { text: "there".into(), color: Color::Aqua, dim: false, speaker: Some(Speaker::Ali) },
            Span { text: "YO".into(), color: Color::Salmon, dim: false, speaker: Some(Speaker::Bader) },
        ];
        let lines = super::story_lines(&t);
        let flat: Vec<String> = lines
            .iter()
            .map(|l| l.spans.iter().map(|s| s.content.to_string()).collect::<String>())
            .collect();
        // exact layout: header, two same-speaker lines, blank spacer, header, line
        assert_eq!(flat, vec!["ALI", "hi", "there", "", "BADER", "YO"]);
    }

    #[test]
    fn system_interjection_keeps_one_header() {
        use crate::app::Span;
        use smobb_core::speaker::{Color, Speaker};
        let t = vec![
            Span { text: "hi".into(), color: Color::Aqua, dim: false, speaker: Some(Speaker::Ali) },
            Span { text: "SYSTEM MSG".into(), color: Color::White, dim: false, speaker: Some(Speaker::System) },
            Span { text: "again".into(), color: Color::Aqua, dim: false, speaker: Some(Speaker::Ali) },
        ];
        let lines = super::story_lines(&t);
        let flat: Vec<String> = lines
            .iter()
            .map(|l| l.spans.iter().map(|s| s.content.to_string()).collect::<String>())
            .collect();
        // System has an empty label, so it renders as a plain line and does NOT
        // reset grouping: only one "ALI" header, no spacer before the second Ali line.
        assert_eq!(flat, vec!["ALI", "hi", "SYSTEM MSG", "again"]);
        assert_eq!(flat.iter().filter(|s| s.trim() == "ALI").count(), 1);
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
        let header = Output::Line { text: "[Clues]".into(), color: Color::White, speaker: None };
        let item = Output::Line { text: "  - a clue".into(), color: Color::White, speaker: None };
        let h = super::sidebar_line(&header);
        let i = super::sidebar_line(&item);
        let bold = |l: &ratatui::text::Line| {
            l.spans.iter().any(|s| s.style.add_modifier.contains(ratatui::style::Modifier::BOLD))
        };
        assert!(bold(&h), "section header should be bold");
        assert!(!bold(&i), "list item should not be bold");
    }

    #[test]
    fn banner_is_letterspaced_and_centered() {
        let lines = super::banner_lines("SELF", 11);
        let flat: Vec<String> = lines
            .iter()
            .map(|l| l.spans.iter().map(|s| s.content.to_string()).collect::<String>())
            .collect();
        // letter-spaced text present
        assert!(flat.iter().any(|s| s.contains("S E L F")));
        // vertically padded (more lines than just the text + box)
        assert_eq!(lines.len(), 11);
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
