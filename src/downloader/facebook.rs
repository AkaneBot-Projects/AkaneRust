// =============================================================================
//  src/downloader/facebook.rs — Facebook via siputzx API
// =============================================================================
//
//  GET /api/d/facebook?url=<url>
//
//  Response:
//    data.title, data.duration, data.thumbnail
//    data.downloads[].quality, .type, .url
//
// =============================================================================

use serde_json::Value;

use super::{
    fetch_api, make_client, DownloadError, DownloadResult, MediaItem, API_BASE,
};

pub async fn download(url: &str) -> Result<DownloadResult, DownloadError> {
    let client  = make_client().map_err(|e| DownloadError::ApiFailed(e.to_string()))?;
    let encoded = urlencoding::encode(url);
    let api_url = format!("{API_BASE}/api/d/facebook?url={encoded}");

    let data = fetch_api(&client, &api_url).await?;
    parse(&data).ok_or(DownloadError::NoMedia)
}

fn parse(data: &Value) -> Option<DownloadResult> {
    let title    = data.get("title").and_then(Value::as_str).unwrap_or("Facebook Video").to_string();
    let duration = data.get("duration").and_then(Value::as_str).map(str::to_string);
    let thumb    = data.get("thumbnail").and_then(Value::as_str).map(str::to_string);

    let downloads = data.get("downloads").and_then(Value::as_array)?;

    let media: Vec<MediaItem> = downloads
        .iter()
        .filter_map(|item| {
            let url     = item.get("url").and_then(Value::as_str)?.to_string();
            let quality = item.get("quality").and_then(Value::as_str).unwrap_or("?").to_string();
            let mtype   = item.get("type").and_then(Value::as_str).unwrap_or("video").to_string();
            Some(MediaItem {
                quality,
                url,
                media_type: mtype,
                backup_url: None,
            })
        })
        .collect();

    if media.is_empty() { return None; }

    Some(DownloadResult {
        platform:  "Facebook",
        title,
        author:    String::new(),
        thumbnail: thumb,
        duration,
        media,
        stats:     None,
    })
}
