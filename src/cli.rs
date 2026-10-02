use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "kuroko",
    about = "🌸 Kuroko (黒子) — Declarative Telegram Bot Fleet Orchestrator",
    version,
    propagate_version = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Show differences between local bots.toml and live Telegram state
    Diff(DiffArgs),

    /// Reconcile and apply declarative configuration to Telegram
    Apply(ApplyArgs),

    /// Display dashboard of all managed bots and live status
    List(ListArgs),

    /// Authenticate `MTProto` user session for `BotFather` automation
    Login(LoginArgs),

    /// Create a new bot via `@BotFather` and save credentials
    New(NewArgs),

    /// Generate a starter bots.toml template
    Init(InitArgs),
}

#[derive(Debug, Args)]
pub struct DiffArgs {
    /// Optional specific bot ID to diff
    pub bot: Option<String>,

    /// Path to declarative bots configuration file
    #[arg(short, long, default_value = "bots.toml")]
    pub config: PathBuf,
}

#[derive(Debug, Args)]
pub struct ApplyArgs {
    /// Optional specific bot ID to apply
    pub bot: Option<String>,

    /// Path to declarative bots configuration file
    #[arg(short, long, default_value = "bots.toml")]
    pub config: PathBuf,

    /// Only apply HTTP Bot API changes (skip `MTProto` avatar and settings)
    #[arg(long)]
    pub http_only: bool,

    /// Preview changes without applying
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Debug, Args)]
pub struct ListArgs {
    /// Path to declarative bots configuration file
    #[arg(short, long, default_value = "bots.toml")]
    pub config: PathBuf,
}

#[derive(Debug, Args)]
pub struct LoginArgs {
    /// Telegram API ID (defaults to config or `TELEGRAM_API_ID`)
    #[arg(long)]
    pub api_id: Option<i32>,

    /// Telegram API Hash (defaults to config or `TELEGRAM_API_HASH`)
    #[arg(long)]
    pub api_hash: Option<String>,
}

#[derive(Debug, Args)]
pub struct NewArgs {
    /// Display name of the new bot
    #[arg(short, long)]
    pub name: Option<String>,

    /// Desired username (ending in bot)
    #[arg(short, long)]
    pub username: Option<String>,
}

#[derive(Debug, Args)]
pub struct InitArgs {
    /// Output file path for initial configuration
    #[arg(short, long, default_value = "bots.toml")]
    pub output: PathBuf,
}
