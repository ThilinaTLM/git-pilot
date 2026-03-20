use std::path::PathBuf;

use anyhow::{Result, anyhow};
use crossterm::event::KeyEvent;

use crate::app::actions::{AppAction, map_key_event};
use crate::app::state::{AppState, Modal, RepositoryState};
use crate::domain::branch::BranchName;
use crate::domain::commit::CommitMessage;
use crate::domain::status::FileSection;
use crate::infrastructure::git_cli::{GitCliRepositoryService, GitRepositoryService};
use crate::infrastructure::repo_discovery::{RepositoryDiscovery, WalkDirRepositoryDiscovery};

pub struct AppController {
    state: AppState,
    discovery: WalkDirRepositoryDiscovery,
    git: GitCliRepositoryService,
    root: PathBuf,
    max_depth: usize,
}

impl AppController {
    pub fn bootstrap(root: PathBuf) -> Result<Self> {
        let mut controller = Self {
            state: AppState::default(),
            discovery: WalkDirRepositoryDiscovery,
            git: GitCliRepositoryService,
            root,
            max_depth: 3,
        };
        controller.refresh_repositories()?;
        Ok(controller)
    }

    pub fn state(&self) -> &AppState {
        &self.state
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        let action = map_key_event(&self.state.active_view, &self.state.modal, key_event);
        if let Err(error) = self.dispatch(action) {
            self.state.set_error(error.to_string());
        }
        Ok(())
    }

    fn dispatch(&mut self, action: AppAction) -> Result<()> {
        match action {
            AppAction::Noop => {}
            AppAction::Quit => self.state.should_quit = true,
            AppAction::RefreshRepos => self.refresh_repositories()?,
            AppAction::SelectNextRepo => self.state.select_next_repo(),
            AppAction::SelectPreviousRepo => self.state.select_previous_repo(),
            AppAction::SelectNextFile => self.state.select_next_file(),
            AppAction::SelectPreviousFile => self.state.select_previous_file(),
            AppAction::SelectNextBranch => self.state.select_next_branch(),
            AppAction::SelectPreviousBranch => self.state.select_previous_branch(),
            AppAction::StageSelected => self.stage_selected()?,
            AppAction::UnstageSelected => self.unstage_selected()?,
            AppAction::StageAll => self.stage_all()?,
            AppAction::UnstageAll => self.unstage_all()?,
            AppAction::ToggleStage => self.toggle_stage()?,
            AppAction::OpenBranchSwitch => self.state.open_branch_switch(),
            AppAction::OpenBranchCreate => self.state.open_branch_create(),
            AppAction::OpenCommitPanel => self.state.open_commit_panel(),
            AppAction::ConfirmModal => self.confirm_modal()?,
            AppAction::CloseModal => self.state.close_modal(),
            AppAction::ToggleHelp => self.state.show_help = !self.state.show_help,
            AppAction::InsertChar(ch) => self.insert_char(ch),
            AppAction::Backspace => self.backspace(),
            AppAction::InsertNewline => self.insert_newline(),
            AppAction::ScrollDiffDown => {
                self.state.diff_scroll = self.state.diff_scroll.saturating_add(5);
            }
            AppAction::ScrollDiffUp => {
                self.state.diff_scroll = self.state.diff_scroll.saturating_sub(5);
            }
            AppAction::SwitchView(view) => self.state.switch_view(view),
            AppAction::NextView => self.state.next_view(),
            AppAction::PreviousView => self.state.previous_view(),
            AppAction::DeleteBranch => self.delete_branch()?,
            AppAction::MergeBranch => self.merge_branch()?,
        }

        self.state.sync_selection();
        self.refresh_diff();
        Ok(())
    }

    fn refresh_repositories(&mut self) -> Result<()> {
        let selected_path = self.state.selected_repo_path();
        let summaries = self.discovery.discover(&self.root, self.max_depth)?;
        let repos = summaries
            .into_iter()
            .map(|summary| match self.git.load_repository(&summary) {
                Ok(details) => RepositoryState::from_details(details),
                Err(error) => RepositoryState::from_error(summary, error.to_string()),
            })
            .collect::<Vec<_>>();

        self.state.set_repositories(repos, selected_path);
        if self.state.repos.is_empty() {
            self.state
                .set_info("No Git repositories found in the current directory or descendants");
        } else {
            self.state
                .set_info(format!("Loaded {} repositories", self.state.repos.len()));
        }
        Ok(())
    }

