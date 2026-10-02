# CODEBASE.md: Kuroko (黒子) Semantic Digest

> **Notice**: This file is an AI-optimized semantic index. Do not write narrative prose. Keep token density high.

## 1. System Topology & Data Flow
```text
CLI Invocation (cli.rs / main.rs)
       │
       ▼
Spec Loader & Validator (domain/spec.rs) ── parses ──> bots.toml
       │
       ▼
Controller Dispatcher (controller.rs)
  ├── Diff / Apply Engine (domain/diff.rs)
  │     ├── Remote Fetcher (infra/http/client.rs) ── queries ──> Telegram Bot API
  │     ├── Diff Calculator (domain/diff.rs) ── computes ──> Vec<DiffAction>
  │     ├── Visual Diff Renderer (ui/diff_view.rs)
  │     ├── Bot API Mutator (infra/http/client.rs) ── writes ──> Names, Commands, Descriptions
  │     └── BotFather Automator (infra/mtproto/botfather.rs) ── automates ──> Avatars, Covers, Settings
  ├── Bot Creator (infra/mtproto/botfather.rs) ── interactive /newbot ──> Telegram MTProto
  └── Session Auth (infra/mtproto/client.rs) ── QR / Phone / 2FA ──> SQLite Session DB (~/.config/kuroko/session.db)
```

## 2. Global Constraints & Architecture Patterns
- **Primary Language & Edition**: Rust 2024 Edition (`1.85+`).
- **Architectural Paradigm**: Strict role-based division (`domain/`, `infra/`, `ui/`, `cli.rs`, `controller.rs`).
- **Hard Constraints**: <400 lines/file (300 soft), <60 lines/fn (40 soft), zero production `unwrap()`/`expect()`, zero dead code warnings (`-D warnings`).
- **State Reconciliation**: Declarative "Bot-as-Code" idempotency; calculate `DiffAction` state changes before mutation.
- **Dual Telegram Client Strategy**:
  - HTTP Bot API (`reqwest`) for fast, non-privileged operations (commands, descriptions, names).
  - MTProto (`grammers-client` with SQLite session) for BotFather backstage orchestration (avatars, covers, privacy settings, bot creation).

## 3. Module & Interface Skeleton

### `src/lib.rs` (Role: root, Lines: 5)
- **Responsibility**: Crate entrypoint declaring public submodules.
- **Imports**: `pub mod cli;`, `pub mod controller;`, `pub mod domain;`, `pub mod infra;`, `pub mod ui;`
- **Consumers**: `src/main.rs`, integration tests.
- **Side Effects / I/O**: None.

### `src/cli.rs` (Role: cli, Lines: 99)
- **Responsibility**: Command-line argument definitions and flag parsing using `clap`.
- **Imports**: `clap::{Parser, Subcommand, ValueEnum}`, `std::path::PathBuf`
- **Types & Enums**:
  ```rust
  pub struct Cli { pub command: Commands }
  pub enum Commands {
      Diff(DiffArgs), Apply(ApplyArgs), List(ListArgs),
      Login(LoginArgs), New(NewBotArgs), Init(InitArgs),
  }
  pub struct DiffArgs { pub file: PathBuf, pub bot: Option<String> }
  pub struct ApplyArgs { pub file: PathBuf, pub bot: Option<String>, pub yes: bool, pub no_mtproto: bool }
  pub struct ListArgs { pub file: PathBuf }
  pub struct LoginArgs { pub config: Option<PathBuf> }
  pub struct NewBotArgs { pub name: Option<String>, pub username: Option<String>, pub output: Option<PathBuf> }
  pub struct InitArgs { pub output: PathBuf }
  ```
- **Consumers**: `src/main.rs`
- **Side Effects / I/O**: None.

