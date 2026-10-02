use crate::domain::diff::{BotDiff, ChangeType};
use colored::Colorize;

pub fn render_diff(diff: &BotDiff) {
    let title = diff
        .username
        .as_deref()
        .map_or_else(|| diff.bot_id.clone(), |u| format!("@{u}"));

    if diff.is_in_sync() {
        println!(
            "  {} Bot [{}] is up to date (0 changes)",
            "✔".green().bold(),
            title.cyan().bold()
        );
        return;
    }

    println!(
        "\n{} Changes for Bot [{}] ({} action(s) planned):",
        "⚙".yellow().bold(),
        title.cyan().bold(),
        diff.actions.len()
    );

    for f in &diff.fields {
        match f.change_type {
            ChangeType::Add => {
                let val = f.new_value.as_deref().unwrap_or("");
                println!(
                    "  {} {}: {}",
                    "+".green().bold(),
                    f.field.bold(),
                    val.green()
                );
            }
            ChangeType::Modify => {
                let old = f.old_value.as_deref().unwrap_or("[none]");
                let new = f.new_value.as_deref().unwrap_or("");
                println!(
                    "  {} {}: {} -> {}",
                    "~".yellow().bold(),
                    f.field.bold(),
                    old.red(),
                    new.green()
                );
            }
            ChangeType::Delete => {
                let old = f.old_value.as_deref().unwrap_or("");
                println!("  {} {}: {}", "-".red().bold(), f.field.bold(), old.red());
            }
            ChangeType::Unchanged => {}
        }
    }

    for c in &diff.commands {
        match c.change_type {
            ChangeType::Add => {
                let desc = c.new_description.as_deref().unwrap_or("");
                println!(
                    "  {} /{} - {}",
                    "+".green().bold(),
                    c.command.green().bold(),
                    desc.green()
                );
            }
            ChangeType::Modify => {
                let old = c.old_description.as_deref().unwrap_or("");
                let new = c.new_description.as_deref().unwrap_or("");
                println!(
                    "  {} /{} - {} -> {}",
                    "~".yellow().bold(),
                    c.command.yellow().bold(),
                    old.red(),
                    new.green()
                );
            }
            ChangeType::Delete => {
                let old = c.old_description.as_deref().unwrap_or("");
                println!(
                    "  {} /{} - {}",
                    "-".red().bold(),
                    c.command.red().bold(),
                    old.red()
                );
            }
            ChangeType::Unchanged => {}
        }
    }
}
