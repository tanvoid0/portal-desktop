//! Read-only host readouts: volume capacity (Dashboard, System page) and CPU /
//! memory / GPU / network / process figures. Deletes nothing — purely informational.

use crate::process_ext::NoWindowExt;
use serde::Serialize;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use sysinfo::{DiskKind, Disks, Networks, ProcessRefreshKind, System};
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
    /// Drive temperature. `None` when the drive or OS won't say without admin rights.
    pub temperature_c: Option<i16>,
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
            // SSDs only: a spinning disk takes seconds to answer, on a 30s poll.
            temperature_c: (d.kind() == DiskKind::SSD)
                .then(|| drive_temperature(&mount))
                .flatten(),
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

/// `STORAGE_TEMPERATURE_DATA_DESCRIPTOR`: sensor count at byte 12, first sensor's reading at 26.
#[cfg(any(windows, test))]
fn parse_drive_temperature(descriptor: &[u8]) -> Option<i16> {
    let word = |at: usize| Some([*descriptor.get(at)?, *descriptor.get(at + 1)?]);
    if u16::from_le_bytes(word(12)?) == 0 {
        return None;
    }
    let celsius = i16::from_le_bytes(word(26)?);
    // Firmware that has no sensor tends to answer 0 or 0x8000.
    (1..=150).contains(&celsius).then_some(celsius)
}

/// `IOCTL_STORAGE_QUERY_PROPERTY` on a handle with no access rights, which needs no elevation.
#[cfg(windows)]
fn drive_temperature(mount: &str) -> Option<i16> {
    use std::os::windows::fs::OpenOptionsExt;
    use std::os::windows::io::{AsRawHandle, RawHandle};

    // One call, declared here rather than pulling a bindings crate into portal-core.
    #[link(name = "kernel32")]
    extern "system" {
        fn DeviceIoControl(
            device: RawHandle,
            code: u32,
            in_buffer: *const u8,
            in_size: u32,
            out_buffer: *mut u8,
            out_size: u32,
            returned: *mut u32,
            overlapped: *mut std::ffi::c_void,
        ) -> i32;
    }
    const IOCTL_STORAGE_QUERY_PROPERTY: u32 = 0x002D_1400;
    const STORAGE_DEVICE_TEMPERATURE_PROPERTY: u32 = 52;

    // Drive letters only: `C:\` -> `\\.\C:`.
    let letter = mount.trim_end_matches('\\');
    if letter.len() != 2 || !letter.ends_with(':') {
        return None;
    }
    let device = std::fs::OpenOptions::new()
        .access_mode(0)
        .open(format!(r"\\.\{letter}"))
        .ok()?;
    // STORAGE_PROPERTY_QUERY { PropertyId, QueryType: PropertyStandardQuery, .. }
    let mut query = [0u8; 12];
    query[..4].copy_from_slice(&STORAGE_DEVICE_TEMPERATURE_PROPERTY.to_le_bytes());
    let mut out = [0u8; 512];
    let mut returned = 0u32;
    // SAFETY: both buffers outlive this synchronous call and their true sizes are passed.
    let ok = unsafe {
        DeviceIoControl(
            device.as_raw_handle(),
            IOCTL_STORAGE_QUERY_PROPERTY,
            query.as_ptr(),
            query.len() as u32,
            out.as_mut_ptr(),
            out.len() as u32,
            &mut returned,
            std::ptr::null_mut(),
        )
    };
    if ok == 0 {
        return None;
    }
    parse_drive_temperature(&out[..(returned as usize).min(out.len())])
}

#[cfg(not(windows))]
fn drive_temperature(_mount: &str) -> Option<i16> {
    None
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
    /// The rest of what that GPU reports.
    pub gpu: Option<GpuStats>,
}

#[derive(Serialize, Clone, Debug, PartialEq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/disk/generated/")]
pub struct GpuStats {
    pub name: String,
    pub temperature_c: Option<f32>,
    #[ts(type = "number | null")]
    pub memory_used_bytes: Option<u64>,
    #[ts(type = "number | null")]
    pub memory_total_bytes: Option<u64>,
    pub power_watts: Option<f32>,
    pub fan_percent: Option<f32>,
}

/// CPU load is a delta between refreshes, so the `System` is kept alive across
/// calls — the first call after startup reports 0 until the second poll.
fn host_system() -> &'static Mutex<System> {
    static SYSTEM: OnceLock<Mutex<System>> = OnceLock::new();
    SYSTEM.get_or_init(|| Mutex::new(System::new()))
}