### `src/domain/spec.rs` (Role: domain, Lines: 219)
- **Responsibility**: Declarative TOML schema representation, deserialization, validation, and token resolution.
- **Imports**: `anyhow::{bail, Context, Result}`, `serde::{Deserialize, Serialize}`, `std::{collections::BTreeMap, env, path::Path}`
- **Types & Enums**:
  ```rust
  pub struct FleetSpec { pub settings: GlobalSettings, pub bots: BTreeMap<String, BotSpec> }
  pub struct GlobalSettings { pub default_language: Option<String> }
  pub struct BotSpec { pub name: Option<String>, pub token: String, pub avatar: Option<String>, pub cover: Option<String>, pub description: Option<DescriptionSpec>, pub short_description: Option<DescriptionSpec>, pub settings: Option<BotSettingsSpec>, pub commands: Vec<BotCommandSpec> }
  pub struct DescriptionSpec { pub text: String, pub language_code: Option<String> }
  pub struct BotSettingsSpec { pub privacy_mode: Option<bool>, pub can_join_groups: Option<bool>, pub inline_mode: Option<bool> }
  pub struct BotCommandSpec { pub command: String, pub description: String, pub language_code: Option<String> }
  ```
- **Public Functions & Signatures**:
  ```rust
  impl FleetSpec {
      pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self>;
      pub fn parse_str(toml_str: &str) -> Result<Self>;
      pub fn validate(&self) -> Result<()>;
  }
  impl BotSpec {
      pub fn resolve_token(&self) -> Option<String>;
  }
  ```
- **Consumers**: `src/controller.rs`, `src/domain/diff.rs`, `src/main.rs`.
- **Side Effects / I/O**: Reads files from disk (`std::fs::read_to_string`), reads environment variables (`std::env::var`), supports smart fallback to `~/.config/ryoiki/telegram.json` and `~/.config/tayori/config.toml` or `file:...` references.

### `src/domain/diff.rs` (Role: domain, Lines: 325)
- **Responsibility**: Reconciles live Telegram state against local desired spec and computes discrete diff actions.
- **Imports**: `super::spec::*`, `crate::infra::http::types::*`, `colored::*`, `std::path::PathBuf`
- **Types & Enums**:
  ```rust
  pub enum DiffAction {
      UpdateName { from: Option<String>, to: String },
      UpdateDescription { language_code: Option<String>, from: Option<String>, to: String },
      UpdateShortDescription { language_code: Option<String>, from: Option<String>, to: String },
      UpdateCommands { language_code: Option<String>, added: Vec<BotCommandSpec>, removed: Vec<BotCommand>, modified: Vec<(BotCommand, BotCommandSpec)> },
      SetAvatar { path: PathBuf },
      SetCover { path: PathBuf },
      UpdatePrivacyMode { to: bool },
      UpdateGroupJoin { to: bool },
      UpdateInlineMode { to: bool },
  }
  pub struct BotDiff { pub bot_id: String, pub actions: Vec<DiffAction> }
  ```
- **Public Functions & Signatures**:
  ```rust
  impl BotDiff {
      pub fn compute(bot_id: &str, desired: &BotSpec, live_state: &LiveBotState) -> Self;
      pub fn is_in_sync(&self) -> bool;
      pub fn has_mtproto_actions(&self) -> bool;
  }
  ```
- **Consumers**: `src/controller.rs`, `src/ui/diff_view.rs`.
- **Side Effects / I/O**: None (pure domain logic).

### `src/infra/config.rs` (Role: infra, Lines: 103)
- **Responsibility**: XDG configuration directory management, credential resolution for MTProto API, and session pathing.
- **Imports**: `anyhow::{Context, Result}`, `serde::{Deserialize, Serialize}`, `std::{env, fs, path::PathBuf}`
- **Types & Enums**:
  ```rust
  pub struct KurokoConfig { pub telegram: TelegramCredentials }
  pub struct TelegramCredentials { pub api_id: i32, pub api_hash: String, pub session_file: Option<PathBuf> }
  ```
- **Public Functions & Signatures**:
  ```rust
  pub fn config_dir() -> Result<PathBuf>;
  pub fn default_session_path() -> Result<PathBuf>;
  impl KurokoConfig {
      pub fn load(explicit_path: Option<&std::path::Path>) -> Result<Self>;
      pub fn save_default() -> Result<PathBuf>;
  }
  ```
- **Consumers**: `src/main.rs`, `src/controller.rs`, `src/infra/mtproto/client.rs`.
- **Side Effects / I/O**: Reads/writes `~/.config/kuroko/config.toml`, creates directory trees.

