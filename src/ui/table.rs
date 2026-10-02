use colored::Colorize;

#[derive(Debug, Clone)]
pub struct FleetBotRow {
    pub bot_id: String,
    pub username: String,
    pub name: String,
    pub commands_count: usize,
    pub avatar_status: String,
    pub sync_status: String,
}

pub fn render_fleet_table(rows: &[FleetBotRow]) {
    if rows.is_empty() {
        println!(
            "{}",
            "No bots configured. Run `kuroko new` or create `bots.toml`.".yellow()
        );
        return;
    }

    println!(
        "\n{}",
        "🌸 Kuroko (黒子) — Telegram Bot Fleet Status".bold().cyan()
    );
    println!("{}", "━".repeat(84).bright_black());
    println!(
        "{:<16} {:<18} {:<24} {:<10} {:<12}",
        "BOT ID".bold(),
        "USERNAME".bold(),
        "NAME".bold(),
        "COMMANDS".bold(),
        "STATUS".bold()
    );
    println!("{}", "─".repeat(84).bright_black());

    for r in rows {
        let status_colored = if r.sync_status == "Synced" {
            r.sync_status.green().bold()
        } else if r.sync_status == "Diverged" {
            r.sync_status.yellow().bold()
        } else {
            r.sync_status.red().bold()
        };

        println!(
            "{:<16} {:<18} {:<24} {:<10} {:<12}",
            r.bot_id.white(),
            format!("@{}", r.username).cyan(),
            truncate(&r.name, 22),
            format!("{} cmds", r.commands_count).blue(),
            status_colored
        );
    }
    println!("{}", "━".repeat(84).bright_black());
    println!(
        "Total managed bots: {}\n",
        rows.len().to_string().green().bold()
    );
}

fn truncate(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_chars.saturating_sub(3)).collect();
        format!("{truncated}...")
    }
}
