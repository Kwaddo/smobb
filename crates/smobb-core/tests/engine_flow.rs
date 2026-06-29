use smobb_core::content;
use smobb_core::engine::{Engine, Output};

#[test]
fn fixture_game_reaches_end() {
    // minimal self-contained content with a path to Quit
    let c = content::from_str(&[r#"ContentPart(
        locations: { "hub": (
            intro: [ Line(Smobb, "HELLO.") ],
            prompt: [ Line(Smobb, "HOW CAN I HELP YOU?") ],
            commands: [
                Branch(keys: ["smobb"], target: Steps([ Enter("smobb_loc") ])),
                Branch(keys: ["end"], target: Steps([ Quit ])),
            ],
        ),
        "smobb_loc": (
            prompt: [ Line(Smobb, "WHAT WOULD YOU LIKE TO KNOW?") ],
            commands: [
                // a command scene just lists lines; when it drains, the engine
                // re-prompts THIS location's menu. Return is only for `end` (pop).
                Branch(keys: ["ali"], target: Steps([ Line(Smobb, "INCOMPETENT.") ])),
                Branch(keys: ["end"], target: Steps([ Return ])),
            ],
        ) },
        scenes: {},
    )"#]);
    let mut e = Engine::new(c);
    let _ = e.start();
    let _ = e.advance("smobb");
    let out = e.advance("ali");
    assert!(out
        .iter()
        .any(|o| matches!(o, Output::Line{text,..} if text=="INCOMPETENT.")));
    let _ = e.advance("end"); // back to hub
    let out = e.advance("end"); // hub "Quit" option asks first
    assert!(!out.contains(&Output::End));
    let out = e.advance("yes"); // confirm close
    assert!(out.contains(&Output::End));
}
