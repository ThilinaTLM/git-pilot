pub mod app;
pub mod domain;
pub mod infrastructure;
pub mod shared;
pub mod ui;

use std::io;
use std::time::{Duration, Instant};

use anyhow::Result;
use app::controller::AppController;
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

pub fn run() -> Result<()> {
    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let root = std::env::current_dir()?;
    let mut controller = AppController::bootstrap(root)?;

    let result = run_loop(&mut terminal, &mut controller);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        DisableMouseCapture,
        LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    controller: &mut AppController,
) -> Result<()> {
    let mut last_auto_fetch = Instant::now();

    loop {
        controller.check_background_results();

        // Auto-fetch if enabled and interval has elapsed
        let settings = &controller.state().settings;
        if settings.auto_fetch_enabled {
            let interval = Duration::from_secs(settings.auto_fetch_interval_secs);
            if last_auto_fetch.elapsed() >= interval {
                controller.auto_fetch();
                last_auto_fetch = Instant::now();
            }
        }

        controller.tick_spinner();
        terminal.draw(|frame| ui::screen::render(frame, controller.state()))?;

        if controller.state().should_quit {
            break;
        }

        if event::poll(Duration::from_millis(250))? {
            match event::read()? {
                Event::Key(key_event) => controller.handle_key_event(key_event)?,
                Event::Mouse(mouse_event) => {
                    let size = terminal.size()?;
                    let area = ratatui::layout::Rect::new(0, 0, size.width, size.height);
                    controller.handle_mouse_event(mouse_event, area)?;
                }
                _ => {}
            }
        }
    }

    Ok(())
}
