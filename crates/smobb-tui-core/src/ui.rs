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

fn output_lines_to_text(outs: &[Output]) -> Vec<Line<'static>> {
    outs.iter()
        .filter_map(|o| match o {
            Output::Line { text, color, .. } => Some(Line::from(RSpan::styled(
                text.clone(),
                Style::default().fg(to_rgb(*color)),
            ))),
            _ => None,
        })
        .collect()
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

    // story transcript
    let story_lines: Vec<Line> = app
        .transcript()
        .iter()
        .map(|s| {
            let mut style = Style::default().fg(to_rgb(s.color));
            if s.dim {
                style = style
                    .add_modifier(Modifier::DIM)
                    .fg(RColor::Rgb(160, 160, 160));
            }
            Line::from(RSpan::styled(s.text.clone(), style))
        })
        .collect();

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
        let side = Paragraph::new(output_lines_to_text(&outs))
            .wrap(Wrap { trim: false })
            .block(Block::bordered().title(title));
        f.render_widget(side, sa);
    }

    // input line
    let hint = "[Tab] sidebar  [hold Space] fast  [F2] instant  ['leave' exits]  [Esc] quit";
    let input = Paragraph::new(Line::from(format!("> {}    {}", app.input(), hint)));
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