    fn reload_selected_repo(&mut self) -> Result<()> {
        let summary = self
            .state
            .selected_repo_ref()
            .map(|repo| repo.summary.clone())
            .ok_or_else(|| anyhow!("no repository selected"))?;
        let details = self.git.load_repository(&summary)?;
        self.state
            .replace_selected_repository(RepositoryState::from_details(details));
        Ok(())
    }

    fn toggle_stage(&mut self) -> Result<()> {
        let section = self
            .state
            .selected_file_section()
            .ok_or_else(|| anyhow!("no file selected"))?;
        if section == FileSection::Staged {
            self.unstage_selected()
        } else {
            self.stage_selected()
        }
    }

    fn stage_selected(&mut self) -> Result<()> {
        let section = self
            .state
            .selected_file_section()
            .ok_or_else(|| anyhow!("no file selected"))?;
        if section == FileSection::Staged {
            return Ok(());
        }
        let repo_path = self
            .state
            .selected_repo_path()
            .ok_or_else(|| anyhow!("no repository selected"))?;
        let file_path = self
            .state
            .selected_file_path()
            .ok_or_else(|| anyhow!("no file selected"))?;
        self.git.stage_file(&repo_path, file_path)?;
        self.reload_selected_repo()?;
        self.state.set_info("Staged selected file");
        Ok(())
    }

    fn unstage_selected(&mut self) -> Result<()> {
        let section = self
            .state
            .selected_file_section()
            .ok_or_else(|| anyhow!("no file selected"))?;
        if section != FileSection::Staged {
            return Ok(());
        }
        let repo_path = self
            .state
            .selected_repo_path()
            .ok_or_else(|| anyhow!("no repository selected"))?;
        let file_path = self
            .state
            .selected_file_path()
            .ok_or_else(|| anyhow!("no file selected"))?;
        self.git.unstage_file(&repo_path, file_path)?;
        self.reload_selected_repo()?;
        self.state.set_info("Unstaged selected file");
        Ok(())
    }

    fn stage_all(&mut self) -> Result<()> {
        let repo_path = self
            .state
            .selected_repo_path()
            .ok_or_else(|| anyhow!("no repository selected"))?;
        self.git.stage_all(&repo_path)?;
        self.reload_selected_repo()?;
        self.state.set_info("Staged all changes");
        Ok(())
    }

    fn unstage_all(&mut self) -> Result<()> {
        let repo_path = self
            .state
            .selected_repo_path()
            .ok_or_else(|| anyhow!("no repository selected"))?;
        self.git.unstage_all(&repo_path)?;
        self.reload_selected_repo()?;
        self.state.set_info("Unstaged all changes");
        Ok(())
    }

    fn delete_branch(&mut self) -> Result<()> {
        let repo_path = self
            .state
            .selected_repo_path()
            .ok_or_else(|| anyhow!("no repository selected"))?;
        let branch_name = self
            .state
            .selected_branch_name()
            .ok_or_else(|| anyhow!("no branch selected"))?
            .to_string();

        if let Some(repo) = self.state.selected_repo_ref()
            && repo
                .current_branch
                .as_deref()
                .is_some_and(|current| current == branch_name)
        {
            return Err(anyhow!("cannot delete the current branch"));
        }

        let branch = BranchName::try_from(branch_name)?;
        self.git.delete_branch(&repo_path, &branch)?;
        self.reload_selected_repo()?;
        self.state
            .set_info(format!("Deleted branch {}", branch.as_str()));
        Ok(())
    }

    fn merge_branch(&mut self) -> Result<()> {
        let repo_path = self
            .state
            .selected_repo_path()
            .ok_or_else(|| anyhow!("no repository selected"))?;
        let branch_name = self
            .state
            .selected_branch_name()
            .ok_or_else(|| anyhow!("no branch selected"))?
            .to_string();
        let branch = BranchName::try_from(branch_name)?;
        self.git.merge_branch(&repo_path, &branch)?;
        self.reload_selected_repo()?;
        self.state.set_info(format!("Merged {}", branch.as_str()));
        Ok(())
    }

