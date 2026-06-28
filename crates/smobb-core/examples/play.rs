use smobb_core::content;
use smobb_core::engine::{Engine, Output};
use std::io::{self, BufRead, Write};
use std::{thread, time::Duration};

fn render(out: &[Output], e: &Engine) -> bool {
    for o in out {
        match o {
            Output::Line { text, color, .. } => println!("{}{}\u{1b}[0m", color.ansi(), text),
            Output::Translation(t) => println!("\u{1b}[2m  \u{21b7} \"{}\"\u{1b}[0m", t),
            Output::Sleep(ms) => thread::sleep(Duration::from_millis(*ms as u64)),
            Output::Recap(_) => {}
            Output::AwaitInput => {
                print!("> ");
                io::stdout().flush().unwrap();
            }
            Output::End => return true,
            Output::Effect(_) => {}
        }
    }
    let _ = e;
    false
}

fn main() {
    let mut e = Engine::new(content::load());
    if render(&e.start(), &e) {
        return;
    }
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let input = line.unwrap_or_default();
        let trimmed = input.trim().to_lowercase();
        let out = match trimmed.as_str() {
            "skipmode" | "sm" => {
                e.set_skip(!e.skip());
                vec![]
            }
            "recap" | "rp" => e.recap_view(),
            "options" | "?" | "look" => e.options_view(),
            "journal" | "j" => e.journal_view(),
            _ => e.advance(&input),
        };
        if render(&out, &e) {
            break;
        }
    }
}