### `src/infra/http/types.rs` (Role: infra, Lines: 57)
- **Responsibility**: Strongly-typed serde models for Telegram HTTP Bot API responses.
- **Imports**: `serde::{Deserialize, Serialize}`
- **Types & Enums**:
  ```rust
  pub struct ApiResponse<T> { pub ok: bool, pub result: Option<T>, pub description: Option<String>, pub error_code: Option<i32> }
  pub struct BotUser { pub id: i64, pub is_bot: bool, pub first_name: String, pub username: Option<String>, pub can_join_groups: Option<bool>, pub can_read_all_group_messages: Option<bool>, pub supports_inline_queries: Option<bool> }
  pub struct BotName { pub name: String }
  pub struct BotDescription { pub description: String }
  pub struct BotShortDescription { pub short_description: String }
  pub struct BotCommand { pub command: String, pub description: String }
  pub struct LiveBotState { pub user: BotUser, pub name: Option<String>, pub description: Option<String>, pub short_description: Option<String>, pub commands: Vec<BotCommand> }
  ```
- **Consumers**: `src/infra/http/client.rs`, `src/domain/diff.rs`.
- **Side Effects / I/O**: None.

### `src/infra/http/client.rs` (Role: infra, Lines: 174)
- **Responsibility**: Asynchronous HTTP client wrapping Telegram Bot API endpoints.
- **Imports**: `super::types::*`, `anyhow::{bail, Context, Result}`, `reqwest::Client`, `serde::Serialize`
- **Types & Enums**:
  ```rust
  pub struct BotApiClient { client: Client, token: String }
  ```
- **Public Functions & Signatures**:
  ```rust
  impl BotApiClient {
      pub fn new(token: String) -> Self;
      pub async fn get_me(&self) -> Result<BotUser>;
      pub async fn get_my_name(&self, language_code: Option<&str>) -> Result<Option<String>>;
      pub async fn get_my_description(&self, language_code: Option<&str>) -> Result<Option<String>>;
      pub async fn get_my_short_description(&self, language_code: Option<&str>) -> Result<Option<String>>;
      pub async fn get_my_commands(&self, language_code: Option<&str>) -> Result<Vec<BotCommand>>;
      pub async fn fetch_live_state(&self, language_code: Option<&str>) -> Result<LiveBotState>;
      pub async fn set_my_name(&self, name: &str, language_code: Option<&str>) -> Result<()>;
      pub async fn set_my_description(&self, description: &str, language_code: Option<&str>) -> Result<()>;
      pub async fn set_my_short_description(&self, short_description: &str, language_code: Option<&str>) -> Result<()>;
      pub async fn set_my_commands(&self, commands: &[BotCommand], language_code: Option<&str>) -> Result<()>;
      pub async fn delete_my_commands(&self, language_code: Option<&str>) -> Result<()>;
  }
  ```
- **Consumers**: `src/controller.rs`.
- **Side Effects / I/O**: Network I/O to `https://api.telegram.org`.

### `src/infra/mtproto/session.rs` (Role: infra, Lines: 14)
- **Responsibility**: SQLite session file management for Grammers MTProto storage.
- **Imports**: `anyhow::{Context, Result}`, `grammers_session::SqliteSession`, `std::{path::Path, sync::Arc}`
- **Public Functions & Signatures**:
  ```rust
  pub async fn open_session(path: &Path) -> Result<Arc<SqliteSession>>;
  ```
- **Consumers**: `src/infra/mtproto/client.rs`.
- **Side Effects / I/O**: SQLite database creation/access on local filesystem.

### `src/infra/mtproto/client.rs` (Role: infra, Lines: 84)
- **Responsibility**: MTProto user client initialization and interactive 2FA/login flow.
- **Imports**: `super::session::open_session`, `anyhow::{Context, Result}`, `dialoguer::{Password, Input}`, `grammers_client::{Client, Config, InitParams}`, `std::path::Path`
- **Types & Enums**:
  ```rust
  pub struct MtprotoClient { client: Client }
  ```
