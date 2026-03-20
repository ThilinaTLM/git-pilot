use anyhow::Result;

pub trait AiService {
    fn generate_commit_message(&self, diff: &str) -> Result<String>;
    fn generate_pr_description(&self, commits: &str, diff: &str) -> Result<(String, String)>;
}
