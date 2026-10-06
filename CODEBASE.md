# CODEBASE.md: kuroko Semantic Digest

> **Notice**: AI-optimized semantic index. Do not write narrative prose. Keep token density high.

## 1. System Topology & Data Flow
```text
Entrypoint ──> CLI/Parser ──> Domain Logic ──> Infra/IO
```

## 2. Global Constraints & Architecture Patterns
- **Primary Language**: Rust 2021 edition
- **Architectural Paradigm**: Role-based (domain/, infra/, api/cli/, tui/)
- **Hard Constraints**: <400 lines/file, <60 lines/fn, zero production unwrap(), 0 warnings.
- **Target Distribution**: Linux x86_64 standalone binary

## 3. Module & Interface Skeleton

### `src/cli.rs` (Role: cli, Lines: 271)
- **Responsibility**: Core cli logic in src/cli.rs
- **Imports**: use clap :: { Args , Parser , Subcommand } , use std :: path :: PathBuf 
- **Types & Enums**:
  ```rust
  pub struct Cli
  pub enum Commands
  pub enum BotSubcommands
  pub struct BotSetArgs
  pub struct BotRmArgs
  pub enum CmdSubcommands
  pub struct CmdAddArgs
  pub struct CmdRmArgs
  pub struct CmdListArgs
  pub struct ImportArgs
  pub struct DiffArgs
  pub struct ApplyArgs
  pub struct ListArgs
  pub struct LoginArgs
  pub struct NewArgs
  pub struct SendArgs
  pub struct ReadArgs
  pub struct InitArgs
  ```

### `src/controller.rs` (Role: general, Lines: 200)
- **Responsibility**: Core general logic in src/controller.rs
- **Imports**: use crate :: domain :: diff :: { BotDiff , DiffAction } , use crate :: domain :: spec :: { BotSpec , BotsConfig } , use crate :: infra :: http :: BotApiClient , use crate :: infra :: mtproto :: { BotFatherClient , MtprotoEngine } , use anyhow :: { bail , Context , Result } , use colored :: Colorize , use std :: path :: Path 
- **Public Functions & Signatures**:
  ```rust
  async fn compute_single_diff (bot_id : & str , bot_spec : & BotSpec) -> Result < BotDiff >
  fn select_target_bots < 'a > (config : & 'a BotsConfig , filter : Option < & str > ,) -> Result < Vec < (String , & 'a BotSpec) > >
  async fn apply_action (bot_id : & str , bot_spec : & BotSpec , action : & DiffAction , http : & BotApiClient , http_only : bool , mtproto : & mut Option < MtprotoEngine > ,) -> Result < () >
  ```

### `src/domain/diff.rs` (Role: domain, Lines: 329)
- **Responsibility**: Core domain logic in src/domain/diff.rs
- **Imports**: use super :: spec :: { BotSpec , CommandSpec } , use serde :: { Deserialize , Serialize } 
- **Types & Enums**:
  ```rust
  pub struct RemoteBotState
  pub enum DiffAction
  pub enum ChangeType
  pub struct FieldDiff
  pub struct CommandDiff
  pub struct BotDiff
  ```
- **Public Functions & Signatures**:
  ```rust
  fn is_in_sync (& self) -> bool
  fn compute (bot_id : & str , desired : & BotSpec , remote : & RemoteBotState) -> Self
  ```

### `src/domain/spec.rs` (Role: domain, Lines: 401)
- **Responsibility**: Core domain logic in src/domain/spec.rs
- **Imports**: use anyhow :: { bail , Context , Result } , use serde :: { Deserialize , Serialize } , use std :: collections :: BTreeMap , use std :: fs , use std :: path :: { Path , PathBuf } 
- **Types & Enums**:
  ```rust
  pub struct BotsConfig
  pub struct GlobalSettings
  pub struct TextSpec
  pub struct BotSettingsSpec
  pub struct CommandSpec
  pub struct BotSpec
  ```
