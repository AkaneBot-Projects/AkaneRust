use std::path::PathBuf;
use std::time::Duration;
use tokio::fs;
use tokio::process::Command;
use tokio::time::timeout;

use super::CommandContext;

#[derive(Debug, Clone, PartialEq)]
pub enum EvalMode { Sync, Async }

const COMPILE_TIMEOUT: u64 = 25;
const RUN_TIMEOUT:     u64 = 5;

pub async fn execute(ctx: &CommandContext<'_>, mode: EvalMode) -> String {
    if !ctx.state.config.owner.is_owner(&ctx.sender) {
        return "Command ini hanya untuk owner.".to_string();
    }

    let rustc_path = match find_rustc() {
        Some(p) => p,
        None    => return "`rustc` tidak ditemukan di sistem.".to_string(),
    };

    let code = extract_code(ctx.text).trim().to_string();
    if code.is_empty() {
        let p = ctx.state.config.first_prefix();
        return format!(
            "*Eval Rust*\n\
             `>> <expr>`      - eval, auto-print hasil\n\
             `=> <code>`      - async eval\n\
             `{p}eval <code>` - sama dengan `>>`\n\n\
             Contoh:\n\
             `>> 2 + 2`\n\
             `>> \"hello\".repeat(3)`\n\
             `>> vec![1,2,3].iter().sum::<i32>()`"
        );
    }

    match run_eval(&code, &mode, &rustc_path).await {
        Ok(out) => {
            let out = out.trim().to_string();
            let display = if out.is_empty() { "(no output)".to_string() } else { truncate(&out, 1500) };
            format!("```\n{display}\n```")
        }
        Err(e) => format!("```\n{}\n```", truncate(&e, 1500)),
    }
}

fn extract_code(text: &str) -> &str {
    let t = text.trim();
    for pfx in &[">> ", "=> ", ".eval ", "!eval ", "eval "] {
        if let Some(r) = t.strip_prefix(pfx) { return r; }
    }
    if t == "eval" || t == ">>" || t == "=>" { return ""; }
    t
}

fn wrap_code(code: &str, mode: &EvalMode, auto_print: bool) -> String {
    let code = code.trim();
    if code.contains("fn main(") || code.contains("fn main ") {
        return code.to_string();
    }
    let t       = code.trim_end();
    let is_expr = !t.ends_with(';')
        && (!t.ends_with('}') || !t.trim_end().strip_suffix('}').unwrap_or("").trim_end().ends_with(';'));
    let allow   = "#[allow(unused_imports, unused_variables, dead_code, unused_must_use)]";

    match mode {
        EvalMode::Sync => {
            if auto_print && is_expr {
                format!("{allow}\nfn main() {{\n    let __r = {{ {code} }};\n    println!(\"{{:?}}\", __r);\n}}")
            } else {
                format!("{allow}\nfn main() {{\n    {code}\n}}")
            }
        }
        EvalMode::Async => {
            if auto_print && is_expr {
                format!("{allow}\n#[tokio::main]\nasync fn main() {{\n    let __r = {{ {code} }};\n    println!(\"{{:?}}\", __r);\n}}")
            } else {
                format!("{allow}\n#[tokio::main]\nasync fn main() {{\n    {code}\n}}")
            }
        }
    }
}

async fn run_eval(code: &str, mode: &EvalMode, rustc: &str) -> Result<String, String> {
    if *mode == EvalMode::Async { return run_async_eval(code).await; }
    // Phase 1: auto-print
    if let Ok(out) = compile_and_run(&wrap_code(code, mode, true), rustc).await { return Ok(out); }
    // Phase 2: statement
    compile_and_run(&wrap_code(code, mode, false), rustc).await
}

