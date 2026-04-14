use super::CommandContext;
pub fn execute(ctx: &CommandContext<'_>) -> String {
    let name     = &ctx.state.config.bot.name;
    let uptime   = ctx.state.uptime_str();
    let prefixes = ctx.state.config.prefixes_display();
    format!(
        "*{name}*\n━━━━━━━━━━━━━━━━━━━━━━━━\n\
         Nama     : {name}\n\
         Runtime  : Rust\n\
         Library  : whatsapp-rust v0.5\n\
         Uptime   : {uptime}\n\
         Prefix   : {prefixes}"
    )
}
