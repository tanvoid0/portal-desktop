//! `tauri.conf.json` is the authority on the bundle identifier; `APP_IDENTIFIER`
//! in portal-core is a copy of it. This asserts they have not drifted.
//!
//! It lives here rather than beside the constant because the conf file belongs to
//! the Tauri shell — portal-core has no business reading it, and reaching up out
//! of that crate would break the moment it moves.
//!
//! An integration test, not a unit test, deliberately: `tests/` builds its own
//! binary, so this keeps running even while the lib test harness cannot load.

use portal_core::app_paths::APP_IDENTIFIER;

/// The bundler only validates the identifier during `tauri build`, which the
/// release smoke job does not run — so a bad identifier gets caught by all four
/// platform builds at once and the tag publishes nothing. That is what killed
/// v0.9.0 and v0.10.0. Assert the rule here, where `cargo test` sees it.
#[test]
fn bundle_identifier_is_valid_and_matches_tauri_conf() {
    let conf: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tauri.conf.json")).unwrap(),
    )
    .unwrap();
    let configured = conf["identifier"].as_str().unwrap();

    assert_eq!(
        configured, APP_IDENTIFIER,
        "tauri.conf.json identifier and APP_IDENTIFIER have drifted; the app data \
         dir and keychain service would not match the bundle"
    );

    // Tauri: alphanumerics, hyphens and periods only. An underscore fails the build.
    assert!(
        configured
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.'),
        "identifier {configured:?} has characters the Tauri bundler rejects"
    );
}
