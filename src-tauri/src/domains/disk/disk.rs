//! Read-only disk capacity readout for the Dashboard. Enumerates mounted
//! volumes and reports total / available bytes. Touches no file contents,
//! deletes nothing — purely informational.

use serde::Serialize;
use sysinfo::Disks;

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
