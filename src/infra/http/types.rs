use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub ok: bool,
    pub result: Option<T>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramUser {
    pub id: i64,
    pub is_bot: bool,
    pub first_name: String,
    pub username: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TelegramBotCommand {
    pub command: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TelegramBotName {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TelegramBotDescription {
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TelegramBotShortDescription {
    pub short_description: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SetNamePayload<'a> {
    pub name: &'a str,
}

#[derive(Debug, Clone, Serialize)]
pub struct SetDescriptionPayload<'a> {
    pub description: &'a str,
}

#[derive(Debug, Clone, Serialize)]
pub struct SetShortDescriptionPayload<'a> {
    pub short_description: &'a str,
}

#[derive(Debug, Clone, Serialize)]
pub struct SetCommandsPayload<'a> {
    pub commands: &'a [TelegramBotCommand],
}