async fn compile_and_run(src: &str, rustc: &str) -> Result<String, String> {
    let tmp  = std::env::temp_dir();
    let ts   = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().subsec_nanos();
    let src_path = tmp.join(format!("akane_{ts}.rs"));
    let bin_path = tmp.join(format!("akane_{ts}"));
    let path_env = build_path_env();

    fs::write(&src_path, src).await.map_err(|e| format!("write: {e}"))?;

    let co = timeout(Duration::from_secs(COMPILE_TIMEOUT),
        Command::new(rustc).arg("--edition=2021").arg("-o").arg(&bin_path).arg(&src_path)
            .env("PATH", &path_env).output()
    ).await;
    let _ = fs::remove_file(&src_path).await;
    let co = co.map_err(|_| format!("compile timeout (>{COMPILE_TIMEOUT}s)"))?.map_err(|e| e.to_string())?;
    if !co.status.success() {
        let _ = fs::remove_file(&bin_path).await;
        return Err(clean_error(&String::from_utf8_lossy(&co.stderr)));
    }

    let ro = timeout(Duration::from_secs(RUN_TIMEOUT),
        Command::new(&bin_path).env("PATH", &path_env).output()
    ).await;
    let _ = fs::remove_file(&bin_path).await;
    let ro = ro.map_err(|_| format!("runtime timeout (>{RUN_TIMEOUT}s)"))?.map_err(|e| e.to_string())?;

    let stdout = String::from_utf8_lossy(&ro.stdout).to_string();
    let stderr = String::from_utf8_lossy(&ro.stderr).to_string();
    if !ro.status.success() {
        return Err(if stderr.is_empty() { format!("exit {}", ro.status.code().unwrap_or(-1)) } else { stderr });
    }
    Ok(if stderr.is_empty() { stdout } else { format!("{stdout}{stderr}") })
}

async fn run_async_eval(code: &str) -> Result<String, String> {
    let tmp  = std::env::temp_dir();
    let ts   = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().subsec_nanos();
    let proj = tmp.join(format!("akane_async_{ts}"));
    let src  = proj.join("src");
    fs::create_dir_all(&src).await.map_err(|e| e.to_string())?;
    fs::write(proj.join("Cargo.toml"),
        "[package]\nname=\"eval\"\nversion=\"0.1.0\"\nedition=\"2021\"\n[dependencies]\ntokio={version=\"1\",features=[\"full\"]}\n"
    ).await.map_err(|e| e.to_string())?;
    let is_expr = !code.trim_end().ends_with(';');
    fs::write(src.join("main.rs"), &wrap_code(code, &EvalMode::Async, is_expr)).await.map_err(|e| e.to_string())?;

    let out = timeout(Duration::from_secs(90),
        Command::new("cargo").args(["run","--release","--quiet"]).current_dir(&proj)
            .env("PATH", build_path_env()).output()
    ).await;
    let _ = fs::remove_dir_all(&proj).await;
    let out = out.map_err(|_| "async timeout (>90s)".to_string())?.map_err(|e| e.to_string())?;
    if !out.status.success() { return Err(clean_error(&String::from_utf8_lossy(&out.stderr))); }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

fn find_rustc() -> Option<String> {
    if let Ok(o) = std::process::Command::new("which").arg("rustc").output() {
        if o.status.success() {
            let p = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if !p.is_empty() { return Some(p); }
        }
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
    for c in &[
        format!("{home}/.cargo/bin/rustc"),
        "/data/data/com.termux/files/home/.cargo/bin/rustc".into(),
        "/usr/local/bin/rustc".into(),
        "/usr/bin/rustc".into(),
    ] {
        if PathBuf::from(c).exists() { return Some(c.clone()); }
    }
    None
}

fn build_path_env() -> String {
    let current = std::env::var("PATH").unwrap_or_default();
    let home    = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
    [
        format!("{home}/.cargo/bin"),
        "/data/data/com.termux/files/home/.cargo/bin".into(),
        "/data/data/com.termux/files/usr/bin".into(),
        "/usr/local/bin".into(),
        "/usr/bin".into(),
        "/bin".into(),
        current,
    ].join(":")
}

fn clean_error(e: &str) -> String {
    e.lines().map(|l| {
        if l.contains("akane_") {
            if let Some(pos) = l.find("akane_") {
                if let Some(end) = l[pos..].find(".rs") {
                    return format!("{}eval.rs{}", &l[..pos], &l[pos+end+3..]);
                }
            }
        }
        l.to_string()
    }).collect::<Vec<_>>().join("\n")
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max { return s.to_string(); }
    format!("{}...\n(truncated)", s.chars().take(max).collect::<String>())
}
