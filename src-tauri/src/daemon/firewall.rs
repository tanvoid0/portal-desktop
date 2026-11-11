//! Windows Firewall self-repair for the LAN listener: on `daemon::spawn`,
//! check whether an inbound-allow rule already exists for this exe and add
//! one if not. Best-effort only — `add rule` silently no-ops without admin
//! elevation; there is no retry loop or UAC prompt here, just a log line
//! either way. [`rule_present`] also backs `network::commands::
//! get_remote_connectivity_status`'s honest status read, so there is one
//! `netsh`-output parser, not two.
//!
//! Deliberately NOT a reachability probe: a same-host TCP connect to this
//! machine's own LAN IP succeeds via Windows' loopback short-circuit
//! regardless of whether an external device could actually get in — exactly
//! the misleading "all clear" that bit the live debugging session this
//! module exists to answer for.

use std::path::{Path, PathBuf};

const RULE_NAME: &str = "Portal Desktop";

/// This process's own exe, canonicalized so it compares equal to the
/// absolute path `netsh` echoes back. `canonicalize()` prepends Windows'
/// `\\?\` extended-length-path prefix, which `netsh`'s own argument parser
/// rejects outright (`program=\\?\D:\...` fails to add, and would never have
/// matched `show rule`'s plain-path output either) — stripped back off here,
/// the one place both `ensure_rule` and `rule_present` get this path from.
pub fn current_exe_path() -> Option<PathBuf> {
    let exe = std::env::current_exe()
        .and_then(|p| p.canonicalize())
        .ok()?;
    let stripped = exe
        .to_string_lossy()
        .strip_prefix(r"\\?\")
        .map(PathBuf::from);
    Some(stripped.unwrap_or(exe))
}

/// Off the async runtime (it shells out), so it never delays the LAN
/// listener or discovery responder starting. Never panics.
pub fn spawn_check(port: u16) {
    tauri::async_runtime::spawn_blocking(move || {
        let Some(exe) = current_exe_path() else {
            eprintln!("[daemon] firewall check: could not resolve current exe path");
            return;
        };
        ensure_rule(&exe, port);
    });
}

#[cfg(target_os = "windows")]
fn ensure_rule(exe: &Path, port: u16) {
    if rule_present(exe) {
        println!("[daemon] firewall rule OK for {}", exe.display());
        return;
    }

    use crate::process_ext::NoWindowExt;
    let output = std::process::Command::new("netsh")
        .args([
            "advfirewall",
            "firewall",
            "add",
            "rule",
            &format!("name={RULE_NAME}"),
            "dir=in",
            "action=allow",
            &format!("program={}", exe.display()),
            "enable=yes",
            "profile=any",
            "protocol=TCP",
            &format!("localport={port}"),
        ])
        .no_window()
        .output();

    match output {
        Ok(out) if out.status.success() => {
            println!("[daemon] firewall rule added for {}", exe.display());
        }
        Ok(out) => eprintln!(
            "[daemon] firewall rule add failed (exit {:?}): {}{}",
            out.status.code(),
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr),
        ),
        Err(e) => eprintln!("[daemon] firewall rule add failed to run netsh: {e}"),
    }
}

#[cfg(not(target_os = "windows"))]
fn ensure_rule(_exe: &Path, _port: u16) {}

/// Whether `netsh`'s rule set already has an enabled Allow rule for `exe`.
/// Shared by [`ensure_rule`] (so it never adds a duplicate) and the
/// connectivity command (so it reports the same thing it would fix).
#[cfg(target_os = "windows")]
pub fn rule_present(exe: &Path) -> bool {
    use crate::process_ext::NoWindowExt;
    let Ok(output) = std::process::Command::new("netsh")
        .args([
            "advfirewall",
            "firewall",
            "show",
            "rule",
            &format!("name={RULE_NAME}"),
            "verbose",
        ])
        .no_window()
        .output()
    else {
        return false;
    };
    parse_rule_present(&String::from_utf8_lossy(&output.stdout), exe)
}

#[cfg(not(target_os = "windows"))]
pub fn rule_present(_exe: &Path) -> bool {
    false
}

