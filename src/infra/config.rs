use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct KurokoConfig {
    pub telegram_api_id: Option<i32>,
    pub telegram_api_hash: Option<String>,
}

impl KurokoConfig {
    pub fn config_dir() -> PathBuf {
        if let Ok(dir) = std::env::var("KUROKO_CONFIG_DIR") {
            PathBuf::from(dir)
        } else {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".config").join("kuroko")
        }
    }

    pub fn config_file() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    pub fn session_file() -> PathBuf {
        Self::config_dir().join("kuroko.session")
    }

    pub fn ensure_dir() -> Result<()> {
        let dir = Self::config_dir();
        if !dir.exists() {
            fs::create_dir_all(&dir)
                .with_context(|| format!("Failed to create config dir: {}", dir.display()))?;
        }
        Ok(())
    }

    pub fn load() -> Self {
        let path = Self::config_file();
        if let Ok(content) = fs::read_to_string(&path) {
            toml::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) -> Result<()> {
        Self::ensure_dir()?;
        let path = Self::config_file();
        let serialized = toml::to_string_pretty(self).context("Failed to serialize config")?;
        fs::write(&path, serialized)
            .with_context(|| format!("Failed to write config: {}", path.display()))?;
        Ok(())
    }

    pub fn get_api_credentials(&self) -> Option<(i32, String)> {
        // First check environment variables
        if let (Ok(id_str), Ok(hash)) = (
            std::env::var("TELEGRAM_API_ID"),
            std::env::var("TELEGRAM_API_HASH"),
        ) {
            if let Ok(id) = id_str.parse::<i32>() {
                if !hash.trim().is_empty() {
                    return Some((id, hash.trim().to_string()));
                }
            }
        }

        // Second check config file
        if let (Some(id), Some(hash)) = (self.telegram_api_id, &self.telegram_api_hash) {
            if !hash.trim().is_empty() {
                return Some((id, hash.trim().to_string()));
            }
        }

        None
    }
}

pub fn resolve_path(p: &str) -> PathBuf {
    if p.starts_with('~') {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        Path::new(&home).join(p.trim_start_matches("~/"))
    } else {
        PathBuf::from(p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_path_expands_tilde() {
        std::env::set_var("HOME", "/tmp/kuroko_home");
        let resolved = resolve_path("~/assets/avatar.png");
        assert_eq!(
            resolved,
            PathBuf::from("/tmp/kuroko_home/assets/avatar.png")
        );
    }
}