- **Public Functions & Signatures**:
  ```rust
  impl MtprotoClient {
      pub async fn connect(api_id: i32, session_path: &Path) -> Result<Self>;
      pub async fn login_interactive(&self, api_id: i32, api_hash: &str) -> Result<()>;
      pub fn client(&self) -> &Client;
  }
  ```
- **Consumers**: `src/main.rs`, `src/controller.rs`.
- **Side Effects / I/O**: Interactive terminal prompts (`dialoguer`), MTProto TCP/TLS connections to Telegram DC.

### `src/infra/mtproto/botfather.rs` (Role: infra, Lines: 206)
- **Responsibility**: Automates backstage `@BotFather` bot operations over MTProto userbot messages.
- **Imports**: `anyhow::{bail, Context, Result}`, `grammers_client::{types::InputMedia, Client}`, `std::{path::Path, time::Duration}`, `tokio::time::sleep`
- **Types & Enums**:
  ```rust
  pub struct BotFatherAutomation<'a> { client: &'a Client }
  ```
- **Public Functions & Signatures**:
  ```rust
  impl<'a> BotFatherAutomation<'a> {
      pub fn new(client: &'a Client) -> Self;
      pub async fn send_and_wait_response(&self, text: &str) -> Result<String>;
      pub async fn set_userpic(&self, bot_username: &str, photo_path: &Path) -> Result<()>;
      pub async fn set_intro(&self, bot_username: &str, media_path: &Path) -> Result<()>;
      pub async fn set_privacy(&self, bot_username: &str, enabled: bool) -> Result<()>;
      pub async fn set_join_groups(&self, bot_username: &str, enabled: bool) -> Result<()>;
      pub async fn set_inline(&self, bot_username: &str, enabled: bool) -> Result<()>;
      pub async fn create_new_bot(&self, name: &str, username: &str) -> Result<String>;
  }
  pub fn extract_token(text: &str) -> Result<String>;
  ```
- **Consumers**: `src/controller.rs`, `src/main.rs`.
- **Side Effects / I/O**: Sends and reads messages in dialogue with `@BotFather` over MTProto.

### `src/ui/diff_view.rs` (Role: ui, Lines: 90)
- **Responsibility**: Rich terminal visualization of calculated diff actions using colored symbols.
- **Imports**: `crate::domain::diff::{BotDiff, DiffAction}`, `colored::*`
- **Public Functions & Signatures**:
  ```rust
  pub fn render_diff(diff: &BotDiff);
  ```
- **Consumers**: `src/controller.rs`.
- **Side Effects / I/O**: Standard output (`println!`).

### `src/ui/table.rs` (Role: ui, Lines: 69)
- **Responsibility**: Renders formatted ASCII dashboard table of all configured bots in fleet.
- **Imports**: `crate::domain::spec::BotSpec`, `crate::infra::http::types::LiveBotState`, `colored::*`
- **Types & Enums**:
  ```rust
  pub struct BotSummaryItem<'a> { pub id: &'a str, pub spec: &'a BotSpec, pub live: Option<&'a LiveBotState> }
  ```
- **Public Functions & Signatures**:
  ```rust
  pub fn render_fleet_table(bots: &[BotSummaryItem]);
  ```
- **Consumers**: `src/controller.rs`.
- **Side Effects / I/O**: Standard output (`println!`).

### `src/controller.rs` (Role: api/dispatcher, Lines: 210)
- **Responsibility**: High-level action coordinator executing reconciliation, diff rendering, and batch updates.
- **Imports**: `crate::domain::{diff::*, spec::*}, crate::infra::http::client::BotApiClient, crate::infra::mtproto::{botfather::BotFatherAutomation, client::MtprotoClient}, crate::ui::{diff_view, table::{render_fleet_table, BotSummaryItem}}`
- **Public Functions & Signatures**:
  ```rust
  pub async fn diff_bot(bot_id: &str, spec: &BotSpec) -> Result<BotDiff>;
  pub async fn apply_bot(bot_id: &str, spec: &BotSpec, diff: &BotDiff, mtproto: Option<&BotFatherAutomation<'_>>) -> Result<()>;
  pub async fn list_fleet(spec: &FleetSpec) -> Result<()>;
  ```
- **Consumers**: `src/main.rs`.
- **Side Effects / I/O**: Network calls via HTTP and MTProto, console outputs.

