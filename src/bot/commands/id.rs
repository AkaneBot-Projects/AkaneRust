use super::CommandContext;

pub fn execute(ctx: &CommandContext<'_>) -> String {
    let sender    = &ctx.sender;
    let chat      = &ctx.chat;
    let chat_type = if chat.ends_with("@g.us") { "Group" } else { "Personal" };

    // Deteksi format JID
    let sender_note = if sender.contains("@lid") {
        let lid_num = sender.split('@').next().unwrap_or(sender);
        format!(
            "`{sender}`\n  (format LID — set config.toml: owner.lid = \"{lid_num}\")"
        )
    } else {
        format!("`{sender}`")
    };

    format!(
        "*JID Info*\n━━━━━━━━━━━━━━━━━━━━━━━━\n\
         Sender : {sender_note}\n\
         Chat   : `{chat}` ({chat_type})"
    )
}
