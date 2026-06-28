//! The dramatic-effect vocabulary, authored into content and rendered per-platform.
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum EffectKind {
    /// One-shot screen flash (~120ms). Tint reuses the speaker palette.
    Flash(crate::speaker::Color),
    /// Corruption burst for N ms (char scramble + web RGB-split/jitter).
    Glitch(u16),
    /// Full-screen takeover headline for a fixed duration. Cross-platform.
    Banner(String),
    /// Sustained palette/tint until `Mood::Normal` or the next command.
    Mood(Mood),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum Mood {
    Normal,
    RedAlert,
    ColdCyan,
    Corruption,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::speaker::Color;

    #[test]
    fn parses_each_effect_variant() {
        assert_eq!(ron::from_str::<EffectKind>("Flash(Red)").unwrap(), EffectKind::Flash(Color::Red));
        assert_eq!(ron::from_str::<EffectKind>("Glitch(900)").unwrap(), EffectKind::Glitch(900));
        assert_eq!(
            ron::from_str::<EffectKind>("Banner(\"DELETING\")").unwrap(),
            EffectKind::Banner("DELETING".to_string())
        );
        assert_eq!(ron::from_str::<EffectKind>("Mood(RedAlert)").unwrap(), EffectKind::Mood(Mood::RedAlert));
    }
}