- **Public Functions & Signatures**:
  ```rust
  fn from_file < P : AsRef < Path > > (path : P) -> Result < Self >
  fn parse_str (raw : & str) -> Result < Self >
  fn validate (& self) -> Result < () >
  fn save_file < P : AsRef < Path > > (& self , path : P) -> Result < () >
  fn get_or_create_bot (& mut self , id : & str) -> & mut BotSpec
  fn remove_bot (& mut self , id : & str) -> bool
  fn add_or_update_command (& mut self , bot_id : & str , command : & str , description : String ,) -> Result < () >
  fn remove_command (& mut self , bot_id : & str , command : & str) -> bool
  fn resolve_token (& self) -> Option < String >
  fn validate (& self , bot_id : & str) -> Result < () >
  ```

### `src/domain.rs` (Role: domain, Lines: 5)
- **Responsibility**: Core domain logic in src/domain.rs
- **Imports**: pub use diff :: { BotDiff , DiffAction , FieldDiff } , pub use spec :: { BotSpec , BotsConfig , CommandSpec } 

### `src/infra/config.rs` (Role: infra, Lines: 117)
- **Responsibility**: Core infra logic in src/infra/config.rs
- **Imports**: use anyhow :: { Context , Result } , use serde :: { Deserialize , Serialize } , use std :: fs , use std :: path :: { Path , PathBuf } 
- **Types & Enums**:
  ```rust
  pub struct KurokoConfig
  ```
- **Public Functions & Signatures**:
  ```rust
  fn config_dir () -> PathBuf
  fn config_file () -> PathBuf
  fn session_file () -> PathBuf
  fn ensure_dir () -> Result < () >
  fn load () -> Self
  fn save (& self) -> Result < () >
  fn get_api_credentials (& self) -> Option < (i32 , String) >
  fn resolve_path (p : & str) -> PathBuf
  fn resolve_bots_config_path (arg_path : & Path) -> PathBuf
  ```

### `src/infra/http/client.rs` (Role: infra, Lines: 174)
- **Responsibility**: Core infra logic in src/infra/http/client.rs
- **Imports**: use super :: types :: { ApiResponse , SetCommandsPayload , SetDescriptionPayload , SetNamePayload , SetShortDescriptionPayload , TelegramBotCommand , TelegramBotDescription , TelegramBotName , TelegramBotShortDescription , TelegramUser , } , use crate :: domain :: diff :: RemoteBotState , use crate :: domain :: spec :: CommandSpec , use anyhow :: { bail , Context , Result } , use reqwest :: Client , use std :: time :: Duration 
- **Types & Enums**:
  ```rust
  pub struct BotApiClient
  ```
- **Public Functions & Signatures**:
  ```rust
  fn new (token : & str) -> Self
  fn token (& self) -> & str
  async fn get_me (& self) -> Result < TelegramUser >
  async fn get_my_name (& self) -> Result < String >
  async fn set_my_name (& self , name : & str) -> Result < () >
  async fn get_my_description (& self) -> Result < String >
  async fn set_my_description (& self , description : & str) -> Result < () >
  async fn get_my_short_description (& self) -> Result < String >
  async fn set_my_short_description (& self , short_description : & str) -> Result < () >
  async fn get_my_commands (& self) -> Result < Vec < CommandSpec > >
  async fn set_my_commands (& self , commands : & [CommandSpec]) -> Result < () >
  async fn fetch_remote_state (& self) -> Result < RemoteBotState >
  ```

### `src/infra/http/types.rs` (Role: infra, Lines: 57)
- **Responsibility**: Core infra logic in src/infra/http/types.rs
- **Imports**: use serde :: { Deserialize , Serialize } 
- **Types & Enums**:
  ```rust
  pub struct ApiResponse
  pub struct TelegramUser
  pub struct TelegramBotCommand
  pub struct TelegramBotName
  pub struct TelegramBotDescription
  pub struct TelegramBotShortDescription
  pub struct SetNamePayload
  pub struct SetDescriptionPayload
  pub struct SetShortDescriptionPayload
  pub struct SetCommandsPayload
  ```

