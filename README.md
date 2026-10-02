# 🌸 Kuroko (黒子) — Declarative Telegram Bot Fleet Orchestrator

> **"Terraform for Telegram Bots." Declarative Bot-as-Code management for commands, descriptions, avatars, covers, and privacy settings with hybrid HTTP Bot API & MTProto `@BotFather` automation.**

<br>

<div align="center">

[![Rust Edition](https://img.shields.io/badge/Rust-2024%20Edition-DEA584?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/Platform-Linux-FCC624?style=for-the-badge&logo=linux&logoColor=black)](https://github.com/Praveensenpai/kuroko)
[![Telegram](https://img.shields.io/badge/Telegram-MTProto%20%2B%20Bot%20API-24A1DE?style=for-the-badge&logo=telegram)](https://core.telegram.org/)
[![License](https://img.shields.io/badge/License-MIT-green?style=for-the-badge)](LICENSE)

<br>

[⚡ Quick Start](#-quick-start) • [✨ Key Features](#-key-features) • [🔄 Architecture](#-architecture--workflow) • [⚡ CLI Workflows](#-zero-manual-cli-workflows) • [💻 CLI Reference](#-cli-commands) • [📜 License](#-license)

</div>

<br>

> [!TIP]
> **Zero Manual BotFather Clicking · 100% Declarative & Idempotent**  
> In traditional Japanese Kabuki theatre, the **Kuroko (黒子)** is the invisible stagehand who maneuvers props and scenery in the background. Kuroko automates the tedious backstage orchestration of Telegram bots—reconciling your desired state against Telegram servers automatically.

---

## 🔄 Architecture & Workflow

Kuroko operates a **hybrid dual-engine architecture**:
1. **HTTP Bot API Engine**: Fast, concurrent, and non-privileged sync for bot names, short descriptions, long descriptions, and command menus.
2. **MTProto Userbot Engine**: Headless Telegram MTProto automation directly controlling `@BotFather` for operations inaccessible via the Bot API (profile pictures, intro video covers, group join permissions, inline mode, and bot creation).

```text
               ┌───────────────────────────────┐
               │    Local Fleet Spec           │
               │    `bots.toml`                │
               └───────────────┬───────────────┘
                               │
                               ▼
               ┌───────────────────────────────┐
               │    🌸 Kuroko Core Engine      │
               │    · Diff Calculator          │
               │    · State Reconciler         │
               └───────┬───────────────┬───────┘
                       │               │
      [1. Bot API Sync]│               │[2. MTProto Automation]
                       ▼               ▼
      ┌─────────────────────────┐     ┌─────────────────────────┐
      │   Telegram Bot API      │     │   @BotFather via MTProto│
      │   · Names & Bios        │     │   · Avatars & Video Pics│
      │   · Commands & Menus    │     │   · Privacy / Join Flags│
      │   · Multilingual Specs  │     │   · Headless /newbot    │
      └─────────────────────────┘     └─────────────────────────┘
```

---

## ✨ Key Features

| Feature | Description |
|---|---|
| **📄 Declarative Bot-as-Code** | Maintain your entire fleet of Telegram bots in a clean, version-controlled `bots.toml` file. |
| **🔍 Colorized `diff` Previews** | Inspect exact additions, removals, and modifications before touching production bots. |
| **🖼️ Headless Avatars & Covers** | Automates uploading profile avatars (`/setuserpic`) and video covers (`/setintro`) through `@BotFather`. |
| **🔒 Safe Secret Resolution** | Store bot tokens as environment variables (`env:MY_BOT_TOKEN`) to prevent leaking keys in Git. |
| **⚡ Multi-Command Management** | Sync complex command lists with custom descriptions and multi-language scopes. |
| **📊 Fleet Dashboard** | View live status, bot usernames, privacy modes, and group join states with `kuroko list`. |
| **🤖 Zero-Friction Bot Creation** | Provision new bots from the command line with `kuroko new` and auto-append credentials to `bots.toml`. |

---

## 🚀 Quick Start

### 🪄 One-Liner Magic (Recommended)

Install `kuroko` in seconds with automatic architecture detection and shell `$PATH` setup:

```bash
curl -fsSL https://raw.githubusercontent.com/Praveensenpai/kuroko/main/install.sh | bash
```

<br>

### 🛠️ Building From Source

Prerequisites: A working Rust toolchain (`cargo` 1.85+ recommended).

```bash
# Clone the repository
git clone https://github.com/Praveensenpai/kuroko.git
cd kuroko

# Build optimized release binary
cargo build --release

# Install to user binary path
install -Dm 755 target/release/kuroko ~/.local/bin/kuroko
```

### 1. Initialize Your Bot Fleet Spec

Generate a starter `bots.toml` template:

```bash
kuroko init --output bots.toml
```

### 2. (Optional) Connect MTProto for Backstage Automation

For avatars, video covers, and privacy settings, authenticate an MTProto user session:

```bash
# Set your Telegram App credentials (from https://my.telegram.org)
export TELEGRAM_API_ID="123456"
export TELEGRAM_API_HASH="0123456789abcdef0123456789abcdef"

# Authenticate session (prompts for phone, code, and 2FA password)
kuroko login
```

### 3. Review Diffs & Apply Changes

```bash
# Preview changes between bots.toml and live Telegram state
kuroko diff

# Apply changes with interactive confirmation
kuroko apply

# Or apply non-interactively in CI/CD pipelines
kuroko apply --yes
```

---

## ⚡ Zero-Manual CLI Workflows

Never write or edit raw TOML files manually. Kuroko provides a complete suite of commands to configure and inspect your bot fleet directly from your terminal:

### 1. Auto-Import Existing Bots
```bash
# Automatically detects local bots (~/.config/ryoiki, ~/.config/tayori) and queries Telegram API
kuroko import --auto

# Or import an individual bot by its token
kuroko import --token "123456:ABC-DEF..." --id my_bot
```

### 2. Configure Bot Metadata
```bash
# Add or update a bot's name, token, and descriptions
kuroko bot set ryoiki \
  --name "領域 Ryoiki Bot" \
  --token "env:RYOIKI_BOT_TOKEN" \
  --desc "🌊 領域 (Ryoiki) — Server Control & Download Manager" \
  --short-desc "Server management & torrent orchestrator"

# Remove a bot from the fleet
kuroko bot rm old_bot
```

### 3. Manage Command Menus
```bash
# Add commands with one clean command
kuroko cmd add ryoiki status "Show system health, CPU, RAM, & active downloads"
kuroko cmd add ryoiki torrents "List all active, completed, or downloading torrents"
kuroko cmd add ryoiki storage "Inspect disk partition usage and drive health"

# List configured commands for any bot
kuroko cmd list ryoiki

# Remove a command
kuroko cmd rm ryoiki old_cmd
```

### 4. Review & Deploy
```bash
# Inspect changes before touching production
kuroko diff

# Apply changes to Telegram immediately
kuroko apply --http-only
```

---

## 💻 CLI Commands

```text
🌸 Kuroko (黒子) — Declarative Telegram Bot Fleet Orchestrator

Usage: kuroko <COMMAND>

Commands:
  diff    Show differences between local bots.toml and live Telegram state
  apply   Reconcile and apply declarative configuration to Telegram
  list    Display dashboard of all managed bots and live status
  bot     Manage bot fleet entries in configuration (set / rm)
  cmd     Manage bot command menus in configuration (add / rm / list)
  import  Import existing bot specifications from live Telegram API or host configs
  login   Authenticate MTProto user session for BotFather automation
  new     Create a new bot via @BotFather and save credentials
  init    Generate a starter bots.toml template
  help    Print this message or the help of the given subcommand(s)
```

### CLI Command Highlights:

- **Configure Bots Programmatically (No Manual TOML Editing)**:
  ```bash
  # Add or update a bot in bots.toml
  kuroko bot set ryoiki \
    --name "領域 Ryoiki Bot" \
    --token "env:RYOIKI_BOT_TOKEN" \
    --desc "🌊 領域 (Ryoiki) — Server Control & Download Manager" \
    --short-desc "Server management & torrent orchestrator"

  # Remove a bot from the fleet
  kuroko bot rm old_bot
  ```

- **Manage Bot Command Menus**:
  ```bash
  # Add or update commands
  kuroko cmd add ryoiki status "Show system health, CPU, RAM, & active downloads"
  kuroko cmd add ryoiki torrents "List all active, completed, or downloading torrents"
  kuroko cmd add ryoiki storage "Inspect disk partition usage and drive health"

  # Remove a command
  kuroko cmd rm ryoiki old_command

  # List configured commands for a bot
  kuroko cmd list ryoiki
  ```

- **Auto-Import Existing Bots**:
  ```bash
  # Automatically discover host configs (~/.config/ryoiki, ~/.config/tayori) and fetch live state
  kuroko import --auto

  # Or import an individual bot by token
  kuroko import --token "123456:ABC-DEF..." --id my_bot
  ```

- **Preview & Reconcile**:
  ```bash
  # Show colored diff
  kuroko diff

  # Apply changes (HTTP Bot API only)
  kuroko apply --http-only

  # Full apply (HTTP + MTProto BotFather)
  kuroko apply
  ```

---

## 📜 License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.  
© Praveen Senpai ([@Praveensenpai](https://github.com/Praveensenpai))
