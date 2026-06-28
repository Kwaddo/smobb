//! Portable S.M.O.B.B. TUI core: the App state machine + ratatui rendering,
//! with no terminal backend (so it builds for native and wasm alike).
pub mod app;
pub mod ui;
pub use app::{App, Key, Span, Tab};
