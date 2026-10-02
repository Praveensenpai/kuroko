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

    /// Manage bot fleet entries in configuration
    #[command(subcommand)]
    Bot(BotSubcommands),

    /// Manage bot command menus in configuration
    #[command(subcommand)]
    Cmd(CmdSubcommands),

    /// Import existing bot specifications from live Telegram Bot API or host configs
    Import(ImportArgs),

    /// Authenticate `MTProto` user session for `BotFather` automation
    Login(LoginArgs),

    /// Create a new bot via `@BotFather` and save credentials
    New(NewArgs),

    /// Generate a starter bots.toml template
    Init(InitArgs),
}

#[derive(Debug, Subcommand)]
pub enum BotSubcommands {
    /// Add or update bot metadata in bots.toml
    Set(BotSetArgs),

    /// Remove a bot from bots.toml
    Rm(BotRmArgs),
}

#[derive(Debug, Args)]
pub struct BotSetArgs {
    /// Bot ID key in bots.toml (e.g. 'ryoiki', 'tayori')
    pub bot_id: String,

    /// Display name of the bot
    #[arg(short, long)]
    pub name: Option<String>,

    /// Bot API token or '`env:VAR_NAME`'
    #[arg(short, long)]
    pub token: Option<String>,

    /// Full bot description
    #[arg(short, long)]
    pub desc: Option<String>,

    /// Short bot description (max 120 chars)
    #[arg(short = 's', long = "short-desc")]
    pub short_desc: Option<String>,

    /// Local file path to profile avatar image
    #[arg(long)]
    pub avatar: Option<String>,

    /// Local file path to intro video cover
    #[arg(long)]
    pub cover: Option<String>,

    /// Privacy mode toggle (true/false)
    #[arg(long)]
    pub privacy: Option<bool>,

    /// Group join toggle (true/false)
    #[arg(long)]
    pub join_groups: Option<bool>,

    /// Path to declarative bots configuration file
    #[arg(short, long, default_value = "bots.toml")]
    pub config: PathBuf,
}

#[derive(Debug, Args)]
pub struct BotRmArgs {
    /// Bot ID key to remove
    pub bot_id: String,

    /// Path to declarative bots configuration file
    #[arg(short, long, default_value = "bots.toml")]
    pub config: PathBuf,
}

#[derive(Debug, Subcommand)]
pub enum CmdSubcommands {
    /// Add or update a command in bots.toml
    Add(CmdAddArgs),

    /// Remove a command from bots.toml
    Rm(CmdRmArgs),

    /// List configured commands for a bot
    List(CmdListArgs),
}

#[derive(Debug, Args)]
pub struct CmdAddArgs {
    /// Target bot ID (e.g. 'ryoiki')
    pub bot_id: String,

    /// Command name without slash (e.g. 'status')
    pub command: String,

    /// Description of what the command does
    pub description: String,

    /// Path to declarative bots configuration file
    #[arg(short, long, default_value = "bots.toml")]
    pub config: PathBuf,
}

#[derive(Debug, Args)]
pub struct CmdRmArgs {
    /// Target bot ID
    pub bot_id: String,

    /// Command name to remove
    pub command: String,

    /// Path to declarative bots configuration file
    #[arg(short, long, default_value = "bots.toml")]
    pub config: PathBuf,
}

#[derive(Debug, Args)]
pub struct CmdListArgs {
    /// Target bot ID
    pub bot_id: String,

    /// Path to declarative bots configuration file
    #[arg(short, long, default_value = "bots.toml")]
    pub config: PathBuf,
}

#[derive(Debug, Args)]
pub struct ImportArgs {
    /// Target bot ID to assign in bots.toml (default: bot username)
    #[arg(short, long)]
    pub id: Option<String>,

    /// Telegram Bot token (or '`env:VAR_NAME`') to import
    #[arg(short, long)]
    pub token: Option<String>,

    /// Automatically discover known bot configurations on host (~/.config/ryoiki, ~/.config/tayori)
    #[arg(short, long)]
    pub auto: bool,

    /// Path to declarative bots configuration file
    #[arg(short, long, default_value = "bots.toml")]
    pub config: PathBuf,
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
