pub mod app;
pub mod domain;
pub mod infrastructure;
pub mod shared;
pub mod ui;

use std::io;
use std::time::Duration;

use anyhow::Result;
use app::controller::AppController;
use crossterm::event::{self, Event};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

pub fn run() -> Result<()> {
    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let root = std::env::current_dir()?;
    let mut controller = AppController::bootstrap(root)?;

    let result = run_loop(&mut terminal, &mut controller);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    controller: &mut AppController,
) -> Result<()> {
    loop {
        controller.check_background_results();
        terminal.draw(|frame| ui::screen::render(frame, controller.state()))?;

        if controller.state().should_quit {
            break;
        }

        if event::poll(Duration::from_millis(250))?
            && let Event::Key(key_event) = event::read()?
        {
            controller.handle_key_event(key_event)?;
        }
    }

    Ok(())
}
