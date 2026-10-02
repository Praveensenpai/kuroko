use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;
use kuroko::cli::{ApplyArgs, Cli, Commands, DiffArgs, InitArgs, ListArgs, LoginArgs, NewArgs};
use kuroko::controller::{apply_action, compute_single_diff, select_target_bots};
use kuroko::domain::diff::BotDiff;
use kuroko::domain::spec::BotsConfig;
use kuroko::infra::config::KurokoConfig;
use kuroko::infra::http::BotApiClient;
use kuroko::infra::mtproto::{BotFatherClient, MtprotoEngine};
use kuroko::ui::table::FleetBotRow;
use kuroko::ui::{render_diff, render_fleet_table};
use std::fs;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init(args) => handle_init(&args)?,
        Commands::Diff(args) => handle_diff(args).await?,
        Commands::Apply(args) => handle_apply(args).await?,
        Commands::List(args) => handle_list(args).await?,
        Commands::Login(args) => handle_login(args).await?,
        Commands::New(args) => handle_new(args).await?,
    }

    Ok(())
}

fn handle_init(args: &InitArgs) -> Result<()> {
    if args.output.exists() {
        println!(
            "  {} Configuration file already exists at {}",
            "!".yellow().bold(),
            args.output.display()
        );
        return Ok(());
    }

    let template = r#"# 🌸 Kuroko (黒子) — Declarative Telegram Bot Fleet Configuration

[settings]
default_language = "en"

# ========================================================
# Example Bot
# ========================================================
[bots.example_bot]
name = "My Example Bot"
token = "env:EXAMPLE_BOT_TOKEN"
avatar = "assets/avatar.png"
cover = "assets/cover.mp4"

[bots.example_bot.description]
text = "A full-featured Telegram bot managed with Kuroko."

[bots.example_bot.short_description]
text = "Declarative bot automation."

[bots.example_bot.settings]
privacy_mode = false
can_join_groups = false
inline_mode = false

[[bots.example_bot.commands]]
command = "start"
description = "Start the bot"

[[bots.example_bot.commands]]
command = "help"
description = "Show help and commands"
"#;

    fs::write(&args.output, template)
        .with_context(|| format!("Failed to create {}", args.output.display()))?;

    println!(
        "  {} Initialized configuration at {}",
        "✔".green().bold(),
        args.output.display().to_string().cyan()
    );
    Ok(())
}

async fn handle_diff(args: DiffArgs) -> Result<()> {
    let config = BotsConfig::from_file(&args.config)?;
    let targets = select_target_bots(&config, args.bot.as_deref())?;

    println!(
        "🔍 Comparing local [{}] with live Telegram servers...",
        args.config.display().to_string().cyan()
    );

    for (bot_id, bot_spec) in targets {
        let diff = compute_single_diff(&bot_id, bot_spec).await?;
        render_diff(&diff);
    }

    Ok(())
}

async fn handle_apply(args: ApplyArgs) -> Result<()> {
    if args.dry_run {
        return handle_diff(DiffArgs {
            bot: args.bot,
            config: args.config,
        })
        .await;
    }

    let config = BotsConfig::from_file(&args.config)?;
    let targets = select_target_bots(&config, args.bot.as_deref())?;

    println!(
        "⚡ Reconciling Telegram fleet with [{}]...",
        args.config.display().to_string().cyan()
    );

    let mut mtproto: Option<MtprotoEngine> = None;

    for (bot_id, bot_spec) in targets {
        let diff = compute_single_diff(&bot_id, bot_spec).await?;
        render_diff(&diff);

        if diff.is_in_sync() {
            continue;
        }

        let token = bot_spec
            .resolve_token()
            .with_context(|| format!("Missing token for bot '{bot_id}'"))?;
        let http_client = BotApiClient::new(&token);

        for action in diff.actions {
            apply_action(
                &bot_id,
                bot_spec,
                &action,
                &http_client,
                args.http_only,
                &mut mtproto,
            )
            .await?;
        }

        println!(
            "  {} [{}] fully synchronized!",
            "✔".green().bold(),
            bot_id.cyan().bold()
        );
    }

    Ok(())
}