### `src/infra/http.rs` (Role: infra, Lines: 4)
- **Responsibility**: Core infra logic in src/infra/http.rs
- **Imports**: pub use client :: BotApiClient 

### `src/infra/mtproto/botfather.rs` (Role: infra, Lines: 206)
- **Responsibility**: Core infra logic in src/infra/mtproto/botfather.rs
- **Imports**: use anyhow :: { bail , Context , Result } , use grammers_client :: message :: InputMessage , use grammers_client :: Client , use grammers_session :: types :: PeerRef , use std :: path :: Path , use std :: time :: Duration , use tokio :: time :: sleep 
- **Types & Enums**:
  ```rust
  pub struct BotFatherClient
  ```
- **Public Functions & Signatures**:
  ```rust
  fn new (client : & 'a Client) -> Self
  async fn get_botfather_peer (& self) -> Result < PeerRef >
  async fn upload_bot_avatar < P : AsRef < Path > > (& self , bot_username : & str , image_path : P ,) -> Result < () >
  async fn upload_bot_cover < P : AsRef < Path > > (& self , bot_username : & str , media_path : P ,) -> Result < () >
  async fn toggle_privacy_mode (& self , bot_username : & str , enabled : bool) -> Result < () >
  async fn toggle_group_joining (& self , bot_username : & str , enabled : bool) -> Result < () >
  async fn create_new_bot (& self , display_name : & str , username : & str) -> Result < String >
  fn extract_token_from_botfather_response (text : & str) -> Option < String >
  ```

### `src/infra/mtproto/chat.rs` (Role: infra, Lines: 145)
- **Responsibility**: Core infra logic in src/infra/mtproto/chat.rs
- **Imports**: use crate :: infra :: config :: KurokoConfig , use crate :: infra :: mtproto :: MtprotoEngine , use anyhow :: { bail , Context , Result } , use grammers_client :: message :: Message , use grammers_client :: tl , use grammers_client :: Client , use grammers_session :: types :: PeerRef , use std :: time :: { Duration , Instant } , use tokio :: time :: sleep 
- **Public Functions & Signatures**:
  ```rust
  async fn connect_authorized () -> Result < MtprotoEngine >
  async fn resolve_peer (client : & Client , username : & str) -> Result < PeerRef >
  fn render_message (msg : & Message) -> String
  async fn send_and_wait (client : & Client , peer : PeerRef , text : & str , timeout : Duration ,) -> Result < () >
  async fn read_latest (client : & Client , peer : PeerRef , count : usize) -> Result < () >
  ```

### `src/infra/mtproto/client.rs` (Role: infra, Lines: 89)
- **Responsibility**: Core infra logic in src/infra/mtproto/client.rs
- **Imports**: use super :: session :: load_or_create_session , use crate :: infra :: config :: KurokoConfig , use anyhow :: { bail , Context , Result } , use dialoguer :: Password , use grammers_client :: { Client , SenderPool } 
- **Types & Enums**:
  ```rust
  pub struct MtprotoEngine
  ```
- **Public Functions & Signatures**:
  ```rust
  async fn connect (api_id : i32 , api_hash : & str) -> Result < Self >
  fn client (& self) -> & Client
  async fn is_authorized (& self) -> Result < bool >
  async fn login_interactive (& self) -> Result < () >
  ```

### `src/infra/mtproto/session.rs` (Role: infra, Lines: 14)
- **Responsibility**: Core infra logic in src/infra/mtproto/session.rs
- **Imports**: use anyhow :: { Context , Result } , use grammers_session :: storages :: SqliteSession , use std :: path :: Path , use std :: sync :: Arc 
- **Public Functions & Signatures**:
  ```rust
  async fn load_or_create_session < P : AsRef < Path > > (path : P) -> Result < Arc < SqliteSession > >
  ```

### `src/infra/mtproto.rs` (Role: infra, Lines: 8)
- **Responsibility**: Core infra logic in src/infra/mtproto.rs
- **Imports**: pub use botfather :: BotFatherClient , pub use chat :: { connect_authorized , read_latest , resolve_peer , send_and_wait } , pub use client :: MtprotoEngine 

