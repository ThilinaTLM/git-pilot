use anyhow::Result;

use crate::domain::ai::{GeneratedCommitMessage, GeneratedPrDescription};

pub trait AiService: Send + Sync {
    fn generate_commit_message(&self, diff: &str) -> Result<GeneratedCommitMessage>;
    fn generate_pr_description(&self, commits: &str, diff: &str) -> Result<GeneratedPrDescription>;
    fn generate_branch_name(&self, diff: &str) -> Result<String>;
}
