use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};

use crate::app::state::AppState;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let Some(repo) = state.selected_repo_ref() else {
        let empty = Paragraph::new("No repositories found under the current directory.")
            .block(Block::default().title("Changes").borders(Borders::ALL));
        frame.render_widget(empty, area);
        return;
    };

    if let Some(error) = &repo.load_error {
        let paragraph = Paragraph::new(error.as_str())
            .block(Block::default().title("Changes").borders(Borders::ALL))
            .wrap(ratatui::widgets::Wrap { trim: true });
        frame.render_widget(paragraph, area);
        return;
    }

    if repo.status_files.is_empty() {
        let empty = Paragraph::new("Working tree is clean.")
            .block(Block::default().title("Changes").borders(Borders::ALL));
        frame.render_widget(empty, area);
        return;
    }

    let items = repo
        .status_files
        .iter()
        .map(|file| {
            let status = match (file.staged, file.unstaged, file.untracked) {
                (_, _, true) => "??",
                (true, true, false) => "SU",
                (true, false, false) => "S ",
                (false, true, false) => " U",
                (false, false, false) => "  ",
            };
            ListItem::new(format!("{status} {}", file.path.display()))
        })
        .collect::<Vec<_>>();

    let list = List::new(items)
        .block(Block::default().title("Changes").borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::Blue).fg(Color::Black))
        .highlight_symbol("> ");

    let mut list_state = ListState::default();
    list_state.select(Some(state.selected_file.min(repo.status_files.len() - 1)));
    frame.render_stateful_widget(list, area, &mut list_state);
}
