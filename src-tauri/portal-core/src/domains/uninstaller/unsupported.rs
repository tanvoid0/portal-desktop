use super::*;

const MSG: &str = "The software uninstaller is only available on Windows.";

pub fn list_programs() -> Result<Vec<InstalledProgram>, String> {
    Err(MSG.into())
}
pub fn scan_leftovers(_id: &str) -> Result<Vec<Leftover>, String> {
    Err(MSG.into())
}
pub fn uninstall(_id: &str, _silent: bool, _cleanup: &[String]) -> Result<UninstallOutcome, String> {
    Err(MSG.into())
}
