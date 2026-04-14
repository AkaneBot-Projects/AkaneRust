// =============================================================================
//  commands/owner.rs — .owner
// =============================================================================
//
//  Reply berupa teks biasa. Handler khusus akan kirim contact card.
//  Lihat bot/client.rs :: send_contact()
//
// =============================================================================

use super::CommandContext;

/// Return (nama, nomor) untuk dikirim sebagai contact card
pub fn contact_info(ctx: &CommandContext<'_>) -> (String, String) {
    let o = &ctx.state.config.owner;
    (o.name.clone(), o.number.clone())
}

pub fn execute(ctx: &CommandContext<'_>) -> String {
    // Ini tidak dipanggil langsung — .owner di-handle sebagai ContactReply
    // tapi tetap ada sebagai fallback teks
    let o = &ctx.state.config.owner;
    format!("*Owner*\n━━━━━━━━━━━━━━━━━━━━━━━━\nNama   : {}\nNomor  : {}", o.name, o.number)
}
