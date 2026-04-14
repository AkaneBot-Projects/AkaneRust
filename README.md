# AkaneBot 🤖

WhatsApp Bot berbasis Rust menggunakan [`whatsapp-rust`](https://github.com/jlucaso1/whatsapp-rust).

---

## 📦 Tech Stack

| Crate | Versi | Fungsi |
|---|---|---|
| `whatsapp-rust` | `0.5` | WhatsApp Web client |
| `whatsapp-rust-sqlite-storage` | `0.5` | Simpan sesi |
| `whatsapp-rust-tokio-transport` | `0.5` | WebSocket transport |
| `whatsapp-rust-ureq-http-client` | `0.5` | HTTP client |
| `sysinfo` | `0.30` | Info RAM/CPU/OS |
| `serde` + `toml` | `1` / `0.8` | Config TOML |
| `tokio` | `1` | Async runtime |
| `anyhow` | `1` | Error handling |

---

## ⚙️ Setup

### 1. Edit config.toml

```toml
[bot]
name   = "AkaneBot"
prefix = "!"
db_path = "whatsapp.db"

[auth]
# Nomor HP format: kode negara + nomor tanpa 0
# Contoh: 62 + 81234567890 → 6281234567890
phone_number = "6281234567890"
```

### 2. Jalankan

```bash
cargo run
```

### 3. Login via Pair Code

Kode 8 digit akan muncul di terminal:

```
╔══════════════════════════════════╗
║   KODE PAIR:    XXXX-XXXX       ║
╚══════════════════════════════════╝
```

Di HP:
1. Buka WhatsApp
2. Settings → Linked Devices → Link a Device
3. Pilih **Tautkan dengan nomor telepon**
4. Masukkan kode dari terminal

---

## 🎮 Perintah

| Command | Deskripsi |
|---|---|
| `!ping` | Cek bot aktif → `pong 🏓` |
| `!help` / `!menu` | Tampilkan semua perintah |
| `!info` | Info RAM, CPU, OS server |
| `!runtime` | Lama bot berjalan |
| `!id` | Lihat JID kamu & chat |
| `!<lain>` | "Command tidak dikenal" |
| Teks biasa | Diabaikan |

---

## 📁 Struktur

```
src/
├── main.rs              # Entry point + bot builder
├── config.rs            # Load & parse config.toml
└── bot/
    ├── mod.rs
    ├── state.rs         # Shared state (config, start_time)
    ├── client.rs        # Helper send_text()
    ├── handler.rs       # Event dispatcher
    └── commands/
        ├── mod.rs       # Router + CommandContext
        ├── ping.rs
        ├── help.rs
        ├── info.rs      # sysinfo: RAM/CPU/OS
        ├── runtime.rs   # uptime bot
        ├── id.rs        # JID info
        └── unknown.rs
```

---

⚠️ **Disclaimer:** `whatsapp-rust` adalah implementasi tidak resmi.
Penggunaan bisa melanggar ToS Meta/WhatsApp. Gunakan dengan risiko sendiri.
