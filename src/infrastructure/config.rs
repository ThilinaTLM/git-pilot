use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::domain::settings::{AppSettings, PartialSettings};

fn user_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("git-pilot").join("settings.toml"))
}

/// Load user-level settings from `~/.config/git-pilot/settings.toml`.
/// Missing or unparseable file falls back to defaults.
pub fn load_user_settings() -> AppSettings {
    let Some(path) = user_config_path() else {
        return AppSettings::default();
    };
    if !path.exists() {
        return AppSettings::default();
    }
    let contents = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return AppSettings::default(),
    };
    toml::from_str(&contents).unwrap_or_default()
}

/// Load team-level partial settings from `.git-pilot.toml` in the repo root.
pub fn load_repo_team_settings(repo_root: &Path) -> Option<PartialSettings> {
    load_partial_from(repo_root.join(".git-pilot.toml"))
}

/// Load personal-level partial settings from `.git/git-pilot.toml`.
pub fn load_repo_personal_settings(repo_root: &Path) -> Option<PartialSettings> {
    load_partial_from(repo_root.join(".git").join("git-pilot.toml"))
}

fn load_partial_from(path: PathBuf) -> Option<PartialSettings> {
    let contents = fs::read_to_string(&path).ok()?;
    toml::from_str(&contents).ok()
}

/// Resolve settings using three-tier merge: defaults → user → team → personal.
pub fn resolve_settings(repo_root: &Path) -> AppSettings {
    let mut settings = load_user_settings();

    if let Some(team) = load_repo_team_settings(repo_root) {
        settings.apply_overrides(&team);
    }
    if let Some(personal) = load_repo_personal_settings(repo_root) {
        settings.apply_overrides(&personal);
    }

    settings
}

/// Save settings to the user-level config file only.
pub fn save_settings(settings: &AppSettings) -> Result<()> {
    let path = user_config_path().context("could not determine config directory")?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("could not create config directory")?;
    }
    let contents = toml::to_string_pretty(settings).context("could not serialize settings")?;
    fs::write(&path, contents).context("could not write settings file")?;
    Ok(())
}
