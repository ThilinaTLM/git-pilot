use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::state::ActivePanel;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppAction {
    Noop,
    Quit,
    RefreshRepos,
    SelectNextRepo,
    SelectPreviousRepo,
    SelectNextFile,
    SelectPreviousFile,
    SelectNextBranch,
    SelectPreviousBranch,
    StageSelected,
    UnstageSelected,
    StageAll,
    UnstageAll,
    OpenBranchSwitch,
    OpenBranchCreate,
    OpenCommitPanel,
    ConfirmPanel,
    ClosePanel,
    ToggleHelp,
    InsertChar(char),
    Backspace,
    InsertNewline,
}

pub fn map_key_event(active_panel: &ActivePanel, key_event: KeyEvent) -> AppAction {
    match active_panel {
        ActivePanel::None => map_normal_mode_key(key_event),
        ActivePanel::BranchSwitch => map_branch_switch_key(key_event),
        ActivePanel::BranchCreate => map_text_input_key(key_event, false),
        ActivePanel::Commit => map_text_input_key(key_event, true),
    }
}

fn map_normal_mode_key(key_event: KeyEvent) -> AppAction {
    match key_event.code {
        KeyCode::Char('q') => AppAction::Quit,
        KeyCode::Char('r') => AppAction::RefreshRepos,
        KeyCode::Char('?') => AppAction::ToggleHelp,
        KeyCode::Char('h') | KeyCode::Left => AppAction::SelectPreviousRepo,
        KeyCode::Char('l') | KeyCode::Right => AppAction::SelectNextRepo,
        KeyCode::Char('j') | KeyCode::Down => AppAction::SelectNextFile,
        KeyCode::Char('k') | KeyCode::Up => AppAction::SelectPreviousFile,
        KeyCode::Char('s') => AppAction::StageSelected,
        KeyCode::Char('u') => AppAction::UnstageSelected,
        KeyCode::Char('S') => AppAction::StageAll,
        KeyCode::Char('U') => AppAction::UnstageAll,
        KeyCode::Char('b') => AppAction::OpenBranchSwitch,
        KeyCode::Char('n') => AppAction::OpenBranchCreate,
        KeyCode::Char('c') => AppAction::OpenCommitPanel,
        _ => AppAction::Noop,
    }
}

fn map_branch_switch_key(key_event: KeyEvent) -> AppAction {
    match key_event.code {
        KeyCode::Esc => AppAction::ClosePanel,
        KeyCode::Enter => AppAction::ConfirmPanel,
        KeyCode::Char('j') | KeyCode::Down => AppAction::SelectNextBranch,
        KeyCode::Char('k') | KeyCode::Up => AppAction::SelectPreviousBranch,
        _ => AppAction::Noop,
    }
}

fn map_text_input_key(key_event: KeyEvent, allow_newline: bool) -> AppAction {
    if allow_newline
        && key_event.modifiers == KeyModifiers::CONTROL
        && matches!(key_event.code, KeyCode::Char('n'))
    {
        return AppAction::InsertNewline;
    }

    match key_event.code {
        KeyCode::Esc => AppAction::ClosePanel,
        KeyCode::Enter => AppAction::ConfirmPanel,
        KeyCode::Backspace => AppAction::Backspace,
        KeyCode::Char(ch) if !key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            AppAction::InsertChar(ch)
        }
        _ => AppAction::Noop,
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use crate::app::state::ActivePanel;

    use super::{AppAction, map_key_event};

    #[test]
    fn maps_stage_all_in_normal_mode() {
        let action = map_key_event(
            &ActivePanel::None,
            KeyEvent::new(KeyCode::Char('S'), KeyModifiers::SHIFT),
        );

        assert_eq!(action, AppAction::StageAll);
    }

    #[test]
    fn maps_commit_newline_shortcut() {
        let action = map_key_event(
            &ActivePanel::Commit,
            KeyEvent::new(KeyCode::Char('n'), KeyModifiers::CONTROL),
        );

        assert_eq!(action, AppAction::InsertNewline);
    }
}
