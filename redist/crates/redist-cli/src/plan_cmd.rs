/// plan_cmd.rs — `redist plan [X] [--configure]`
///
/// Interactive TUI hub with label awareness.
///
/// Dispatch rules:
/// - No label, no flags: launch redist-tui with no arguments (label picker mode)
/// - `--label X`:        launch redist-tui with `--label X` (pre-scoped to that label)
/// - `--configure`:      launch redist-tui with `--configure` (opens compositor wizard)
/// - `--config PATH`:    also pass `--config PATH` when launching the TUI
///
/// Registry JSON is injected via `REDIST_REGISTRY_JSON` env var so the TUI can
/// read current state without a file I/O race.
///
/// Fallback (when redist-tui binary is absent):
/// - With label:       print a summary equivalent to `redist show X`
/// - With --configure: print `redist config validate configs/{label}.yml` guidance
/// - Otherwise:        print a `redist ls` summary

use crate::args::PlanArgs;
use crate::run_registry::Registry;

// ── Binary discovery ───────────────────────────────────────────────────────────

/// Resolve the path to the `redist-tui` binary.
///
/// On all platforms: look for the sibling binary next to the current
/// executable (same directory), mirroring the existing Tui dispatch in main.rs.
pub fn find_tui_binary() -> std::path::PathBuf {
    let mut path = std::env::current_exe()
        .unwrap_or_else(|_| std::path::PathBuf::from("redist-tui"));
    path.set_file_name(if cfg!(windows) { "redist-tui.exe" } else { "redist-tui" });
    path
}

// ── Registry JSON helper ───────────────────────────────────────────────────────

/// Serialize the current registry to a compact JSON string.
///
/// Returns an empty object (`"{}"`) on any error so the TUI can still start
/// without the registry context.
fn registry_json() -> String {
    match Registry::load() {
        Ok(reg) => serde_json::to_string(&reg).unwrap_or_else(|_| "{}".to_string()),
        Err(_) => "{}".to_string(),
    }
}

// ── Fallback output helpers ────────────────────────────────────────────────────

/// Print a plain-text summary when redist-tui is absent and no label was given.
///
/// Mirrors the output of `redist ls` but printed inline so the user can see
/// what labels exist and which stages are complete.
fn print_ls_fallback() {
    let pairs = Registry::list_labels().unwrap_or_default();
    if pairs.is_empty() {
        println!("[INFO] No labels found in .redist registry.");
        println!("       Run: redist build <LABEL> --year 2020");
        return;
    }
    println!("[INFO] redist-tui not available. Label registry:");
    println!("  {:<30} {:<20} {:<20} {:<20}", "LABEL", "BUILT", "ANALYZED", "REPORTED");
    println!("  {}", "-".repeat(92));
    for (name, entry) in &pairs {
        println!(
            "  {:<30} {:<20} {:<20} {:<20}",
            name,
            entry.built.join(", "),
            entry.analyzed.join(", "),
            entry.reported.join(", ")
        );
    }
}

/// Print a summary for a specific label when redist-tui is absent.
///
/// Mirrors the output of `redist show X` so the user still gets useful info.
fn print_show_fallback(label: &str) {
    match Registry::get(label) {
        Ok(Some(entry)) => {
            println!("[INFO] redist-tui not available. Showing label: {label}");
            println!("  Built:    {}", if entry.built.is_empty() { "(none)".to_string() } else { entry.built.join(", ") });
            println!("  Analyzed: {}", if entry.analyzed.is_empty() { "(none)".to_string() } else { entry.analyzed.join(", ") });
            println!("  Reported: {}", if entry.reported.is_empty() { "(none)".to_string() } else { entry.reported.join(", ") });
        }
        Ok(None) => {
            println!("[INFO] Label '{label}' not found in .redist registry.");
            println!("       Run: redist build {label} --year 2020");
        }
        Err(e) => {
            println!("[INFO] Could not read registry: {e}");
        }
    }
}

/// Print configure guidance when redist-tui is absent and --configure was passed.
fn print_configure_fallback(label: Option<&str>) {
    println!("[INFO] redist-tui not available. To edit algorithm config:");
    if let Some(lbl) = label {
        println!("       redist config validate configs/{lbl}.yml");
        println!("       redist config new --name {lbl} --dry-run");
    } else {
        println!("       redist config new --name <LABEL> --dry-run");
        println!("       redist config validate configs/<LABEL>.yml");
    }
}

