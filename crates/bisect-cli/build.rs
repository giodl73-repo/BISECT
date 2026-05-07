//! Build script that captures provenance metadata for the running binary.
//!
//! Sets these env vars at compile time, available via env!() in the binary:
//! - REDIST_BUILD_COMMIT: git rev-parse HEAD (or "unknown" if not a git checkout)
//! - REDIST_BUILD_COMMIT_SHORT: first 12 chars of the commit
//! - REDIST_BUILD_DATE: ISO-8601 UTC timestamp of the build
//! - REDIST_RUSTC_VERSION: rustc --version output
//!
//! Re-runs whenever HEAD or the index changes, so the embedded commit
//! always matches the working tree at build time.

use std::process::Command;

fn main() {
    // Deposition Prep plan Task 8 (B-07): honor an explicit
    // `REDIST_BUILD_COMMIT_OVERRIDE` env at build time. This lets reproducible-
    // build packagers pin the recorded commit to a release tag without going
    // through git. Production builds outside CI/release should leave this unset.
    let commit_full = if let Ok(override_val) = std::env::var("REDIST_BUILD_COMMIT_OVERRIDE") {
        // Honor override verbatim; do NOT append the -dirty marker (the
        // override is the authoritative attestation).
        println!("cargo:warning=using REDIST_BUILD_COMMIT_OVERRIDE={override_val}");
        override_val
    } else {
        // Default: derive from git rev-parse HEAD + dirty-marker suffix.
        let commit = git_command(&["rev-parse", "HEAD"])
            .unwrap_or_else(|| "unknown".to_string());
        let dirty = match Command::new("git").args(["diff", "--quiet"]).status() {
            Ok(s) if !s.success() => "-dirty",
            _ => "",
        };
        format!("{commit}{dirty}")
    };
    let commit_short_full = if commit_full.len() >= 12 {
        commit_full[..12].to_string()
    } else {
        commit_full.clone()
    };

    println!("cargo:rerun-if-env-changed=REDIST_BUILD_COMMIT_OVERRIDE");
    println!("cargo:rustc-env=REDIST_BUILD_COMMIT={commit_full}");
    println!("cargo:rustc-env=REDIST_BUILD_COMMIT_SHORT={commit_short_full}");

    // build timestamp — ISO-8601 UTC
    // We don't take a deterministic-build dependency (chrono / time crate) here;
    // formatting unix epoch by hand keeps build.rs dependency-free.
    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    println!("cargo:rustc-env=REDIST_BUILD_UNIX={now_secs}");

    // rustc version
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let rustc_version = Command::new(&rustc)
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=REDIST_RUSTC_VERSION={rustc_version}");

    // Re-run if HEAD or the index changes.
    println!("cargo:rerun-if-changed=../../.git/HEAD");
    println!("cargo:rerun-if-changed=../../.git/index");
    println!("cargo:rerun-if-changed=build.rs");
}

fn git_command(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout).ok().map(|s| s.trim().to_string())
}
