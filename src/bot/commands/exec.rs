use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

use super::CommandContext;

const TIMEOUT_SECS: u64 = 10;
const MAX_OUTPUT:   usize = 3000;

pub async fn execute(ctx: &CommandContext<'_>) -> String {
    if !ctx.state.config.owner.is_owner(&ctx.sender) {
        return "Command `$` hanya untuk owner.".to_string();
    }

    let cmd = ctx.text.trim().strip_prefix("$ ").unwrap_or("").trim();
    if cmd.is_empty() {
        return "Usage: `$ <command>`\n\nContoh: `$ ls -la`, `$ df -h`".to_string();
    }

    println!("[exec] {cmd}");

    let shell = find_shell();
    let run   = timeout(
        Duration::from_secs(TIMEOUT_SECS),
        Command::new(&shell)
            .arg("-c")
            .arg(cmd)
            .env("PATH", build_path_env())
            .output(),
    ).await;

    match run {
        Err(_)       => format!("timeout (>{TIMEOUT_SECS}s)"),
        Ok(Err(e))   => format!("spawn error: {e}"),
        Ok(Ok(out))  => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            let code   = out.status.code().unwrap_or(-1);

            let mut output = stdout;
            if !stderr.is_empty() {
                if !output.is_empty() { output.push('\n'); }
                output.push_str(&stderr);
            }

            let output = if output.trim().is_empty() { "(no output)".to_string() }
                         else { truncate(output.trim(), MAX_OUTPUT) };

            let status = if code == 0 { "".to_string() } else { format!("[exit {code}] ") };
            format!("{status}```\n{output}\n```")
        }
    }
}

fn find_shell() -> String {
    for s in &[
        "/data/data/com.termux/files/usr/bin/bash",
        "/usr/bin/bash", "/bin/bash", "/bin/sh",
    ] {
        if std::path::Path::new(s).exists() { return s.to_string(); }
    }
    "sh".to_string()
}

fn build_path_env() -> String {
    let current = std::env::var("PATH").unwrap_or_default();
    let home    = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
    [
        format!("{home}/.cargo/bin"),
        "/data/data/com.termux/files/home/.cargo/bin".into(),
        "/data/data/com.termux/files/usr/bin".into(),
        "/usr/local/bin".into(), "/usr/bin".into(), "/bin".into(),
        current,
    ].join(":")
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max { return s.to_string(); }
    format!("{}...\n({} chars total, truncated)", s.chars().take(max).collect::<String>(), s.len())
}
