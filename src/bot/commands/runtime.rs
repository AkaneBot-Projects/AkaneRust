use super::CommandContext;
pub fn execute(ctx: &CommandContext<'_>) -> String {
    format!("Uptime: *{}*", ctx.state.uptime_str())
}
