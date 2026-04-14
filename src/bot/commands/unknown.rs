use super::CommandContext;
pub fn execute(cmd: &str, ctx: &CommandContext<'_>) -> String {
    let p = ctx.state.config.first_prefix();
    format!("Command tidak dikenal: `{cmd}`\nKetik `{p}help` untuk daftar perintah.")
}
