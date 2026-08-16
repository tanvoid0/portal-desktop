fn main() {
    manifest_for_test_binaries();
    tauri_build::build()
}

/// Give `cargo test` binaries the same common-controls manifest the app gets.
///
/// `tauri_build::build()` embeds a Windows manifest into the *application*
/// binary, and part of what that manifest asks for is comctl32 version 6. The
/// dialog plugin imports `TaskDialogIndirect`, which only v6 exports — without
/// the manifest the loader binds comctl32 5.82 instead and the process dies at
/// startup with `STATUS_ENTRYPOINT_NOT_FOUND`, before `main` runs.
///
/// Test harnesses are separately linked binaries that never saw that manifest,
/// so `cargo test` failed while `cargo build` was fine. `rustc-link-arg-tests`
/// scopes this to test targets, which keeps it out of the shipped binary's link
/// and — unlike `RUSTFLAGS` — does not invalidate every dependency's build.
fn manifest_for_test_binaries() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() != Ok("msvc") {
        return;
    }

    // Note: *not* `/MANIFEST:EMBED`. `tauri_build` embeds a manifest resource in
    // the app binary, and a second embedded one is a hard link error
    // (`CVT1100: duplicate resource`). Left external, link.exe writes
    // `<binary>.manifest` beside each output: Windows reads it for the test
    // harness, which has no embedded manifest, and the app binary keeps using
    // the richer one Tauri gave it.
    println!(
        "cargo:rustc-link-arg=/MANIFESTDEPENDENCY:type='win32' \
         name='Microsoft.Windows.Common-Controls' version='6.0.0.0' \
         processorArchitecture='*' publicKeyToken='6595b64144ccf1df' language='*'"
    );
}
