use anyhow::{bail, Context, Result};
use grammers_client::message::InputMessage;
use grammers_client::Client;
use grammers_session::types::PeerRef;
use std::path::Path;
use std::time::Duration;
use tokio::time::sleep;

pub struct BotFatherClient<'a> {
    client: &'a Client,
}

impl<'a> BotFatherClient<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn get_botfather_peer(&self) -> Result<PeerRef> {
        let peer = self
            .client
            .resolve_username("BotFather")
            .await
            .context("Failed to resolve @BotFather")?
            .context("@BotFather not found on Telegram")?;

        let peer_ref = peer
            .to_ref()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to convert BotFather to PeerRef: {e}"))?
            .context("BotFather PeerRef missing")?;

        Ok(peer_ref)
    }

    pub async fn upload_bot_avatar<P: AsRef<Path>>(
        &self,
        bot_username: &str,
        image_path: P,
    ) -> Result<()> {
        let peer = self.get_botfather_peer().await?;
        let clean_username = bot_username.strip_prefix('@').unwrap_or(bot_username);

        // Step 1: Send /setuserpic command
        self.client.send_message(peer, "/setuserpic").await?;
        sleep(Duration::from_millis(800)).await;

        // Step 2: Choose the bot
        let target = format!("@{clean_username}");
        self.client.send_message(peer, target.as_str()).await?;
        sleep(Duration::from_millis(800)).await;

        // Step 3: Upload the image file
        let uploaded = self
            .client
            .upload_file(image_path.as_ref())
            .await
            .with_context(|| {
                format!(
                    "Failed to upload avatar file: {}",
                    image_path.as_ref().display()
                )
            })?;

        let input_msg = InputMessage::new().text("").photo(uploaded);
        self.client.send_message(peer, input_msg).await?;
        sleep(Duration::from_millis(500)).await;

        Ok(())
    }

    pub async fn upload_bot_cover<P: AsRef<Path>>(
        &self,
        bot_username: &str,
        media_path: P,
    ) -> Result<()> {
        let peer = self.get_botfather_peer().await?;
        let clean_username = bot_username.strip_prefix('@').unwrap_or(bot_username);

        // Step 1: Send /setintro
        self.client.send_message(peer, "/setintro").await?;
        sleep(Duration::from_millis(800)).await;

        // Step 2: Choose the bot
        let target = format!("@{clean_username}");
        self.client.send_message(peer, target.as_str()).await?;
        sleep(Duration::from_millis(800)).await;

        // Step 3: Upload the cover media
        let uploaded = self
            .client
            .upload_file(media_path.as_ref())
            .await
            .with_context(|| {
                format!(
                    "Failed to upload cover file: {}",
                    media_path.as_ref().display()
                )
            })?;

        let input_msg = InputMessage::new().text("").photo(uploaded);
        self.client.send_message(peer, input_msg).await?;
        sleep(Duration::from_millis(500)).await;

        Ok(())
    }

    pub async fn toggle_privacy_mode(&self, bot_username: &str, enabled: bool) -> Result<()> {
        let peer = self.get_botfather_peer().await?;
        let clean = bot_username.strip_prefix('@').unwrap_or(bot_username);

        self.client.send_message(peer, "/setprivacy").await?;
        sleep(Duration::from_millis(800)).await;

        let target = format!("@{clean}");
        self.client.send_message(peer, target.as_str()).await?;
        sleep(Duration::from_millis(800)).await;

        let choice = if enabled { "Enable" } else { "Disable" };
        self.client.send_message(peer, choice).await?;
        Ok(())
    }

    pub async fn toggle_group_joining(&self, bot_username: &str, enabled: bool) -> Result<()> {
        let peer = self.get_botfather_peer().await?;
        let clean = bot_username.strip_prefix('@').unwrap_or(bot_username);

        self.client.send_message(peer, "/setjoingroups").await?;
        sleep(Duration::from_millis(800)).await;

        let target = format!("@{clean}");
        self.client.send_message(peer, target.as_str()).await?;
        sleep(Duration::from_millis(800)).await;

        let choice = if enabled { "Enable" } else { "Disable" };
        self.client.send_message(peer, choice).await?;
        Ok(())
    }

    pub async fn create_new_bot(&self, display_name: &str, username: &str) -> Result<String> {
        let peer = self.get_botfather_peer().await?;

        // 1. Send /newbot
        self.client.send_message(peer, "/newbot").await?;
        sleep(Duration::from_millis(1000)).await;

        // 2. Send display name
        self.client.send_message(peer, display_name).await?;
        sleep(Duration::from_millis(1000)).await;

        // 3. Send username (must end in bot)
        let clean_username = if username.ends_with("bot") || username.ends_with("Bot") {
            username.to_string()
        } else {
            format!("{username}_bot")
        };
        self.client
            .send_message(peer, clean_username.as_str())
            .await?;
        sleep(Duration::from_millis(1500)).await;

        // Extract token from latest message
        let mut messages = self.client.iter_messages(peer).limit(3);
        while let Some(msg) = messages.next().await? {
            let text = msg.text();
            if let Some(token) = extract_token_from_botfather_response(text) {
                return Ok(token);
            }
        }

        bail!("Bot created, but could not automatically parse API token from @BotFather response");
    }
}

pub fn extract_token_from_botfather_response(text: &str) -> Option<String> {
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(pos) = trimmed.find(':') {
            let (prefix, suffix) = trimmed.split_at(pos);
            let suffix = &suffix[1..];
            if prefix.chars().all(|c| c.is_ascii_digit())
                && suffix.len() >= 30
                && suffix
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
            {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_token_from_botfather_response() {
        let msg = "Done! Congratulations on your new bot.\nUse this token to access the HTTP API:\n7123456789:AAFlkjf9023_sdkjfdskjf90234_sdfkljsdf\nKeep your token secure.";
        let token = extract_token_from_botfather_response(msg);
        assert_eq!(
            token,
            Some("7123456789:AAFlkjf9023_sdkjfdskjf90234_sdfkljsdf".to_string())
        );
    }
}
