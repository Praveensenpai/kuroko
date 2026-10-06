use super::session::load_or_create_session;
use crate::infra::config::KurokoConfig;
use anyhow::{bail, Context, Result};
use dialoguer::Password;
use grammers_client::{Client, SenderPool};

pub struct MtprotoEngine {
    client: Client,
    api_hash: String,
}

impl MtprotoEngine {
    pub async fn connect(api_id: i32, api_hash: &str) -> Result<Self> {
        KurokoConfig::ensure_dir()?;
        let session_path = KurokoConfig::session_file();
        let session = load_or_create_session(&session_path).await?;

        // `SenderPool::new` only builds the pool. Its runner must be driven for
        // the connection to carry any request, so spawn it here; otherwise the
        // pool is dropped when this function returns and every invocation fails
        // with `InvocationError::Dropped`.
        let pool = SenderPool::new(session, api_id);
        let client = Client::new(pool.handle.clone());
        tokio::spawn(pool.runner.run());

        Ok(Self {
            client,
            api_hash: api_hash.to_string(),
        })
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub async fn is_authorized(&self) -> Result<bool> {
        self.client
            .is_authorized()
            .await
            .context("Failed to check authorization status")
    }

    pub async fn login_interactive(&self) -> Result<()> {
        if self.is_authorized().await? {
            println!("  ✔ Already authorized via active MTProto session.");
            return Ok(());
        }

        let phone: String = dialoguer::Input::new()
            .with_prompt("Enter your Telegram phone number (international format e.g. +1234567890)")
            .interact_text()
            .context("Failed to read phone number")?;

        let token = self
            .client
            .request_login_code(phone.trim(), &self.api_hash)
            .await
            .context("Failed to request login code from Telegram")?;

        let code: String = dialoguer::Input::new()
            .with_prompt("Enter the login code received on Telegram")
            .interact_text()
            .context("Failed to read login code")?;

        match self.client.sign_in(&token, code.trim()).await {
            Ok(_) => {
                println!("  ✔ Successfully signed in to Telegram.");
                Ok(())
            }
            Err(grammers_client::SignInError::PasswordRequired(password_token)) => {
                let password = Password::new()
                    .with_prompt("Enter your 2FA Cloud Password")
                    .interact()
                    .context("Failed to read 2FA password")?;

                self.client
                    .check_password(password_token, password.trim())
                    .await
                    .context("Failed to verify 2FA password")?;

                println!("  ✔ Successfully signed in with 2FA password.");
                Ok(())
            }
            Err(e) => {
                bail!("Sign in failed: {e}");
            }
        }
    }
}
