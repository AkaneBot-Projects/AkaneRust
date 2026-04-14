mod bot;
mod config;
mod downloader;

use anyhow::Result;
use std::sync::Arc;
use whatsapp_rust::bot::Bot;
use whatsapp_rust::pair_code::{PairCodeOptions, PlatformId};
use whatsapp_rust::store::SqliteStore;
use whatsapp_rust::TokioRuntime;
use whatsapp_rust_tokio_transport::TokioWebSocketTransportFactory;
use whatsapp_rust_ureq_http_client::UreqHttpClient;

use bot::handler::handle_event;
use bot::state::AppState;

#[tokio::main]
async fn main() -> Result<()> {
    println!("===================================");
    println!("  AkaneBot v0.1.0");
    println!("===================================");
    println!();

    let cfg = config::Config::load()?;
    println!("[ok] Config loaded");
    println!("     name   : {}", cfg.bot.name);
    println!("     prefix : {}", cfg.prefixes_display());
    println!("     phone  : {}", cfg.auth.phone_number);
    println!("     db     : {}", cfg.bot.db_path);
    println!();

    let state   = Arc::new(AppState::new(cfg.clone()));
    let backend = Arc::new(
        SqliteStore::new(&cfg.bot.db_path).await
            .map_err(|e| anyhow::anyhow!("SQLite: {e}"))?
    );
    println!("[ok] Storage: {}", cfg.bot.db_path);

    let state_clone = Arc::clone(&state);
    let mut bot = Bot::builder()
        .with_backend(backend)
        .with_transport_factory(TokioWebSocketTransportFactory::new())
        .with_http_client(UreqHttpClient::new())
        .with_runtime(TokioRuntime)
        .with_pair_code(PairCodeOptions {
            phone_number:           cfg.auth.phone_number.clone(),
            show_push_notification: true,
            custom_code:            None,
            platform_id:            PlatformId::Chrome,
            platform_display:       "Chrome (Linux)".to_string(),
        })
        .on_event(move |event, client| {
            let state = Arc::clone(&state_clone);
            async move { handle_event(event, client, state).await }
        })
        .build().await
        .map_err(|e| anyhow::anyhow!("Bot build: {e}"))?;

    println!("[ok] Bot ready, waiting for pair code...");
    println!();
    println!("  1. Open WhatsApp on phone");
    println!("  2. Settings -> Linked Devices -> Link a Device");
    println!("  3. Link with phone number");
    println!("  4. Enter the 8-digit code shown above");
    println!();

    bot.run().await
        .map_err(|e| anyhow::anyhow!("{e}"))?
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(())
}
