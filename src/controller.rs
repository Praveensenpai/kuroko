use crate::domain::diff::{BotDiff, DiffAction};
use crate::domain::spec::{BotSpec, BotsConfig};
use crate::infra::config::KurokoConfig;
use crate::infra::http::BotApiClient;
use crate::infra::mtproto::{BotFatherClient, MtprotoEngine};
use anyhow::{bail, Context, Result};
use colored::Colorize;
use std::path::Path;

pub async fn compute_single_diff(bot_id: &str, bot_spec: &BotSpec) -> Result<BotDiff> {
    let token = bot_spec
        .resolve_token()
        .with_context(|| format!("Missing token for bot '{bot_id}'"))?;

    let client = BotApiClient::new(&token);
    let remote = client
        .fetch_remote_state()
        .await
        .with_context(|| format!("Failed to fetch remote state for bot '{bot_id}'"))?;

    Ok(BotDiff::compute(bot_id, bot_spec, &remote))
}

pub fn select_target_bots<'a>(
    config: &'a BotsConfig,
    filter: Option<&str>,
) -> Result<Vec<(String, &'a BotSpec)>> {
    if let Some(target) = filter {
        if let Some(bot) = config.bots.get(target) {
            Ok(vec![(target.to_string(), bot)])
        } else {
            bail!("Bot '{target}' not found in configuration file");
        }
    } else {
        let mut list: Vec<(String, &'a BotSpec)> =
            config.bots.iter().map(|(k, v)| (k.clone(), v)).collect();
        list.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(list)
    }
}

pub async fn apply_action(
    bot_id: &str,
    bot_spec: &BotSpec,
    action: &DiffAction,
    http: &BotApiClient,
    http_only: bool,
    mtproto: &mut Option<MtprotoEngine>,
) -> Result<()> {
    match action {
        DiffAction::SetName(name) => {
            print!("  -> Updating name to '{name}' ... ");
            http.set_my_name(name).await?;
            println!("{}", "OK".green());
        }
        DiffAction::SetDescription(desc) => {
            print!("  -> Updating description ... ");
            http.set_my_description(desc).await?;
            println!("{}", "OK".green());
        }
        DiffAction::SetShortDescription(short) => {
            print!("  -> Updating short description ... ");
            http.set_my_short_description(short).await?;
            println!("{}", "OK".green());
        }
        DiffAction::SetCommands(cmds) => {
            print!("  -> Updating {} command(s) ... ", cmds.len());
            http.set_my_commands(cmds).await?;
            println!("{}", "OK".green());
        }
        DiffAction::UploadAvatar(path) => {
            apply_avatar_upload(bot_id, bot_spec, path, http, http_only, mtproto).await?;
        }
        DiffAction::UploadCover(path) => {
            apply_cover_upload(bot_id, bot_spec, path, http, http_only, mtproto).await?;
        }
        DiffAction::SetPrivacyMode(enabled) => {
            apply_privacy_setting(bot_id, bot_spec, *enabled, http, http_only, mtproto).await?;
        }
        DiffAction::SetCanJoinGroups(enabled) => {
            apply_group_setting(bot_id, bot_spec, *enabled, http, http_only, mtproto).await?;
        }
        DiffAction::SetInlineMode(_) => {
            println!("  ! Inline mode configuration available in upcoming release");
        }
    }
    Ok(())
}

async fn apply_avatar_upload(
    bot_id: &str,
    bot_spec: &BotSpec,
    path: &str,
    http: &BotApiClient,
    http_only: bool,
    mtproto: &mut Option<MtprotoEngine>,
) -> Result<()> {
    if http_only {
        println!(
            "  {} Skipping avatar upload (--http-only mode)",
            "!".yellow()
        );
        return Ok(());
    }
    ensure_mtproto_ready(mtproto).await?;
    if let Some(engine) = mtproto {
        let bf = BotFatherClient::new(engine.client());
        let username = resolve_username(bot_id, bot_spec, http).await?;
        print!("  -> Uploading avatar from '{path}' via BotFather ... ");
        bf.upload_bot_avatar(&username, Path::new(path)).await?;
        println!("{}", "OK".green());
    }
    Ok(())
}

async fn apply_cover_upload(
    bot_id: &str,
    bot_spec: &BotSpec,
    path: &str,
    http: &BotApiClient,
    http_only: bool,
    mtproto: &mut Option<MtprotoEngine>,
) -> Result<()> {
    if http_only {
        println!(
            "  {} Skipping cover upload (--http-only mode)",
            "!".yellow()
        );
        return Ok(());
    }
    ensure_mtproto_ready(mtproto).await?;
    if let Some(engine) = mtproto {
        let bf = BotFatherClient::new(engine.client());
        let username = resolve_username(bot_id, bot_spec, http).await?;
        print!("  -> Uploading cover media from '{path}' via BotFather ... ");
        bf.upload_bot_cover(&username, Path::new(path)).await?;
        println!("{}", "OK".green());
    }
    Ok(())
}

async fn apply_privacy_setting(
    bot_id: &str,
    bot_spec: &BotSpec,
    enabled: bool,
    http: &BotApiClient,
    http_only: bool,
    mtproto: &mut Option<MtprotoEngine>,
) -> Result<()> {
    if http_only {
        return Ok(());
    }
    ensure_mtproto_ready(mtproto).await?;
    if let Some(engine) = mtproto {
        let bf = BotFatherClient::new(engine.client());
        let username = resolve_username(bot_id, bot_spec, http).await?;
        print!("  -> Setting privacy mode to {enabled} ... ");
        bf.toggle_privacy_mode(&username, enabled).await?;
        println!("{}", "OK".green());
    }
    Ok(())
}

async fn apply_group_setting(
    bot_id: &str,
    bot_spec: &BotSpec,
    enabled: bool,
    http: &BotApiClient,
    http_only: bool,
    mtproto: &mut Option<MtprotoEngine>,
) -> Result<()> {
    if http_only {
        return Ok(());
    }
    ensure_mtproto_ready(mtproto).await?;
    if let Some(engine) = mtproto {
        let bf = BotFatherClient::new(engine.client());
        let username = resolve_username(bot_id, bot_spec, http).await?;
        print!("  -> Setting group joining to {enabled} ... ");
        bf.toggle_group_joining(&username, enabled).await?;
        println!("{}", "OK".green());
    }
    Ok(())
}

async fn resolve_username(bot_id: &str, _spec: &BotSpec, http: &BotApiClient) -> Result<String> {
    if let Ok(me) = http.get_me().await {
        if let Some(u) = me.username {
            return Ok(u);
        }
    }
    Ok(bot_id.to_string())
}

async fn ensure_mtproto_ready(engine: &mut Option<MtprotoEngine>) -> Result<()> {
    if engine.is_some() {
        return Ok(());
    }
    let config = KurokoConfig::load();
    let (api_id, api_hash) = config.get_api_credentials().context(
        "MTProto operation requires Telegram API credentials. Run `kuroko login` first or set TELEGRAM_API_ID & TELEGRAM_API_HASH",
    )?;

    let client = MtprotoEngine::connect(api_id, &api_hash).await?;
    if !client.is_authorized().await? {
        bail!("MTProto session not authorized. Please run `kuroko login` first.");
    }
    *engine = Some(client);
    Ok(())
}
