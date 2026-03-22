use serde::{Deserialize, Serialize};

// ── Resolved settings (all fields concrete) ─────────────────────────

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default)]
    pub auto_refresh: AutoRefreshSettings,
    #[serde(default)]
    pub ai: AiSettings,
    #[serde(default)]
    pub pull_requests: PrSettings,
    #[serde(default)]
    pub commit: CommitSettings,
    #[serde(default)]
    pub ui: UiSettings,
}

impl AppSettings {
    pub fn apply_overrides(&mut self, partial: &PartialSettings) {
        if let Some(ar) = &partial.auto_refresh {
            ar.apply_to(&mut self.auto_refresh);
        }
        if let Some(ai) = &partial.ai {
            ai.apply_to(&mut self.ai);
        }
        if let Some(pr) = &partial.pull_requests {
            pr.apply_to(&mut self.pull_requests);
        }
        if let Some(c) = &partial.commit {
            c.apply_to(&mut self.commit);
        }
        if let Some(u) = &partial.ui {
            u.apply_to(&mut self.ui);
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AutoRefreshSettings {
    #[serde(default)]
    pub fetch_enabled: bool,
    #[serde(default = "default_fetch_interval")]
    pub fetch_interval_secs: u64,
    #[serde(default)]
    pub status_enabled: bool,
    #[serde(default = "default_status_interval")]
    pub status_interval_secs: u64,
    #[serde(default)]
    pub branches_enabled: bool,
    #[serde(default = "default_branches_interval")]
    pub branches_interval_secs: u64,
    #[serde(default)]
    pub prs_enabled: bool,
    #[serde(default = "default_prs_interval")]
    pub prs_interval_secs: u64,
}

fn default_fetch_interval() -> u64 {
    300
}
fn default_status_interval() -> u64 {
    60
}
fn default_branches_interval() -> u64 {
    300
}
fn default_prs_interval() -> u64 {
    300
}

impl Default for AutoRefreshSettings {
    fn default() -> Self {
        Self {
            fetch_enabled: false,
            fetch_interval_secs: default_fetch_interval(),
            status_enabled: false,
            status_interval_secs: default_status_interval(),
            branches_enabled: false,
            branches_interval_secs: default_branches_interval(),
            prs_enabled: false,
            prs_interval_secs: default_prs_interval(),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AiSettings {
    pub branch_name_instructions: Option<String>,
    pub commit_message_instructions: Option<String>,
    pub pr_title_instructions: Option<String>,
    pub pr_description_instructions: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PrSettings {
    pub default_base_branch: Option<String>,
    #[serde(default)]
    pub draft_by_default: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommitSettings {
    #[serde(default = "default_subject_max_length")]
    pub subject_max_length: usize,
}

fn default_subject_max_length() -> usize {
    72
}

impl Default for CommitSettings {
    fn default() -> Self {
        Self {
            subject_max_length: default_subject_max_length(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UiSettings {
    #[serde(default = "default_view")]
    pub default_view: String,
}

fn default_view() -> String {
    "changes".to_string()
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            default_view: default_view(),
        }
    }
}

// ── Partial settings (all fields optional, for overlay/merge) ────────

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PartialSettings {
    pub auto_refresh: Option<PartialAutoRefresh>,
    pub ai: Option<PartialAiSettings>,
    pub pull_requests: Option<PartialPrSettings>,
    pub commit: Option<PartialCommitSettings>,
    pub ui: Option<PartialUiSettings>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PartialAutoRefresh {
    pub fetch_enabled: Option<bool>,
    pub fetch_interval_secs: Option<u64>,
    pub status_enabled: Option<bool>,
    pub status_interval_secs: Option<u64>,
    pub branches_enabled: Option<bool>,
    pub branches_interval_secs: Option<u64>,
    pub prs_enabled: Option<bool>,
    pub prs_interval_secs: Option<u64>,
}

impl PartialAutoRefresh {
    fn apply_to(&self, target: &mut AutoRefreshSettings) {
        if let Some(v) = self.fetch_enabled {
            target.fetch_enabled = v;
        }
        if let Some(v) = self.fetch_interval_secs {
            target.fetch_interval_secs = v;
        }
        if let Some(v) = self.status_enabled {
            target.status_enabled = v;
        }
        if let Some(v) = self.status_interval_secs {
            target.status_interval_secs = v;
        }
        if let Some(v) = self.branches_enabled {
            target.branches_enabled = v;
        }
        if let Some(v) = self.branches_interval_secs {
            target.branches_interval_secs = v;
        }
        if let Some(v) = self.prs_enabled {
            target.prs_enabled = v;
        }
        if let Some(v) = self.prs_interval_secs {
            target.prs_interval_secs = v;
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PartialAiSettings {
    pub branch_name_instructions: Option<String>,
    pub commit_message_instructions: Option<String>,
    pub pr_title_instructions: Option<String>,
    pub pr_description_instructions: Option<String>,
}

impl PartialAiSettings {
    fn apply_to(&self, target: &mut AiSettings) {
        if self.branch_name_instructions.is_some() {
            target.branch_name_instructions = self.branch_name_instructions.clone();
        }
        if self.commit_message_instructions.is_some() {
            target.commit_message_instructions = self.commit_message_instructions.clone();
        }
        if self.pr_title_instructions.is_some() {
            target.pr_title_instructions = self.pr_title_instructions.clone();
        }
        if self.pr_description_instructions.is_some() {
            target.pr_description_instructions = self.pr_description_instructions.clone();
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PartialPrSettings {
    pub default_base_branch: Option<String>,
    pub draft_by_default: Option<bool>,
}

impl PartialPrSettings {
    fn apply_to(&self, target: &mut PrSettings) {
        if self.default_base_branch.is_some() {
            target.default_base_branch = self.default_base_branch.clone();
        }
        if let Some(v) = self.draft_by_default {
            target.draft_by_default = v;
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PartialCommitSettings {
    pub subject_max_length: Option<usize>,
}

impl PartialCommitSettings {
    fn apply_to(&self, target: &mut CommitSettings) {
        if let Some(v) = self.subject_max_length {
            target.subject_max_length = v;
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PartialUiSettings {
    pub default_view: Option<String>,
}

impl PartialUiSettings {
    fn apply_to(&self, target: &mut UiSettings) {
        if let Some(v) = &self.default_view {
            target.default_view = v.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings_have_expected_values() {
        let settings = AppSettings::default();
        assert!(!settings.auto_refresh.fetch_enabled);
        assert_eq!(settings.auto_refresh.fetch_interval_secs, 300);
        assert!(!settings.auto_refresh.status_enabled);
        assert_eq!(settings.auto_refresh.status_interval_secs, 60);
        assert_eq!(settings.commit.subject_max_length, 72);
        assert_eq!(settings.ui.default_view, "changes");
        assert!(!settings.pull_requests.draft_by_default);
    }

    #[test]
    fn apply_overrides_merges_partial_fields() {
        let mut settings = AppSettings::default();
        let partial = PartialSettings {
            auto_refresh: Some(PartialAutoRefresh {
                fetch_enabled: Some(true),
                fetch_interval_secs: Some(120),
                ..Default::default()
            }),
            commit: Some(PartialCommitSettings {
                subject_max_length: Some(50),
            }),
            ..Default::default()
        };

        settings.apply_overrides(&partial);

        assert!(settings.auto_refresh.fetch_enabled);
        assert_eq!(settings.auto_refresh.fetch_interval_secs, 120);
        // Unset fields remain default
        assert!(!settings.auto_refresh.status_enabled);
        assert_eq!(settings.auto_refresh.status_interval_secs, 60);
        assert_eq!(settings.commit.subject_max_length, 50);
    }

    #[test]
    fn apply_overrides_chains_correctly() {
        let mut settings = AppSettings::default();

        // Team config sets fetch enabled
        let team = PartialSettings {
            auto_refresh: Some(PartialAutoRefresh {
                fetch_enabled: Some(true),
                fetch_interval_secs: Some(600),
                ..Default::default()
            }),
            ..Default::default()
        };
        settings.apply_overrides(&team);

        // Personal config overrides interval only
        let personal = PartialSettings {
            auto_refresh: Some(PartialAutoRefresh {
                fetch_interval_secs: Some(60),
                ..Default::default()
            }),
            ..Default::default()
        };
        settings.apply_overrides(&personal);

        assert!(settings.auto_refresh.fetch_enabled); // from team
        assert_eq!(settings.auto_refresh.fetch_interval_secs, 60); // from personal
    }

    #[test]
    fn roundtrip_toml_serialization() {
        let settings = AppSettings::default();
        let toml_str = toml::to_string_pretty(&settings).unwrap();
        let parsed: AppSettings = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.auto_refresh.fetch_interval_secs, 300);
        assert_eq!(parsed.commit.subject_max_length, 72);
    }

    #[test]
    fn partial_settings_deserializes_from_minimal_toml() {
        let toml_str = r#"
[auto_refresh]
fetch_enabled = true

[commit]
subject_max_length = 50
"#;
        let partial: PartialSettings = toml::from_str(toml_str).unwrap();
        assert_eq!(partial.auto_refresh.unwrap().fetch_enabled, Some(true));
        assert_eq!(partial.commit.unwrap().subject_max_length, Some(50));
        assert!(partial.ai.is_none());
        assert!(partial.ui.is_none());
    }
}
