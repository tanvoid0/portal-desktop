//! OS facts the vault lock policy needs: which OS session this is, and how long
//! the user has been idle.

use std::sync::Arc;
use std::time::Duration;

use portal_core::domains::credentials::services::{OsSession, VaultSession};
use tauri::{AppHandle, Emitter};

const CHECK_EVERY: Duration = Duration::from_secs(15);

/// Locks the vault once the user has been idle for the policy's `idle_minutes`.
pub fn spawn_idle_lock(app: AppHandle, session: Arc<VaultSession>) {
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(CHECK_EVERY).await;
            let minutes = session.policy().idle_minutes;
            if minutes == 0 || !session.is_unlocked() {
                continue;
            }
            // Real input idle, so background key reads can't postpone the lock.
            // Without an OS input clock, the vault going unused is all there is.
            let idle = system_idle().unwrap_or_else(|| session.since_last_use());
            if idle >= Duration::from_secs(u64::from(minutes) * 60) {
                session.lock();
                let _ = app.emit("vault://locked", ());
            }
        }
    });
}

/// Time since the last keyboard or mouse input anywhere on the desktop.
#[cfg(windows)]
fn system_idle() -> Option<Duration> {
    use windows::Win32::System::SystemInformation::GetTickCount;
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};

    let mut info = LASTINPUTINFO {
        cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
        dwTime: 0,
    };
    unsafe {
        GetLastInputInfo(&mut info)
            .as_bool()
            .then(|| Duration::from_millis(u64::from(GetTickCount().wrapping_sub(info.dwTime))))
    }
}

#[cfg(not(windows))]
fn system_idle() -> Option<Duration> {
    None
}

/// Boot time alone misses a Windows shutdown with Fast Startup (uptime carries
/// over), so the id there is the logon session, which does not.
pub fn os_session() -> OsSession {
    OsSession {
        id: logon_id().unwrap_or_default(),
        boot_time: sysinfo::System::boot_time(),
    }
}

#[cfg(windows)]
fn logon_id() -> Option<String> {
    use windows::Win32::Foundation::{CloseHandle, HANDLE};
    use windows::Win32::Security::{
        GetTokenInformation, TokenStatistics, TOKEN_QUERY, TOKEN_STATISTICS,
    };
    use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    unsafe {
        let mut token = HANDLE::default();
        OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).ok()?;
        let mut stats = TOKEN_STATISTICS::default();
        let mut len = 0;
        let read = GetTokenInformation(
            token,
            TokenStatistics,
            Some((&mut stats as *mut TOKEN_STATISTICS).cast()),
            std::mem::size_of::<TOKEN_STATISTICS>() as u32,
            &mut len,
        );
        let _ = CloseHandle(token);
        read.ok()?;
        let luid = stats.AuthenticationId;
        Some(format!("{:x}-{:x}", luid.HighPart, luid.LowPart))
    }
}

#[cfg(not(windows))]
fn logon_id() -> Option<String> {
    None
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn windows_session_facts_are_readable() {
        assert!(logon_id().is_some());
        assert!(system_idle().is_some());
        assert!(os_session().boot_time > 0);
    }
}