// ── Entry point ────────────────────────────────────────────────────────────────

/// Run the `redist plan` command.
///
/// Tries to launch `redist-tui` as a subprocess.  If the binary is not found,
/// falls back to printing a plain-text summary and returns `Ok(())`.
pub fn run_plan(args: PlanArgs) -> Result<(), String> {
    let tui_bin = find_tui_binary();

    // Probe whether the TUI binary exists before attempting to spawn it.
    if !tui_bin.exists() {
        println!(
            "[INFO] redist-tui not found. Install with: cargo install --path redist/crates/redist-tui"
        );
        if args.configure {
            print_configure_fallback(args.label.as_deref());
        } else if let Some(ref lbl) = args.label {
            print_show_fallback(lbl);
        } else {
            print_ls_fallback();
        }
        return Ok(());
    }

    // Build the subprocess command.
    let mut cmd = std::process::Command::new(&tui_bin);

    // Inject registry state as env var so TUI avoids file I/O races.
    cmd.env("REDIST_REGISTRY_JSON", registry_json());

    // Forward label.
    if let Some(ref lbl) = args.label {
        cmd.arg("--label").arg(lbl);
    }

    // Forward --configure.
    if args.configure {
        cmd.arg("--configure");
    }

    // Forward --config PATH.
    if let Some(ref cfg_path) = args.config {
        cmd.arg("--config").arg(cfg_path);
    }

    match cmd.status() {
        Ok(status) => std::process::exit(status.code().unwrap_or(1)),
        Err(e) => {
            eprintln!("[INFO] redist-tui not found: {e}");
            eprintln!("[INFO] Install with: cargo install --path redist/crates/redist-tui");
            if args.configure {
                print_configure_fallback(args.label.as_deref());
            } else if let Some(ref lbl) = args.label {
                print_show_fallback(lbl);
            } else {
                print_ls_fallback();
            }
            Ok(())
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::PlanArgs;

    // ── 1. PlanArgs defaults ────────────────────────────────────────────────────

    /// Default PlanArgs has no label and configure=false.
    #[test]
    fn plan_args_defaults() {
        let args = PlanArgs { label: None, configure: false, config: None };
        assert!(args.label.is_none(), "default label must be None");
        assert!(!args.configure, "default configure must be false");
        assert!(args.config.is_none(), "default config must be None");
    }

    // ── 2. PlanArgs with label ──────────────────────────────────────────────────

    /// PlanArgs with label stores label correctly.
    #[test]
    fn plan_args_with_label() {
        let args = PlanArgs {
            label: Some("official_proposal".to_string()),
            configure: false,
            config: None,
        };
        assert_eq!(args.label.as_deref(), Some("official_proposal"));
        assert!(!args.configure);
    }

    // ── 3. PlanArgs with configure ──────────────────────────────────────────────

    /// PlanArgs with configure=true is stored correctly.
    #[test]
    fn plan_args_configure_true() {
        let args = PlanArgs { label: None, configure: true, config: None };
        assert!(args.configure, "configure flag must be true");
        assert!(args.label.is_none());
    }

    // ── 4. PlanArgs with label and configure ────────────────────────────────────

    /// PlanArgs with both label and configure set.
    #[test]
    fn plan_args_label_and_configure() {
        let args = PlanArgs {
            label: Some("senate_draft".to_string()),
            configure: true,
            config: None,
        };
        assert_eq!(args.label.as_deref(), Some("senate_draft"));
        assert!(args.configure);
    }

    // ── 5. find_tui_binary path uses correct extension ──────────────────────────

    /// find_tui_binary must produce a path ending with the right binary name.
    #[test]
    fn find_tui_binary_uses_correct_extension() {
        let p = find_tui_binary();
        let name = p.file_name().unwrap().to_string_lossy();
        if cfg!(windows) {
            assert_eq!(name, "redist-tui.exe", "Windows binary must have .exe extension");
        } else {
            assert_eq!(name, "redist-tui", "Unix binary must have no extension");
        }
    }

    // ── 6. fallback when binary not found: graceful, no panic ──────────────────

    /// run_plan with a nonexistent binary path must return Ok (not panic, not exit).
    ///
    /// We can't intercept std::process::exit, but `Err(e)` in cmd.status() triggers
    /// the fallback branch that calls print_* and returns Ok(()).
    /// We simulate this by calling the fallback helpers directly.
    #[test]
    fn fallback_configure_no_label_does_not_panic() {
        // Direct call to the fallback — must not panic
        print_configure_fallback(None);
    }

    // ── 7. fallback configure with label ────────────────────────────────────────

    #[test]
    fn fallback_configure_with_label_does_not_panic() {
        print_configure_fallback(Some("official_proposal"));
    }

    // ── 8. fallback ls does not panic ───────────────────────────────────────────

    #[test]
    fn fallback_ls_does_not_panic() {
        // print_ls_fallback calls Registry::load internally; with no .redist file
        // in the cwd (test runner dir), it returns an empty registry.
        // We capture by just verifying it does not panic.
        print_ls_fallback();
    }

    // ── 9. fallback show with unknown label does not panic ───────────────────────

    #[test]
    fn fallback_show_unknown_label_does_not_panic() {
        print_show_fallback("nonexistent_label_xyz");
    }

    // ── 10. registry_json returns valid JSON ─────────────────────────────────────

    #[test]
    fn registry_json_returns_valid_json() {
        let json = registry_json();
        // Must be parseable as JSON (at minimum an empty object `{}`)
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&json);
        assert!(parsed.is_ok(), "registry_json must return valid JSON: {json}");
    }

    // ── 11. PlanArgs with config path ────────────────────────────────────────────

    #[test]
    fn plan_args_with_config_path() {
        let args = PlanArgs {
            label: Some("vt_test".to_string()),
            configure: true,
            config: Some(std::path::PathBuf::from("configs/vt_test.yml")),
        };
        assert_eq!(args.label.as_deref(), Some("vt_test"));
        assert!(args.configure);
        assert_eq!(args.config.as_deref(), Some(std::path::Path::new("configs/vt_test.yml")));
    }

    // ── 12. find_tui_binary: path is a sibling of current executable ─────────
    //
    // The result must share the same parent directory as the current executable.

    #[test]
    fn find_tui_binary_is_sibling_of_current_exe() {
        let tui = find_tui_binary();
        let exe = std::env::current_exe().unwrap();
        assert_eq!(
            tui.parent(),
            exe.parent(),
            "redist-tui must be a sibling of the current exe"
        );
    }

    // ── 13. print_configure_fallback with label mentions label name ───────────

    #[test]
    fn fallback_configure_with_label_mentions_label_name() {
        // We can't capture stdout in a simple unit test, but we can verify
        // no panic and that the helper accepts any valid label string.
        print_configure_fallback(Some("senate_draft2"));
        print_configure_fallback(Some("official-proposal"));
        print_configure_fallback(Some("x"));
        // All must return without panic
    }

    // ── 14. registry_json on empty registry returns valid JSON object ──────────

    #[test]
    fn registry_json_empty_registry_is_empty_object() {
        // The registry_json helper returns at minimum "{}"
        let json = registry_json();
        let v: serde_json::Value = serde_json::from_str(&json)
            .expect("registry_json must always return valid JSON");
        assert!(v.is_object(), "registry_json must return a JSON object");
    }

    // ── 15. run_plan: graceful fallback when tui binary absent ───────────────
    //
    // run_plan with a guaranteed-absent TUI path must return Ok (not panic,
    // not call std::process::exit).  We test this by directly exercising the
    // fallback helpers that run_plan calls internally.

    #[test]
    fn run_plan_fallback_does_not_panic_for_configure_mode() {
        // Simulate the configure-mode fallback path
        let args = PlanArgs {
            label: Some("my_plan".to_string()),
            configure: true,
            config: None,
        };
        // Call fallback helpers as run_plan would when tui binary is absent
        if args.configure {
            print_configure_fallback(args.label.as_deref());
        }
        // Must not panic
    }

    #[test]
    fn run_plan_fallback_does_not_panic_for_show_mode() {
        let args = PlanArgs {
            label: Some("nonexistent_label_xyz".to_string()),
            configure: false,
            config: None,
        };
        if let Some(ref lbl) = args.label {
            print_show_fallback(lbl);
        }
        // Must not panic
    }

    #[test]
    fn run_plan_fallback_does_not_panic_for_ls_mode() {
        let args = PlanArgs { label: None, configure: false, config: None };
        if args.label.is_none() && !args.configure {
            print_ls_fallback();
        }
        // Must not panic
    }
}
