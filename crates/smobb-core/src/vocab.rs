//! vocab.rs — single source of truth for navigation/meta/name vocabulary.
//! Maps any typed alias to a canonical id and a canonical id to its display
//! label. Passwords and secret/cheat codes are deliberately ABSENT here: they
//! match by exact equality and never surface as labels.

struct Group {
    /// The id stored/compared internally and used to look up a label.
    canonical: &'static str,
    /// Human-facing name shown in menus/options. Always one of `aliases`
    /// (after normalization) so the label is itself typeable.
    label: &'static str,
    /// Every spelling a player may type. MUST include the canonical.
    aliases: &'static [&'static str],
}

const GROUPS: &[Group] = &[
    // ---- sections ----
    Group {
        canonical: "extra",
        label: "Stories",
        aliases: &["extra", "extras", "story", "stories"],
    },
    Group {
        canonical: "guide",
        label: "Guide",
        aliases: &["guide", "help"],
    },
    Group {
        canonical: "guide_names",
        label: "Names",
        aliases: &["guide_names", "names", "name"],
    },
    Group {
        canonical: "settings",
        label: "Settings",
        aliases: &["settings"],
    },
    // ---- characters ----
    Group {
        canonical: "ali",
        label: "Ali",
        aliases: &["ali"],
    },
    Group {
        canonical: "bader",
        label: "Bader",
        aliases: &["bader"],
    },
    Group {
        canonical: "bevil",
        label: "Bevil",
        aliases: &["bevil"],
    },
    Group {
        canonical: "hasan",
        label: "Hasan",
        aliases: &["hasan"],
    },
    Group {
        canonical: "josh",
        label: "Josh",
        aliases: &["josh"],
    },
    Group {
        canonical: "qassim",
        label: "Qassim",
        aliases: &["qassim", "qasim"],
    },
    Group {
        canonical: "smobb",
        label: "S.M.O.B.B.",
        aliases: &["smobb"],
    },
    Group {
        canonical: "mavis",
        label: "M.A.V.I.S.",
        aliases: &["mavis"],
    },
    Group {
        canonical: "jaffar",
        label: "Jaffar",
        aliases: &["jaffar", "ja3ffar", "ja3ffer"],
    },
    Group {
        canonical: "hub",
        label: "Hub",
        aliases: &["hub"],
    },
    // ---- meta-commands ----
    Group {
        canonical: "recap",
        label: "Recap",
        aliases: &["recap", "rp"],
    },
    Group {
        canonical: "skipmode",
        label: "Skip mode",
        aliases: &["skipmode", "sm"],
    },
    Group {
        canonical: "journal",
        label: "Journal",
        aliases: &["journal", "j"],
    },
    Group {
        canonical: "options",
        label: "Options",
        aliases: &["options", "choices", "choice"],
    },
    Group {
        canonical: "play",
        label: "Play",
        aliases: &["play", "pl"],
    },
    // ---- affirmative / negative (consumed by the Gate step in Phase 2) ----
    Group {
        canonical: "yes",
        label: "Yes",
        aliases: &["yes", "y", "yeah", "yep", "yup", "yessir"],
    },
    Group {
        canonical: "no",
        label: "No",
        aliases: &["no", "n", "nope", "nah", "nuhhuh"],
    },
];

/// Lowercase, trim, and drop `.` so dotted forms like `M.A.V.I.S.` collapse to
/// `mavis`. Used ONLY on the registry-canonical comparison path — never on the
/// exact-match path that guards passwords and secret codes.
pub fn normalize(token: &str) -> String {
    token.trim().to_lowercase().replace('.', "")
}

/// The display label for a canonical id, or `None` if unknown.
pub fn label(canonical: &str) -> Option<&'static str> {
    GROUPS
        .iter()
        .find(|g| g.canonical == canonical)
        .map(|g| g.label)
}

/// Canonical ids that are people (drives the journal's "People" list and
/// visited tracking). Kept here so the set lives in ONE place.
const CHARACTERS: &[&str] = &["ali", "bader", "bevil", "hasan", "josh", "qassim", "smobb"];

/// True if a canonical id names a character.
pub fn is_character(canonical: &str) -> bool {
    CHARACTERS.contains(&canonical)
}

