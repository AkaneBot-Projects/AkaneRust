use super::CommandContext;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

pub fn execute(_ctx: &CommandContext<'_>) -> String {
    let mut sys = System::new_with_specifics(
        RefreshKind::nothing()
            .with_memory(MemoryRefreshKind::everything())
            .with_cpu(CpuRefreshKind::everything()),
    );
    sys.refresh_cpu_usage();
    std::thread::sleep(std::time::Duration::from_millis(200));
    sys.refresh_cpu_usage();
    sys.refresh_memory();

    let total_mb  = sys.total_memory() / 1024 / 1024;
    let used_mb   = sys.used_memory()  / 1024 / 1024;
    let free_mb   = total_mb.saturating_sub(used_mb);
    let ram_pct   = if total_mb > 0 { (used_mb * 100) / total_mb } else { 0 };
    let cpu_count = sys.cpus().len();
    let cpu_usage = if cpu_count > 0 {
        sys.cpus().iter().map(|c| c.cpu_usage()).sum::<f32>() / cpu_count as f32
    } else { 0.0 };
    let cpu_name = sys.cpus().first().map(|c| c.brand()).unwrap_or("Unknown");
    let os_name  = System::name().unwrap_or_else(|| "Unknown".into());
    let os_ver   = System::os_version().unwrap_or_else(|| "".into());
    let hostname = System::host_name().unwrap_or_else(|| "Unknown".into());
    let kernel   = System::kernel_version().unwrap_or_else(|| "Unknown".into());

    format!(
        "*System Info*\n━━━━━━━━━━━━━━━━━━━━━━━━\n\n\
         OS\n\
         Sistem    : {os_name} {os_ver}\n\
         Kernel    : {kernel}\n\
         Hostname  : {hostname}\n\n\
         CPU\n\
         Model     : {cpu_name}\n\
         Core      : {cpu_count}\n\
         Usage     : {cpu_usage:.1}%\n\n\
         RAM\n\
         Total     : {total_mb} MB\n\
         Used      : {used_mb} MB ({ram_pct}%)\n\
         Free      : {free_mb} MB"
    )
}
