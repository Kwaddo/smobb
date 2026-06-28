use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum Speaker {
    Ali,
    Bader,
    BaderMc,
    Bevil,
    Hasan,
    Josh,
    JoshMc,
    Qassim,
    Smobb,
    Jaffar,
    Mavis,
    System,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Purple,
    Cyan,
    Aqua,
    White,
    Salmon,
    Salmon2,
    Gold,
    LightGreen,
    Silver,
    Aquamarine,
}

impl Speaker {
    /// Short upper-case display label for the transcript turn header.
    /// Empty for `System` (narration shows no header).
    pub fn label(self) -> &'static str {
        match self {
            Speaker::Ali => "ALI",
            Speaker::Bader | Speaker::BaderMc => "BADER",
            Speaker::Bevil => "BEVIL",
            Speaker::Hasan => "HASAN",
            Speaker::Josh | Speaker::JoshMc => "JOSH",
            Speaker::Qassim => "QASSIM",
            Speaker::Smobb => "SMOBB",
            Speaker::Jaffar => "JAFFAR",
            Speaker::Mavis => "MAVIS",
            Speaker::System => "",
        }
    }
    pub fn color(self) -> Color {
        match self {
            Speaker::Ali => Color::Aqua,
            Speaker::Bader => Color::Salmon,
            Speaker::BaderMc => Color::Salmon2,
            Speaker::Bevil => Color::Gold,
            Speaker::Hasan => Color::Purple,
            Speaker::Josh => Color::LightGreen,
            Speaker::JoshMc => Color::Aquamarine,
            Speaker::Qassim => Color::Silver,
            Speaker::Smobb => Color::White,
            Speaker::Jaffar => Color::Red,
            Speaker::Mavis => Color::Blue,
            Speaker::System => Color::White,
        }
    }
}

impl Color {
    /// ANSI escape (matches Bits.java codes).
    pub fn ansi(self) -> &'static str {
        match self {
            Color::Black => "\u{1b}[30m",
            Color::Red => "\u{1b}[31m",
            Color::Green => "\u{1b}[32m",
            Color::Yellow => "\u{1b}[33m",
            Color::Blue => "\u{1b}[34m",
            Color::Purple => "\u{1b}[35m",
            Color::Cyan => "\u{1b}[36m",
            Color::Aqua => "\u{1b}[38;5;14m",
            Color::White => "\u{1b}[38;5;15m",
            Color::Salmon => "\u{1b}[38;5;209m",
            Color::Salmon2 => "\u{1b}[38;5;210m",
            Color::Gold => "\u{1b}[38;5;220m",
            Color::LightGreen => "\u{1b}[38;5;119m",
            Color::Silver => "\u{1b}[38;5;7m",
            Color::Aquamarine => "\u{1b}[38;5;122m",
        }
    }
    /// CSS hex (xterm 256 approximations) for the future web frontend.
    pub fn hex(self) -> &'static str {
        match self {
            Color::Black => "#000000",
            Color::Red => "#cd0000",
            Color::Green => "#00cd00",
            Color::Yellow => "#cdcd00",
            Color::Blue => "#0000ee",
            Color::Purple => "#cd00cd",
            Color::Cyan => "#00cdcd",
            Color::Aqua => "#00ffff",
            Color::White => "#ffffff",
            Color::Salmon => "#ff875f",
            Color::Salmon2 => "#ff8787",
            Color::Gold => "#ffd700",
            Color::LightGreen => "#87ff5f",
            Color::Silver => "#c0c0c0",
            Color::Aquamarine => "#87ffd7",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn speaker_labels() {
        assert_eq!(Speaker::Ali.label(), "ALI");
        assert_eq!(Speaker::Smobb.label(), "SMOBB");
        assert_eq!(Speaker::BaderMc.label(), "BADER");
        assert_eq!(Speaker::System.label(), ""); // system has no banner label
    }
    #[test]
    fn speaker_colors_match_bits() {
        assert_eq!(Speaker::Ali.color(), Color::Aqua);
        assert_eq!(Speaker::Bader.color(), Color::Salmon);
        assert_eq!(Speaker::BaderMc.color(), Color::Salmon2);
        assert_eq!(Speaker::Bevil.color(), Color::Gold);
        assert_eq!(Speaker::Hasan.color(), Color::Purple);
        assert_eq!(Speaker::Josh.color(), Color::LightGreen);
        assert_eq!(Speaker::JoshMc.color(), Color::Aquamarine);
        assert_eq!(Speaker::Qassim.color(), Color::Silver);
        assert_eq!(Speaker::Smobb.color(), Color::White);
        assert_eq!(Speaker::Jaffar.color(), Color::Red);
        assert_eq!(Speaker::Mavis.color(), Color::Blue);
        assert_eq!(Speaker::System.color(), Color::White);
    }
    #[test]
    fn color_codes_present() {
        assert_eq!(Color::Aqua.ansi(), "\u{1b}[38;5;14m");
        assert_eq!(Color::White.hex(), "#ffffff");
    }
}
