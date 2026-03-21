use crate::app::state::{AppState, CreateRepoStep, Modal, View};
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
            Modal::BranchManage => {
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
                    key_hint: "Esc",
                    label: "close",
                });
                suggestions
            }
            Modal::CommitLog => vec![
                Suggestion {
                    key_hint: "j/k",
                    label: "navigate",
                },
                Suggestion {
                    key_hint: "Ctrl+d/u",
                    label: "scroll detail",
                },
                Suggestion {
                    key_hint: "Esc",
                    label: "close",
                },
            ],
            Modal::Settings => vec![
                Suggestion {
                    key_hint: "j/k",
                    label: "navigate",
                },
                Suggestion {
                    key_hint: "Space",
                    label: "toggle",
                },
                Suggestion {
                    key_hint: "+/-",
                    label: "adjust",
                },
                Suggestion {
                    key_hint: "Esc",
                    label: "close",
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
            Modal::CopilotLogin => vec![Suggestion {
                key_hint: "Esc",
                label: "cancel",
            }],
            Modal::CreateRepo(ref step) => compute_create_repo_suggestions(step),
            Modal::None => vec![],
        };
    }

    match state.active_view {
        View::Changes => compute_changes_suggestions(state),
        View::Pr => compute_pr_suggestions(state),
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
        key_hint: "B",
        label: "branches",
    });
    suggestions.push(Suggestion {
        key_hint: "L",
        label: "log",
    });

    if let Some(repo) = state.selected_repo_ref()
        && !repo.has_origin_remote
    {
        suggestions.push(Suggestion {
            key_hint: "R",
            label: "create repo",
        });
    }

    suggestions.push(Suggestion {
        key_hint: "?",
        label: "help",
    });

    suggestions
}

fn compute_pr_suggestions(state: &AppState) -> Vec<Suggestion> {
    let mut suggestions = Vec::new();

    if let Some(repo) = state.selected_repo_ref()
        && !repo.pull_requests.is_empty()
    {
        suggestions.push(Suggestion {
            key_hint: "j/k",
            label: "navigate",
        });
        suggestions.push(Suggestion {
            key_hint: "Enter",
            label: "open in browser",
        });
    }

    suggestions.push(Suggestion {
        key_hint: "r",
        label: "refresh PRs",
    });
    suggestions.push(Suggestion {
        key_hint: "?",
        label: "help",
    });

    suggestions
}

fn compute_create_repo_suggestions(step: &CreateRepoStep) -> Vec<Suggestion> {
    match step {
        CreateRepoStep::Owner | CreateRepoStep::RepoName => vec![
            Suggestion {
                key_hint: "Enter",
                label: "next",
            },
            Suggestion {
                key_hint: "Esc",
                label: "back",
            },
        ],
        CreateRepoStep::Visibility => vec![
            Suggestion {
                key_hint: "Space",
                label: "toggle",
            },
            Suggestion {
                key_hint: "Enter",
                label: "next",
            },
            Suggestion {
                key_hint: "Esc",
                label: "back",
            },
        ],
        CreateRepoStep::Confirm => vec![
            Suggestion {
                key_hint: "Enter",
                label: "create",
            },
            Suggestion {
                key_hint: "Esc",
                label: "back",
            },
        ],
    }
}
