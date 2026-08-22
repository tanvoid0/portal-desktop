/// The version of the running binary. `env!` expands in this crate, whose
/// version is the one `tauri.conf.json` ships — reading it from a library
/// crate would report that library's version instead.
#[portal_rpc_macro::rpc]
pub async fn get_app_version_command() -> Result<String, String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}
