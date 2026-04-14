use super::CommandContext;

pub fn execute(ctx: &CommandContext<'_>) -> String {
    let sender    = &ctx.sender;
    let chat      = &ctx.chat;
    let chat_type = if chat.ends_with("@g.us") { "Group" } else { "Personal" };
    format!(
        "*JID Info*\n━━━━━━━━━━━━━━━━━━━━━━━━\n\
         Sender : `{sender}`\n\
         Chat   : `{chat}` ({chat_type})"
    )
}
