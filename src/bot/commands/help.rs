use super::CommandContext;
pub fn execute(ctx: &CommandContext<'_>) -> String {
    let name     = &ctx.state.config.bot.name;
    let prefixes = ctx.state.config.prefixes_display();
    let p        = ctx.state.config.first_prefix();
    format!(
        "*{name}*\n━━━━━━━━━━━━━━━━━━━━━━━━\n\n\
         *Umum*\n\
         • `{p}ping`       - Cek bot aktif\n\
         • `{p}help`       - Menu ini\n\n\
         *System*\n\
         • `{p}info`       - RAM, CPU, OS\n\
         • `{p}runtime`    - Uptime bot\n\n\
         *Identitas*\n\
         • `{p}id`         - Info JID\n\
         • `{p}owner`      - Info owner\n\
         • `{p}about`      - Tentang bot\n\n\
         *Downloader*\n\
         • `{p}tt <url>`   - TikTok / FB / IG\n\n\
         *Owner*\n\
         • `>> <code>`     - Eval Rust (sync)\n\
         • `=> <code>`     - Eval Rust (async)\n\
         • `$ <cmd>`       - Shell execute\n\n\
         ─────────────────────────\n\
         Prefix: {prefixes}"
    )
}