/// Adapter-name/description substrings that mean "this is a VPN tunnel,"
/// found by reading real `ipconfig /all` output against known VPN products.
/// Heuristic, not exhaustive — good enough to name the likely culprit in a
/// banner, not a security boundary.
const VPN_KEYWORDS: &[&str] = &[
    "vpn",
    "tap-windows",
    "tap adapter",
    "wintun",
    "ppp adapter",
    "nordvpn",
    "expressvpn",
    "avast secureline",
    "private internet access",
    "openvpn",
    "wireguard",
    "anyconnect",
    "globalprotect",
    "pulse secure",
    "forticlient",
    "hamachi",
    "zerotier",
    "tailscale",
];

/// The name of an active (connected) VPN-looking network adapter, if any.
/// Some VPN clients drop inbound LAN traffic as an anti-leak default even
/// while otherwise idle — this is report-only, there is no "fix" an app can
/// apply to someone else's VPN client, just naming it so the user knows what
/// to try disconnecting.
#[cfg(target_os = "windows")]
pub fn active_vpn_adapter() -> Option<String> {
    use crate::process_ext::NoWindowExt;
    let output = std::process::Command::new("ipconfig")
        .arg("/all")
        .no_window()
        .output()
        .ok()?;
    parse_active_vpn_adapter(&String::from_utf8_lossy(&output.stdout))
}

#[cfg(not(target_os = "windows"))]
pub fn active_vpn_adapter() -> Option<String> {
    None
}

/// Pure parser for [`active_vpn_adapter`]. `ipconfig /all` prints one
/// unindented header line per adapter (`"<Type> adapter <Name>:"`) followed
/// by indented fields; a disconnected adapter's block is just a lone `Media
/// State . . . : Media disconnected` line, nothing else. Finds the first
/// *connected* block whose header or body mentions a known VPN product.
fn parse_active_vpn_adapter(output: &str) -> Option<String> {
    let normalized = output.replace("\r\n", "\n");
    let mut blocks: Vec<(&str, String)> = Vec::new();
    let mut header: Option<&str> = None;
    let mut body = String::new();

    for line in normalized.lines() {
        if !line.is_empty() && !line.starts_with(' ') && !line.starts_with('\t') {
            if let Some(h) = header.take() {
                blocks.push((h, std::mem::take(&mut body)));
            }
            header = Some(line.trim_end_matches(':').trim());
        } else {
            body.push_str(line);
            body.push('\n');
        }
    }
    if let Some(h) = header {
        blocks.push((h, body));
    }

    blocks.into_iter().find_map(|(header, body)| {
        if body.to_lowercase().contains("media disconnected") {
            return None;
        }
        let haystack = format!("{header}\n{body}").to_lowercase();
        VPN_KEYWORDS
            .iter()
            .any(|kw| haystack.contains(kw))
            .then(|| header.to_string())
    })
}

/// Pure parser: `netsh ... show rule verbose` prints one blank-line-
/// separated block per matching rule name (there can be several, including
/// stale duplicates — see the module doc on `daemon::mod`'s `spawn`). Finds
/// a block whose `Program:` matches `exe` (case-insensitive path compare)
/// and is `Enabled: Yes`, `Action: Allow`. Split out from `rule_present` so
/// it's unit-testable without shelling out — the piece most likely to break
/// silently if a Windows update reformats `netsh`'s output.
fn parse_rule_present(output: &str, exe: &Path) -> bool {
    let exe_lower = exe.to_string_lossy().to_lowercase();
    output.replace("\r\n", "\n").split("\n\n").any(|block| {
        field(block, "Program:").map(str::to_lowercase).as_deref() == Some(exe_lower.as_str())
            && field(block, "Enabled:") == Some("Yes")
            && field(block, "Action:") == Some("Allow")
    })
}

