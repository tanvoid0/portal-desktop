//! Windows Hello consent gate for the vault's keychain unlock path.
//!
//! Button-triggered only — the frontend never fires this on dialog mount.
//! `verify` blocks on a WinRT call, so both it and `available` hop onto
//! `spawn_blocking` rather than block the async runtime.

#[cfg(windows)]
use windows::core::{factory, HSTRING};
#[cfg(windows)]
use windows::Security::Credentials::UI::{
    UserConsentVerificationResult, UserConsentVerifier, UserConsentVerifierAvailability,
};
#[cfg(windows)]
use windows::Win32::Foundation::HWND;
#[cfg(windows)]
use windows::Win32::System::WinRT::{
    IUserConsentVerifierInterop, RoInitialize, RO_INIT_MULTITHREADED,
};

/// WinRT calls into `UserConsentVerifier` need the calling thread COM/WinRT
/// initialized, which the tokio blocking pool's threads are not by default.
/// Best-effort and idempotent: ignore the result either way (already
/// initialized, or initialized under a different model — nothing to do
/// about the latter from here).
#[cfg(windows)]
fn ensure_winrt_initialized() {
    unsafe {
        let _ = RoInitialize(RO_INIT_MULTITHREADED);
    }
}

/// Whether Windows Hello (or another configured Windows credential gesture)
/// is available on this machine. Any failure reads as unavailable — callers
/// only need a yes/no to decide whether to show the button.
#[cfg(windows)]
pub async fn available() -> bool {
    tauri::async_runtime::spawn_blocking(|| {
        ensure_winrt_initialized();
        UserConsentVerifier::CheckAvailabilityAsync()
            .and_then(|op| op.join())
            .map(|a| a == UserConsentVerifierAvailability::Available)
            .unwrap_or(false)
    })
    .await
    .unwrap_or(false)
}

#[cfg(not(windows))]
pub async fn available() -> bool {
    false
}

/// Prompts the OS consent UI anchored to `hwnd`, blocking until the user
/// responds or cancels.
///
/// The plain `RequestVerificationAsync` doesn't work from a Win32 desktop
/// process — it needs the `ForWindow` interop variant to anchor the prompt
/// to a real window.
// `HWND` wraps a raw pointer, so it isn't `Send` — an async fn parameter of
// that type poisons the whole returned future (the pre-poll state holds every
// parameter, before any code, including the move into `spawn_blocking`, has
// run). Callers pass the bare handle value instead and we rebuild `HWND` only
// inside the blocking closure.
#[cfg(windows)]
pub async fn verify(hwnd: isize, message: &str) -> Result<bool, String> {
    let message = message.to_string();
    tauri::async_runtime::spawn_blocking(move || {
        ensure_winrt_initialized();
        let hwnd = HWND(hwnd as *mut _);
        let interop: IUserConsentVerifierInterop =
            factory::<UserConsentVerifier, IUserConsentVerifierInterop>()
                .map_err(|e| e.to_string())?;
        let op = unsafe {
            interop
                .RequestVerificationForWindowAsync::<windows_future::IAsyncOperation<UserConsentVerificationResult>>(
                    hwnd,
                    &HSTRING::from(message),
                )
                .map_err(|e| e.to_string())?
        };
        let result = op.join().map_err(|e| e.to_string())?;
        Ok(result == UserConsentVerificationResult::Verified)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(not(windows))]
pub async fn verify(_hwnd: isize, _message: &str) -> Result<bool, String> {
    Err("Windows Hello is not available on this platform".to_string())
}
