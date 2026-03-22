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

struct AutoRefreshTimers {
    last_fetch: Instant,
    last_status: Instant,
    last_branches: Instant,
    last_prs: Instant,
}

impl AutoRefreshTimers {
    fn new() -> Self {
        let now = Instant::now();
        Self {
            last_fetch: now,
            last_status: now,
            last_branches: now,
            last_prs: now,
        }
    }
}

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
    let mut timers = AutoRefreshTimers::new();

    loop {
        controller.check_background_results();

        // Auto-refresh checks (clone to avoid borrow conflict)
        let ar = controller.state().settings.auto_refresh.clone();
        if ar.fetch_enabled
            && timers.last_fetch.elapsed() >= Duration::from_secs(ar.fetch_interval_secs)
        {
            controller.auto_fetch();
            timers.last_fetch = Instant::now();
        }
        if ar.status_enabled
            && timers.last_status.elapsed() >= Duration::from_secs(ar.status_interval_secs)
        {
            controller.auto_refresh_status();
            timers.last_status = Instant::now();
        }
        if ar.branches_enabled
            && timers.last_branches.elapsed() >= Duration::from_secs(ar.branches_interval_secs)
        {
            controller.auto_refresh_branches();
            timers.last_branches = Instant::now();
        }
        if ar.prs_enabled && timers.last_prs.elapsed() >= Duration::from_secs(ar.prs_interval_secs)
        {
            controller.auto_refresh_prs();
            timers.last_prs = Instant::now();
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
