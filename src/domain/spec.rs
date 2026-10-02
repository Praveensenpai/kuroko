use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BotsConfig {
    #[serde(default)]
    pub settings: Option<GlobalSettings>,
    #[serde(default)]
    pub bots: HashMap<String, BotSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GlobalSettings {
    #[serde(default = "default_language")]
    pub default_language: String,
}

fn default_language() -> String {
    "en".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TextSpec {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct BotSettingsSpec {
    pub privacy_mode: Option<bool>,
    pub can_join_groups: Option<bool>,
    pub inline_mode: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommandSpec {
    pub command: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct BotSpec {
    pub name: Option<String>,
    pub token: Option<String>,
    pub avatar: Option<String>,
    pub cover: Option<String>,
    pub description: Option<TextSpec>,
    pub short_description: Option<TextSpec>,
    pub settings: Option<BotSettingsSpec>,
    #[serde(default)]
    pub commands: Vec<CommandSpec>,
}

impl BotsConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read config file: {}", path.as_ref().display()))?;
        Self::parse_str(&content)
    }

    pub fn parse_str(raw: &str) -> Result<Self> {
        <Self as std::str::FromStr>::from_str(raw)
    }
}

impl std::str::FromStr for BotsConfig {
    type Err = anyhow::Error;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let config: BotsConfig = toml::from_str(raw).context("Failed to parse TOML config")?;
        config.validate()?;
        Ok(config)
    }
}

impl BotsConfig {
    pub fn validate(&self) -> Result<()> {
        for (id, bot) in &self.bots {
            bot.validate(id)?;
        }
        Ok(())
    }
}

impl BotSpec {
    pub fn resolve_token(&self) -> Option<String> {
        let raw = self.token.as_ref()?.trim();
        if let Some(var) = raw.strip_prefix("env:") {
            std::env::var(var.trim()).ok()
        } else if !raw.is_empty() {
            Some(raw.to_string())
        } else {
            None
        }
    }

    pub fn validate(&self, bot_id: &str) -> Result<()> {
        if let Some(desc) = &self.description {
            if desc.text.len() > 512 {
                bail!(
                    "Bot '{bot_id}' description exceeds 512 characters (found {})",
                    desc.text.len()
                );
            }
        }

        if let Some(short) = &self.short_description {
            if short.text.len() > 120 {
                bail!(
                    "Bot '{bot_id}' short_description exceeds 120 characters (found {})",
                    short.text.len()
                );
            }
        }

        for cmd in &self.commands {
            validate_command_name(&cmd.command, bot_id)?;
            if cmd.description.is_empty() || cmd.description.len() > 256 {
                bail!(
                    "Bot '{bot_id}' command '/{}' description must be 1-256 characters",
                    cmd.command
                );
            }
        }

        Ok(())
    }
}

fn validate_command_name(name: &str, bot_id: &str) -> Result<()> {
    let clean = name.strip_prefix('/').unwrap_or(name);
    if clean.is_empty() || clean.len() > 32 {
        bail!("Bot '{bot_id}' command '/{clean}' name must be 1-32 characters");
    }
    let valid = clean
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
    if !valid {
        bail!("Bot '{bot_id}' command '/{clean}' must contain only lowercase a-z, 0-9, and _");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_spec() {
        let toml_data = r#"
        [settings]
        default_language = "en"

        [bots.ryoiki]
        name = "Ryoiki Server Bot"
        token = "123456:ABC-DEF"
        avatar = "assets/avatar.png"

        [bots.ryoiki.description]
        text = "Server control center"

        [bots.ryoiki.short_description]
        text = "Server control"

        [[bots.ryoiki.commands]]
        command = "status"
        description = "Show telemetry"
        "#;

        let res = BotsConfig::parse_str(toml_data);
        assert!(res.is_ok());
        if let Ok(cfg) = res {
            assert!(cfg.bots.contains_key("ryoiki"));
            if let Some(bot) = cfg.bots.get("ryoiki") {
                assert_eq!(bot.name.as_deref(), Some("Ryoiki Server Bot"));
                assert_eq!(bot.commands.len(), 1);
                assert_eq!(bot.commands[0].command, "status");
            }
        }
    }

    #[test]
    fn test_rejects_long_short_description() {
        let long_text = "a".repeat(125);
        let toml_data = format!(
            r#"
            [bots.bad]
            [bots.bad.short_description]
            text = "{long_text}"
            "#
        );
        let res = BotsConfig::parse_str(&toml_data);
        assert!(res.is_err());
    }

    #[test]
    fn test_rejects_invalid_command_characters() {
        let toml_data = r#"
        [bots.bad]
        [[bots.bad.commands]]
        command = "Status!"
        description = "Invalid exclamation"
        "#;
        let res = BotsConfig::parse_str(toml_data);
        assert!(res.is_err());
    }

    #[test]
    fn test_token_resolution_from_env() {
        std::env::set_var("TEST_BOT_TOKEN_VAR", "secret123");
        let bot = BotSpec {
            token: Some("env:TEST_BOT_TOKEN_VAR".to_string()),
            ..Default::default()
        };
        assert_eq!(bot.resolve_token(), Some("secret123".to_string()));
    }
}
