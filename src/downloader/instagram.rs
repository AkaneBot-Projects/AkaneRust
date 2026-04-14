// =============================================================================
//  src/downloader/instagram.rs — Instagram via siputzx API
// =============================================================================
//
//  GET /api/d/ig?url=<url>
//
//  Respons bervariasi:
//    - Single video: data.url (string)
//    - Carousel/multi: data.urls (array)
//    - Dengan: data.thumbnail, data.caption, data.username
//
// =============================================================================

use serde_json::Value;

use super::{
    fetch_api, make_client, DownloadError, DownloadResult, MediaItem, API_BASE,
};

pub async fn download(url: &str) -> Result<DownloadResult, DownloadError> {
    let client  = make_client().map_err(|e| DownloadError::ApiFailed(e.to_string()))?;
    let encoded = urlencoding::encode(url);
    let api_url = format!("{API_BASE}/api/d/ig?url={encoded}");

    let data = fetch_api(&client, &api_url).await?;
    parse(&data).ok_or(DownloadError::NoMedia)
}

fn parse(data: &Value) -> Option<DownloadResult> {
    let author  = data.get("username").and_then(Value::as_str).unwrap_or("").to_string();
    let title   = data.get("caption").and_then(Value::as_str).unwrap_or("").to_string();
    let thumb   = data.get("thumbnail").and_then(Value::as_str).map(str::to_string);

    let mut media = vec![];

    // ── Single video / foto ───────────────────────────────────────────────
    if let Some(url) = data.get("url").and_then(Value::as_str) {
        if !url.is_empty() {
            media.push(MediaItem {
                quality:    "Original".into(),
                url:        url.to_string(),
                media_type: "video".into(),
                backup_url: None,
            });
        }
    }

    // ── Carousel / multiple media ─────────────────────────────────────────
    if let Some(urls) = data.get("urls").and_then(Value::as_array) {
        for (i, item) in urls.iter().enumerate() {
            // Bisa berupa string URL langsung atau object {url, type}
            let (url, mtype) = if let Some(s) = item.as_str() {
                (s.to_string(), "media".to_string())
            } else {
                let u = item.get("url").and_then(Value::as_str).unwrap_or("").to_string();
                let t = item.get("type").and_then(Value::as_str).unwrap_or("media").to_string();
                (u, t)
            };

            if !url.is_empty() {
                media.push(MediaItem {
                    quality:    format!("#{}", i + 1),
                    url,
                    media_type: mtype,
                    backup_url: None,
                });
            }
        }
    }

    if media.is_empty() { return None; }

    Some(DownloadResult {
        platform:  "Instagram",
        title,
        author,
        thumbnail: thumb,
        duration:  None,
        media,
        stats:     None,
    })
}
