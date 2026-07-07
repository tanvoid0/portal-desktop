use std::collections::HashSet;
use std::process::Command;

use super::{
    CleanedItem, DevCleaner, DevCleanerGroup, DevCleanerItem, DevCleanerScanContext,
    DevCleanerWalkHooks, FailedCleanItem,
};
use crate::domains::disk::classify::Risk;

/// Shared scanner/cleaner for OCI container runtimes (Docker, Podman).
pub struct ContainerRuntimeCleaner {
    binary: &'static str,
    label: &'static str,
}

impl ContainerRuntimeCleaner {
    pub fn docker() -> Self {
        Self {
            binary: "docker",
            label: "Docker",
        }
    }

    pub fn podman() -> Self {
        Self {
            binary: "podman",
            label: "Podman",
        }
    }
}

impl DevCleaner for ContainerRuntimeCleaner {
    fn id(&self) -> &'static str {
        self.binary
    }

    fn label(&self) -> &'static str {
        self.label
    }

    fn needs_roots(&self) -> bool {
        false
    }

    fn scan(
        &self,
        _ctx: &DevCleanerScanContext,
        _hooks: Option<&DevCleanerWalkHooks<'_>>,
    ) -> Result<DevCleanerGroup, String> {
        Ok(match self.check_available() {
            Ok(()) => {
                let mut items = Vec::new();
                items.extend(self.scan_stopped_containers());
                items.extend(self.scan_dangling_images());
                items.extend(self.scan_unused_images());
                items.extend(self.scan_dangling_volumes());
                items.extend(self.scan_dangling_networks());
                items.extend(self.scan_build_cache());

                items.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
                let total_bytes = items.iter().map(|i| i.size_bytes).sum();
                DevCleanerGroup {
                    cleaner_id: self.id().to_string(),
                    label: self.label().to_string(),
                    available: true,
                    unavailable_reason: None,
                    total_bytes,
                    items,
                }
            }
            Err(reason) => DevCleanerGroup {
                cleaner_id: self.id().to_string(),
                label: self.label().to_string(),
                available: false,
                unavailable_reason: Some(reason),
                total_bytes: 0,
                items: vec![],
            },
        })
    }

    fn clean(&self, items: &[DevCleanerItem]) -> (Vec<CleanedItem>, Vec<FailedCleanItem>) {
        let mut cleaned = Vec::new();
        let mut failed = Vec::new();

        for item in items {
            let result = match item.kind.as_str() {
                "stopped-container" | "created-container" => {
                    self.run(&["rm", "-f", &item.path])
                }
                "dangling-image" | "unused-image" => self.run(&["rmi", "-f", &item.path]),
                "dangling-volume" => self.run(&["volume", "rm", "-f", &item.path]),
                "dangling-network" => self.run(&["network", "rm", &item.path]),
                "build-cache" => self.run(&["builder", "prune", "-af"]),
                other => Err(format!("unsupported kind: {other}")),
            };

            match result {
                Ok(_) => cleaned.push(CleanedItem {
                    path: item.path.clone(),
                    kind: item.kind.clone(),
                    size_bytes: item.size_bytes,
                }),
                Err(e) => failed.push(FailedCleanItem {
                    path: item.path.clone(),
                    kind: item.kind.clone(),
                    error: e,
                }),
            }
        }

        (cleaned, failed)
    }
}

impl ContainerRuntimeCleaner {
    fn check_available(&self) -> Result<(), String> {
        self.run(&["info", "--format", "{{.ID}}"])
            .map(|_| ())
            .map_err(|e| {
                if e.contains("not found") || e.contains("program") {
                    format!("{} is not installed or not on PATH", self.label)
                } else {
                    format!("{} daemon not running: {e}", self.label)
                }
            })
    }

    fn run(&self, args: &[&str]) -> Result<String, String> {
        let out = Command::new(self.binary)
            .args(args)
            .output()
            .map_err(|e| format!("{} not found: {e}", self.binary))?;
        if out.status.success() {
            Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
        } else {
            let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
            let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
            let msg = if stderr.is_empty() { stdout } else { stderr };
            Err(if msg.is_empty() {
                format!("{} {:?} failed", self.binary, args)
            } else {
                msg
            })
        }
    }

