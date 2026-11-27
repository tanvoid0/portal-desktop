//! Read-only disk capacity readout for the Dashboard. Enumerates mounted
//! volumes and reports total / available bytes. Touches no file contents,
//! deletes nothing — purely informational.

use crate::process_ext::NoWindowExt;
use serde::Serialize;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;
use sysinfo::{Disks, System};
use ts_rs::TS;

#[derive(Serialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/disk/generated/")]
pub struct DiskUsage {
    /// Mount point / drive root, e.g. "C:\\" or "/".
    pub mount_point: String,
    /// Volume label or device name (may be empty).
    pub name: String,
    /// Filesystem, e.g. "NTFS" | "ext4".
    pub fs_kind: String,
    #[ts(type = "number")]
    pub total_bytes: u64,
    #[ts(type = "number")]
    pub available_bytes: u64,
    pub is_removable: bool,
}

/// Snapshot of every mounted volume's capacity. De-duplicated by mount point
/// (some platforms list the same mount twice).
pub fn disk_usage() -> Vec<DiskUsage> {
    let disks = Disks::new_with_refreshed_list();
    let mut out: Vec<DiskUsage> = Vec::new();

    for d in &disks {
        let mount = d.mount_point().to_string_lossy().into_owned();
        if out.iter().any(|u| u.mount_point == mount) {
            continue;
        }
        out.push(DiskUsage {
            mount_point: mount,
            name: d.name().to_string_lossy().into_owned(),
            fs_kind: d.file_system().to_string_lossy().into_owned(),
            total_bytes: d.total_space(),
            available_bytes: d.available_space(),
            is_removable: d.is_removable(),
        });
    }

    // Biggest volumes first — the ones worth cleaning.
    out.sort_by(|a, b| b.total_bytes.cmp(&a.total_bytes));
    out
}

/// Host CPU / memory / GPU snapshot for the sidebar status readout. Read-only.
#[derive(Serialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/disk/generated/")]
pub struct HostStats {
    /// Whole-machine CPU load, 0-100.
    pub cpu_percent: f32,
    #[ts(type = "number")]
    pub memory_used_bytes: u64,
    #[ts(type = "number")]
    pub memory_total_bytes: u64,
    /// GPU load, 0-100. `None` when nothing on this machine can report it.
    pub gpu_percent: Option<f32>,
}

/// CPU load is a delta between refreshes, so the `System` is kept alive across
/// calls — the first call after startup reports 0 until the second poll.
fn host_system() -> &'static Mutex<System> {
    static SYSTEM: OnceLock<Mutex<System>> = OnceLock::new();
    SYSTEM.get_or_init(|| Mutex::new(System::new()))
}

/// ponytail: NVIDIA only, one `nvidia-smi` spawn per poll. AMD/Intel read as `None` and the UI
/// leaves the GPU out; Windows' "GPU Engine" performance counters are the vendor-neutral upgrade.
async fn gpu_percent() -> Option<f32> {
    let mut cmd = tokio::process::Command::new("nvidia-smi");
    cmd.args([
        "--query-gpu=utilization.gpu",
        "--format=csv,noheader,nounits",
    ])
    .no_window()
    .kill_on_drop(true);
    // A wedged driver can hang nvidia-smi; the other readings must still arrive.
    let out = tokio::time::timeout(Duration::from_secs(2), cmd.output())
        .await
        .ok()?
        .ok()?;
    if !out.status.success() {
        return None;
    }
    parse_gpu_percent(&String::from_utf8_lossy(&out.stdout))
}

/// One line per GPU, the busiest wins. `[N/A]` lines are skipped.
fn parse_gpu_percent(csv: &str) -> Option<f32> {
    csv.lines()
        .filter_map(|line| line.trim().parse::<f32>().ok())
        .reduce(f32::max)
}

pub async fn host_stats() -> HostStats {
    // Before the lock: a std mutex guard must not live across an await.
    let gpu_percent = gpu_percent().await;
    let mut sys = host_system().lock().unwrap_or_else(|e| e.into_inner());
    sys.refresh_cpu();
    sys.refresh_memory();
    HostStats {
        cpu_percent: sys.global_cpu_info().cpu_usage(),
        memory_used_bytes: sys.used_memory(),
        memory_total_bytes: sys.total_memory(),
        gpu_percent,
    }
}

#[cfg(test)]
mod tests {
    use super::parse_gpu_percent;

    #[test]
    fn gpu_percent_takes_the_busiest_readable_gpu() {
        assert_eq!(parse_gpu_percent("5\r\n"), Some(5.0));
        assert_eq!(parse_gpu_percent("[N/A]\n12\n80\n"), Some(80.0));
        assert_eq!(parse_gpu_percent("[N/A]\n"), None);
        assert_eq!(parse_gpu_percent(""), None);
    }
}
