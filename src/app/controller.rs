use std::path::PathBuf;

use anyhow::{Result, anyhow};
use crossterm::event::KeyEvent;

use crate::app::actions::{AppAction, map_key_event};
use crate::app::state::{ActivePanel, AppState, RepositoryState};
use crate::domain::branch::BranchName;
use crate::domain::commit::CommitMessage;
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
        let action = map_key_event(&self.state.active_panel, key_event);
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
            AppAction::OpenBranchSwitch => self.state.open_branch_switch(),
            AppAction::OpenBranchCreate => self.state.open_branch_create(),
            AppAction::OpenCommitPanel => self.state.open_commit_panel(),
            AppAction::ConfirmPanel => self.confirm_panel()?,
            AppAction::ClosePanel => self.state.close_panel(),
            AppAction::ToggleHelp => self.state.show_help = !self.state.show_help,
            AppAction::InsertChar(ch) => self.insert_char(ch),
            AppAction::Backspace => self.backspace(),
            AppAction::InsertNewline => self.insert_newline(),
        }

        self.state.sync_selection();
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

    fn stage_selected(&mut self) -> Result<()> {
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

    fn confirm_panel(&mut self) -> Result<()> {
        match self.state.active_panel {
            ActivePanel::None => Ok(()),
            ActivePanel::BranchSwitch => self.confirm_branch_switch(),
            ActivePanel::BranchCreate => self.confirm_branch_create(),
            ActivePanel::Commit => self.confirm_commit(),
        }
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
        self.state.close_panel();
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
        self.state.close_panel();
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
        self.state.close_panel();
        self.state
            .set_info(format!("Committed {}", message.subject()));
        Ok(())
    }

    fn insert_char(&mut self, ch: char) {
        match self.state.active_panel {
            ActivePanel::BranchCreate => self.state.branch_name_input.push(ch),
            ActivePanel::Commit => self.state.commit_message_input.push(ch),
            ActivePanel::None | ActivePanel::BranchSwitch => {}
        }
    }

    fn backspace(&mut self) {
        match self.state.active_panel {
            ActivePanel::BranchCreate => {
                self.state.branch_name_input.pop();
            }
            ActivePanel::Commit => {
                self.state.commit_message_input.pop();
            }
            ActivePanel::None | ActivePanel::BranchSwitch => {}
        }
    }

    fn insert_newline(&mut self) {
        if matches!(self.state.active_panel, ActivePanel::Commit) {
            self.state.commit_message_input.push('\n');
        }
    }
}