    fn item_id(&self, kind: &str, path: &str) -> String {
        format!("{}:{}:{}", self.binary, kind, path)
    }

    fn scan_stopped_containers(&self) -> Vec<DevCleanerItem> {
        let mut items = Vec::new();
        for status in ["exited", "created", "dead"] {
            let Ok(out) = self.run(&[
                "ps",
                "-a",
                "-s",
                "--no-trunc",
                "--filter",
                &format!("status={status}"),
                "--format",
                "{{.ID}}\t{{.Names}}\t{{.Size}}",
            ]) else {
                continue;
            };
            for line in out.lines().filter(|l| !l.trim().is_empty()) {
                let parts: Vec<&str> = line.splitn(3, '\t').collect();
                if parts.len() < 2 {
                    continue;
                }
                let id = parts[0].trim();
                let name = parts[1].trim();
                let size = parts.get(2).map(|s| parse_human_size(s)).unwrap_or(0);
                items.push(DevCleanerItem {
                    id: self.item_id(status, id),
                    cleaner_id: self.id().to_string(),
                    path: id.to_string(),
                    kind: format!("{status}-container"),
                    reason: if name.is_empty() {
                        format!("{status} container — safe to remove")
                    } else {
                        format!("{status} container ({name}) — safe to remove")
                    },
                    size_bytes: size,
                    file_count: 0,
                    risk: Risk::Safe,
                    group_label: None,
                });
            }
        }
        items
    }

    fn scan_dangling_images(&self) -> Vec<DevCleanerItem> {
        let Ok(out) = self.run(&[
            "images",
            "--filter",
            "dangling=true",
            "--format",
            "{{.ID}}\t{{.Repository}}:{{.Tag}}\t{{.Size}}",
        ]) else {
            return vec![];
        };

        out.lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(3, '\t').collect();
                if parts.len() < 2 {
                    return None;
                }
                let id = parts[0].trim();
                let tag = parts[1].trim();
                let size = parts.get(2).map(|s| parse_human_size(s)).unwrap_or(0);
                Some(DevCleanerItem {
                    id: self.item_id("dangling-image", id),
                    cleaner_id: self.id().to_string(),
                    path: id.to_string(),
                    kind: "dangling-image".to_string(),
                    reason: format!("untagged intermediate layer ({tag})"),
                    size_bytes: size,
                    file_count: 0,
                    risk: Risk::Safe,
                    group_label: None,
                })
            })
            .collect()
    }

    fn scan_unused_images(&self) -> Vec<DevCleanerItem> {
        let in_use = self.images_in_use();
        let Ok(out) = self.run(&[
            "images",
            "--format",
            "{{.ID}}\t{{.Repository}}:{{.Tag}}\t{{.Size}}",
        ]) else {
            return vec![];
        };

        out.lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(3, '\t').collect();
                if parts.len() < 2 {
                    return None;
                }
                let id = parts[0].trim();
                let tag = parts[1].trim();
                if tag.contains("<none>") {
                    return None; // dangling — already listed
                }
                if in_use.contains(id) || in_use.contains(tag) {
                    return None;
                }
                let size = parts.get(2).map(|s| parse_human_size(s)).unwrap_or(0);
                Some(DevCleanerItem {
                    id: self.item_id("unused-image", id),
                    cleaner_id: self.id().to_string(),
                    path: id.to_string(),
                    kind: "unused-image".to_string(),
                    reason: format!("not referenced by any container ({tag})"),
                    size_bytes: size,
                    file_count: 0,
                    risk: Risk::Review,
                    group_label: None,
                })
            })
            .collect()
    }

    fn images_in_use(&self) -> HashSet<String> {
        let mut set = HashSet::new();
        if let Ok(out) = self.run(&["ps", "-a", "--format", "{{.Image}}"]) {
            for line in out.lines() {
                let s = line.trim();
                if !s.is_empty() {
                    set.insert(s.to_string());
                }
            }
        }
        set
    }

    fn scan_dangling_volumes(&self) -> Vec<DevCleanerItem> {
        let Ok(out) = self.run(&[
            "volume",
            "ls",
            "--filter",
            "dangling=true",
            "--format",
            "{{.Name}}",
        ]) else {
            return vec![];
        };

        out.lines()
            .filter(|l| !l.trim().is_empty())
            .map(|name| {
                let name = name.trim();
                DevCleanerItem {
                    id: self.item_id("dangling-volume", name),
                    cleaner_id: self.id().to_string(),
                    path: name.to_string(),
                    kind: "dangling-volume".to_string(),
                    reason: "volume not attached to any container — review before removing".to_string(),
                    size_bytes: 0,
                    file_count: 0,
                    risk: Risk::Review,
                    group_label: None,
                }
            })
            .collect()
    }

    fn scan_dangling_networks(&self) -> Vec<DevCleanerItem> {
        let Ok(out) = self.run(&[
            "network",
            "ls",
            "--filter",
            "dangling=true",
            "--format",
            "{{.ID}}\t{{.Name}}",
        ]) else {
            return vec![];
        };

        out.lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(2, '\t').collect();
                if parts.is_empty() {
                    return None;
                }
                let id = parts[0].trim();
                let name = parts.get(1).map(|s| s.trim()).unwrap_or(id);
                Some(DevCleanerItem {
                    id: self.item_id("dangling-network", id),
                    cleaner_id: self.id().to_string(),
                    path: id.to_string(),
                    kind: "dangling-network".to_string(),
                    reason: format!("unused network ({name})"),
                    size_bytes: 0,
                    file_count: 0,
                    risk: Risk::Safe,
                    group_label: None,
                })
            })
            .collect()
    }

    fn scan_build_cache(&self) -> Vec<DevCleanerItem> {
        let Ok(out) = self.run(&[
            "system",
            "df",
            "--format",
            "{{.Type}}\t{{.Size}}\t{{.Reclaimable}}",
        ]) else {
            return vec![];
        };

        for line in out.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() < 3 {
                continue;
            }
            let kind = parts[0].trim().to_lowercase();
            if kind.contains("build") && kind.contains("cache") {
                let reclaimable = parse_human_size(parts[2].trim());
                if reclaimable == 0 {
                    return vec![];
                }
                return vec![DevCleanerItem {
                    id: self.item_id("build-cache", "all"),
                    cleaner_id: self.id().to_string(),
                    path: "build-cache".to_string(),
                    kind: "build-cache".to_string(),
                    reason: format!(
                        "reclaimable build cache ({})",
                        parts[2].trim()
                    ),
                    size_bytes: reclaimable,
                    file_count: 0,
                    risk: Risk::Safe,
                    group_label: None,
                }];
            }
        }
        vec![]
    }
}

