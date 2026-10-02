use anyhow::{bail, Context, Result};
use colored::Colorize;
use std::fs;
use std::path::Path;

use crate::cli::{BotRmArgs, BotSetArgs, CmdAddArgs, CmdListArgs, CmdRmArgs, ImportArgs};
use crate::domain::spec::{BotsConfig, TextSpec};
use crate::infra::config::resolve_bots_config_path;
use crate::infra::http::client::BotApiClient;

pub fn handle_bot_set(args: &BotSetArgs) -> Result<()> {
    let path = resolve_bots_config_path(&args.config);
    let mut config = if path.exists() {
        BotsConfig::from_file(&path)?
    } else {
        BotsConfig::default()
    };

    let bot = config.get_or_create_bot(&args.bot_id);

    if let Some(name) = &args.name {
        bot.name = Some(name.clone());
    }
    if let Some(token) = &args.token {
        bot.token = Some(token.clone());
    }
    if let Some(desc) = &args.desc {
        bot.description = Some(TextSpec { text: desc.clone() });
    }
    if let Some(short) = &args.short_desc {
        bot.short_description = Some(TextSpec {
            text: short.clone(),
        });
    }
    if let Some(avatar) = &args.avatar {
        bot.avatar = Some(avatar.clone());
    }
    if let Some(cover) = &args.cover {
        bot.cover = Some(cover.clone());
    }

    if args.privacy.is_some() || args.join_groups.is_some() {
        let settings = bot.settings.get_or_insert_with(Default::default);
        if let Some(p) = args.privacy {
            settings.privacy_mode = Some(p);
        }
        if let Some(j) = args.join_groups {
            settings.can_join_groups = Some(j);
        }
    }

    config.validate()?;
    config.save_file(&path)?;

    println!(
        "  {} Bot [{}] saved in [{}]",
        "✔".green().bold(),
        args.bot_id.cyan().bold(),
        path.display().to_string().yellow()
    );

    Ok(())
}

pub fn handle_bot_rm(args: &BotRmArgs) -> Result<()> {
    let path = resolve_bots_config_path(&args.config);
    if !path.exists() {
        bail!("Config file [{}] not found", path.display());
    }

    let mut config = BotsConfig::from_file(&path)?;
    if config.remove_bot(&args.bot_id) {
        config.save_file(&path)?;
        println!(
            "  {} Bot [{}] removed from [{}]",
            "✔".green().bold(),
            args.bot_id.cyan().bold(),
            path.display().to_string().yellow()
        );
    } else {
        println!(
            "  {} Bot [{}] was not found in [{}]",
            "ℹ".yellow().bold(),
            args.bot_id.cyan().bold(),
            path.display().to_string().yellow()
        );
    }

    Ok(())
}

pub fn handle_cmd_add(args: &CmdAddArgs) -> Result<()> {
    let path = resolve_bots_config_path(&args.config);
    let mut config = if path.exists() {
        BotsConfig::from_file(&path)?
    } else {
        BotsConfig::default()
    };

    config.add_or_update_command(&args.bot_id, &args.command, args.description.clone())?;
    config.save_file(&path)?;

    let clean = args.command.strip_prefix('/').unwrap_or(&args.command);
    println!(
        "  {} Command [/{}] configured for bot [{}] in [{}]",
        "✔".green().bold(),
        clean.cyan(),
        args.bot_id.cyan().bold(),
        path.display().to_string().yellow()
    );

    Ok(())
}

pub fn handle_cmd_rm(args: &CmdRmArgs) -> Result<()> {
    let path = resolve_bots_config_path(&args.config);
    if !path.exists() {
        bail!("Config file [{}] not found", path.display());
    }

    let mut config = BotsConfig::from_file(&path)?;
    let clean = args.command.strip_prefix('/').unwrap_or(&args.command);
    if config.remove_command(&args.bot_id, clean) {
        config.save_file(&path)?;
        println!(
            "  {} Command [/{}] removed from bot [{}]",
            "✔".green().bold(),
            clean.cyan(),
            args.bot_id.cyan().bold()
        );
    } else {
        println!(
            "  {} Command [/{}] was not found on bot [{}]",
            "ℹ".yellow().bold(),
            clean.cyan(),
            args.bot_id.cyan().bold()
        );
    }

    Ok(())
}

