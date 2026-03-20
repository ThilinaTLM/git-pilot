use ratatui::prelude::*;
use ratatui::widgets::{List, ListItem, ListState, Paragraph};

use crate::app::state::AppState;
use crate::domain::status::FileSection;
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let block = theme::pane_block("Changes");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(repo) = state.selected_repo_ref() else {
        let empty = Paragraph::new("No repositories found under the current directory.")
            .style(theme::muted_text_style())
            .wrap(ratatui::widgets::Wrap { trim: true });
        frame.render_widget(empty, inner);
        return;
    };

    if let Some(error) = &repo.load_error {
        let paragraph = Paragraph::new(error.as_str())
            .style(theme::error_text_style())
            .wrap(ratatui::widgets::Wrap { trim: true });
        frame.render_widget(paragraph, inner);
        return;
    }

    if state.grouped_files.entries.is_empty() {
        let empty = Paragraph::new("Working tree is clean.").style(theme::muted_text_style());
        frame.render_widget(empty, inner);
        return;
    }

    // Build list items with section headers interleaved
    let mut items: Vec<ListItem> = Vec::new();
    // Track which visual row maps to which entry index (None = section header)
    let mut row_to_entry: Vec<Option<usize>> = Vec::new();
    let mut current_section: Option<FileSection> = None;

    for (entry_idx, entry) in state.grouped_files.entries.iter().enumerate() {
        // Insert section header when section changes
        if current_section != Some(entry.section) {
            current_section = Some(entry.section);
            let count = state.grouped_files.section_count(entry.section);
            let header_text = match entry.section {
                FileSection::Staged => format!("Staged Changes ({count})"),
                FileSection::Unstaged => format!("Unstaged Changes ({count})"),
                FileSection::Untracked => format!("Untracked ({count})"),
            };
            items.push(ListItem::new(Line::from(Span::styled(
                header_text,
                theme::section_header_style(),
            ))));
            row_to_entry.push(None);
        }

        let file = &repo.status_files[entry.file_index];
        let icon = status_icon(&file.status_code, entry.section);
        let icon_style = match entry.section {
            FileSection::Staged => theme::success_text_style(),
            FileSection::Unstaged | FileSection::Untracked => theme::warning_text_style(),
        };

        items.push(ListItem::new(Line::from(vec![
            Span::styled(format!(" {icon} "), icon_style),
            Span::styled(file.path.display().to_string(), theme::text_style()),
        ])));
        row_to_entry.push(Some(entry_idx));
    }

    // Map selected_file entry index to visual row
    let visual_row = row_to_entry
        .iter()
        .position(|r| *r == Some(state.selected_file));

    let list = List::new(items)
        .style(theme::text_style())
        .highlight_style(theme::selected_list_style())
        .highlight_symbol("▸ ");

    let mut list_state = ListState::default();
    list_state.select(visual_row);
    frame.render_stateful_widget(list, inner, &mut list_state);
}

fn status_icon(status_code: &str, section: FileSection) -> char {
    let bytes = status_code.as_bytes();
    match section {
        FileSection::Staged => match bytes.first().copied().unwrap_or(b' ') {
            b'M' => 'M',
            b'A' => 'A',
            b'D' => 'D',
            b'R' => 'R',
            b'C' => 'C',
            _ => 'M',
        },
        FileSection::Unstaged => match bytes.get(1).copied().unwrap_or(b' ') {
            b'M' => 'M',
            b'D' => 'D',
            _ => 'M',
        },
        FileSection::Untracked => '?',
    }
}
