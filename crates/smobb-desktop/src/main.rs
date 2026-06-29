// Release builds run as a GUI app with no console window (Windows). Debug keeps
// the console so logs are visible.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod input;
mod prefs;
mod save;
mod settings;

use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::{Duration, Instant as StdInstant};

use ratatui_wgpu::ratatui::style::Color as WgpuColor;
use ratatui_wgpu::ratatui::Terminal;
use ratatui_wgpu::{Builder, Dimensions, Font, WgpuBackend};
use settings::{Settings, FONT_DEFAULT, FONT_MAX, FONT_MIN, FONT_STEP};
use smobb_core::content;
use smobb_core::engine::Engine;
use smobb_tui_core::app::{App, Key};
use smobb_tui_core::ui;
use web_time::Instant;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key as WKey, ModifiersState, NamedKey};
use winit::window::{Fullscreen, Window, WindowAttributes, WindowId};

const TICK: Duration = Duration::from_millis(30);
/// Mouse-wheel lines per detent (trackpads use a pixel accumulator instead).
const WHEEL_LINES: i32 = 3;

type DesktopTerminal = Terminal<WgpuBackend<'static, 'static>>;

/// Build a fresh terminal + wgpu backend for `window` at font size `px`. Used at
/// startup and on every font change — rebuilding from scratch is the reliable
/// way to resize the font, since the backend's in-place `update_fonts` leaves
/// per-cell render caches stale (it can't tell the grid changed) and corrupts
/// the screen. All faces MUST be monospace with the same aspect ratio: the
/// backend sets the cell width to the *minimum* char width across every font, so
/// a proportional fallback would shrink the cell and overlap glyphs.
fn build_terminal(window: Arc<Window>, px: u32) -> DesktopTerminal {
    let size = window.inner_size();
    let width = NonZeroU32::new(size.width.max(1)).unwrap();
    let height = NonZeroU32::new(size.height.max(1)).unwrap();

    let backend = pollster::block_on(
        Builder::from_font(Font::new(dejavu::sans_mono::regular()).expect("mono font"))
            .with_bold_fonts([Font::new(dejavu::sans_mono::bold()).expect("mono bold")])
            .with_italic_fonts([Font::new(dejavu::sans_mono::oblique()).expect("mono italic")])
            .with_bold_italic_fonts([
                Font::new(dejavu::sans_mono::bold_oblique()).expect("mono bold-italic")
            ])
            // ratatui-wgpu defaults to a WHITE background; a terminal's `Reset`
            // is dark, so recolor the default cell to match.
            .with_bg_color(WgpuColor::Rgb(8, 8, 8))
            .with_fg_color(WgpuColor::Rgb(220, 220, 220))
            .with_font_size_px(px)
            .with_width_and_height(Dimensions { width, height })
            .build_with_target(window),
    )
    .expect("build wgpu backend");

    Terminal::new(backend).expect("terminal")
}

struct AppHandler {
    app: App,
    window: Option<Arc<Window>>,
    terminal: Option<DesktopTerminal>,
    mods: ModifiersState,
    settings: Settings,
    cursor_x: f64,
    wheel_accum: f32,
    font_px: u32,
    fullscreen: bool,
}

impl AppHandler {
    fn new(app: App, prefs: prefs::WindowPrefs) -> Self {
        Self {
            app,
            window: None,
            terminal: None,
            mods: ModifiersState::empty(),
            settings: Settings::new(prefs.font_px, prefs.fullscreen),
            cursor_x: 0.0,
            wheel_accum: 0.0,
            font_px: prefs.font_px,
            fullscreen: prefs.fullscreen,
        }
    }

    fn request_redraw(&self) {
        if let Some(w) = self.window.as_ref() {
            w.request_redraw();
        }
    }

    /// Draw one frame synchronously (story + optional settings overlay). Used
    /// after a state change that must be reflected immediately instead of
    /// waiting for the next queued redraw (avoids a stale/black frame).
    fn redraw_now(&mut self) {
        let Self {
            terminal,
            app,
            settings,
            ..
        } = self;
        if let Some(t) = terminal.as_mut() {
            let _ = t.draw(|f| {
                ui::render(f, app);
                if settings.open {
                    settings.render(f);
                }
            });
        }
    }

    fn persist(&self) {
        prefs::save(&prefs::WindowPrefs {
            font_px: self.font_px,
            fullscreen: self.fullscreen,
        });
    }

    /// Resize the font by rebuilding the backend from scratch. In-place
    /// `update_fonts` leaves the backend's per-cell render caches stale — it only
    /// invalidates them when `resize` *detects* a grid change, but a font change
    /// keeps the same pixel size so the change goes unseen and the grid corrupts.
    /// A fresh backend always starts clean.
    fn apply_font(&mut self, px: u32) {
        self.font_px = px;
        self.settings.font_px = px;
        if let Some(w) = self.window.clone() {
            self.terminal = Some(build_terminal(w, px));
        }
        self.redraw_now();
        self.persist();
    }

    fn apply_fullscreen(&mut self, on: bool) {
        self.fullscreen = on;
        self.settings.fullscreen = on;
        if let Some(w) = self.window.as_ref() {
            w.set_fullscreen(on.then(|| Fullscreen::Borderless(None)));
        }
        self.request_redraw();
        self.persist();
    }

