use std::sync::Arc;
use wacore::download::MediaType;
use whatsapp_rust::Client;

use super::{CommandContext, CommandReply, MediaReply};
use crate::downloader::{facebook, instagram, tiktok, DownloadResult, MediaItem};

pub async fn execute(ctx: &CommandContext<'_>, _client: &Arc<Client>) -> CommandReply {
    let parts: Vec<&str> = ctx.text.trim().splitn(2, ' ').collect();
    let url = parts.get(1).copied().unwrap_or("").trim();

    if url.is_empty() {
        let p = ctx.state.config.first_prefix();
        return CommandReply::Text(format!(
            "*Downloader*\n\
             `{p}tt <url>` - TikTok, Facebook, Instagram"
        ));
    }

    let result = if url.contains("tiktok.com") || url.contains("vt.tiktok.com") {
        tiktok::download(url).await
    } else if url.contains("facebook.com") || url.contains("fb.watch") {
        facebook::download(url).await
    } else if url.contains("instagram.com") {
        instagram::download(url).await
    } else {
        return CommandReply::Text("Platform tidak didukung. (TikTok, Facebook, Instagram)".to_string());
    };

    match result {
        Err(e)   => CommandReply::Text(e.message()),
        Ok(data) => build_reply(data).await,
    }
}

async fn build_reply(data: DownloadResult) -> CommandReply {
    let mut parts = vec![];
    if !data.author.is_empty() { parts.push(format!("@{}", data.author)); }
    if !data.title.is_empty() && data.title != "No description" {
        parts.push(data.title.chars().take(100).collect::<String>());
    }
    if let Some(s) = &data.stats {
        if !s.play_count.is_empty() {
            parts.push(format!("{} views  {} likes  {} comments", s.play_count, s.like_count, s.comment_count));
        }
    }
    let caption = parts.join("\n");

    if data.media.len() > 1 && data.platform == "Instagram" {
        return build_multi(&data.media, &caption).await;
    }

    let best = pick_best(&data.media);
    let item = match best {
        Some(m) => m,
        None    => return CommandReply::Text("Tidak ada media yang bisa diunduh.".to_string()),
    };

    match fetch_bytes(&item.url).await {
        Ok(bytes) => {
            let mtype    = guess_media_type(&item.media_type, &item.url);
            let filename = Some(format!("{}.{}", data.platform.to_lowercase(), ext_for(&mtype)));
            CommandReply::Media { bytes, media_type: mtype, caption, filename }
        }
        Err(e) => CommandReply::Text(format!("Download gagal: {e}\n\n{}", item.url)),
    }
}

async fn build_multi(items: &[MediaItem], caption: &str) -> CommandReply {
    let mut replies = vec![];
    for (i, item) in items.iter().enumerate() {
        if let Ok(bytes) = fetch_bytes(&item.url).await {
            let mtype = guess_media_type(&item.media_type, &item.url);
            replies.push(MediaReply {
                bytes,
                media_type: mtype,
                caption:    if i == 0 { caption.to_string() } else { String::new() },
                filename:   None,
            });
        }
    }
    if replies.is_empty() {
        CommandReply::Text("Semua media gagal diunduh.".to_string())
    } else {
        CommandReply::MultiMedia(replies)
    }
}

fn pick_best(items: &[MediaItem]) -> Option<&MediaItem> {
    for p in &["HD", "No WM", "no wm", "720p", "SD", "Original", "#1"] {
        if let Some(m) = items.iter().find(|i| i.quality.to_lowercase().contains(&p.to_lowercase())) {
            return Some(m);
        }
    }
    items.first()
}

fn guess_media_type(type_str: &str, url: &str) -> MediaType {
    let t = type_str.to_lowercase();
    let u = url.to_lowercase();
    if t.contains("audio") || u.contains(".mp3") || u.contains(".m4a") { MediaType::Audio }
    else if t.contains("image") || t.contains("photo") || u.contains(".jpg") || u.contains(".jpeg") || u.contains(".png") { MediaType::Image }
    else { MediaType::Video }
}

fn ext_for(mt: &MediaType) -> &'static str {
    match mt {
        MediaType::Video => "mp4",
        MediaType::Image => "jpg",
        MediaType::Audio => "mp3",
        _                => "bin",
    }
}

async fn fetch_bytes(url: &str) -> Result<Vec<u8>, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .user_agent("Mozilla/5.0")
        .build().map_err(|e| e.to_string())?;

    let resp = client.get(url).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() { return Err(format!("HTTP {}", resp.status())); }

    let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
    if bytes.len() > 50 * 1024 * 1024 { return Err("file terlalu besar (>50MB)".to_string()); }
    Ok(bytes.to_vec())
}
