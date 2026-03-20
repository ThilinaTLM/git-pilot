use std::path::{Path, PathBuf};

use crate::domain::repo::{RepositoryDetails, RepositorySummary};
use crate::domain::status::ChangedFile;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActivePanel {
    None,
    BranchSwitch,
    BranchCreate,
    Commit,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MessageLevel {
    Info,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlashMessage {
    pub level: MessageLevel,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepositoryState {
    pub summary: RepositorySummary,
    pub current_branch: Option<String>,
    pub branches: Vec<String>,
    pub status_files: Vec<ChangedFile>,
    pub load_error: Option<String>,
}

impl RepositoryState {
    pub fn from_details(details: RepositoryDetails) -> Self {
        Self {
            branches: details
                .branches
                .into_iter()
                .map(|branch| branch.name.to_string())
                .collect(),
            current_branch: details.current_branch,
            status_files: details.status.files,
            summary: details.summary,
            load_error: None,
        }
    }

    pub fn from_error(summary: RepositorySummary, error: String) -> Self {
        Self {
            summary,
            current_branch: None,
            branches: Vec::new(),
            status_files: Vec::new(),
            load_error: Some(error),
        }
    }
}

#[derive(Clone, Debug)]
pub struct AppState {
    pub repos: Vec<RepositoryState>,
    pub selected_repo: usize,
    pub selected_file: usize,
    pub selected_branch: usize,
    pub active_panel: ActivePanel,
    pub branch_name_input: String,
    pub commit_message_input: String,
    pub message: Option<FlashMessage>,
    pub show_help: bool,
    pub should_quit: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            repos: Vec::new(),
            selected_repo: 0,
            selected_file: 0,
            selected_branch: 0,
            active_panel: ActivePanel::None,
            branch_name_input: String::new(),
            commit_message_input: String::new(),
            message: None,
            show_help: false,
            should_quit: false,
        }
    }
}

impl AppState {
    pub fn set_repositories(
        &mut self,
        repos: Vec<RepositoryState>,
        selected_path: Option<PathBuf>,
    ) {
        self.repos = repos;
        if self.repos.is_empty() {
            self.selected_repo = 0;
            self.selected_file = 0;
            self.selected_branch = 0;
            return;
        }

        if let Some(path) = selected_path {
            if let Some(index) = self.repos.iter().position(|repo| repo.summary.path == path) {
                self.selected_repo = index;
            } else {
                self.selected_repo = self.selected_repo.min(self.repos.len().saturating_sub(1));
            }
        } else {
            self.selected_repo = self.selected_repo.min(self.repos.len().saturating_sub(1));
        }

        self.sync_selection();
    }

    pub fn replace_selected_repository(&mut self, repo: RepositoryState) {
        if let Some(selected) = self.repos.get_mut(self.selected_repo) {
            *selected = repo;
        } else {
            self.repos.push(repo);
            self.selected_repo = self.repos.len().saturating_sub(1);
        }
        self.sync_selection();
    }

    pub fn sync_selection(&mut self) {
        if let Some((file_len, branch_len)) = self
            .selected_repo_ref()
            .map(|repo| (repo.status_files.len(), repo.branches.len()))
        {
            self.selected_file = self.selected_file.min(file_len.saturating_sub(1));
            self.selected_branch = self.selected_branch.min(branch_len.saturating_sub(1));
        } else {
            self.selected_file = 0;
            self.selected_branch = 0;
        }
    }

    pub fn select_next_repo(&mut self) {
        if !self.repos.is_empty() {
            self.selected_repo = (self.selected_repo + 1) % self.repos.len();
            self.selected_file = 0;
            self.selected_branch = self.current_branch_index().unwrap_or(0);
        }
    }

    pub fn select_previous_repo(&mut self) {
        if !self.repos.is_empty() {
            self.selected_repo = if self.selected_repo == 0 {
                self.repos.len() - 1
            } else {
                self.selected_repo - 1
            };
            self.selected_file = 0;
            self.selected_branch = self.current_branch_index().unwrap_or(0);
        }
    }

    pub fn select_next_file(&mut self) {
        if let Some(repo) = self.selected_repo_ref()
            && !repo.status_files.is_empty()
        {
            self.selected_file = (self.selected_file + 1) % repo.status_files.len();
        }
    }

    pub fn select_previous_file(&mut self) {
        if let Some(repo) = self.selected_repo_ref()
            && !repo.status_files.is_empty()
        {
            self.selected_file = if self.selected_file == 0 {
                repo.status_files.len() - 1
            } else {
                self.selected_file - 1
            };
        }
    }

    pub fn select_next_branch(&mut self) {
        if let Some(repo) = self.selected_repo_ref()
            && !repo.branches.is_empty()
        {
            self.selected_branch = (self.selected_branch + 1) % repo.branches.len();
        }
    }

    pub fn select_previous_branch(&mut self) {
        if let Some(repo) = self.selected_repo_ref()
            && !repo.branches.is_empty()
        {
            self.selected_branch = if self.selected_branch == 0 {
                repo.branches.len() - 1
            } else {
                self.selected_branch - 1
            };
        }
    }

    pub fn open_branch_switch(&mut self) {
        self.active_panel = ActivePanel::BranchSwitch;
        self.selected_branch = self.current_branch_index().unwrap_or(0);
    }

    pub fn open_branch_create(&mut self) {
        self.active_panel = ActivePanel::BranchCreate;
        self.branch_name_input.clear();
    }

    pub fn open_commit_panel(&mut self) {
        self.active_panel = ActivePanel::Commit;
        self.commit_message_input.clear();
    }

    pub fn close_panel(&mut self) {
        self.active_panel = ActivePanel::None;
        self.branch_name_input.clear();
        self.commit_message_input.clear();
    }

    pub fn current_branch_index(&self) -> Option<usize> {
        let repo = self.selected_repo_ref()?;
        let current = repo.current_branch.as_deref()?;
        repo.branches.iter().position(|branch| branch == current)
    }

    pub fn selected_repo_ref(&self) -> Option<&RepositoryState> {
        self.repos.get(self.selected_repo)
    }

    pub fn selected_repo_path(&self) -> Option<PathBuf> {
        self.selected_repo_ref()
            .map(|repo| repo.summary.path.clone())
    }

    pub fn selected_file_path(&self) -> Option<&Path> {
        self.selected_repo_ref()?
            .status_files
            .get(self.selected_file)
            .map(|file| file.path.as_path())
    }

    pub fn selected_branch_name(&self) -> Option<&str> {
        self.selected_repo_ref()?
            .branches
            .get(self.selected_branch)
            .map(String::as_str)
    }

    pub fn set_info(&mut self, message: impl Into<String>) {
        self.message = Some(FlashMessage {
            level: MessageLevel::Info,
            text: message.into(),
        });
    }

    pub fn set_error(&mut self, message: impl Into<String>) {
        self.message = Some(FlashMessage {
            level: MessageLevel::Error,
            text: message.into(),
        });
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::domain::repo::{RepositoryId, RepositorySummary};

    use super::{AppState, RepositoryState};

    #[test]
    fn preserves_selected_repo_when_repository_list_refreshes() {
        let first = repo_state("alpha");
        let second = repo_state("beta");
        let mut state = AppState::default();
        state.set_repositories(vec![first.clone(), second.clone()], None);
        state.selected_repo = 1;

        state.set_repositories(
            vec![first, second.clone()],
            Some(second.summary.path.clone()),
        );

        assert_eq!(state.selected_repo, 1);
    }

    fn repo_state(name: &str) -> RepositoryState {
        let path = PathBuf::from(format!("/tmp/{name}"));
        RepositoryState {
            summary: RepositorySummary {
                id: RepositoryId(path.clone()),
                name: name.to_string(),
                path: path.clone(),
                relative_path: PathBuf::from(name),
            },
            current_branch: Some("main".to_string()),
            branches: vec!["main".to_string()],
            status_files: Vec::new(),
            load_error: None,
        }
    }
}
