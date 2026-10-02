use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BotsConfig {
    #[serde(default)]
    pub settings: Option<GlobalSettings>,
    #[serde(default)]
    pub bots: BTreeMap<String, BotSpec>,
}

impl Default for BotsConfig {
    fn default() -> Self {
        Self {
            settings: Some(GlobalSettings {
                default_language: default_language(),
            }),
            bots: BTreeMap::new(),
        }
    }
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

    pub fn save_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content =
            toml::to_string_pretty(self).context("Failed to serialize BotsConfig to TOML")?;
        if let Some(parent) = path.as_ref().parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
            }
        }
        fs::write(path.as_ref(), content)
            .with_context(|| format!("Failed to write config file: {}", path.as_ref().display()))?;
        Ok(())
    }

    pub fn get_or_create_bot(&mut self, id: &str) -> &mut BotSpec {
        self.bots.entry(id.to_string()).or_default()
    }

    pub fn remove_bot(&mut self, id: &str) -> bool {
        self.bots.remove(id).is_some()
    }

    pub fn add_or_update_command(
        &mut self,
        bot_id: &str,
        command: &str,
        description: String,
    ) -> Result<()> {
        validate_command_name(command, bot_id)?;
        if description.is_empty() || description.len() > 256 {
            bail!("Bot '{bot_id}' command description must be 1-256 characters");
        }
        let clean_cmd = command.strip_prefix('/').unwrap_or(command).to_string();
        let bot = self.get_or_create_bot(bot_id);
        if let Some(existing) = bot.commands.iter_mut().find(|c| c.command == clean_cmd) {
            existing.description = description;
        } else {
            bot.commands.push(CommandSpec {
                command: clean_cmd,
                description,
            });
        }
        Ok(())
    }

    pub fn remove_command(&mut self, bot_id: &str, command: &str) -> bool {
        let clean_cmd = command.strip_prefix('/').unwrap_or(command);
        if let Some(bot) = self.bots.get_mut(bot_id) {
            let initial_len = bot.commands.len();
            bot.commands.retain(|c| c.command != clean_cmd);
            bot.commands.len() < initial_len
        } else {
            false
        }
    }
}

impl BotSpec {
    pub fn resolve_token(&self) -> Option<String> {
        let raw = self.token.as_ref()?.trim();
        if let Some(var) = raw.strip_prefix("env:") {
            if let Ok(val) = std::env::var(var.trim()) {
                if !val.trim().is_empty() {
                    return Some(val);
                }
            }
            resolve_known_token_fallback(var.trim())
        } else if let Some(file_ref) = raw.strip_prefix("file:") {
            resolve_file_token(file_ref)
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

fn resolve_known_token_fallback(var_name: &str) -> Option<String> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    match var_name {
        "RYOIKI_BOT_TOKEN" => {
            let path = Path::new(&home).join(".config/ryoiki/telegram.json");
            resolve_token_from_json(&path, "bot_token")
        }
        "TAYORI_BOT_TOKEN" => {
            let path = Path::new(&home).join(".config/tayori/config.toml");
            resolve_token_from_toml(&path, "bot_token")
        }
        _ => None,
    }
}

fn resolve_file_token(file_ref: &str) -> Option<String> {
    let parts: Vec<&str> = file_ref.splitn(2, ':').collect();
    if parts.len() != 2 {
        return None;
    }
    let (file_path_str, key) = (parts[0], parts[1]);
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let path = if let Some(stripped) = file_path_str.strip_prefix("~/") {
        Path::new(&home).join(stripped)
    } else {
        PathBuf::from(file_path_str)
    };

    if Path::new(file_path_str)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
    {
        resolve_token_from_json(&path, key)
    } else {
        resolve_token_from_toml(&path, key)
    }
}

fn resolve_token_from_json(path: &Path, key: &str) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    let val: serde_json::Value = serde_json::from_str(&content).ok()?;
    val.get(key)
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string)
}

fn resolve_token_from_toml(path: &Path, key: &str) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(key) {
            let clean = trimmed
                .split('=')
                .nth(1)?
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if !clean.is_empty() {
                return Some(clean.to_string());
            }
        }
    }
    None
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

    #[test]
    fn test_mutation_add_and_remove_commands() {
        let mut cfg = BotsConfig::default();
        let res = cfg.add_or_update_command("mybot", "status", "Check health".to_string());
        assert!(res.is_ok());

        let bot = cfg.bots.get("mybot");
        assert!(bot.is_some());
        if let Some(b) = bot {
            assert_eq!(b.commands.len(), 1);
            assert_eq!(b.commands[0].command, "status");
            assert_eq!(b.commands[0].description, "Check health");
        }

        // Update description
        let res_update =
            cfg.add_or_update_command("mybot", "/status", "Updated health".to_string());
        assert!(res_update.is_ok());
        assert_eq!(cfg.bots["mybot"].commands[0].description, "Updated health");

        // Remove command
        assert!(cfg.remove_command("mybot", "status"));
        assert_eq!(cfg.bots["mybot"].commands.len(), 0);

        // Remove bot
        assert!(cfg.remove_bot("mybot"));
        assert!(!cfg.bots.contains_key("mybot"));
    }

    #[test]
    fn test_token_resolution_from_file() {
        let temp_dir = std::env::temp_dir();
        let json_file = temp_dir.join("test_token.json");
        std::fs::write(&json_file, r#"{"my_token": "secret_file_123"}"#).unwrap();

        let bot = BotSpec {
            token: Some(format!("file:{}:my_token", json_file.display())),
            ..Default::default()
        };
        assert_eq!(bot.resolve_token(), Some("secret_file_123".to_string()));
        let _ = std::fs::remove_file(json_file);
    }
}
