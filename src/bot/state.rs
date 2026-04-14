use std::time::Instant;
use crate::config::Config;

pub struct AppState {
    pub config:   Config,
    pub start_at: Instant,
}

impl AppState {
    pub fn new(config: Config) -> Self { Self { config, start_at: Instant::now() } }

    pub fn uptime_str(&self) -> String {
        let total = self.start_at.elapsed().as_secs();
        let days  = total / 86400;
        let hours = (total % 86400) / 3600;
        let mins  = (total % 3600) / 60;
        let secs  = total % 60;
        if days > 0 { format!("{days} hari {hours}j {mins}m {secs}d") }
        else { format!("{hours}j {mins}m {secs}d") }
    }
}