    fn confirm_modal(&mut self) -> Result<()> {
        match self.state.modal {
            Modal::None => self.confirm_branches_view_switch(),
            Modal::BranchSwitch => self.confirm_branch_switch(),
            Modal::BranchCreate => self.confirm_branch_create(),
            Modal::Commit => self.confirm_commit(),
        }
    }

    fn confirm_branches_view_switch(&mut self) -> Result<()> {
        let repo_path = self
            .state
            .selected_repo_path()
            .ok_or_else(|| anyhow!("no repository selected"))?;
        let branch_name = self
            .state
            .selected_branch_name()
            .ok_or_else(|| anyhow!("no branch selected"))?
            .to_string();
        let branch = BranchName::try_from(branch_name)?;
        self.git.switch_branch(&repo_path, &branch)?;
        self.reload_selected_repo()?;
        self.state
            .set_info(format!("Switched to {}", branch.as_str()));
        Ok(())
    }

    fn confirm_branch_switch(&mut self) -> Result<()> {
        let repo_path = self
            .state
            .selected_repo_path()
            .ok_or_else(|| anyhow!("no repository selected"))?;
        let branch_name = self
            .state
            .selected_branch_name()
            .ok_or_else(|| anyhow!("no branch selected"))?
            .to_string();
        let branch = BranchName::try_from(branch_name)?;
        self.git.switch_branch(&repo_path, &branch)?;
        self.reload_selected_repo()?;
        self.state.close_modal();
        self.state
            .set_info(format!("Switched to {}", branch.as_str()));
        Ok(())
    }

    fn confirm_branch_create(&mut self) -> Result<()> {
        let repo_path = self
            .state
            .selected_repo_path()
            .ok_or_else(|| anyhow!("no repository selected"))?;
        let branch = BranchName::try_from(self.state.branch_name_input.clone())?;
        self.git.create_branch(&repo_path, &branch)?;
        self.reload_selected_repo()?;
        self.state.close_modal();
        self.state
            .set_info(format!("Created and switched to {}", branch.as_str()));
        Ok(())
    }

    fn confirm_commit(&mut self) -> Result<()> {
        let repo = self
            .state
            .selected_repo_ref()
            .ok_or_else(|| anyhow!("no repository selected"))?;
        if repo.status_files.iter().all(|file| !file.staged) {
            return Err(anyhow!("no staged changes to commit"));
        }

        let message = CommitMessage::try_from(self.state.commit_message_input.clone())?;
        self.git.commit(&repo.summary.path, &message)?;
        self.reload_selected_repo()?;
        self.state.close_modal();
        self.state
            .set_info(format!("Committed {}", message.subject()));
        Ok(())
    }

    fn insert_char(&mut self, ch: char) {
        match self.state.modal {
            Modal::BranchCreate => self.state.branch_name_input.push(ch),
            Modal::Commit => self.state.commit_message_input.push(ch),
            Modal::None | Modal::BranchSwitch => {}
        }
    }

    fn backspace(&mut self) {
        match self.state.modal {
            Modal::BranchCreate => {
                self.state.branch_name_input.pop();
            }
            Modal::Commit => {
                self.state.commit_message_input.pop();
            }
            Modal::None | Modal::BranchSwitch => {}
        }
    }

    fn insert_newline(&mut self) {
        if matches!(self.state.modal, Modal::Commit) {
            self.state.commit_message_input.push('\n');
        }
    }

    fn refresh_diff(&mut self) {
        let Some(repo_path) = self.state.selected_repo_path() else {
            self.state.diff_content = None;
            return;
        };
        let Some(entry) = self.state.selected_file_entry().cloned() else {
            self.state.diff_content = None;
            return;
        };
        let Some(file_path) = self
            .state
            .selected_repo_ref()
            .and_then(|repo| repo.status_files.get(entry.file_index))
            .map(|f| f.path.clone())
        else {
            self.state.diff_content = None;
            return;
        };

        match self.git.diff_file(&repo_path, &file_path, entry.section) {
            Ok(diff) if diff.is_empty() => self.state.diff_content = None,
            Ok(diff) => self.state.diff_content = Some(diff),
            Err(_) => self.state.diff_content = None,
        }
    }
}
