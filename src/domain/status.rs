use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChangedFile {
    pub path: PathBuf,
    pub staged: bool,
    pub unstaged: bool,
    pub untracked: bool,
    pub status_code: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RepositoryStatus {
    pub files: Vec<ChangedFile>,
}

impl RepositoryStatus {
    pub fn is_clean(&self) -> bool {
        self.files.is_empty()
    }
}