### `src/mutate.rs` (Role: domain/mutation, Lines: 299)
- **Responsibility**: Programmatic CLI mutations of bots.toml (bot metadata, command menus, and live host bot auto-import).
- **Imports**: `crate::cli::*, crate::domain::spec::*, crate::infra::config::*, crate::infra::http::client::BotApiClient`
- **Public Functions & Signatures**:
  ```rust
  pub fn handle_bot_set(args: &BotSetArgs) -> Result<()>;
  pub fn handle_bot_rm(args: &BotRmArgs) -> Result<()>;
  pub fn handle_cmd_add(args: &CmdAddArgs) -> Result<()>;
  pub fn handle_cmd_rm(args: &CmdRmArgs) -> Result<()>;
  pub fn handle_cmd_list(args: &CmdListArgs) -> Result<()>;
  pub async fn handle_import(args: ImportArgs) -> Result<()>;
  ```
- **Consumers**: `src/main.rs`.
- **Side Effects / I/O**: Reads/writes `bots.toml`, reads host configs (`~/.config/ryoiki`, `~/.config/tayori`), queries Bot API.

### `src/main.rs` (Role: entrypoint, Lines: 298)
- **Responsibility**: Binary entrypoint parsing CLI commands, resolving configs, prompting confirmations, and delegating execution.
- **Imports**: `anyhow::{Context, Result}, clap::Parser, colored::*, dialoguer::Confirm, kuroko::{cli::*, controller::*, domain::spec::FleetSpec, infra::config::*, infra::mtproto::client::MtprotoClient, infra::mtproto::botfather::BotFatherAutomation}`
- **Public Functions & Signatures**:
  ```rust
  #[tokio::main]
  async fn main() -> Result<()>;
  ```
- **Consumers**: CLI binary runner.
- **Side Effects / I/O**: Reads CLI arguments, loads config files, terminal I/O, invokes controllers.

## 4. Execution Lifecycle Trace
1. **Startup**: `main()` parses CLI options via `Cli::parse()`.
2. **Subcommand Dispatch**:
   - `init`: Emits well-commented `bots.toml` starter configuration template.
   - `login`: Connects MTProto client, challenges user for phone/code/password, persists session to SQLite.
   - `new`: Authenticates MTProto, conducts `/newbot` dialogue with `@BotFather`, extracts API token, and optionally appends to `bots.toml`.
   - `list`: Iterates all bots defined in `bots.toml`, calls `getMe` concurrently, prints consolidated ASCII status table.
   - `diff`: Loads `bots.toml`, resolves tokens from env/plain, fetches live state via `fetch_live_state()`, computes `BotDiff`, renders colorized diff.
   - `apply`: Computes diffs; prompts user for confirmation unless `--yes` is passed. Applies Bot API updates (`setMyCommands`, `setMyDescription`, etc.) and invokes MTProto `@BotFather` automation for avatars, covers, and group/privacy settings.
3. **Exit**: Flushes output and terminates with exit code 0 or typed `anyhow` error.

## 5. Verification Commands
```bash
# Build
cargo build --release

# Test Suite
cargo test --all-targets

# Lint & Format
cargo clippy --all-targets -- -D warnings
cargo fmt --check

# Smoke Test
cargo run -- --help
cargo run -- init --output test_fleet.toml
```

## 6. Recent Iteration Changes
- **2026-10-02**: Initial architecture & full implementation of Kuroko:
  - Added `Cargo.toml` with strict linting rules and locked dependencies.
  - Implemented domain spec parser and validator (`src/domain/spec.rs`).
  - Implemented state reconciler and diff generator (`src/domain/diff.rs`).
  - Implemented Telegram HTTP Bot API client (`src/infra/http/`).
  - Implemented Grammers MTProto client and `@BotFather` automation (`src/infra/mtproto/`).
  - Implemented terminal dashboard table and colorized diff view (`src/ui/`).
  - Implemented subcommands (`diff`, `apply`, `list`, `login`, `new`, `init`) with modular controller (`src/controller.rs`, `src/main.rs`).
  - Tested test suite (8 passed, 0 warnings).
