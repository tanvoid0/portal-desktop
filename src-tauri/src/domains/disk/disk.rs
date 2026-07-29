//! Read-only disk capacity readout for the Dashboard. Enumerates mounted
//! volumes and reports total / available bytes. Touches no file contents,
//! deletes nothing — purely informational.

use serde::Serialize;
use std::sync::{Mutex, OnceLock};
use sysinfo::{Disks, System};

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DiskUsage {
    /// Mount point / drive root, e.g. "C:\\" or "/".
    pub mount_point: String,
    /// Volume label or device name (may be empty).
    pub name: String,
    /// Filesystem, e.g. "NTFS" | "ext4".
    pub fs_kind: String,
    pub total_bytes: u64,
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

/// Host CPU / memory snapshot for the sidebar status chips. Read-only.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HostStats {
    /// Whole-machine CPU load, 0-100.
    pub cpu_percent: f32,
    pub memory_used_bytes: u64,
    pub memory_total_bytes: u64,
}

/// CPU load is a delta between refreshes, so the `System` is kept alive across
/// calls — the first call after startup reports 0 until the second poll.
fn host_system() -> &'static Mutex<System> {
    static SYSTEM: OnceLock<Mutex<System>> = OnceLock::new();
    SYSTEM.get_or_init(|| Mutex::new(System::new()))
}

pub fn host_stats() -> HostStats {
    let mut sys = host_system().lock().unwrap_or_else(|e| e.into_inner());
    sys.refresh_cpu();
    sys.refresh_memory();
    HostStats {
        cpu_percent: sys.global_cpu_info().cpu_usage(),
        memory_used_bytes: sys.used_memory(),
        memory_total_bytes: sys.total_memory(),
    }
}
