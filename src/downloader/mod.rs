// =============================================================================
//  src/downloader/mod.rs
// =============================================================================

pub mod facebook;
pub mod instagram;
pub mod tiktok;

// Re-export mod_ untuk akses dari tiktok command

use anyhow::{anyhow, Result};
use reqwest::Client;
use serde_json::Value;

pub const API_BASE: &str = "https://api.siputzx.my.id";

#[derive(Debug, Clone)]
pub struct MediaItem {
    pub quality:    String,
    pub url:        String,
    pub media_type: String,
    pub backup_url: Option<String>,
}

#[derive(Debug)]
pub struct DownloadResult {
    pub platform:  &'static str,
    pub title:     String,
    pub author:    String,
    pub thumbnail: Option<String>,
    pub duration:  Option<String>,
    pub media:     Vec<MediaItem>,
    pub stats:     Option<MediaStats>,
}

#[derive(Debug, Default)]
pub struct MediaStats {
    pub play_count:    String,
    pub like_count:    String,
    pub comment_count: String,
    pub share_count:   String,
}

#[derive(Debug)]
pub enum DownloadError {
    NotFound,
    ApiFailed(String),
    InvalidUrl,
    NoMedia,
}

impl DownloadError {
    pub fn message(self) -> String {
        match self {
            Self::NotFound        => "[err] Media tidak ditemukan atau link expired.".into(),
            Self::ApiFailed(msg)  => format!("[err] API error: {msg}"),
            Self::InvalidUrl      => "[err] URL tidak valid.".into(),
            Self::NoMedia         => "[err] Tidak ada media yang bisa diunduh.".into(),
        }
    }
}

pub fn make_client() -> Result<Client> {
    Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| anyhow!("Gagal buat HTTP client: {e}"))
}

pub async fn fetch_api(client: &Client, url: &str) -> Result<Value, DownloadError> {
    let resp = client.get(url).send().await
        .map_err(|e| DownloadError::ApiFailed(e.to_string()))?;

    if !resp.status().is_success() {
        return Err(DownloadError::ApiFailed(format!("HTTP {}", resp.status())));
    }

    let json: Value = resp.json().await
        .map_err(|e| DownloadError::ApiFailed(e.to_string()))?;

    let ok = json.get("status").and_then(Value::as_bool).unwrap_or(false);
    if !ok {
        let msg = json.get("message").and_then(Value::as_str).unwrap_or("unknown").to_string();
        return Err(DownloadError::ApiFailed(msg));
    }

    json.get("data").cloned().ok_or(DownloadError::NoMedia)
}