    fn apply_settings_action(&mut self, action: settings::Action) {
        match action {
            settings::Action::FontChanged(px) => self.apply_font(px),
            settings::Action::ToggleFullscreen => self.apply_fullscreen(self.settings.fullscreen),
            settings::Action::Close | settings::Action::None => {}
        }
    }

    /// Window-management shortcuts that work without opening the panel. Returns
    /// true if the key was consumed.
    fn handle_global_shortcut(&mut self, key: Key) -> bool {
        // Cmd (macOS) or Ctrl (elsewhere) + zoom keys.
        if self.mods.control_key() || self.mods.super_key() {
            match key {
                Key::Char('=') | Key::Char('+') => {
                    self.apply_font((self.font_px + FONT_STEP).min(FONT_MAX));
                    return true;
                }
                Key::Char('-') | Key::Char('_') => {
                    self.apply_font(self.font_px.saturating_sub(FONT_STEP).max(FONT_MIN));
                    return true;
                }
                Key::Char('0') => {
                    self.apply_font(FONT_DEFAULT);
                    return true;
                }
                _ => {}
            }
        }
        false
    }

    fn scroll_by_wheel(&mut self, delta: MouseScrollDelta) {
        let window_w = self
            .window
            .as_ref()
            .map(|w| w.inner_size().width as f64)
            .unwrap_or(1.0);
        let pane = input::pane_at(window_w, self.cursor_x, self.app.sidebar_visible());
        // Positive wheel/trackpad delta = scroll up (toward earlier text), so a
        // positive physical delta maps to a negative line offset.
        let lines = match delta {
            MouseScrollDelta::LineDelta(_, y) => -(y.round() as i32) * WHEEL_LINES,
            MouseScrollDelta::PixelDelta(p) => {
                self.wheel_accum += p.y as f32;
                let cell = self.font_px.max(1) as f32;
                let whole = (self.wheel_accum / cell).trunc();
                self.wheel_accum -= whole * cell;
                -(whole as i32)
            }
        };
        if lines != 0 {
            self.app.scroll_pane(pane, lines);
            self.request_redraw();
        }
    }
}

impl ApplicationHandler for AppHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return; // already initialised (e.g. resume from suspend)
        }
        let window = Arc::new(
            event_loop
                .create_window(WindowAttributes::default().with_title("S.M.O.B.B."))
                .expect("create window"),
        );
        if self.fullscreen {
            window.set_fullscreen(Some(Fullscreen::Borderless(None)));
        }

        self.terminal = Some(build_terminal(window.clone(), self.font_px));
        self.window = Some(window);

        // Reveal the first frame immediately so the window isn't blank until the
        // event loop happens to tick.
        self.app.tick(Instant::now());
        self.redraw_now();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                self.persist();
                save::write_save(&self.app);
                event_loop.exit();
            }
            WindowEvent::ModifiersChanged(m) => self.mods = m.state(),
            WindowEvent::CursorMoved { position, .. } => self.cursor_x = position.x,
            WindowEvent::MouseWheel { delta, .. } => self.scroll_by_wheel(delta),
            WindowEvent::Resized(size) => {
                if let Some(t) = self.terminal.as_mut() {
                    t.backend_mut().resize(size.width, size.height);
                }
                self.request_redraw();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state != ElementState::Pressed {
                    return;
                }

                // F4 toggles the settings panel; F11 toggles fullscreen. These
                // are desktop-only window keys with no app-level meaning, so map
                // them from the raw winit key before the shared key translation.
                match &event.logical_key {
                    WKey::Named(NamedKey::F4) => {
                        if self.settings.open {
                            self.settings.open = false;
                        } else {
                            self.settings.open(self.font_px, self.fullscreen);
                        }
                        self.request_redraw();
                        return;
                    }
                    WKey::Named(NamedKey::F11) => {
                        self.apply_fullscreen(!self.fullscreen);
                        return;
                    }
                    _ => {}
                }

                let Some(key) = input::to_key(&event.logical_key, self.mods) else {
                    return;
                };

                // While the panel is open it owns the keyboard, so Esc closes it
                // instead of quitting the app.
                if self.settings.open {
                    let action = self.settings.handle_key(key);
                    self.apply_settings_action(action);
                    self.request_redraw();
                    return;
                }

                if self.handle_global_shortcut(key) {
                    return;
                }

                // Fast-forward keys want auto-repeat; other keys act on the
                // initial press only (mirrors the terminal frontend).
                let is_fast = matches!(key, Key::Shift | Key::Char(' '));
                if event.repeat && !is_fast {
                    return;
                }
                self.app.key(key);
                self.request_redraw();
            }
            WindowEvent::RedrawRequested => self.redraw_now(),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.app.tick(Instant::now());
        if self.app.dirty() {
            save::write_save(&self.app);
            self.app.clear_dirty();
        }
        if self.app.quit() {
            self.persist();
            save::write_save(&self.app);
            event_loop.exit();
            return;
        }
        self.request_redraw();
        event_loop.set_control_flow(ControlFlow::WaitUntil(StdInstant::now() + TICK));
    }
}

fn main() {
    let mut engine = Engine::new(content::load());
    if let Some(state) = save::load_save() {
        engine.restore(state); // restore the notebook before the intro runs
    }
    let app = App::new(engine);

    let event_loop = EventLoop::new().expect("event loop");
    let mut handler = AppHandler::new(app, prefs::load());
    event_loop.run_app(&mut handler).expect("run app");
}
