use std::path::{Path, PathBuf};

use crate::domain::commit::LogEntry;
use crate::domain::pull_request::PrInfo;
use crate::domain::remote::RemoteInfo;
use crate::domain::repo::{RepositoryDetails, RepositorySummary};
use crate::domain::status::{ChangedFile, FileSection};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum View {
    #[default]
    Changes,
    Branches,
    Log,
    Remotes,
}

impl View {
    pub const ALL: &[View] = &[View::Changes, View::Branches, View::Log, View::Remotes];

    pub fn index(&self) -> usize {
        match self {
            View::Changes => 0,
            View::Branches => 1,
            View::Log => 2,
            View::Remotes => 3,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            View::Changes => "Changes",
            View::Branches => "Branches",
            View::Log => "Log",
            View::Remotes => "Remotes",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CreateRepoStep {
    Owner,
    RepoName,
    Visibility,
    Confirm,
}

#[derive(Clone, Debug)]
pub struct CreateRepoState {
    pub step: CreateRepoStep,
    pub owner_input: String,
    pub repo_name_input: String,
    pub is_public: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Modal {
    None,
    BranchSwitch,
    BranchCreate,
    Commit,
    CopilotLogin,
    CreateRepo(CreateRepoStep),
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
    pub log_entries: Vec<LogEntry>,
    pub remotes: Vec<RemoteInfo>,
    pub pull_requests: Vec<PrInfo>,
    pub has_origin_remote: bool,
    pub load_error: Option<String>,
}

impl RepositoryState {
    pub fn from_details(details: RepositoryDetails) -> Self {
        let has_origin_remote = details.remotes.iter().any(|r| r.name == "origin");
        Self {
            branches: details
                .branches
                .into_iter()
                .map(|branch| branch.name.to_string())
                .collect(),
            current_branch: details.current_branch,
            status_files: details.status.files,
            log_entries: details.log_entries,
            remotes: details.remotes,
            pull_requests: Vec::new(),
            has_origin_remote,
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
            log_entries: Vec::new(),
            remotes: Vec::new(),
            pull_requests: Vec::new(),
            has_origin_remote: false,
            load_error: Some(error),
        }
    }

    pub fn latest_commit_timestamp(&self) -> i64 {
        self.log_entries.first().map(|e| e.timestamp).unwrap_or(0)
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
    pub selected_log_entry: usize,
    pub selected_remote: usize,
    pub log_scroll: u16,
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
    pub ai_loading: bool,
    pub device_code: Option<crate::app::background::DeviceCodeInfo>,
    pub copilot_authenticated: bool,
    pub create_repo_state: Option<CreateRepoState>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            repos: Vec::new(),
            selected_repo: 0,
            selected_file: 0,
            selected_branch: 0,
            selected_log_entry: 0,
            selected_remote: 0,
            log_scroll: 0,
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
            ai_loading: false,
            device_code: None,
            copilot_authenticated: false,
            create_repo_state: None,
        }
    }
}

impl AppState {
    pub fn set_repositories(
        &mut self,
        mut repos: Vec<RepositoryState>,
        selected_path: Option<PathBuf>,
    ) {
        repos.sort_by(|a, b| {
            b.latest_commit_timestamp()
                .cmp(&a.latest_commit_timestamp())
        });
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
            let log_len = repo.log_entries.len();
            let remote_len = repo.remotes.len();
            self.grouped_files = grouped;
            self.selected_file = self.selected_file.min(entry_len.saturating_sub(1));
            self.selected_branch = self.selected_branch.min(branch_len.saturating_sub(1));
            self.selected_log_entry = self.selected_log_entry.min(log_len.saturating_sub(1));
            self.selected_remote = self.selected_remote.min(remote_len.saturating_sub(1));
        } else {
            self.grouped_files = GroupedFileList::default();
            self.selected_file = 0;
            self.selected_branch = 0;
            self.selected_log_entry = 0;
            self.selected_remote = 0;
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

    pub fn select_next_log_entry(&mut self) {
        if let Some(repo) = self.selected_repo_ref()
            && !repo.log_entries.is_empty()
        {
            self.selected_log_entry = (self.selected_log_entry + 1) % repo.log_entries.len();
        }
    }

    pub fn select_previous_log_entry(&mut self) {
        if let Some(repo) = self.selected_repo_ref()
            && !repo.log_entries.is_empty()
        {
            self.selected_log_entry = if self.selected_log_entry == 0 {
                repo.log_entries.len() - 1
            } else {
                self.selected_log_entry - 1
            };
        }
    }

    pub fn select_next_remote(&mut self) {
        if let Some(repo) = self.selected_repo_ref()
            && !repo.remotes.is_empty()
        {
            self.selected_remote = (self.selected_remote + 1) % repo.remotes.len();
        }
    }

    pub fn select_previous_remote(&mut self) {
        if let Some(repo) = self.selected_repo_ref()
            && !repo.remotes.is_empty()
        {
            self.selected_remote = if self.selected_remote == 0 {
                repo.remotes.len() - 1
            } else {
                self.selected_remote - 1
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
        self.create_repo_state = None;
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
            log_entries: Vec::new(),
            remotes: Vec::new(),
            pull_requests: Vec::new(),
            has_origin_remote: false,
            load_error: None,
        }
    }
}
