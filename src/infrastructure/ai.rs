use anyhow::Result;

use crate::domain::ai::{GeneratedCommitMessage, GeneratedPrDescription};
use crate::domain::settings::AiSettings;

pub trait AiService: Send + Sync {
    fn generate_commit_message(
        &self,
        diff: &str,
        ai_settings: &AiSettings,
    ) -> Result<GeneratedCommitMessage>;
    fn generate_pr_description(
        &self,
        commits: &str,
        diff: &str,
        ai_settings: &AiSettings,
    ) -> Result<GeneratedPrDescription>;
    fn generate_branch_name(&self, diff: &str, ai_settings: &AiSettings) -> Result<String>;
}
