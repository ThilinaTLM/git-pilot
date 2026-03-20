use crate::app::state::{AppState, Modal, View};
use crate::domain::status::FileSection;

pub struct Suggestion {
    pub label: &'static str,
    pub key_hint: &'static str,
}

pub fn compute_suggestions(state: &AppState) -> Vec<Suggestion> {
    if state.modal != Modal::None {
        return match state.modal {
            Modal::BranchSwitch => vec![
                Suggestion {
                    key_hint: "j/k",
                    label: "move",
                },
                Suggestion {
                    key_hint: "Enter",
                    label: "switch",
                },
                Suggestion {
                    key_hint: "Esc",
                    label: "cancel",
                },
            ],
            Modal::BranchCreate => vec![
                Suggestion {
                    key_hint: "Enter",
                    label: "create",
                },
                Suggestion {
                    key_hint: "Esc",
                    label: "cancel",
                },
            ],
            Modal::Commit => vec![
                Suggestion {
                    key_hint: "Enter",
                    label: "commit",
                },
                Suggestion {
                    key_hint: "Ctrl+n",
                    label: "newline",
                },
                Suggestion {
                    key_hint: "Esc",
                    label: "cancel",
                },
            ],
            Modal::None => vec![],
        };
    }

    match state.active_view {
        View::Changes => compute_changes_suggestions(state),
        View::Branches => compute_branches_suggestions(state),
    }
}

fn compute_changes_suggestions(state: &AppState) -> Vec<Suggestion> {
    let mut suggestions = Vec::new();

    if state.repos.is_empty() {
        suggestions.push(Suggestion {
            key_hint: "r",
            label: "refresh",
        });
        suggestions.push(Suggestion {
            key_hint: "?",
            label: "help",
        });
        return suggestions;
    }

    let has_unstaged = state.grouped_files.has_section(FileSection::Unstaged)
        || state.grouped_files.has_section(FileSection::Untracked);
    let has_staged = state.grouped_files.has_section(FileSection::Staged);

    if has_unstaged {
        suggestions.push(Suggestion {
            key_hint: "Space",
            label: "stage",
        });
        suggestions.push(Suggestion {
            key_hint: "S",
            label: "stage all",
        });
    }

    if has_staged {
        suggestions.push(Suggestion {
            key_hint: "c",
            label: "commit",
        });
    }

    if !has_unstaged && !has_staged {
        suggestions.push(Suggestion {
            key_hint: "n",
            label: "new branch",
        });
        suggestions.push(Suggestion {
            key_hint: "b",
            label: "switch branch",
        });
    }

    suggestions.push(Suggestion {
        key_hint: "?",
        label: "help",
    });

    suggestions
}

fn compute_branches_suggestions(state: &AppState) -> Vec<Suggestion> {
    let mut suggestions = Vec::new();

    if state.selected_branch_name().is_some() {
        suggestions.push(Suggestion {
            key_hint: "Enter",
            label: "switch",
        });
        suggestions.push(Suggestion {
            key_hint: "d",
            label: "delete",
        });
        suggestions.push(Suggestion {
            key_hint: "m",
            label: "merge",
        });
    }

    suggestions.push(Suggestion {
        key_hint: "n",
        label: "new branch",
    });
    suggestions.push(Suggestion {
        key_hint: "?",
        label: "help",
    });

    suggestions
}
