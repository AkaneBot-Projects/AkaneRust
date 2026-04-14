// =============================================================================
//  src/bot/commands/mod.rs — Command Router
// =============================================================================

pub mod about;
pub mod eval;
pub mod exec;
pub mod help;
pub mod id;
pub mod info;
pub mod owner;
pub mod ping;
pub mod runtime;
pub mod tiktok;
pub mod unknown;

use std::sync::Arc;
use wacore::download::MediaType;
use wacore_binary::jid::Jid;
use whatsapp_rust::Client;

use crate::bot::state::AppState;
use eval::EvalMode;

pub struct CommandContext<'a> {
    pub text:     &'a str,
    pub sender:   String,
    pub chat:     String,
    pub chat_jid: Jid,
    pub state:    &'a AppState,
}

pub enum CommandReply {
    Text(String),
    /// Kirim contact card
    ContactCard { name: String, number: String },
    Media {
        bytes:      Vec<u8>,
        media_type: MediaType,
        caption:    String,
        filename:   Option<String>,
    },
    MultiMedia(Vec<MediaReply>),
    None,
}

pub struct MediaReply {
    pub bytes:      Vec<u8>,
    pub media_type: MediaType,
    pub caption:    String,
    pub filename:   Option<String>,
}

pub fn detect_eval_prefix(text: &str) -> Option<EvalMode> {
    let t = text.trim();
    if t.starts_with(">> ") || t == ">>" { return Some(EvalMode::Sync); }
    if t.starts_with("=> ") || t == "=>" { return Some(EvalMode::Async); }
    None
}

fn command_keyword(text: &str, prefix: &str) -> String {
    text.trim().strip_prefix(prefix).unwrap_or(text)
        .split_whitespace().next().unwrap_or("").to_lowercase()
}

pub fn route(ctx: &CommandContext<'_>) -> Option<CommandReply> {
    let text   = ctx.text.trim();
    let prefix = ctx.state.config.match_prefix(text)?;
    let cmd    = command_keyword(text, prefix);
    match cmd.as_str() {
        "ping"             => Some(CommandReply::Text(ping::execute())),
        "help" | "menu"    => Some(CommandReply::Text(help::execute(ctx))),
        "info"             => Some(CommandReply::Text(info::execute(ctx))),
        "runtime"|"uptime" => Some(CommandReply::Text(runtime::execute(ctx))),
        "id"               => Some(CommandReply::Text(id::execute(ctx))),
        "about"            => Some(CommandReply::Text(about::execute(ctx))),
        "owner"            => {
            let (name, number) = owner::contact_info(ctx);
            Some(CommandReply::ContactCard { name, number })
        },
        "tt" | "eval"      => None, // async
        _                  => None, // unknown: diam
    }
}

pub fn is_async_command(text: &str, state: &AppState) -> bool {
    if detect_eval_prefix(text).is_some() { return true; }
    if text.trim().starts_with("$ ")      { return true; }
    let prefix = match state.config.match_prefix(text.trim()) { Some(p) => p, None => return false };
    matches!(command_keyword(text, prefix).as_str(), "tt" | "eval")
}

pub async fn route_async(ctx: &CommandContext<'_>, client: &Arc<Client>) -> CommandReply {
    let text = ctx.text.trim();

    // Shell exec
    if text.starts_with("$ ") {
        return CommandReply::Text(exec::execute(ctx).await);
    }

    // Eval prefix >> / =>
    if let Some(mode) = detect_eval_prefix(text) {
        return CommandReply::Text(eval::execute(ctx, mode).await);
    }

    let prefix = match ctx.state.config.match_prefix(text) {
        Some(p) => p,
        None    => return CommandReply::None,
    };
    let cmd = command_keyword(text, prefix);

    match cmd.as_str() {
        "eval" => CommandReply::Text(eval::execute(ctx, EvalMode::Sync).await),
        "tt"   => tiktok::execute(ctx, client).await,
        _      => CommandReply::None,
    }
}
