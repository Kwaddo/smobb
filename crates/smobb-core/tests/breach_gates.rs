use smobb_core::content;
use smobb_core::scene::Step;

/// Recursively count Gate steps in a step list (including nested gates inside
/// Gate on_deploy/on_missing/on_decline, If branches, and Prompt branch targets).
fn count_gates(steps: &[Step]) -> usize {
    let mut n = 0;
    for s in steps {
        match s {
            Step::Gate(g) => {
                n += 1;
                n += count_gates(&g.on_deploy);
                n += count_gates(&g.on_missing);
                n += count_gates(&g.on_decline);
            }
            Step::If(_, a, b) => n += count_gates(a) + count_gates(b),
            Step::Prompt(branches) => {
                for br in branches {
                    if let smobb_core::scene::Target::Steps(inner) = &br.target {
                        n += count_gates(inner);
                    }
                }
            }
            _ => {}
        }
    }
    n
}

#[test]
fn real_content_has_breach_gates() {
    let c = content::load();
    let mut total = 0;

    // Walk all location commands (recurse into nested steps).
    for loc in c.locations.values() {
        for b in &loc.commands {
            if let smobb_core::scene::Target::Steps(steps) = &b.target {
                total += count_gates(steps);
            }
        }
    }

    // Also walk scenes (e.g. josh's bader_josh scene contains a gate).
    for scene in c.scenes.values() {
        total += count_gates(&scene.steps);
    }

    println!("breach_gates: observed gate count = {total}");

    // Verify the migration invariant on at least one gate: on_missing == on_decline
    // (same length is the strongest check possible without PartialEq on Step).
    let mut found_invariant = false;
    'outer: for loc in c.locations.values() {
        for b in &loc.commands {
            if let smobb_core::scene::Target::Steps(steps) = &b.target {
                for s in steps {
                    if let Step::Gate(g) = s {
                        assert_eq!(
                            g.on_missing.len(),
                            g.on_decline.len(),
                            "migration invariant violated: on_missing.len() != on_decline.len()"
                        );
                        found_invariant = true;
                        break 'outer;
                    }
                }
            }
        }
    }
    assert!(
        found_invariant,
        "no top-level gate found to check invariant"
    );

    assert!(
        total >= 10,
        "expected all 10 migrated breach gates, found {total}"
    );
}
