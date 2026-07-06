/*!
 * Process spawn extension: suppress the console window that Windows attaches to
 * child processes when the host is a GUI app (no attached console).
 *
 * Without this, every `Command` spawn in a release build flashes a `cmd`/console
 * window for a split second. `CREATE_NO_WINDOW` (0x0800_0000) tells Windows not to
 * create a console for the child. On non-Windows this is a no-op.
 *
 * Implemented for both `std::process::Command` and `tokio::process::Command` so it
 * chains directly after `Command::new(...)`:
 *
 * ```ignore
 * Command::new("git").no_window().arg("status").output()
 * ```
 */

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Adds `.no_window()` to a command builder.
pub trait NoWindowExt {
    fn no_window(&mut self) -> &mut Self;
}

impl NoWindowExt for std::process::Command {
    #[cfg(windows)]
    fn no_window(&mut self) -> &mut Self {
        use std::os::windows::process::CommandExt;
        self.creation_flags(CREATE_NO_WINDOW)
    }

    #[cfg(not(windows))]
    fn no_window(&mut self) -> &mut Self {
        self
    }
}

impl NoWindowExt for tokio::process::Command {
    #[cfg(windows)]
    fn no_window(&mut self) -> &mut Self {
        self.creation_flags(CREATE_NO_WINDOW)
    }

    #[cfg(not(windows))]
    fn no_window(&mut self) -> &mut Self {
        self
    }
}
