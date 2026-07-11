use crate::domains::environment::types::{EnvApplyResult, EnvChange, EnvPermissions, EnvVariable};

pub trait PlatformEnv: Send + Sync {
    fn list_variables(&self) -> Result<Vec<EnvVariable>, String>;
    fn get_permissions(&self) -> Result<EnvPermissions, String>;
    fn set_variable(&self, name: &str, value: &str, scope: &str) -> Result<(), String>;
    fn delete_variable(&self, name: &str, scope: &str) -> Result<(), String>;
    fn apply_changes_elevated(&self, changes: &[EnvChange]) -> Result<EnvApplyResult, String>;
    fn refresh_process_environment(&self) -> Result<(), String>;
}

pub fn platform_env() -> Box<dyn PlatformEnv> {
    #[cfg(target_os = "windows")]
    {
        Box::new(super::windows::WindowsEnv)
    }
    #[cfg(not(target_os = "windows"))]
    {
        Box::new(super::unix::UnixEnv)
    }
}
