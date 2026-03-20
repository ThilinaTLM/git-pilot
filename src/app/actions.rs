use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::state::{Modal, View};

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
    ToggleStage,
    OpenBranchSwitch,
    OpenBranchCreate,
    OpenCommitPanel,
    ConfirmModal,
    CloseModal,
    ToggleHelp,
    InsertChar(char),
    Backspace,
    InsertNewline,
    ScrollDiffDown,
    ScrollDiffUp,
    SwitchView(View),
    NextView,
    PreviousView,
    DeleteBranch,
    MergeBranch,
    GenerateCommitMessage,
    CopilotLogin,
}

pub fn map_key_event(view: &View, modal: &Modal, key_event: KeyEvent) -> AppAction {
    if *modal != Modal::None {
        return map_modal_key(modal, key_event);
    }
    if let Some(action) = map_global_key(key_event) {
        return action;
    }
    match view {
        View::Changes => map_changes_key(key_event),
        View::Branches => map_branches_key(key_event),
    }
}

fn map_modal_key(modal: &Modal, key_event: KeyEvent) -> AppAction {
    match modal {
        Modal::None => AppAction::Noop,
        Modal::BranchSwitch => map_branch_switch_key(key_event),
        Modal::BranchCreate => map_text_input_key(key_event, false),
        Modal::Commit => map_commit_input_key(key_event),
        Modal::CopilotLogin => match key_event.code {
            KeyCode::Esc => AppAction::CloseModal,
            _ => AppAction::Noop,
        },
    }
}

fn map_global_key(key_event: KeyEvent) -> Option<AppAction> {
    match key_event.code {
        KeyCode::Char('q') => Some(AppAction::Quit),
        KeyCode::Char('?') => Some(AppAction::ToggleHelp),
        KeyCode::Char('r') => Some(AppAction::RefreshRepos),
        KeyCode::Left | KeyCode::Char('h') => Some(AppAction::SelectPreviousRepo),
        KeyCode::Right | KeyCode::Char('l') => Some(AppAction::SelectNextRepo),
        KeyCode::Tab => Some(AppAction::NextView),
        KeyCode::BackTab => Some(AppAction::PreviousView),
        KeyCode::Char('1') => Some(AppAction::SwitchView(View::Changes)),
        KeyCode::Char('2') => Some(AppAction::SwitchView(View::Branches)),
        _ => None,
    }
}

fn map_changes_key(key_event: KeyEvent) -> AppAction {
    // Ctrl+d / Ctrl+u for diff scrolling
    if key_event.modifiers.contains(KeyModifiers::CONTROL) {
        return match key_event.code {
            KeyCode::Char('d') => AppAction::ScrollDiffDown,
            KeyCode::Char('u') => AppAction::ScrollDiffUp,
            _ => AppAction::Noop,
        };
    }

    match key_event.code {
        KeyCode::Down | KeyCode::Char('j') => AppAction::SelectNextFile,
        KeyCode::Up | KeyCode::Char('k') => AppAction::SelectPreviousFile,
        KeyCode::Char(' ') => AppAction::ToggleStage,
        KeyCode::Char('s') => AppAction::StageSelected,
        KeyCode::Char('u') => AppAction::UnstageSelected,
        KeyCode::Char('S') => AppAction::StageAll,
        KeyCode::Char('U') => AppAction::UnstageAll,
        KeyCode::Char('c') => AppAction::OpenCommitPanel,
        KeyCode::Char('b') => AppAction::OpenBranchSwitch,
        KeyCode::Char('n') => AppAction::OpenBranchCreate,
        KeyCode::PageDown => AppAction::ScrollDiffDown,
        KeyCode::PageUp => AppAction::ScrollDiffUp,
        _ => AppAction::Noop,
    }
}

fn map_branches_key(key_event: KeyEvent) -> AppAction {
    match key_event.code {
        KeyCode::Down | KeyCode::Char('j') => AppAction::SelectNextBranch,
        KeyCode::Up | KeyCode::Char('k') => AppAction::SelectPreviousBranch,
        KeyCode::Enter => AppAction::ConfirmModal,
        KeyCode::Char('n') => AppAction::OpenBranchCreate,
        KeyCode::Char('d') => AppAction::DeleteBranch,
        KeyCode::Char('m') => AppAction::MergeBranch,
        _ => AppAction::Noop,
    }
}

fn map_branch_switch_key(key_event: KeyEvent) -> AppAction {
    match key_event.code {
        KeyCode::Esc => AppAction::CloseModal,
        KeyCode::Enter => AppAction::ConfirmModal,
        KeyCode::Down | KeyCode::Char('j') => AppAction::SelectNextBranch,
        KeyCode::Up | KeyCode::Char('k') => AppAction::SelectPreviousBranch,
        _ => AppAction::Noop,
    }
}

fn map_commit_input_key(key_event: KeyEvent) -> AppAction {
    if key_event.modifiers == KeyModifiers::CONTROL {
        return match key_event.code {
            KeyCode::Char('n') => AppAction::InsertNewline,
            KeyCode::Char('g') => AppAction::GenerateCommitMessage,
            KeyCode::Char('l') => AppAction::CopilotLogin,
            _ => AppAction::Noop,
        };
    }

    match key_event.code {
        KeyCode::Esc => AppAction::CloseModal,
        KeyCode::Enter => AppAction::ConfirmModal,
        KeyCode::Backspace => AppAction::Backspace,
        KeyCode::Char(ch) => AppAction::InsertChar(ch),
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
        KeyCode::Esc => AppAction::CloseModal,
        KeyCode::Enter => AppAction::ConfirmModal,
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

    use crate::app::state::{Modal, View};

    use super::{AppAction, map_key_event};

    #[test]
    fn maps_stage_all_in_normal_mode() {
        let action = map_key_event(
            &View::Changes,
            &Modal::None,
            KeyEvent::new(KeyCode::Char('S'), KeyModifiers::SHIFT),
        );

        assert_eq!(action, AppAction::StageAll);
    }

    #[test]
    fn maps_commit_newline_shortcut() {
        let action = map_key_event(
            &View::Changes,
            &Modal::Commit,
            KeyEvent::new(KeyCode::Char('n'), KeyModifiers::CONTROL),
        );

        assert_eq!(action, AppAction::InsertNewline);
    }

    #[test]
    fn maps_tab_to_next_view() {
        let action = map_key_event(
            &View::Changes,
            &Modal::None,
            KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE),
        );

        assert_eq!(action, AppAction::NextView);
    }

    #[test]
    fn maps_space_to_toggle_stage() {
        let action = map_key_event(
            &View::Changes,
            &Modal::None,
            KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE),
        );

        assert_eq!(action, AppAction::ToggleStage);
    }

    #[test]
    fn maps_branches_view_delete() {
        let action = map_key_event(
            &View::Branches,
            &Modal::None,
            KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE),
        );

        assert_eq!(action, AppAction::DeleteBranch);
    }

    #[test]
    fn maps_number_key_to_switch_view() {
        let action = map_key_event(
            &View::Changes,
            &Modal::None,
            KeyEvent::new(KeyCode::Char('2'), KeyModifiers::NONE),
        );

        assert_eq!(action, AppAction::SwitchView(View::Branches));
    }
}