### `src/infra.rs` (Role: infra, Lines: 7)
- **Responsibility**: Core infra logic in src/infra.rs
- **Imports**: pub use config :: KurokoConfig , pub use http :: BotApiClient , pub use mtproto :: MtprotoEngine 

### `src/lib.rs` (Role: general, Lines: 6)
- **Responsibility**: Core general logic in src/lib.rs

### `src/main.rs` (Role: general, Lines: 334)
- **Responsibility**: Core general logic in src/main.rs
- **Imports**: use anyhow :: { Context , Result } , use clap :: Parser , use colored :: Colorize , use kuroko :: cli :: { ApplyArgs , Cli , Commands , DiffArgs , InitArgs , ListArgs , LoginArgs , NewArgs , ReadArgs , SendArgs , } , use kuroko :: controller :: { apply_action , compute_single_diff , select_target_bots } , use kuroko :: domain :: diff :: BotDiff , use kuroko :: domain :: spec :: BotsConfig , use kuroko :: infra :: config :: KurokoConfig , use kuroko :: infra :: http :: BotApiClient , use kuroko :: infra :: mtproto :: { BotFatherClient , MtprotoEngine } , use kuroko :: ui :: table :: FleetBotRow , use kuroko :: ui :: { render_diff , render_fleet_table } , use std :: fs , use std :: time :: Duration 

### `src/mutate.rs` (Role: general, Lines: 299)
- **Responsibility**: Core general logic in src/mutate.rs
- **Imports**: use anyhow :: { bail , Context , Result } , use colored :: Colorize , use std :: fs , use std :: path :: Path , use crate :: cli :: { BotRmArgs , BotSetArgs , CmdAddArgs , CmdListArgs , CmdRmArgs , ImportArgs } , use crate :: domain :: spec :: { BotsConfig , TextSpec } , use crate :: infra :: config :: resolve_bots_config_path , use crate :: infra :: http :: client :: BotApiClient 
- **Public Functions & Signatures**:
  ```rust
  fn handle_bot_set (args : & BotSetArgs) -> Result < () >
  fn handle_bot_rm (args : & BotRmArgs) -> Result < () >
  fn handle_cmd_add (args : & CmdAddArgs) -> Result < () >
  fn handle_cmd_rm (args : & CmdRmArgs) -> Result < () >
  fn handle_cmd_list (args : & CmdListArgs) -> Result < () >
  async fn handle_import (args : ImportArgs) -> Result < () >
  ```

### `src/ui/diff_view.rs` (Role: tui, Lines: 90)
- **Responsibility**: Core tui logic in src/ui/diff_view.rs
- **Imports**: use crate :: domain :: diff :: { BotDiff , ChangeType } , use colored :: Colorize 
- **Public Functions & Signatures**:
  ```rust
  fn render_diff (diff : & BotDiff)
  ```

### `src/ui/table.rs` (Role: tui, Lines: 69)
- **Responsibility**: Core tui logic in src/ui/table.rs
- **Imports**: use colored :: Colorize 
- **Types & Enums**:
  ```rust
  pub struct FleetBotRow
  ```
- **Public Functions & Signatures**:
  ```rust
  fn render_fleet_table (rows : & [FleetBotRow])
  ```

### `src/ui.rs` (Role: tui, Lines: 5)
- **Responsibility**: Core tui logic in src/ui.rs
- **Imports**: pub use diff_view :: render_diff , pub use table :: render_fleet_table 

## 4. Execution Lifecycle Trace
1. **Startup**: Entrypoint parses CLI flags & dispatches command.
2. **Execution**: Core domain logic processes inputs and evaluates rules.
3. **Persistence / I/O**: Domain logic calls infra for disk/terminal I/O.
4. **Exit**: Graceful termination with standard exit codes.

## 5. Verification Commands
```bash
cargo build --release --target x86_64-unknown-linux-gnu
cargo test --all-targets
cargo clippy --all-targets -- -D warnings && cargo fmt --check
```
