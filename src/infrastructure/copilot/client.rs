use std::sync::Mutex;

use anyhow::{Context, Result, anyhow};

use crate::domain::ai::{GeneratedCommitMessage, GeneratedPrDescription};
use crate::infrastructure::ai::AiService;

use super::auth::CopilotTokenManager;
use super::prompts;
use super::types::{ChatCompletionRequest, ChatCompletionResponse, ChatMessage};

const MAX_PR_DIFF_CHARS: usize = 12000;

pub struct CopilotAiService {
    token_manager: Mutex<CopilotTokenManager>,
    model: String,
}

impl CopilotAiService {
    pub fn new(token_manager: CopilotTokenManager, model: String) -> Self {
        Self {
            token_manager: Mutex::new(token_manager),
            model,
        }
    }

    fn chat_completion(&self, system_prompt: &str, user_content: &str) -> Result<String> {
        let token = self
            .token_manager
            .lock()
            .map_err(|e| anyhow!("lock error: {e}"))?
            .get_token()?;

        let request = ChatCompletionRequest {
            model: self.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: user_content.to_string(),
                },
            ],
            temperature: 0.3,
        };

        let body = serde_json::to_string(&request)?;

        let mut response = ureq::post("https://api.githubcopilot.com/chat/completions")
            .header("Authorization", &format!("Bearer {token}"))
            .header("Content-Type", "application/json")
            .header("Copilot-Integration-Id", "vscode-chat")
            .header("Editor-Version", "vscode/1.90.0")
            .send(body.as_bytes())?;

        let resp: ChatCompletionResponse = response
            .body_mut()
            .read_json()
            .context("failed to parse chat completion response")?;

        resp.choices
            .into_iter()
            .next()
            .and_then(|c| c.message.content)
            .ok_or_else(|| anyhow!("no response content from Copilot"))
    }
}

impl AiService for CopilotAiService {
    fn generate_commit_message(&self, diff: &str) -> Result<GeneratedCommitMessage> {
        let content = self.chat_completion(prompts::commit_message_system_prompt(), diff)?;
        let content = content.trim();

        // Parse subject and optional body
        if let Some(pos) = content.find("\n\n") {
            Ok(GeneratedCommitMessage {
                subject: content[..pos].trim().to_string(),
                body: Some(content[pos + 2..].trim().to_string()),
            })
        } else {
            Ok(GeneratedCommitMessage {
                subject: content.to_string(),
                body: None,
            })
        }
    }

    fn generate_pr_description(&self, commits: &str, diff: &str) -> Result<GeneratedPrDescription> {
        let truncated_diff = truncate_pr_diff(diff);
        let user_content = format!("## Commits\n{commits}\n\n## Diff\n{truncated_diff}");
        let content =
            self.chat_completion(prompts::pr_description_system_prompt(), &user_content)?;

        // Parse TITLE: and BODY: sections
        let title = content
            .lines()
            .find(|l| l.starts_with("TITLE:"))
            .map(|l| l.trim_start_matches("TITLE:").trim().to_string())
            .unwrap_or_else(|| "Update".to_string());

        let body = if let Some(pos) = content.find("BODY:") {
            content[pos + 5..].trim().to_string()
        } else {
            content.clone()
        };

        Ok(GeneratedPrDescription { title, body })
    }
}

fn truncate_pr_diff(diff: &str) -> &str {
    if diff.len() <= MAX_PR_DIFF_CHARS {
        diff
    } else {
        let truncated = &diff[..MAX_PR_DIFF_CHARS];
        truncated
            .rfind('\n')
            .map(|pos| &diff[..pos])
            .unwrap_or(truncated)
    }
}