/// `"  Label:   value  "` -> `"value"`, for one `netsh`-style `Label:` line
/// in a block (column-aligned with padding, hence the trim on both sides).
fn field<'a>(block: &'a str, label: &str) -> Option<&'a str> {
    block
        .lines()
        .find_map(|l| l.trim().strip_prefix(label))
        .map(str::trim)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Realistic `netsh advfirewall firewall show rule name="Portal Desktop"
    // verbose` output: one matching rule, one stale duplicate pointing at an
    // old install path, matching the "6 duplicate entries" found live.
    const FIXTURE: &str = "\
Rule Name:                            Portal Desktop
----------------------------------------------------------------------
Enabled:                              Yes
Direction:                            In
Profiles:                             Domain,Private,Public
Grouping:
LocalIP:                              Any
RemoteIP:                             Any
Protocol:                             TCP
LocalPort:                            18420
RemotePort:                           Any
Edge traversal:                       No
Program:                              C:\\Users\\tanvo\\AppData\\Local\\Portal Desktop\\portal-desktop.exe
Action:                               Allow

Rule Name:                            Portal Desktop
----------------------------------------------------------------------
Enabled:                              No
Direction:                            In
Profiles:                             Domain,Private,Public
Grouping:
LocalIP:                              Any
RemoteIP:                             Any
Protocol:                             TCP
LocalPort:                            18420
RemotePort:                           Any
Edge traversal:                       No
Program:                              C:\\old\\stale\\path\\portal-desktop.exe
Action:                               Allow

";

    #[test]
    fn finds_matching_enabled_allow_rule() {
        let exe = Path::new(r"C:\Users\tanvo\AppData\Local\Portal Desktop\portal-desktop.exe");
        assert!(parse_rule_present(FIXTURE, exe));
    }

    #[test]
    fn case_insensitive_path_match() {
        let exe = Path::new(r"c:\users\tanvo\appdata\local\portal desktop\PORTAL-DESKTOP.EXE");
        assert!(parse_rule_present(FIXTURE, exe));
    }

    #[test]
    fn rejects_when_no_block_matches_this_exe() {
        let exe = Path::new(r"C:\some\other\app.exe");
        assert!(!parse_rule_present(FIXTURE, exe));
    }

    #[test]
    fn rejects_disabled_rule_even_if_path_matches() {
        let exe = Path::new(r"C:\old\stale\path\portal-desktop.exe");
        assert!(!parse_rule_present(FIXTURE, exe));
    }

    #[test]
    fn rejects_empty_output() {
        assert!(!parse_rule_present("", Path::new(r"C:\app.exe")));
    }

    // Realistic `ipconfig /all` excerpt: a disconnected physical adapter, a
    // normal connected Wi-Fi adapter, and a connected VPN tunnel — mirrors
    // what this session found live (Avast VPN's `SecureLine` service running).
    const IPCONFIG_FIXTURE: &str = "\
Windows IP Configuration

Wireless LAN adapter Local Area Connection* 3:

   Media State . . . . . . . . . . . : Media disconnected
   Connection-specific DNS Suffix  . :

Wireless LAN adapter Wi-Fi:

   Connection-specific DNS Suffix  . : Home
   Description . . . . . . . . . . . : Intel(R) Wi-Fi 6 AX200
   IPv4 Address. . . . . . . . . . . : 192.168.0.137

PPP adapter SecureLine VPN:

   Connection-specific DNS Suffix  . :
   Description . . . . . . . . . . . : Avast SecureLine VPN Adapter
   IPv4 Address. . . . . . . . . . . : 10.8.0.2

";

    #[test]
    fn finds_connected_vpn_adapter_by_name() {
        assert_eq!(
            parse_active_vpn_adapter(IPCONFIG_FIXTURE).as_deref(),
            Some("PPP adapter SecureLine VPN")
        );
    }

    #[test]
    fn ignores_disconnected_adapters_and_plain_wifi() {
        let no_vpn = IPCONFIG_FIXTURE
            .lines()
            .take_while(|l| !l.starts_with("PPP adapter"))
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(parse_active_vpn_adapter(&no_vpn), None);
    }

    #[test]
    fn vpn_parser_rejects_empty_output() {
        assert_eq!(parse_active_vpn_adapter(""), None);
    }
}