pub fn handle_cmd_list(args: &CmdListArgs) -> Result<()> {
    let path = resolve_bots_config_path(&args.config);
    if !path.exists() {
        bail!("Config file [{}] not found", path.display());
    }

    let config = BotsConfig::from_file(&path)?;
    let Some(bot) = config.bots.get(&args.bot_id) else {
        bail!("Bot [{}] not found in [{}]", args.bot_id, path.display());
    };

    println!("\nCommands for bot [{}]:", args.bot_id.cyan().bold());
    if bot.commands.is_empty() {
        println!("  (No commands configured)");
    } else {
        for cmd in &bot.commands {
            println!("  /{:<18} {}", cmd.command.green(), cmd.description);
        }
    }
    println!();

    Ok(())
}

pub async fn handle_import(args: ImportArgs) -> Result<()> {
    let path = resolve_bots_config_path(&args.config);
    let mut config = if path.exists() {
        BotsConfig::from_file(&path)?
    } else {
        BotsConfig::default()
    };

    if args.auto {
        println!("🔍 Scanning host for Telegram bot configurations...");
        import_host_bots(&mut config).await?;
    } else if let Some(token) = &args.token {
        let client = BotApiClient::new(token);
        let remote = client
            .fetch_remote_state()
            .await
            .context("Failed to query live bot state from Telegram")?;

        let bot_id = args.id.unwrap_or_else(|| {
            remote
                .username
                .clone()
                .unwrap_or_else(|| "imported_bot".to_string())
        });

        println!(
            "  Importing @{} into [{}]...",
            remote.username.as_deref().unwrap_or("unknown"),
            bot_id.cyan()
        );

        let bot = config.get_or_create_bot(&bot_id);
        bot.name = remote.name;
        bot.token = Some(token.clone());
        if let Some(desc) = remote.description {
            if !desc.is_empty() {
                bot.description = Some(TextSpec { text: desc });
            }
        }
        if let Some(short) = remote.short_description {
            if !short.is_empty() {
                bot.short_description = Some(TextSpec { text: short });
            }
        }
        bot.commands = remote.commands;
    } else {
        bail!("Specify either --auto or --token <TOKEN> to import");
    }

    config.validate()?;
    config.save_file(&path)?;

    println!(
        "  {} Successfully imported bots into [{}]",
        "✔".green().bold(),
        path.display().to_string().yellow()
    );

    Ok(())
}

async fn import_host_bots(config: &mut BotsConfig) -> Result<()> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());

    // 1. Ryoiki Bot
    let ryoiki_json = Path::new(&home).join(".config/ryoiki/telegram.json");
    if ryoiki_json.exists() {
        if let Ok(content) = fs::read_to_string(&ryoiki_json) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(token) = val.get("bot_token").and_then(|t| t.as_str()) {
                    let client = BotApiClient::new(token);
                    if let Ok(remote) = client.fetch_remote_state().await {
                        println!(
                            "  Found Ryoiki bot: @{}",
                            remote.username.as_deref().unwrap_or("unknown").cyan()
                        );
                        let bot = config.get_or_create_bot("ryoiki");
                        bot.name = remote.name;
                        bot.token = Some("env:RYOIKI_BOT_TOKEN".to_string());
                        if let Some(desc) = remote.description {
                            if !desc.is_empty() {
                                bot.description = Some(TextSpec { text: desc });
                            }
                        }
                        if let Some(short) = remote.short_description {
                            if !short.is_empty() {
                                bot.short_description = Some(TextSpec { text: short });
                            }
                        }
                        bot.commands = remote.commands;
                    }
                }
            }
        }
    }

    // 2. Tayori Bot
    let tayori_toml = Path::new(&home).join(".config/tayori/config.toml");
    if tayori_toml.exists() {
        if let Ok(content) = fs::read_to_string(&tayori_toml) {
            for line in content.lines() {
                if line.contains("bot_token") {
                    if let Some(tok) = line.split('=').nth(1) {
                        let clean_tok = tok.trim().trim_matches('"').trim_matches('\'');
                        let client = BotApiClient::new(clean_tok);
                        if let Ok(remote) = client.fetch_remote_state().await {
                            println!(
                                "  Found Tayori bot: @{}",
                                remote.username.as_deref().unwrap_or("unknown").cyan()
                            );
                            let bot = config.get_or_create_bot("tayori");
                            bot.name = remote.name;
                            bot.token = Some("env:TAYORI_BOT_TOKEN".to_string());
                            if let Some(desc) = remote.description {
                                if !desc.is_empty() {
                                    bot.description = Some(TextSpec { text: desc });
                                }
                            }
                            if let Some(short) = remote.short_description {
                                if !short.is_empty() {
                                    bot.short_description = Some(TextSpec { text: short });
                                }
                            }
                            bot.commands = remote.commands;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
