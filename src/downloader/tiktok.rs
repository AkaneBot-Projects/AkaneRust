// =============================================================================
//  src/downloader/tiktok.rs — TikTok via siputzx API
// =============================================================================
//
//  Endpoint v2 lebih lengkap (ada HD, stats), tapi v1 sebagai fallback.
//
//  v2: GET /api/d/tiktok/v2?url=<url>
//  v1: GET /api/d/tiktok?url=<url>
//
// =============================================================================

use serde_json::Value;

use super::{
    fetch_api, make_client, DownloadError, DownloadResult, MediaItem, MediaStats, API_BASE,
};

pub async fn download(url: &str) -> Result<DownloadResult, DownloadError> {
    let client = make_client().map_err(|e| DownloadError::ApiFailed(e.to_string()))?;

    let encoded = urlencoding::encode(url);

    // ── Coba v2 dulu (lebih lengkap) ──────────────────────────────────────
    let api_v2 = format!("{API_BASE}/api/d/tiktok/v2?url={encoded}");
    if let Ok(data) = fetch_api(&client, &api_v2).await {
        if let Some(result) = parse_v2(&data) {
            return Ok(result);
        }
    }

    // ── Fallback ke v1 ────────────────────────────────────────────────────
    let api_v1 = format!("{API_BASE}/api/d/tiktok?url={encoded}");
    let data    = fetch_api(&client, &api_v1).await?;
    parse_v1(&data).ok_or(DownloadError::NoMedia)
}

/// Parse response v2 (lebih detail)
fn parse_v2(data: &Value) -> Option<DownloadResult> {
    let get = |key: &str| {
        data.get(key)
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string()
    };

    let author = get("author_nickname");
    if author.is_empty() { return None; }

    let mut media = vec![];

    // No-watermark SD
    let nwm = get("no_watermark_link");
    if !nwm.is_empty() {
        media.push(MediaItem {
            quality:    "SD (No WM)".into(),
            url:        nwm,
            media_type: "video".into(),
            backup_url: None,
        });
    }

    // No-watermark HD
    let hd = get("no_watermark_link_hd");
    if !hd.is_empty() {
        media.push(MediaItem {
            quality:    "HD (No WM)".into(),
            url:        hd,
            media_type: "video".into(),
            backup_url: None,
        });
    }

    // Music
    let music = get("music_link");
    if !music.is_empty() {
        media.push(MediaItem {
            quality:    "Audio".into(),
            url:        music,
            media_type: "audio".into(),
            backup_url: None,
        });
    }

    if media.is_empty() { return None; }

    let stats = MediaStats {
        play_count:    get("play_count"),
        like_count:    get("like_count"),
        comment_count: get("comment_count"),
        share_count:   get("share_count"),
    };

    // Duration: API v2 return milidetik sebagai string
    let duration = data.get("duration")
        .and_then(Value::as_str)
        .and_then(|s| s.parse::<u64>().ok())
        .map(|ms| {
            let secs = ms / 1000;
            format!("{}:{:02}", secs / 60, secs % 60)
        });

    Some(DownloadResult {
        platform:  "TikTok",
        title:     get("text"),
        author,
        thumbnail: {
            let t = get("cover_link");
            if t.is_empty() { None } else { Some(t) }
        },
        duration,
        media,
        stats: Some(stats),
    })
}

/// Parse response v1 (lebih sederhana)
fn parse_v1(data: &Value) -> Option<DownloadResult> {
    let title  = data.get("title").and_then(Value::as_str).unwrap_or("").to_string();
    let author = data.get("author").and_then(Value::as_str).unwrap_or("").to_string();
    let thumb  = data.get("thumbnail").and_then(Value::as_str).map(str::to_string);

    let media_arr = data.get("media").and_then(Value::as_array)?;

    let media: Vec<MediaItem> = media_arr
        .iter()
        .filter_map(|item| {
            let url = item.get("url").and_then(Value::as_str)?.to_string();
            let quality    = item.get("quality").and_then(Value::as_str).unwrap_or("?").to_string();
            let media_type = item.get("type").and_then(Value::as_str).unwrap_or("video").to_string();
            let backup_url = item.get("backup").and_then(Value::as_str).map(str::to_string);
            Some(MediaItem { quality, url, media_type, backup_url })
        })
        .collect();

    if media.is_empty() { return None; }

    Some(DownloadResult {
        platform:  "TikTok",
        title,
        author,
        thumbnail: thumb,
        duration:  None,
        media,
        stats:     None,
    })
}
