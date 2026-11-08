use std::process::{Command, Output};

use crate::process_ext::NoWindowExt;

use super::types::ElevatedRunResult;

pub fn is_elevated() -> bool {
    Command::new("id")
        .no_window()
        .args(["-u"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok().map(|s| s.trim() == "0"))
        .unwrap_or(false)
}

pub fn run_elevated(exe: &str, args: &[String]) -> Result<ElevatedRunResult, String> {
    run(exe, args, false)
}

pub fn restart_elevated() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| format!("Failed to resolve current exe: {}", e))?;
    let args: Vec<String> = std::env::args().skip(1).collect();
    run(&exe.to_string_lossy(), &args, true).map(|_| ())
}

fn run(exe: &str, args: &[String], detach: bool) -> Result<ElevatedRunResult, String> {
    if cfg!(target_os = "macos") {
        run_macos(exe, args, detach)
    } else {
        run_linux(exe, args)
    }
}

fn run_linux(exe: &str, args: &[String]) -> Result<ElevatedRunResult, String> {
    let mut cmd_args = vec![exe.to_string()];
    cmd_args.extend(args.iter().cloned());

    let output = Command::new("pkexec")
        .no_window()
        .args(&cmd_args)
        .output()
        .or_else(|_| Command::new("sudo").no_window().args(&cmd_args).output());

    finish(output)
}

fn run_macos(exe: &str, args: &[String], detach: bool) -> Result<ElevatedRunResult, String> {
    let joined = std::iter::once(exe.to_string())
        .chain(args.iter().cloned())
        .map(|a| a.replace('"', "\\\""))
        .collect::<Vec<_>>()
        .join(" ");
    // Detached (app restart): launch and return immediately instead of
    // waiting on a process that is about to outlive this one.
    let script = format!(
        r#"do shell script "{}{}" with administrator privileges"#,
        joined,
        if detach { " > /dev/null 2>&1 &" } else { "" }
    );
    let output = Command::new("osascript")
        .no_window()
        .args(["-e", &script])
        .output();

    finish(output)
}

fn finish(output: std::io::Result<Output>) -> Result<ElevatedRunResult, String> {
    let output = output.map_err(|e| format!("Failed to run elevation helper: {}", e))?;
    if output.status.success() {
        Ok(ElevatedRunResult {
            success: true,
            cancelled: false,
            message: "Completed with elevated privileges.".into(),
        })
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if stderr.to_lowercase().contains("cancel") {
            Ok(ElevatedRunResult {
                success: false,
                cancelled: true,
                message: "Elevation was cancelled.".into(),
            })
        } else {
            Err(stderr)
        }
    }
}
