// =============================================================================
//  src/bot/client.rs — Message & Media Helpers
// =============================================================================

use std::sync::Arc;
use wacore::download::MediaType;
use wacore::proto_helpers::build_quote_context;
use wacore_binary::jid::Jid;
use waproto::whatsapp as wa;
use waproto::whatsapp::message::{
    AudioMessage, ContactMessage, DocumentMessage, ExtendedTextMessage,
    ImageMessage, VideoMessage,
};
use whatsapp_rust::Client;

#[derive(Clone)]
pub struct QuoteInfo {
    pub msg_id: String,
    pub sender: String,
    pub msg:    wa::Message,
}

// ── Text ──────────────────────────────────────────────────────────────────────

pub fn make_text(text: impl Into<String>) -> wa::Message {
    wa::Message { conversation: Some(text.into()), ..Default::default() }
}

pub async fn send_text(client: &Arc<Client>, chat: Jid, text: impl Into<String>) {
    if let Err(e) = client.send_message(chat, make_text(text)).await {
        eprintln!("[err] send_text: {e}");
    }
}

/// Kirim teks dengan reply/quote ke pesan asli
pub async fn send_reply(
    client: &Arc<Client>,
    chat:   Jid,
    quote:  &QuoteInfo,
    text:   impl Into<String>,
) {
    let context = build_quote_context(&quote.msg_id, &quote.sender, &quote.msg);
    let msg = wa::Message {
        extended_text_message: Some(Box::new(ExtendedTextMessage {
            text:         Some(text.into()),
            context_info: Some(Box::new(context)),
            ..Default::default()
        })),
        ..Default::default()
    };
    if let Err(e) = client.send_message(chat, msg).await {
        eprintln!("[err] send_reply: {e}");
    }
}

// ── Contact card ──────────────────────────────────────────────────────────────

/// Kirim kontak sebagai contact card WhatsApp (vCard)
pub async fn send_contact(
    client:  &Arc<Client>,
    chat:    Jid,
    quote:   &QuoteInfo,
    name:    &str,
    number:  &str,
) {
    // Nomor WA dalam format +62... untuk vCard
    let wa_number = if number.starts_with('+') {
        number.to_string()
    } else {
        format!("+{number}")
    };

    // Standard vCard 3.0 — format yang WA bisa parse jadi contact card
    let vcard = format!(
        "BEGIN:VCARD\r\n\
         VERSION:3.0\r\n\
         FN:{name}\r\n\
         TEL;type=CELL;type=VOICE;waid={number}:{wa_number}\r\n\
         END:VCARD"
    );

    let context = build_quote_context(&quote.msg_id, &quote.sender, &quote.msg);

    let msg = wa::Message {
        contact_message: Some(Box::new(ContactMessage {
            display_name: Some(name.to_string()),
            vcard:        Some(vcard),
            context_info: Some(Box::new(context)),
            ..Default::default()
        })),
        ..Default::default()
    };

    let chat_clone = chat.clone();

    if let Err(e) = client.send_message(chat, msg).await {
        eprintln!("[err] send_contact: {e}");

        // fallback ke text kalau gagal
        send_reply(
            client,
            chat_clone,
            quote,
            format!("{name}: {wa_number}")
        ).await;
    }
}

// ── Media ─────────────────────────────────────────────────────────────────────

pub async fn send_media(
    client:     &Arc<Client>,
    chat:       Jid,
    bytes:      Vec<u8>,
    media_type: MediaType,
    caption:    &str,
    filename:   Option<&str>,
    quote:      Option<&QuoteInfo>,
) {
    let upload = match client.upload(bytes, media_type.clone()).await {
        Ok(u)  => u,
        Err(e) => {
            eprintln!("[err] upload: {e}");
            let q = match quote { Some(q) => q.clone(), None => return };
            send_reply(client, chat, &q, format!("[err] Upload gagal: {e}\n\n{caption}")).await;
            return;
        }
    };

    let ctx_info: Option<Box<wa::ContextInfo>> = quote.map(|q| {
        Box::new(build_quote_context(&q.msg_id, &q.sender, &q.msg))
    });

    let msg = match media_type {
        MediaType::Video => wa::Message {
            video_message: Some(Box::new(VideoMessage {
                url:             Some(upload.url),
                direct_path:     Some(upload.direct_path),
                media_key:       Some(upload.media_key),
                file_sha256:     Some(upload.file_sha256),
                file_enc_sha256: Some(upload.file_enc_sha256),
                file_length:     Some(upload.file_length),
                mimetype:        Some("video/mp4".to_string()),
                caption:         Some(caption.to_string()),
                context_info:    ctx_info,
                ..Default::default()
            })),
            ..Default::default()
        },
        MediaType::Image => wa::Message {
            image_message: Some(Box::new(ImageMessage {
                url:             Some(upload.url),
                direct_path:     Some(upload.direct_path),
                media_key:       Some(upload.media_key),
                file_sha256:     Some(upload.file_sha256),
                file_enc_sha256: Some(upload.file_enc_sha256),
                file_length:     Some(upload.file_length),
                mimetype:        Some("image/jpeg".to_string()),
                caption:         Some(caption.to_string()),
                context_info:    ctx_info,
                ..Default::default()
            })),
            ..Default::default()
        },
        MediaType::Audio => wa::Message {
            audio_message: Some(Box::new(AudioMessage {
                url:             Some(upload.url),
                direct_path:     Some(upload.direct_path),
                media_key:       Some(upload.media_key),
                file_sha256:     Some(upload.file_sha256),
                file_enc_sha256: Some(upload.file_enc_sha256),
                file_length:     Some(upload.file_length),
                mimetype:        Some("audio/mpeg".to_string()),
                context_info:    ctx_info,
                ..Default::default()
            })),
            ..Default::default()
        },
        _ => wa::Message {
            document_message: Some(Box::new(DocumentMessage {
                url:             Some(upload.url),
                direct_path:     Some(upload.direct_path),
                media_key:       Some(upload.media_key),
                file_sha256:     Some(upload.file_sha256),
                file_enc_sha256: Some(upload.file_enc_sha256),
                file_length:     Some(upload.file_length),
                mimetype:        Some("application/octet-stream".to_string()),
                title:           Some(filename.unwrap_or("file").to_string()),
                file_name:       Some(filename.unwrap_or("file").to_string()),
                caption:         Some(caption.to_string()),
                context_info:    ctx_info,
                ..Default::default()
            })),
            ..Default::default()
        },
    };

    if let Err(e) = client.send_message(chat, msg).await {
        eprintln!("[err] send_media: {e}");
    }
}
