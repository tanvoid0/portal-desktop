use crate::process_ext::NoWindowExt;

use super::types::ElevatedRunResult;

pub fn is_elevated() -> bool {
    run_powershell(
        "([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)",
    )
    .map(|out| out.trim().eq_ignore_ascii_case("true"))
    .unwrap_or(false)
}

pub fn run_elevated(exe: &str, args: &[String]) -> Result<ElevatedRunResult, String> {
    let script = format!(
        "$p = Start-Process -FilePath '{exe}' {args} -Verb RunAs -Wait -PassThru; exit $p.ExitCode",
        exe = escape(exe),
        args = argument_list(args),
    );
    match run_powershell(&script) {
        Ok(_) => Ok(ElevatedRunResult {
            success: true,
            cancelled: false,
            message: "Completed with elevated privileges.".into(),
        }),
        Err(err) if is_cancelled(&err) => Ok(ElevatedRunResult {
            success: false,
            cancelled: true,
            message: "Elevation was cancelled.".into(),
        }),
        Err(err) => Err(err),
    }
}

pub fn restart_elevated() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| format!("Failed to resolve current exe: {}", e))?;
    let args: Vec<String> = std::env::args().skip(1).collect();
    let script = format!(
        "Start-Process -FilePath '{exe}' {args} -Verb RunAs",
        exe = escape(&exe.to_string_lossy()),
        args = argument_list(&args),
    );
    match run_powershell(&script) {
        Ok(_) => Ok(()),
        Err(err) if is_cancelled(&err) => Err("Elevation was cancelled.".into()),
        Err(err) => Err(err),
    }
}

fn argument_list(args: &[String]) -> String {
    if args.is_empty() {
        return String::new();
    }
    let quoted: Vec<String> = args.iter().map(|a| format!("'{}'", escape(a))).collect();
    format!("-ArgumentList {}", quoted.join(","))
}

fn is_cancelled(err: &str) -> bool {
    let lower = err.to_lowercase();
    lower.contains("cancel") || lower.contains("1223")
}

fn escape(value: &str) -> String {
    value.replace('\'', "''")
}

fn run_powershell(script: &str) -> Result<String, String> {
    let output = std::process::Command::new("powershell")
        .no_window()
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .output()
        .map_err(|e| format!("Failed to run PowerShell: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if output.status.success() {
        Ok(stdout)
    } else if !stderr.is_empty() {
        Err(stderr)
    } else if !stdout.is_empty() {
        Err(stdout)
    } else {
        Err(format!(
            "PowerShell exited with code {:?}",
            output.status.code()
        ))
    }
}