/// ponytail: NVIDIA only, one `nvidia-smi` spawn per poll. AMD/Intel read as `None` and the UI
/// leaves the GPU out; Windows' "GPU Engine" performance counters are the vendor-neutral upgrade.
async fn gpu_reading() -> GpuReading {
    let mut cmd = tokio::process::Command::new("nvidia-smi");
    cmd.args([
        "--query-gpu=name,utilization.gpu,temperature.gpu,memory.used,memory.total,power.draw,fan.speed",
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
    parse_gpu(&String::from_utf8_lossy(&out.stdout))
}

type GpuReading = Option<(Option<f32>, GpuStats)>;

/// Several pollers (this window, paired devices) share one spawn. Shorter than the 2s poll, so a
/// lone poller always gets a new reading.
async fn shared_gpu_reading() -> GpuReading {
    static LAST: tokio::sync::Mutex<Option<(Instant, GpuReading)>> =
        tokio::sync::Mutex::const_new(None);
    let mut last = LAST.lock().await;
    if let Some((at, reading)) = &*last {
        if at.elapsed() < Duration::from_millis(1500) {
            return reading.clone();
        }
    }
    let reading = gpu_reading().await;
    *last = Some((Instant::now(), reading.clone()));
    reading
}

/// One line per GPU, the busiest wins. Each `[N/A]` field is `None` on its own.
fn parse_gpu(csv: &str) -> GpuReading {
    csv.lines()
        .filter_map(|line| {
            let f: Vec<&str> = line.split(',').map(str::trim).collect();
            let [name, load, temp, used, total, power, fan] = f[..] else {
                return None;
            };
            let num = |s: &str| s.parse::<f32>().ok();
            let mib = |s: &str| s.parse::<u64>().ok().map(|v| v * 1024 * 1024);
            let stats = GpuStats {
                name: name.to_string(),
                temperature_c: num(temp),
                memory_used_bytes: mib(used),
                memory_total_bytes: mib(total),
                power_watts: num(power),
                fan_percent: num(fan),
            };
            Some((num(load), stats))
        })
        .reduce(|a, b| if b.0 > a.0 { b } else { a })
}

pub async fn host_stats() -> HostStats {
    // Before the lock: a std mutex guard must not live across an await.
    let (gpu_percent, gpu) = match shared_gpu_reading().await {
        Some((load, stats)) => (load, Some(stats)),
        None => (None, None),
    };
    let mut sys = host_system().lock().unwrap_or_else(|e| e.into_inner());
    sys.refresh_cpu();
    sys.refresh_memory();
    HostStats {
        cpu_percent: sys.global_cpu_info().cpu_usage(),
        memory_used_bytes: sys.used_memory(),
        memory_total_bytes: sys.total_memory(),
        gpu_percent,
        gpu,
    }
}

#[derive(Serialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/disk/generated/")]
pub struct ProcessLoad {
    pub name: String,
    /// Share of the whole machine, 0-100.
    pub cpu_percent: f32,
    #[ts(type = "number")]
    pub memory_bytes: u64,
}

/// What only the System page needs, kept out of the app-wide `host_stats` poll.
#[derive(Serialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/disk/generated/")]
pub struct SystemDetails {
    pub cpu_name: String,
    pub os_name: String,
    #[ts(type = "number")]
    pub uptime_secs: u64,
    /// `false` on the first call, or the first after a pause: every rate below needs two recent
    /// samples.
    pub measured: bool,
    pub core_percents: Vec<f32>,
    pub net_down_bytes_per_sec: f64,
    pub net_up_bytes_per_sec: f64,
    /// The five busiest processes.
    pub processes: Vec<ProcessLoad>,
}

struct DetailSampler {
    system: System,
    networks: Networks,
    last: Option<(Instant, SystemDetails)>,
}

/// A reading younger than this is handed out again, so extra pollers don't shorten the window.
const REUSE_FOR: Duration = Duration::from_secs(1);
/// Beyond this the previous sample is too old to call a difference from it a current rate.
const STALE_AFTER: Duration = Duration::from_secs(10);

/// Its own `System`, so this command and `host_stats` never shorten each other's CPU interval.
/// Network rates divide by the real elapsed time.
pub fn system_details() -> SystemDetails {
    static SAMPLER: OnceLock<Mutex<DetailSampler>> = OnceLock::new();
    let mut s = SAMPLER
        .get_or_init(|| {
            Mutex::new(DetailSampler {
                system: System::new(),
                networks: Networks::new_with_refreshed_list(),
                last: None,
            })
        })
        .lock()
        .unwrap_or_else(|e| e.into_inner());

    if let Some((at, details)) = &s.last {
        if at.elapsed() < REUSE_FOR {
            return details.clone();
        }
    }

    s.system.refresh_cpu();
    s.system
        .refresh_processes_specifics(ProcessRefreshKind::new().with_cpu().with_memory());
    // Not `refresh()`: this also picks up adapters that came up since, and updates the counters
    // of the known ones. Calling both would zero every delta.
    s.networks.refresh_list();
    let sampled_at = Instant::now();
    let seconds = s
        .last
        .as_ref()
        .map(|(at, _)| sampled_at.duration_since(*at))
        .filter(|elapsed| *elapsed <= STALE_AFTER)
        .map(|elapsed| elapsed.as_secs_f64());
    let (down, up) = s.networks.iter().fold((0u64, 0u64), |(d, u), (_, n)| {
        (d + n.received(), u + n.transmitted())
    });
    let rate = |bytes: u64| seconds.map_or(0.0, |s| bytes as f64 / s);

    let cores = s.system.cpus().len().max(1) as f32;
    let mut processes: Vec<ProcessLoad> = s
        .system
        .processes()
        .iter()
        // Pid 0 is Windows' idle process.
        .filter(|(pid, _)| pid.as_u32() != 0)
        .map(|(_, p)| ProcessLoad {
            name: p.name().to_string(),
            cpu_percent: p.cpu_usage() / cores,
            memory_bytes: p.memory(),
        })
        .collect();
    processes.sort_by(|a, b| {
        b.cpu_percent
            .total_cmp(&a.cpu_percent)
            .then(b.memory_bytes.cmp(&a.memory_bytes))
    });
    processes.truncate(5);

    let details = SystemDetails {
        cpu_name: s
            .system
            .cpus()
            .first()
            .map(|c| c.brand().trim().to_string())
            .unwrap_or_default(),
        os_name: System::long_os_version().unwrap_or_default(),
        uptime_secs: System::uptime(),
        measured: seconds.is_some(),
        core_percents: s.system.cpus().iter().map(|c| c.cpu_usage()).collect(),
        net_down_bytes_per_sec: rate(down),
        net_up_bytes_per_sec: rate(up),
        processes,
    };
    s.last = Some((sampled_at, details.clone()));
    details
}

#[cfg(test)]
mod tests {
    use super::{parse_drive_temperature, parse_gpu};

    #[test]
    fn gpu_takes_the_busiest_gpu_and_tolerates_unavailable_fields() {
        let (load, gpu) =
            parse_gpu("A, [N/A], 40, 1, 2, [N/A], [N/A]\r\nB, 80, 29, 3896, 16303, 22.69, 0\r\n")
                .unwrap();
        assert_eq!(load, Some(80.0));
        assert_eq!(gpu.name, "B");
        assert_eq!(gpu.temperature_c, Some(29.0));
        assert_eq!(gpu.memory_used_bytes, Some(3896 * 1024 * 1024));
        assert_eq!(gpu.fan_percent, Some(0.0));
        let (load, gpu) = parse_gpu("A, [N/A], 40, 1, 2, [N/A], [N/A]\n").unwrap();
        assert_eq!((load, gpu.power_watts), (None, None));
        assert_eq!(parse_gpu("No devices were found\n"), None);
        assert_eq!(parse_gpu(""), None);
    }

    #[test]
    fn drive_temperature_reads_the_first_sensor() {
        let mut d = [0u8; 40];
        d[12] = 1;
        d[26] = 41;
        assert_eq!(parse_drive_temperature(&d), Some(41));
        d[26] = 0;
        assert_eq!(parse_drive_temperature(&d), None);
        d[12] = 0;
        d[26] = 41;
        assert_eq!(parse_drive_temperature(&d), None);
        assert_eq!(parse_drive_temperature(&d[..8]), None);
    }

    /// Real hardware readout: `cargo test live_readings -- --ignored --nocapture`.
    #[tokio::test]
    #[ignore]
    async fn live_readings() {
        for d in super::disk_usage() {
            println!("{} {:?} C", d.mount_point, d.temperature_c);
        }
        println!("{:?}", super::host_stats().await.gpu);
        super::system_details();
        std::thread::sleep(std::time::Duration::from_secs(2));
        let s = super::system_details();
        println!("{} | {} | up {}s", s.cpu_name, s.os_name, s.uptime_secs);
        println!(
            "cores {} down {:.0} up {:.0}",
            s.core_percents.len(),
            s.net_down_bytes_per_sec,
            s.net_up_bytes_per_sec
        );
        for p in s.processes {
            println!("{} {:.1}% {}", p.name, p.cpu_percent, p.memory_bytes);
        }
    }
}
