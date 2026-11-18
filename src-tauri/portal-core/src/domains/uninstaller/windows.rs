use std::fs;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::*;
use crate::process_ext::NoWindowExt;

const KEY_PREFIXES: [&str; 3] = [
    r"HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\",
    r"HKEY_LOCAL_MACHINE\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\",
    r"HKEY_CURRENT_USER\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\",
];

fn ps(script: &str) -> Result<String, String> {
    let script = format!("[Console]::OutputEncoding=[Text.Encoding]::UTF8;{script}");
    let out = Command::new("powershell")
        .no_window()
        .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", &script])
        .output()
        .map_err(|e| format!("Failed to run PowerShell: {e}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

/// PowerShell single-quote escape.
fn q(s: &str) -> String {
    s.replace('\'', "''")
}

pub fn list_programs() -> Result<Vec<InstalledProgram>, String> {
    let script = r#"
$paths='HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*','HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*','HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*'
$r = @(Get-ItemProperty $paths -ErrorAction SilentlyContinue | Where-Object { $_.DisplayName -and $_.UninstallString -and -not $_.ParentKeyName } | ForEach-Object {
  [PSCustomObject]@{
    id = ($_.PSPath -replace '^.*?::','')
    name = [string]$_.DisplayName; version = [string]$_.DisplayVersion; publisher = [string]$_.Publisher
    installLocation = [string]$_.InstallLocation; installDate = [string]$_.InstallDate
    sizeKb = [uint64]($_.EstimatedSize -as [uint64]); uninstallString = [string]$_.UninstallString
    quietUninstallString = [string]$_.QuietUninstallString
    isMsi = (($_.WindowsInstaller -eq 1) -or ([string]$_.UninstallString -match 'msiexec'))
    systemComponent = ($_.SystemComponent -eq 1)
  } })
ConvertTo-Json -InputObject $r -Compress -Depth 3
"#;
    let out = ps(script)?;
    let mut v: Vec<InstalledProgram> =
        serde_json::from_str(&out).map_err(|e| format!("Failed to parse program list: {e}"))?;
    v.sort_by_key(|p| p.name.to_lowercase());
    Ok(v)
}

/// The id must be an Uninstall key path; resolve it against the live registry
/// so the command we run is always the one the installer registered.
fn find(id: &str) -> Result<InstalledProgram, String> {
    if !KEY_PREFIXES.iter().any(|p| id.starts_with(p)) {
        return Err("Not an uninstall registry key.".into());
    }
    list_programs()?
        .into_iter()
        .find(|p| p.id.eq_ignore_ascii_case(id))
        .ok_or_else(|| "Program not found (already uninstalled?).".to_string())
}

fn key_exists(id: &str) -> bool {
    let p = id
        .replacen("HKEY_LOCAL_MACHINE", "HKLM:", 1)
        .replacen("HKEY_CURRENT_USER", "HKCU:", 1);
    ps(&format!("Test-Path -LiteralPath '{}'", q(&p))).map(|s| s == "True").unwrap_or(true)
}

/// `cleanup`: leftover paths to remove after a successful uninstall. They must
/// be among what a scan finds *before* uninstalling — afterwards the registry
/// entry that scan keys off is gone.
pub fn uninstall(id: &str, silent: bool, cleanup: &[String]) -> Result<UninstallOutcome, String> {
    let p = find(id)?;
    let before = if cleanup.is_empty() { Vec::new() } else { scan_leftovers(id)? };
    let (cmd, used_silent) = if p.is_msi {
        let guid = msi_guid(&p.uninstall_string)
            .ok_or("MSI program without a product code; cannot uninstall safely.")?;
        let flags = if silent { "/qn /norestart" } else { "" };
        (format!("msiexec.exe /x {guid} {flags}"), silent)
    } else if silent && !p.quiet_uninstall_string.is_empty() {
        (p.quiet_uninstall_string.clone(), true)
    } else {
        (p.uninstall_string.clone(), false)
    };
    // cmd /C so the registered string's own quoting is honoured verbatim.
    let status = Command::new("cmd")
        .no_window()
        .raw_arg(format!("/C \"{cmd}\""))
        .status()
        .map_err(|e| format!("Failed to start uninstaller: {e}"))?;
    let removed = !key_exists(&p.id);
    let message = if removed {
        "Uninstalled.".to_string()
    } else if silent && !used_silent {
        "No silent mode registered; ran the normal uninstaller, which may be waiting on a dialog or was cancelled.".to_string()
    } else {
        "Uninstaller finished but the program is still registered (cancelled or failed).".to_string()
    };
    let cleanup = (removed && !cleanup.is_empty()).then(|| delete_leftovers(&before, cleanup));
    Ok(UninstallOutcome { exit_code: status.code(), removed, silent: used_silent, message, cleanup })
}

// --- leftovers -------------------------------------------------------------

fn env_path(name: &str) -> Option<PathBuf> {
    std::env::var_os(name).map(PathBuf::from)
}

/// Roots a leftover folder may live directly under.
fn roots() -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = ["ProgramFiles", "ProgramFiles(x86)", "ProgramW6432", "ProgramData", "APPDATA", "LOCALAPPDATA"]
        .iter()
        .filter_map(|n| env_path(n))
        .collect();
    if let Some(l) = env_path("LOCALAPPDATA") {
        v.push(l.join("Programs"));
    }
    v.sort();
    v.dedup();
    v
}

fn safe_name(s: &str) -> bool {
    let s = s.trim();
    s.len() >= 3 && !s.contains(['<', '>', ':', '"', '/', '\\', '|', '?', '*'])
}

fn dir_size(p: &Path) -> u64 {
    let Ok(rd) = fs::read_dir(p) else { return 0 };
    rd.flatten()
        .map(|e| match e.metadata() {
            Ok(m) if m.is_dir() => dir_size(&e.path()),
            Ok(m) => m.len(),
            Err(_) => 0,
        })
        .sum()
}

/// A registered InstallLocation is deletable only if it is not a known root,
/// not under the Windows directory, and at least two levels deep.
fn install_dir_ok(install: &Path, roots: &[PathBuf]) -> bool {
    let norm = |p: &Path| p.to_string_lossy().trim_end_matches('\\').to_lowercase();
    let i = norm(install);
    if roots.iter().any(|r| norm(r) == i) {
        return false;
    }
    if let Some(w) = env_path("SystemRoot") {
        if i.starts_with(&norm(&w)) {
            return false;
        }
    }
    install.components().count() >= 3
}

pub fn scan_leftovers(id: &str) -> Result<Vec<Leftover>, String> {
    let p = find(id)?;
    let roots = roots();
    let mut folders: Vec<PathBuf> = Vec::new();

    let loc = p.install_location.trim().trim_matches('"');
    if !loc.is_empty() {
        let loc = PathBuf::from(loc);
        if loc.is_dir() && install_dir_ok(&loc, &roots) {
            folders.push(loc);
        }
    }
    if safe_name(&p.name) {
        for r in &roots {
            folders.push(r.join(p.name.trim()));
            if safe_name(&p.publisher) {
                folders.push(r.join(p.publisher.trim()).join(p.name.trim()));
            }
        }
    }
    folders.sort();
    folders.dedup();
    let mut out: Vec<Leftover> = folders
        .into_iter()
        .filter(|f| f.is_dir())
        .map(|f| Leftover { kind: "folder".into(), size_bytes: dir_size(&f), path: f.to_string_lossy().into() })
        .collect();

    // Software\<Name> and Software\<Publisher>\<Name> in each hive.
    if safe_name(&p.name) {
        let n = q(p.name.trim());
        let pubr = if safe_name(&p.publisher) { q(p.publisher.trim()) } else { String::new() };
        let script = format!(
            r#"$n='{n}';$p='{pubr}'
$c=@();foreach($h in 'HKCU:\SOFTWARE','HKLM:\SOFTWARE','HKLM:\SOFTWARE\WOW6432Node'){{ $c+="$h\$n"; if($p){{$c+="$h\$p\$n"}} }}
$e=@($c | Where-Object {{ Test-Path -LiteralPath $_ }})
ConvertTo-Json -InputObject $e -Compress"#
        );
        let keys: Vec<String> = serde_json::from_str(&ps(&script)?).unwrap_or_default();
        out.extend(keys.into_iter().map(|k| Leftover { kind: "registry".into(), path: k, size_bytes: 0 }));
    }
    Ok(out)
}

fn delete_leftovers(allowed: &[Leftover], paths: &[String]) -> CleanupResult {
    let backup = dirs::data_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("portal_desktop-uninstall-backups");
    let mut res = CleanupResult::default();
    for path in paths {
        let Some(l) = allowed.iter().find(|l| l.path.eq_ignore_ascii_case(path)) else {
            res.failed.push(format!("{path}: not a leftover of this program"));
            continue;
        };
        let r = if l.kind == "folder" {
            trash::delete(&l.path).map_err(|e| e.to_string())
        } else {
            remove_key(&l.path, &backup, &mut res)
        };
        match r {
            Ok(()) => res.removed.push(l.path.clone()),
            Err(e) => res.failed.push(format!("{}: {e}", l.path)),
        }
    }
    res
}

fn remove_key(ps_path: &str, backup: &Path, res: &mut CleanupResult) -> Result<(), String> {
    fs::create_dir_all(backup).map_err(|e| e.to_string())?;
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let file = backup.join(format!("{stamp}-{}.reg", res.removed.len() + res.failed.len()));
    let native = ps_path.replacen("HKCU:", "HKCU", 1).replacen("HKLM:", "HKLM", 1);
    let ok = Command::new("reg")
        .no_window()
        .args(["export", &native, &file.to_string_lossy(), "/y"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !ok {
        return Err("could not back up key; left in place".into());
    }
    res.backup_dir = backup.to_string_lossy().into();
    ps(&format!("Remove-Item -LiteralPath '{}' -Recurse -Force", q(ps_path))).map(|_| ())
}
