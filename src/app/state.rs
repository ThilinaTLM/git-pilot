use std::path::{Path, PathBuf};

use crate::domain::repo::{RepositoryDetails, RepositorySummary};
use crate::domain::status::{ChangedFile, FileSection};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum View {
    #[default]
    Changes,
    Branches,
}

impl View {
    pub const ALL: &[View] = &[View::Changes, View::Branches];

    pub fn index(&self) -> usize {
        match self {
            View::Changes => 0,
            View::Branches => 1,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            View::Changes => "Changes",
            View::Branches => "Branches",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Modal {
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileEntry {
    pub file_index: usize,
    pub section: FileSection,
}

#[derive(Clone, Debug, Default)]
pub struct GroupedFileList {
    pub entries: Vec<FileEntry>,
}

impl GroupedFileList {
    pub fn build(files: &[ChangedFile]) -> Self {
        let mut entries = Vec::new();

        // Staged
        for (i, file) in files.iter().enumerate() {
            if file.staged {
                entries.push(FileEntry {
                    file_index: i,
                    section: FileSection::Staged,
                });
            }
        }

        // Unstaged (tracked, not untracked)
        for (i, file) in files.iter().enumerate() {
            if file.unstaged && !file.untracked {
                entries.push(FileEntry {
                    file_index: i,
                    section: FileSection::Unstaged,
                });
            }
        }

        // Untracked
        for (i, file) in files.iter().enumerate() {
            if file.untracked {
                entries.push(FileEntry {
                    file_index: i,
                    section: FileSection::Untracked,
                });
            }
        }

        Self { entries }
    }

    pub fn section_count(&self, section: FileSection) -> usize {
        self.entries.iter().filter(|e| e.section == section).count()
    }

    pub fn has_section(&self, section: FileSection) -> bool {
        self.entries.iter().any(|e| e.section == section)
    }
}

#[derive(Clone, Debug)]
pub struct AppState {
    pub repos: Vec<RepositoryState>,
    pub selected_repo: usize,
    pub selected_file: usize,
    pub selected_branch: usize,
    pub active_view: View,
    pub modal: Modal,
    pub branch_name_input: String,
    pub commit_message_input: String,
    pub message: Option<FlashMessage>,
    pub show_help: bool,
    pub should_quit: bool,
    pub grouped_files: GroupedFileList,
    pub diff_content: Option<String>,
    pub diff_scroll: u16,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            repos: Vec::new(),
            selected_repo: 0,
            selected_file: 0,
            selected_branch: 0,
            active_view: View::default(),
            modal: Modal::None,
            branch_name_input: String::new(),
            commit_message_input: String::new(),
            message: None,
            show_help: false,
            should_quit: false,
            grouped_files: GroupedFileList::default(),
            diff_content: None,
            diff_scroll: 0,
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
        if let Some(repo) = self.repos.get(self.selected_repo) {
            let grouped = GroupedFileList::build(&repo.status_files);
            let entry_len = grouped.entries.len();
            let branch_len = repo.branches.len();
            self.grouped_files = grouped;
            self.selected_file = self.selected_file.min(entry_len.saturating_sub(1));
            self.selected_branch = self.selected_branch.min(branch_len.saturating_sub(1));
        } else {
            self.grouped_files = GroupedFileList::default();
            self.selected_file = 0;
            self.selected_branch = 0;
        }
    }

    pub fn switch_view(&mut self, view: View) {
        if self.active_view != view {
            self.active_view = view;
            self.diff_scroll = 0;
        }
    }

    pub fn next_view(&mut self) {
        let all = View::ALL;
        let current = self.active_view.index();
        let next = (current + 1) % all.len();
        self.switch_view(all[next].clone());
    }

    pub fn previous_view(&mut self) {
        let all = View::ALL;
        let current = self.active_view.index();
        let prev = if current == 0 {
            all.len() - 1
        } else {
            current - 1
        };
        self.switch_view(all[prev].clone());
    }

    pub fn select_next_repo(&mut self) {
        if !self.repos.is_empty() {
            self.selected_repo = (self.selected_repo + 1) % self.repos.len();
            self.selected_file = 0;
            self.selected_branch = self.current_branch_index().unwrap_or(0);
            self.diff_content = None;
            self.diff_scroll = 0;
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
            self.diff_content = None;
            self.diff_scroll = 0;
        }
    }

    pub fn select_next_file(&mut self) {
        let len = self.grouped_files.entries.len();
        if len > 0 {
            self.selected_file = (self.selected_file + 1) % len;
        }
    }

    pub fn select_previous_file(&mut self) {
        let len = self.grouped_files.entries.len();
        if len > 0 {
            self.selected_file = if self.selected_file == 0 {
                len - 1
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
        self.modal = Modal::BranchSwitch;
        self.selected_branch = self.current_branch_index().unwrap_or(0);
    }

    pub fn open_branch_create(&mut self) {
        self.modal = Modal::BranchCreate;
        self.branch_name_input.clear();
    }

    pub fn open_commit_panel(&mut self) {
        self.modal = Modal::Commit;
        self.commit_message_input.clear();
    }

    pub fn close_modal(&mut self) {
        self.modal = Modal::None;
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

    pub fn selected_file_entry(&self) -> Option<&FileEntry> {
        self.grouped_files.entries.get(self.selected_file)
    }

    pub fn selected_file_path(&self) -> Option<&Path> {
        let entry = self.selected_file_entry()?;
        self.selected_repo_ref()?
            .status_files
            .get(entry.file_index)
            .map(|file| file.path.as_path())
    }

    pub fn selected_file_section(&self) -> Option<FileSection> {
        self.selected_file_entry().map(|e| e.section)
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
