use std::sync::Arc;
use wacore::proto_helpers::MessageExt;
use wacore::types::events::Event;
use whatsapp_rust::Client;

use super::client::{send_contact, send_media, send_reply, QuoteInfo};
use super::commands::{is_async_command, route, route_async, CommandContext, CommandReply};
use super::state::AppState;

pub async fn handle_event(event: Event, client: Arc<Client>, state: Arc<AppState>) {
    match event {
        Event::PairingCode { code, .. } => {
            println!();
            println!("╔══════════════════════════════════╗");
            println!("║  PAIR CODE:  {:^18}  ║", code);
            println!("╚══════════════════════════════════╝");
            println!("  -> WA -> Settings -> Linked Devices -> Link a Device");
            println!("  -> Pilih 'Tautkan dengan nomor telepon'");
            println!();
        }
        Event::PairingQrCode { code, .. } => { println!("\n[QR fallback]\n{code}\n"); }
        Event::PairSuccess(_)  => { println!("[ok] Pairing berhasil!"); }
        Event::Connected(_)    => { println!("[ok] Bot terhubung."); }
        Event::Disconnected(_) => { println!("[--] Terputus, mencoba reconnect..."); }
        Event::LoggedOut(_)    => {
            eprintln!("[!!] Logged out. Hapus {} lalu restart.", state.config.bot.db_path);
        }

        Event::Message(msg, info) => {
            let Some(text) = msg.text_content()
                .filter(|t| !t.is_empty())
                .or_else(|| msg.conversation.as_deref().filter(|t| !t.is_empty()))
            else { return };

            let sender_str = info.source.sender.to_string();
            let chat_str   = info.source.chat.to_string();
            let chat_jid   = info.source.chat.clone();

            println!("[msg] [{}] {}: {}", chat_str, sender_str, text);

            let has_prefix  = state.config.match_prefix(text.trim()).is_some();
            let has_eval_px = super::commands::detect_eval_prefix(text).is_some();
            let has_exec_px = text.trim().starts_with("$ ");
            if !has_prefix && !has_eval_px && !has_exec_px { return; }

            let quote = QuoteInfo {
                msg_id: info.id.clone(),
                sender: sender_str.clone(),
                msg:    *msg.clone(),
            };

            let ctx = CommandContext {
                text,
                sender:   sender_str,
                chat:     chat_str,
                chat_jid: chat_jid.clone(),
                state:    &state,
            };

            // ── Async command ─────────────────────────────────────────────
            if is_async_command(text, &state) || has_exec_px {
                send_reply(&client, chat_jid.clone(), &quote, "processing...").await;
                match route_async(&ctx, &client).await {
                    CommandReply::Text(t)  => {
                        send_reply(&client, chat_jid, &quote, t).await;
                    }
                    CommandReply::ContactCard { name, number } => {
                        send_contact(&client, chat_jid, &quote, &name, &number).await;
                    }
                    CommandReply::Media { bytes, media_type, caption, filename } => {
                        send_media(&client, chat_jid, bytes, media_type, &caption,
                                   filename.as_deref(), Some(&quote)).await;
                    }
                    CommandReply::MultiMedia(items) => {
                        for (i, item) in items.into_iter().enumerate() {
                            let q = if i == 0 { Some(&quote) } else { None };
                            send_media(&client, chat_jid.clone(), item.bytes,
                                       item.media_type, &item.caption,
                                       item.filename.as_deref(), q).await;
                        }
                    }
                    CommandReply::None => {}
                }
                return;
            }

            // ── Sync command ──────────────────────────────────────────────
            match route(&ctx) {
                Some(CommandReply::Text(t)) => {
                    send_reply(&client, chat_jid, &quote, t).await;
                }
                Some(CommandReply::ContactCard { name, number }) => {
                    send_contact(&client, chat_jid, &quote, &name, &number).await;
                }
                Some(other) => {
                    // Media dari sync — tidak seharusnya terjadi, fallback teks
                    if let CommandReply::Text(t) = other {
                        send_reply(&client, chat_jid, &quote, t).await;
                    }
                }
                None => {}
            }
        }

        _ => {}
    }
}
