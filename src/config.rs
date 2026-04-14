use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub bot:   BotConfig,
    pub auth:  AuthConfig,
    pub owner: OwnerConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BotConfig {
    pub name:     String,
    pub prefixes: Vec<String>,
    pub db_path:  String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    pub phone_number: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OwnerConfig {
    pub name:   String,
    /// Nomor HP owner (format internasional, tanpa +)
    pub number: String,
    /// LID WhatsApp owner — diisi jika sender pakai format @lid
    /// Dapatkan dari: kirim ".id", ambil angka sebelum "@lid"
    #[serde(default)]
    pub lid:    String,
}

impl Config {
    pub fn load() -> Result<Self> {
        let raw = fs::read_to_string("config.toml").context("Gagal membaca config.toml")?;
        toml::from_str(&raw).context("config.toml format tidak valid")
    }

    pub fn match_prefix<'a>(&self, text: &'a str) -> Option<&str> {
        let text = text.trim();
        for p in &self.bot.prefixes {
            if text.starts_with(p.as_str()) { return Some(p.as_str()); }
        }
        None
    }

    pub fn first_prefix(&self) -> &str {
        self.bot.prefixes.first().map(String::as_str).unwrap_or("!")
    }

    pub fn prefixes_display(&self) -> String {
        self.bot.prefixes.iter().map(|p| format!("`{p}`")).collect::<Vec<_>>().join(", ")
    }
}

impl OwnerConfig {
    /// Cek apakah sender adalah owner.
    ///
    /// Mendukung dua format sender JID:
    ///   - `6285691464024@s.whatsapp.net` → bandingkan dengan `number`
    ///   - `227835566395639@lid`          → bandingkan dengan `lid`
    pub fn is_owner(&self, sender_jid: &str) -> bool {
        // Ambil bagian sebelum "@"
        let id_part = sender_jid.split('@').next().unwrap_or(sender_jid);

        // Cek nomor HP
        if !self.number.is_empty() && id_part == self.number.as_str() {
            return true;
        }

        // Cek LID
        if !self.lid.is_empty() && id_part == self.lid.as_str() {
            return true;
        }

        false
    }
}