/// True when a branch's authored key and the player's input denote the same
/// command. Exact lowercased equality first (so passwords and secret codes,
/// which are absent from the registry, behave exactly as before); then a
/// canonical comparison so any alias of a registered word matches any other.
pub fn key_matches(branch_key: &str, input: &str) -> bool {
    let bk = branch_key.trim().to_lowercase();
    let inp = input.trim().to_lowercase();
    if bk == inp {
        return true;
    }
    match (canonical(&bk), canonical(&inp)) {
        (Some(a), Some(b)) => a == b,
        _ => false,
    }
}

/// The canonical id for any known alias, or `None` if the token is not a
/// registered navigation/meta/name word (e.g. a password or secret code).
pub fn canonical(token: &str) -> Option<&'static str> {
    let norm = normalize(token);
    GROUPS
        .iter()
        .find(|g| g.aliases.iter().any(|a| normalize(a) == norm))
        .map(|g| g.canonical)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_groups_section_aliases() {
        assert_eq!(canonical("stories"), Some("extra"));
        assert_eq!(canonical("story"), Some("extra"));
        assert_eq!(canonical("extra"), Some("extra"));
        assert_eq!(canonical("extras"), Some("extra"));
    }

    #[test]
    fn canonical_groups_name_variants() {
        assert_eq!(canonical("qasim"), Some("qassim"));
        assert_eq!(canonical("qassim"), Some("qassim"));
        assert_eq!(canonical("ja3ffar"), Some("jaffar"));
        assert_eq!(canonical("ja3ffer"), Some("jaffar"));
    }

    #[test]
    fn canonical_returns_none_for_passwords_and_secrets() {
        assert_eq!(canonical("7549009"), None);
        assert_eq!(canonical("!save"), None);
        assert_eq!(canonical("9285005"), None);
        assert_eq!(canonical("<---------->"), None);
    }

    #[test]
    fn label_lookup_by_canonical() {
        assert_eq!(label("extra"), Some("Stories"));
        assert_eq!(label("qassim"), Some("Qassim"));
        assert_eq!(label("smobb"), Some("S.M.O.B.B."));
        assert_eq!(label("nonexistent"), None);
    }

    #[test]
    fn is_character_flags_people_only() {
        assert!(is_character("qassim"));
        assert!(is_character("ali"));
        assert!(!is_character("extra"));
        assert!(!is_character("recap"));
    }

    #[test]
    fn every_label_resolves_back_to_its_canonical() {
        // Anti-drift guarantee: any label that is a typeable single token must
        // resolve back to its own canonical. Multi-word meta labels (e.g.
        // "Skip mode") legitimately canonicalize to None and are skipped — no
        // hand-kept allowlist.
        for g in GROUPS {
            if let Some(resolved) = canonical(g.label) {
                assert_eq!(
                    resolved, g.canonical,
                    "label '{}' resolves to '{}', expected '{}'",
                    g.label, resolved, g.canonical
                );
            }
        }
    }

    #[test]
    fn every_canonical_resolves_to_itself() {
        // Each group's canonical id must be one of its own aliases.
        for g in GROUPS {
            assert_eq!(
                canonical(g.canonical),
                Some(g.canonical),
                "canonical '{}' does not resolve to itself (missing from its own aliases?)",
                g.canonical
            );
        }
    }

    #[test]
    fn key_matches_exact_and_via_canonical() {
        // exact (covers passwords / secrets not in the registry)
        assert!(key_matches("7549009", "7549009"));
        assert!(!key_matches("7549009", "754"));
        assert!(key_matches("!save", "!save"));
        // canonical: a branch keyed "extra" is reachable by typing the label word
        assert!(key_matches("extra", "stories"));
        assert!(key_matches("extra", "story"));
        // canonical: dotted name forms collapse
        assert!(key_matches("mavis", "m.a.v.i.s."));
        assert!(key_matches("m.a.v.i.s.", "mavis"));
        // canonical: alias both directions
        assert!(key_matches("qassim", "qasim"));
        // distinct canonicals do NOT cross-match
        assert!(!key_matches("ali", "bader"));
        // a registry word must not match an unrelated raw password
        assert!(!key_matches("extra", "7549009"));
    }
}