async fn handle_list(args: ListArgs) -> Result<()> {
    let config = BotsConfig::from_file(&args.config)?;
    let mut rows = Vec::new();

    println!("Querying status for {} bot(s)...", config.bots.len());

    for (bot_id, bot_spec) in &config.bots {
        let Some(token) = bot_spec.resolve_token() else {
            rows.push(FleetBotRow {
                bot_id: bot_id.clone(),
                username: "[No Token]".to_string(),
                name: bot_spec.name.clone().unwrap_or_default(),
                commands_count: bot_spec.commands.len(),
                avatar_status: "-".to_string(),
                sync_status: "No Token".to_string(),
            });
            continue;
        };

        let client = BotApiClient::new(&token);
        match client.fetch_remote_state().await {
            Ok(remote) => {
                let diff = BotDiff::compute(bot_id, bot_spec, &remote);
                let sync_status = if diff.is_in_sync() {
                    "Synced".to_string()
                } else {
                    "Diverged".to_string()
                };

                rows.push(FleetBotRow {
                    bot_id: bot_id.clone(),
                    username: remote.username.unwrap_or_else(|| "Unknown".to_string()),
                    name: remote
                        .name
                        .or_else(|| bot_spec.name.clone())
                        .unwrap_or_default(),
                    commands_count: remote.commands.len(),
                    avatar_status: if bot_spec.avatar.is_some() {
                        "Configured"
                    } else {
                        "-"
                    }
                    .to_string(),
                    sync_status,
                });
            }
            Err(_) => {
                rows.push(FleetBotRow {
                    bot_id: bot_id.clone(),
                    username: "[Error]".to_string(),
                    name: bot_spec.name.clone().unwrap_or_default(),
                    commands_count: bot_spec.commands.len(),
                    avatar_status: "-".to_string(),
                    sync_status: "API Error".to_string(),
                });
            }
        }
    }

    render_fleet_table(&rows);
    Ok(())
}

async fn handle_login(args: LoginArgs) -> Result<()> {
    let mut config = KurokoConfig::load();
    let api_id = if let Some(id) = args.api_id.or(config.telegram_api_id) {
        id
    } else {
        let input: String = dialoguer::Input::new()
            .with_prompt("Enter your Telegram API ID (from https://my.telegram.org)")
            .interact_text()
            .context("Failed to read API ID")?;
        input
            .trim()
            .parse::<i32>()
            .context("Invalid integer API ID")?
    };

    let api_hash = if let Some(hash) = args.api_hash.or_else(|| config.telegram_api_hash.clone()) {
        hash
    } else {
        let input: String = dialoguer::Input::new()
            .with_prompt("Enter your Telegram API Hash (from https://my.telegram.org)")
            .interact_text()
            .context("Failed to read API Hash")?;
        input.trim().to_string()
    };

    println!("Connecting to Telegram MTProto server...");
    let engine = MtprotoEngine::connect(api_id, &api_hash).await?;
    engine.login_interactive().await?;

    config.telegram_api_id = Some(api_id);
    config.telegram_api_hash = Some(api_hash);
    config.save()?;

    println!(
        "  {} Credentials and session saved to ~/.config/kuroko/",
        "✔".green().bold()
    );
    Ok(())
}

async fn handle_new(args: NewArgs) -> Result<()> {
    let config = KurokoConfig::load();
    let (api_id, api_hash) = config.get_api_credentials().context(
        "Creating a bot requires MTProto. Please run `kuroko login` first or set TELEGRAM_API_ID and TELEGRAM_API_HASH.",
    )?;

    let name = match args.name {
        Some(n) => n,
        None => dialoguer::Input::new()
            .with_prompt("Enter the display name for your new bot")
            .interact_text()?,
    };

    let username = match args.username {
        Some(u) => u,
        None => dialoguer::Input::new()
            .with_prompt("Enter the username (must end in 'bot')")
            .interact_text()?,
    };

    println!("Connecting to @BotFather via MTProto...");
    let engine = MtprotoEngine::connect(api_id, &api_hash).await?;
    let bf = BotFatherClient::new(engine.client());

    println!("Creating bot '@{username}' with name '{name}'...");
    let token = bf.create_new_bot(&name, &username).await?;

    println!("\n{}", "🌸 Bot Successfully Created!".green().bold());
    println!("  • Username: @{}", username.cyan());
    println!("  • Name: {name}");
    println!("  • Token: {}\n", token.yellow().bold());
    println!("Add the token to your `bots.toml` or environment:");
    println!(
        "  export {}_TOKEN=\"{token}\"",
        username.to_uppercase().replace('-', "_")
    );

    Ok(())
}
