use super::types::{
    ApiResponse, SetCommandsPayload, SetDescriptionPayload, SetNamePayload,
    SetShortDescriptionPayload, TelegramBotCommand, TelegramBotDescription, TelegramBotName,
    TelegramBotShortDescription, TelegramUser,
};
use crate::domain::diff::RemoteBotState;
use crate::domain::spec::CommandSpec;
use anyhow::{bail, Context, Result};
use reqwest::Client;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct BotApiClient {
    token: String,
    base_url: String,
    client: Client,
}

impl BotApiClient {
    pub fn new(token: &str) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_default();

        Self {
            token: token.trim().to_string(),
            base_url: format!("https://api.telegram.org/bot{}", token.trim()),
            client,
        }
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    pub async fn get_me(&self) -> Result<TelegramUser> {
        let url = format!("{}/getMe", self.base_url);
        self.send_get(&url).await
    }

    pub async fn get_my_name(&self) -> Result<String> {
        let url = format!("{}/getMyName", self.base_url);
        let resp: TelegramBotName = self.send_get(&url).await?;
        Ok(resp.name)
    }

    pub async fn set_my_name(&self, name: &str) -> Result<()> {
        let url = format!("{}/setMyName", self.base_url);
        let payload = SetNamePayload { name };
        self.send_post_status(&url, &payload).await
    }

    pub async fn get_my_description(&self) -> Result<String> {
        let url = format!("{}/getMyDescription", self.base_url);
        let resp: TelegramBotDescription = self.send_get(&url).await?;
        Ok(resp.description)
    }

    pub async fn set_my_description(&self, description: &str) -> Result<()> {
        let url = format!("{}/setMyDescription", self.base_url);
        let payload = SetDescriptionPayload { description };
        self.send_post_status(&url, &payload).await
    }

    pub async fn get_my_short_description(&self) -> Result<String> {
        let url = format!("{}/getMyShortDescription", self.base_url);
        let resp: TelegramBotShortDescription = self.send_get(&url).await?;
        Ok(resp.short_description)
    }

    pub async fn set_my_short_description(&self, short_description: &str) -> Result<()> {
        let url = format!("{}/setMyShortDescription", self.base_url);
        let payload = SetShortDescriptionPayload { short_description };
        self.send_post_status(&url, &payload).await
    }

    pub async fn get_my_commands(&self) -> Result<Vec<CommandSpec>> {
        let url = format!("{}/getMyCommands", self.base_url);
        let cmds: Vec<TelegramBotCommand> = self.send_get(&url).await?;
        let mapped = cmds
            .into_iter()
            .map(|c| CommandSpec {
                command: c.command,
                description: c.description,
            })
            .collect();
        Ok(mapped)
    }

    pub async fn set_my_commands(&self, commands: &[CommandSpec]) -> Result<()> {
        let url = format!("{}/setMyCommands", self.base_url);
        let dtos: Vec<TelegramBotCommand> = commands
            .iter()
            .map(|c| TelegramBotCommand {
                command: c
                    .command
                    .strip_prefix('/')
                    .unwrap_or(&c.command)
                    .to_string(),
                description: c.description.clone(),
            })
            .collect();
        let payload = SetCommandsPayload { commands: &dtos };
        self.send_post_status(&url, &payload).await
    }

    pub async fn fetch_remote_state(&self) -> Result<RemoteBotState> {
        let me = self.get_me().await.context("Failed to query getMe")?;
        let name = self.get_my_name().await.ok();
        let description = self.get_my_description().await.ok();
        let short_description = self.get_my_short_description().await.ok();
        let commands = self.get_my_commands().await.unwrap_or_default();

        Ok(RemoteBotState {
            username: me.username,
            name,
            description,
            short_description,
            commands,
            has_avatar: false,
            has_cover: false,
        })
    }

    async fn send_get<T: serde::de::DeserializeOwned>(&self, url: &str) -> Result<T> {
        let res = self
            .client
            .get(url)
            .send()
            .await
            .with_context(|| format!("HTTP GET failed for {url}"))?;

        let api_resp: ApiResponse<T> = res
            .json()
            .await
            .context("Failed to deserialize Telegram API response")?;

        if !api_resp.ok {
            let desc = api_resp
                .description
                .unwrap_or_else(|| "Unknown API error".to_string());
            bail!("Telegram Bot API error: {desc}");
        }

        api_resp
            .result
            .context("Telegram Bot API returned empty result")
    }

    async fn send_post_status<P: serde::Serialize>(&self, url: &str, payload: &P) -> Result<()> {
        let res = self
            .client
            .post(url)
            .json(payload)
            .send()
            .await
            .with_context(|| format!("HTTP POST failed for {url}"))?;

        let api_resp: ApiResponse<bool> = res
            .json()
            .await
            .context("Failed to deserialize Telegram API response")?;

        if !api_resp.ok {
            let desc = api_resp
                .description
                .unwrap_or_else(|| "Unknown API error".to_string());
            bail!("Telegram Bot API error: {desc}");
        }

        Ok(())
    }
}
