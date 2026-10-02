use super::spec::{BotSpec, CommandSpec};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RemoteBotState {
    pub username: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub short_description: Option<String>,
    pub commands: Vec<CommandSpec>,
    pub has_avatar: bool,
    pub has_cover: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffAction {
    SetName(String),
    SetDescription(String),
    SetShortDescription(String),
    SetCommands(Vec<CommandSpec>),
    UploadAvatar(String),
    UploadCover(String),
    SetPrivacyMode(bool),
    SetCanJoinGroups(bool),
    SetInlineMode(bool),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeType {
    Add,
    Modify,
    Delete,
    Unchanged,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldDiff {
    pub field: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub change_type: ChangeType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandDiff {
    pub command: String,
    pub old_description: Option<String>,
    pub new_description: Option<String>,
    pub change_type: ChangeType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BotDiff {
    pub bot_id: String,
    pub username: Option<String>,
    pub fields: Vec<FieldDiff>,
    pub commands: Vec<CommandDiff>,
    pub actions: Vec<DiffAction>,
}

impl BotDiff {
    pub fn is_in_sync(&self) -> bool {
        self.actions.is_empty()
    }

    pub fn compute(bot_id: &str, desired: &BotSpec, remote: &RemoteBotState) -> Self {
        let mut fields = Vec::new();
        let mut actions = Vec::new();

        diff_name(desired, remote, &mut fields, &mut actions);
        diff_descriptions(desired, remote, &mut fields, &mut actions);
        diff_media(desired, remote, &mut fields, &mut actions);
        diff_settings(desired, &mut actions);

        let command_diffs = diff_commands(&desired.commands, &remote.commands, &mut actions);

        Self {
            bot_id: bot_id.to_string(),
            username: remote.username.clone(),
            fields,
            commands: command_diffs,
            actions,
        }
    }
}

fn diff_name(
    desired: &BotSpec,
    remote: &RemoteBotState,
    fields: &mut Vec<FieldDiff>,
    actions: &mut Vec<DiffAction>,
) {
    if let Some(desired_name) = &desired.name {
        let current = remote.name.as_deref().unwrap_or("");
        if desired_name != current {
            fields.push(FieldDiff {
                field: "name".to_string(),
                old_value: remote.name.clone(),
                new_value: Some(desired_name.clone()),
                change_type: if remote.name.is_none() {
                    ChangeType::Add
                } else {
                    ChangeType::Modify
                },
            });
            actions.push(DiffAction::SetName(desired_name.clone()));
        }
    }
}

fn diff_descriptions(
    desired: &BotSpec,
    remote: &RemoteBotState,
    fields: &mut Vec<FieldDiff>,
    actions: &mut Vec<DiffAction>,
) {
    if let Some(desc) = &desired.description {
        let current = remote.description.as_deref().unwrap_or("");
        if desc.text.trim() != current.trim() {
            fields.push(FieldDiff {
                field: "description".to_string(),
                old_value: remote.description.clone(),
                new_value: Some(desc.text.clone()),
                change_type: if remote.description.is_none() {
                    ChangeType::Add
                } else {
                    ChangeType::Modify
                },
            });
            actions.push(DiffAction::SetDescription(desc.text.clone()));
        }
    }

    if let Some(short) = &desired.short_description {
        let current = remote.short_description.as_deref().unwrap_or("");
        if short.text.trim() != current.trim() {
            fields.push(FieldDiff {
                field: "short_description".to_string(),
                old_value: remote.short_description.clone(),
                new_value: Some(short.text.clone()),
                change_type: if remote.short_description.is_none() {
                    ChangeType::Add
                } else {
                    ChangeType::Modify
                },
            });
            actions.push(DiffAction::SetShortDescription(short.text.clone()));
        }
    }
}

fn diff_media(
    desired: &BotSpec,
    remote: &RemoteBotState,
    fields: &mut Vec<FieldDiff>,
    actions: &mut Vec<DiffAction>,
) {
    if let Some(avatar_path) = &desired.avatar {
        if !avatar_path.is_empty() {
            fields.push(FieldDiff {
                field: "avatar".to_string(),
                old_value: if remote.has_avatar {
                    Some("[Present]".to_string())
                } else {
                    None
                },
                new_value: Some(avatar_path.clone()),
                change_type: ChangeType::Modify,
            });
            actions.push(DiffAction::UploadAvatar(avatar_path.clone()));
        }
    }

    if let Some(cover_path) = &desired.cover {
        if !cover_path.is_empty() {
            fields.push(FieldDiff {
                field: "cover".to_string(),
                old_value: if remote.has_cover {
                    Some("[Present]".to_string())
                } else {
                    None
                },
                new_value: Some(cover_path.clone()),
                change_type: ChangeType::Modify,
            });
            actions.push(DiffAction::UploadCover(cover_path.clone()));
        }
    }
}

fn diff_settings(desired: &BotSpec, actions: &mut Vec<DiffAction>) {
    if let Some(settings) = &desired.settings {
        if let Some(priv_mode) = settings.privacy_mode {
            actions.push(DiffAction::SetPrivacyMode(priv_mode));
        }
        if let Some(join) = settings.can_join_groups {
            actions.push(DiffAction::SetCanJoinGroups(join));
        }
        if let Some(inline) = settings.inline_mode {
            actions.push(DiffAction::SetInlineMode(inline));
        }
    }
}

fn diff_commands(
    desired_cmds: &[CommandSpec],
    remote_cmds: &[CommandSpec],
    actions: &mut Vec<DiffAction>,
) -> Vec<CommandDiff> {
    let mut diffs = Vec::new();
    let mut has_changes = false;

    for cmd in desired_cmds {
        let clean = cmd.command.strip_prefix('/').unwrap_or(&cmd.command);
        let remote_match = remote_cmds
            .iter()
            .find(|rc| rc.command.strip_prefix('/').unwrap_or(&rc.command) == clean);

        match remote_match {
            None => {
                has_changes = true;
                diffs.push(CommandDiff {
                    command: clean.to_string(),
                    old_description: None,
                    new_description: Some(cmd.description.clone()),
                    change_type: ChangeType::Add,
                });
            }
            Some(rc) if rc.description != cmd.description => {
                has_changes = true;
                diffs.push(CommandDiff {
                    command: clean.to_string(),
                    old_description: Some(rc.description.clone()),
                    new_description: Some(cmd.description.clone()),
                    change_type: ChangeType::Modify,
                });
            }
            Some(_) => {
                diffs.push(CommandDiff {
                    command: clean.to_string(),
                    old_description: Some(cmd.description.clone()),
                    new_description: Some(cmd.description.clone()),
                    change_type: ChangeType::Unchanged,
                });
            }
        }
    }

    for rc in remote_cmds {
        let clean = rc.command.strip_prefix('/').unwrap_or(&rc.command);
        let in_desired = desired_cmds
            .iter()
            .any(|c| c.command.strip_prefix('/').unwrap_or(&c.command) == clean);
        if !in_desired {
            has_changes = true;
            diffs.push(CommandDiff {
                command: clean.to_string(),
                old_description: Some(rc.description.clone()),
                new_description: None,
                change_type: ChangeType::Delete,
            });
        }
    }

    if has_changes {
        actions.push(DiffAction::SetCommands(desired_cmds.to_vec()));
    }

    diffs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_detects_name_and_command_changes() {
        let desired = BotSpec {
            name: Some("New Name".to_string()),
            commands: vec![CommandSpec {
                command: "start".to_string(),
                description: "Start bot".to_string(),
            }],
            ..Default::default()
        };

        let remote = RemoteBotState {
            name: Some("Old Name".to_string()),
            commands: vec![],
            ..Default::default()
        };

        let diff = BotDiff::compute("test_bot", &desired, &remote);
        assert!(!diff.is_in_sync());
        assert_eq!(diff.fields.len(), 1);
        assert_eq!(diff.fields[0].field, "name");
        assert_eq!(diff.fields[0].change_type, ChangeType::Modify);
        assert_eq!(diff.commands.len(), 1);
        assert_eq!(diff.commands[0].change_type, ChangeType::Add);
    }

    #[test]
    fn test_diff_in_sync_when_identical() {
        let desired = BotSpec {
            name: Some("Sync Bot".to_string()),
            commands: vec![CommandSpec {
                command: "help".to_string(),
                description: "Help command".to_string(),
            }],
            ..Default::default()
        };

        let remote = RemoteBotState {
            name: Some("Sync Bot".to_string()),
            commands: vec![CommandSpec {
                command: "help".to_string(),
                description: "Help command".to_string(),
            }],
            ..Default::default()
        };

        let diff = BotDiff::compute("sync_bot", &desired, &remote);
        assert!(diff.is_in_sync());
    }
}
