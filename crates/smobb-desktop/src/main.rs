// Release builds run as a GUI app with no console window (Windows). Debug keeps
// the console so logs are visible.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod input;
mod save;

use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::{Duration, Instant as StdInstant};

use ratatui_wgpu::ratatui::style::Color as WgpuColor;
use ratatui_wgpu::ratatui::Terminal;
use ratatui_wgpu::{Builder, Dimensions, Font, WgpuBackend};
use smobb_core::content;
use smobb_core::engine::Engine;
use smobb_tui_core::app::{App, Key};
use smobb_tui_core::ui;
use web_time::Instant;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::ModifiersState;
use winit::window::{Window, WindowAttributes, WindowId};

const TICK: Duration = Duration::from_millis(30);

type DesktopTerminal = Terminal<WgpuBackend<'static, 'static>>;

struct AppHandler {
    app: App,
    window: Option<Arc<Window>>,
    terminal: Option<DesktopTerminal>,
    mods: ModifiersState,
}

impl AppHandler {
    fn new(app: App) -> Self {
        Self {
            app,
            window: None,
            terminal: None,
            mods: ModifiersState::empty(),
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
        let size = window.inner_size();
        let width = NonZeroU32::new(size.width.max(1)).unwrap();
        let height = NonZeroU32::new(size.height.max(1)).unwrap();

        let backend = pollster::block_on(
            Builder::from_font(Font::new(dejavu::sans_mono::regular()).expect("mono font"))
                // All faces MUST be monospace with the same aspect ratio: the
                // backend sets the grid cell width to the *minimum* char width
                // across every font, so a proportional fallback would shrink the
                // cell and make glyphs overlap. Use the mono bold/italic faces.
                .with_bold_fonts([Font::new(dejavu::sans_mono::bold()).expect("mono bold")])
                .with_italic_fonts([Font::new(dejavu::sans_mono::oblique()).expect("mono italic")])
                .with_bold_italic_fonts([
                    Font::new(dejavu::sans_mono::bold_oblique()).expect("mono bold-italic"),
                ])
                // ratatui-wgpu defaults to a WHITE background; a terminal's
                // `Reset` is dark, so recolor the default cell to match.
                .with_bg_color(WgpuColor::Rgb(8, 8, 8))
                .with_fg_color(WgpuColor::Rgb(220, 220, 220))
                .with_font_size_px(16)
                .with_width_and_height(Dimensions { width, height })
                .build_with_target(window.clone()),
        )
        .expect("build wgpu backend");

        self.terminal = Some(Terminal::new(backend).expect("terminal"));
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                save::write_save(&self.app);
                event_loop.exit();
            }
            WindowEvent::ModifiersChanged(m) => self.mods = m.state(),
            WindowEvent::Resized(size) => {
                if let Some(t) = self.terminal.as_mut() {
                    t.backend_mut().resize(size.width, size.height);
                }
                if let Some(w) = self.window.as_ref() {
                    w.request_redraw();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state != ElementState::Pressed {
                    return;
                }
                if let Some(key) = input::to_key(&event.logical_key, self.mods) {
                    // Fast-forward keys want auto-repeat; other keys act on the
                    // initial press only (mirrors the terminal frontend).
                    let is_fast = matches!(key, Key::Shift | Key::Char(' '));
                    if event.repeat && !is_fast {
                        return;
                    }
                    self.app.key(key);
                    if let Some(w) = self.window.as_ref() {
                        w.request_redraw();
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                // disjoint field borrows so the draw closure can read `app`
                let Self { terminal, app, .. } = self;
                if let Some(t) = terminal.as_mut() {
                    let _ = t.draw(|f| ui::render(f, app));
                }
            }
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
            save::write_save(&self.app);
            event_loop.exit();
            return;
        }
        if let Some(w) = self.window.as_ref() {
            w.request_redraw();
        }
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
    let mut handler = AppHandler::new(app);
    event_loop.run_app(&mut handler).expect("run app");
}
