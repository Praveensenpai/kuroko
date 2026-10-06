//! Interactive `MTProto` chat helpers for driving `@BotFather` step by step.
//!
//! Unlike [`super::botfather::BotFatherClient`], these primitives send a single
//! message and then wait for the actual reply, rendering any inline buttons so
//! the caller can decide the next move instead of guessing.

use crate::infra::config::KurokoConfig;
use crate::infra::mtproto::MtprotoEngine;
use anyhow::{bail, Context, Result};
use grammers_client::message::Message;
use grammers_client::tl;
use grammers_client::Client;
use grammers_session::types::PeerRef;
use std::time::{Duration, Instant};
use tokio::time::sleep;

const POLL_INTERVAL: Duration = Duration::from_millis(800);
const POLL_LIMIT: usize = 5;

/// Connects using stored credentials and fails if the session is not authorized.
pub async fn connect_authorized() -> Result<MtprotoEngine> {
    let config = KurokoConfig::load();
    let (api_id, api_hash) = config.get_api_credentials().context(
        "MTProto requires credentials. Run `kuroko login` first or set TELEGRAM_API_ID & TELEGRAM_API_HASH",
    )?;
    let engine = MtprotoEngine::connect(api_id, &api_hash).await?;
    if !engine.is_authorized().await? {
        bail!("MTProto session not authorized. Run `kuroko login` first.");
    }
    Ok(engine)
}

/// Resolves a username (with or without leading `@`) into a sendable peer.
pub async fn resolve_peer(client: &Client, username: &str) -> Result<PeerRef> {
    let clean = username.trim().trim_start_matches('@');
    let peer = client
        .resolve_username(clean)
        .await
        .with_context(|| format!("Failed to resolve @{clean}"))?
        .with_context(|| format!("@{clean} not found on Telegram"))?;
    peer.to_ref()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to convert @{clean} to PeerRef: {e}"))?
        .with_context(|| format!("@{clean} PeerRef missing"))
}

/// Renders a message as text plus any inline buttons and their callback data.
#[must_use]
pub fn render_message(msg: &Message) -> String {
    use std::fmt::Write as _;

    let who = if msg.outgoing() { "me" } else { "them" };
    let mut out = String::new();
    let _ = writeln!(out, "[{who}] {}", msg.text());
    match msg.reply_markup() {
        Some(tl::enums::ReplyMarkup::ReplyInlineMarkup(markup)) => {
            for row in markup.rows {
                let tl::enums::KeyboardButtonRow::Row(r) = row;
                for btn in r.buttons {
                    match btn {
                        tl::enums::KeyboardButton::Callback(cb) => {
                            let data = String::from_utf8_lossy(&cb.data);
                            let text = cb.text;
                            let _ = writeln!(out, "   [button] \"{text}\" data=\"{data}\"");
                        }
                        other => {
                            let _ = writeln!(out, "   [button] {other:?}");
                        }
                    }
                }
            }
        }
        Some(tl::enums::ReplyMarkup::ReplyKeyboardMarkup(markup)) => {
            for row in markup.rows {
                let tl::enums::KeyboardButtonRow::Row(r) = row;
                for btn in r.buttons {
                    if let tl::enums::KeyboardButton::Button(b) = btn {
                        let _ = writeln!(out, "   [key] \"{}\"", b.text);
                    }
                }
            }
        }
        Some(
            tl::enums::ReplyMarkup::ReplyKeyboardHide(_)
            | tl::enums::ReplyMarkup::ReplyKeyboardForceReply(_),
        )
        | None => {}
    }
    out
}

/// Sends one message, waits for the reply, and prints it with any buttons.
pub async fn send_and_wait(
    client: &Client,
    peer: PeerRef,
    text: &str,
    timeout: Duration,
) -> Result<()> {
    let sent = client
        .send_message(peer, text)
        .await
        .with_context(|| format!("Failed to send message: {text}"))?;
    let sent_id = sent.id();
    println!("  ➤ sent: {text}");

    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        sleep(POLL_INTERVAL).await;
        let replies = incoming_after(client, peer, sent_id).await?;
        if !replies.is_empty() {
            for reply in replies {
                print!("{}", render_message(&reply));
            }
            return Ok(());
        }
    }
    bail!("No reply within {}s", timeout.as_secs())
}

/// Prints the most recent messages in a chat, oldest first.
pub async fn read_latest(client: &Client, peer: PeerRef, count: usize) -> Result<()> {
    let mut messages = client.iter_messages(peer).limit(count);
    let mut collected = Vec::new();
    while let Some(msg) = messages.next().await? {
        collected.push(msg);
    }
    collected.reverse();
    for msg in collected {
        print!("{}", render_message(&msg));
    }
    Ok(())
}

/// Incoming messages newer than `after_id`, returned oldest first.
async fn incoming_after(client: &Client, peer: PeerRef, after_id: i32) -> Result<Vec<Message>> {
    let mut messages = client.iter_messages(peer).limit(POLL_LIMIT);
    let mut out = Vec::new();
    while let Some(msg) = messages.next().await? {
        if !msg.outgoing() && msg.id() > after_id {
            out.push(msg);
        }
    }
    out.reverse();
    Ok(out)
}
