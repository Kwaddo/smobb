use smobb_core::content;
use smobb_core::engine::{Engine, Output};

fn texts(out: &[Output]) -> Vec<String> {
    out.iter()
        .filter_map(|o| match o {
            Output::Line { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect()
}

/// The reported bug: at the hub, the section labelled "Stories" must be
/// enterable by typing the displayed word, not only the internal id — and it
/// must land straight on the numbered story list, with no extra "search for?"
/// layer in between.
#[test]
fn stories_label_is_enterable_from_hub() {
    let mut e = Engine::new(content::load());
    e.set_skip(true);
    let _ = e.start();
    let out = e.advance("stories");
    // Typing the displayed label "stories" must ENTER the Stories/extra location,
    // not fall through to the unknown-command feedback path.
    let t = texts(&out);
    assert!(
        !t.iter().any(|s| s.contains("I don't understand")),
        "typing 'stories' triggered unknown-command feedback — canonical routing is broken"
    );
    // Entering the section shows the numbered story prompt directly.
    assert!(
        t.iter().any(|s| s.contains("THERE ARE 12 STORIES")),
        "entering Stories did not present the numbered list directly (extra layer still there?)"
    );
    // ...and a number plays that story immediately — no intermediate keyword.
    let out = e.advance("1");
    let t = texts(&out);
    assert!(
        !t.iter().any(|s| s.contains("I don't understand")),
        "typing a story number right after entering did not select a story"
    );
    assert!(
        t.iter().any(|s| s.contains("THERE WERE TWO MEN")),
        "story 1 did not play when its number was typed directly"
    );
}

/// Josh's `end7312` branch must route to the ending (audit #6 — not just parse).
#[test]
fn josh_end7312_routes_to_ending() {
    let mut e = Engine::new(content::load());
    e.set_skip(true);
    let _ = e.start();
    let _ = e.advance("josh"); // hub -> josh
    let out = e.advance("end7312");
    let t = texts(&out);
    assert!(
        !t.iter().any(|s| s.contains("I don't understand")),
        "'end7312' triggered unknown-command feedback in josh — routing is broken"
    );
    assert!(
        !t.is_empty(),
        "advancing 'end7312' from josh produced no output"
    );
    // A unique line from the ending scene confirms we reached it.
    assert!(
        t.iter().any(|s| s.contains("WELCOME TO THE END.")),
        "'end7312' from josh did not reach the ending scene"
    );
}

/// `qasim` (alias) must work in every location that exposes `qassim`.
#[test]
fn qasim_alias_works_in_guide_names() {
    let mut e = Engine::new(content::load());
    e.set_skip(true);
    let _ = e.start();
    let _ = e.advance("guide"); // hub -> guide
    let _ = e.advance("names"); // guide -> guide_names
    let out = e.advance("qasim"); // alias of qassim
    let t = texts(&out);
    assert!(
        !t.iter().any(|s| s.contains("I don't understand")),
        "'qasim' triggered unknown-command feedback in guide_names — alias is not registered there"
    );
    assert!(
        t.iter().any(|s| s.contains("QASSIM MOHAMMED HAMEED")),
        "'qasim' did not reach qassim's entry in guide_names"
    );
}
