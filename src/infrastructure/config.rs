use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::domain::settings::AppSettings;

fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("git-tui").join("settings.toml"))
}

pub fn load_settings() -> AppSettings {
    let Some(path) = config_path() else {
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

pub fn save_settings(settings: &AppSettings) -> Result<()> {
    let path = config_path().context("could not determine config directory")?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("could not create config directory")?;
    }
    let contents = toml::to_string_pretty(settings).context("could not serialize settings")?;
    fs::write(&path, contents).context("could not write settings file")?;
    Ok(())
}