/// Parses Docker/Podman human-readable sizes like "1.2GB", "500MB", "0B".
fn parse_human_size(s: &str) -> u64 {
    let s = s.trim();
    if s.is_empty() || s == "0" || s == "0B" {
        return 0;
    }
    // Strip parenthetical "(virtual ...)" suffix from container sizes.
    let s = s.split('(').next().unwrap_or(s).trim();
    let (num_str, unit) = s
        .chars()
        .position(|c| c.is_alphabetic())
        .map(|i| (&s[..i], s[i..].trim()))
        .unwrap_or((s, "B"));
    let Ok(mut val) = num_str.trim().parse::<f64>() else {
        return 0;
    };
    let mult: f64 = match unit.to_uppercase().as_str() {
        "B" => 1.0,
        "KB" | "KIB" => 1024.0,
        "MB" | "MIB" => 1024.0_f64.powi(2),
        "GB" | "GIB" => 1024.0_f64.powi(3),
        "TB" | "TIB" => 1024.0_f64.powi(4),
        _ => 1.0,
    };
    val *= mult;
    if val < 0.0 {
        0
    } else {
        val.round() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sizes() {
        assert_eq!(parse_human_size("0B"), 0);
        assert_eq!(parse_human_size("1.5GB"), 1_610_612_736); // 1.5 * 1024^3 rounded
        assert_eq!(parse_human_size("500MB"), 524_288_000);
        assert_eq!(parse_human_size("2.5kB"), 2_560);
        assert_eq!(parse_human_size("1.2GB (virtual 2GB)"), 1_288_490_189);
    }
}
