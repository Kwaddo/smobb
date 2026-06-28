use smobb_core::content;
use smobb_core::engine::{Engine, Output};

fn lines(out: &[Output]) -> String {
    out.iter()
        .filter_map(|o| match o {
            Output::Line { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn learning_a_password_surfaces_in_options_and_journal() {
    let mut e = Engine::new(content::load());
    e.set_skip(true);
    let _ = e.start();
    // hub -> hasan -> ali teaches the Tranava-files password 7549009
    let _ = e.advance("hasan");
    let _ = e.advance("ali");
    let _ = e.advance("end"); // back to hub
    let opts = lines(&e.options_view());
    assert!(
        opts.contains("7549009"),
        "options should list the learned password; got:\n{opts}"
    );
    let jrnl = lines(&e.journal_view());
    assert!(
        jrnl.contains("7549009"),
        "journal [Passwords] should include 7549009"
    );
    assert!(
        jrnl.contains("Hasan"),
        "journal [People] should include a visited character"
    );
}

#[test]
fn jaffar_line_decodes_into_journal() {
    let mut e = Engine::new(content::load());
    e.set_skip(true);
    let _ = e.start();
    // Jaffar's hex stays gibberish until the player learns it's hex at Bevil.
    // Before that reveal, the wrong-breach path must NOT translate.
    let _ = e.advance("ali");
    let _ = e.advance("!save");
    let pre = e.advance("wrong");
    assert!(
        !pre.iter().any(|o| matches!(o, Output::Translation(_))),
        "Jaffar should read as raw hex before the decode is unlocked"
    );
    let _ = e.advance("end"); // back to hub
                              // learn it's hexadecimal at Bevil (Bevil[jaffar] runs UnlockDecode)
    let _ = e.advance("bevil");
    let _ = e.advance("jaffar");
    let _ = e.advance("end"); // back to hub
                              // now the same wrong-breach path auto-translates
    let _ = e.advance("ali");
    let _ = e.advance("!save");
    let out = e.advance("wrong");
    let has_translation = out.iter().any(|o| matches!(o, Output::Translation(_)));
    assert!(
        has_translation,
        "after the Bevil hex reveal, a wrong breach answer should emit Jaffar translations"
    );
    let jrnl = lines(&e.journal_view());
    assert!(jrnl.contains("[Decoded]"));
}
