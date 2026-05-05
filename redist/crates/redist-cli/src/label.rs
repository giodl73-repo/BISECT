/// `label.rs` — Label-based path convention functions.
///
/// A **label** is the user-facing name for a run (`official_proposal`, `vt_test`).
/// Every `redist` command resolves its input/output directories from the label alone
/// through the fixed conventions encoded here.  No path arguments are needed in
/// normal use.
///
/// Directory layout (Spec 7 §2.1):
/// ```text
/// runs/{label}/              ← build outputs
/// analysis/{label}/          ← analysis outputs
/// reports/{label}/           ← report outputs
///
/// runs/{label}/index.json    ← build index
/// analysis/{label}/index.json
/// reports/{label}/index.json
///
/// runs/{label}/{year}/
/// analysis/{label}/{year}/
/// reports/{label}/{year}/
///
/// runs/{label}/{year}/{state_name}/
/// analysis/{label}/{year}/{state_name}/
/// ```

use std::path::PathBuf;

// ── Validation ────────────────────────────────────────────────────────────────

/// Reserved names that cannot be used as labels (they are the output directories
/// themselves, so a label with one of these names would collide with the layout).
const RESERVED: &[&str] = &["runs", "analysis", "reports", ".redist", "configs"];

/// Validate a label name.
///
/// A valid label:
/// - is non-empty
/// - contains only `[a-zA-Z0-9_-]`
/// - does not start with `'.'`
/// - is not one of the [`RESERVED`] names
///
/// Returns `Ok(())` on success or `Err(human-readable message)` on failure.
pub fn validate_label_name(label: &str) -> Result<(), String> {
    if label.is_empty() {
        return Err("label must not be empty".to_string());
    }

    if label.starts_with('.') {
        return Err(format!(
            "label '{label}' must not start with '.'"
        ));
    }

    if let Some(bad) = label
        .chars()
        .find(|c| !c.is_ascii_alphanumeric() && *c != '_' && *c != '-')
    {
        return Err(format!(
            "label '{label}' contains invalid character '{bad}'; \
             only [a-zA-Z0-9_-] are allowed"
        ));
    }

    if RESERVED.contains(&label) {
        return Err(format!(
            "'{label}' is a reserved name and cannot be used as a label"
        ));
    }

    Ok(())
}

// ── Top-level directory paths ─────────────────────────────────────────────────

/// `runs/{label}/`
pub fn runs_dir(label: &str) -> PathBuf {
    PathBuf::from("runs").join(label)
}

/// `analysis/{label}/`
pub fn analysis_dir(label: &str) -> PathBuf {
    PathBuf::from("analysis").join(label)
}

/// `reports/{label}/`
pub fn reports_dir(label: &str) -> PathBuf {
    PathBuf::from("reports").join(label)
}

// ── Year-level directory paths ────────────────────────────────────────────────

/// `runs/{label}/{year}/`
pub fn year_runs_dir(label: &str, year: &str) -> PathBuf {
    runs_dir(label).join(year)
}

/// `analysis/{label}/{year}/`
pub fn year_analysis_dir(label: &str, year: &str) -> PathBuf {
    analysis_dir(label).join(year)
}

/// `reports/{label}/{year}/`
pub fn year_reports_dir(label: &str, year: &str) -> PathBuf {
    reports_dir(label).join(year)
}

// ── State-level directory paths ───────────────────────────────────────────────

/// `runs/{label}/{year}/{state_name}/`
pub fn state_runs_dir(label: &str, year: &str, state_name: &str) -> PathBuf {
    year_runs_dir(label, year).join(state_name)
}

/// `analysis/{label}/{year}/{state_name}/`
pub fn state_analysis_dir(label: &str, year: &str, state_name: &str) -> PathBuf {
    year_analysis_dir(label, year).join(state_name)
}

// ── Index file paths ──────────────────────────────────────────────────────────

/// Path to the build index file: `runs/{label}/index.json`
pub fn build_index_path(label: &str) -> PathBuf {
    runs_dir(label).join("index.json")
}

/// Path to the analysis index file: `analysis/{label}/index.json`
pub fn analysis_index_path(label: &str) -> PathBuf {
    analysis_dir(label).join("index.json")
}

/// Path to the report index file: `reports/{label}/index.json`
pub fn report_index_path(label: &str) -> PathBuf {
    reports_dir(label).join("index.json")
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── validate_label_name — valid cases ──────────────────────────────────────

    #[test]
    fn valid_alphanumeric() {
        assert!(validate_label_name("abc123").is_ok());
    }

    #[test]
    fn valid_underscores() {
        assert!(validate_label_name("official_proposal").is_ok());
    }

    #[test]
    fn valid_hyphens() {
        assert!(validate_label_name("senate-draft2").is_ok());
    }

    #[test]
    fn valid_mixed() {
        assert!(validate_label_name("vt_test-2020").is_ok());
    }

    #[test]
    fn valid_single_char() {
        assert!(validate_label_name("x").is_ok());
    }

    // ── validate_label_name — invalid: empty ──────────────────────────────────

    #[test]
    fn invalid_empty() {
        let err = validate_label_name("").unwrap_err();
        assert!(err.contains("empty"), "expected 'empty' in: {err}");
    }

    // ── validate_label_name — invalid: reserved names ─────────────────────────

    #[test]
    fn invalid_reserved_runs() {
        let err = validate_label_name("runs").unwrap_err();
        assert!(err.contains("reserved"), "expected 'reserved' in: {err}");
    }

    #[test]
    fn invalid_reserved_analysis() {
        let err = validate_label_name("analysis").unwrap_err();
        assert!(err.contains("reserved"), "expected 'reserved' in: {err}");
    }

    #[test]
    fn invalid_reserved_reports() {
        assert!(validate_label_name("reports").unwrap_err().contains("reserved"));
    }

    #[test]
    fn invalid_reserved_redist() {
        assert!(validate_label_name(".redist").unwrap_err().contains("'.'"));
    }

    #[test]
    fn invalid_reserved_configs() {
        assert!(validate_label_name("configs").unwrap_err().contains("reserved"));
    }

    // ── validate_label_name — invalid: path separators ───────────────────────

    #[test]
    fn invalid_forward_slash() {
        let err = validate_label_name("foo/bar").unwrap_err();
        assert!(err.contains('/'), "expected '/' mentioned in: {err}");
    }

    #[test]
    fn invalid_backslash() {
        let err = validate_label_name("foo\\bar").unwrap_err();
        assert!(err.contains('\\'), "expected backslash mentioned in: {err}");
    }

    #[test]
    fn invalid_space() {
        let err = validate_label_name("my label").unwrap_err();
        assert!(err.contains(' '), "expected space mentioned in: {err}");
    }

    // ── validate_label_name — invalid: starts with dot ───────────────────────

    #[test]
    fn invalid_starts_with_dot() {
        let err = validate_label_name(".hidden").unwrap_err();
        assert!(err.contains("'.'"), "expected dot rule in: {err}");
    }

    // ── runs_dir ──────────────────────────────────────────────────────────────

    #[test]
    fn runs_dir_basic() {
        assert_eq!(
            runs_dir("official_proposal"),
            PathBuf::from("runs/official_proposal")
        );
    }

    // ── analysis_dir ──────────────────────────────────────────────────────────

    #[test]
    fn analysis_dir_basic() {
        assert_eq!(
            analysis_dir("official_proposal"),
            PathBuf::from("analysis/official_proposal")
        );
    }

    // ── reports_dir ───────────────────────────────────────────────────────────

    #[test]
    fn reports_dir_basic() {
        assert_eq!(
            reports_dir("official_proposal"),
            PathBuf::from("reports/official_proposal")
        );
    }

    // ── year_runs_dir ─────────────────────────────────────────────────────────

    #[test]
    fn year_runs_dir_basic() {
        assert_eq!(
            year_runs_dir("x", "2020"),
            PathBuf::from("runs/x/2020")
        );
    }

    #[test]
    fn year_analysis_dir_basic() {
        assert_eq!(
            year_analysis_dir("x", "2020"),
            PathBuf::from("analysis/x/2020")
        );
    }

    #[test]
    fn year_reports_dir_basic() {
        assert_eq!(
            year_reports_dir("x", "2020"),
            PathBuf::from("reports/x/2020")
        );
    }

    // ── state-level paths ─────────────────────────────────────────────────────

    #[test]
    fn state_runs_dir_basic() {
        assert_eq!(
            state_runs_dir("official_proposal", "2020", "vermont"),
            PathBuf::from("runs/official_proposal/2020/vermont")
        );
    }

    #[test]
    fn state_analysis_dir_basic() {
        assert_eq!(
            state_analysis_dir("official_proposal", "2020", "vermont"),
            PathBuf::from("analysis/official_proposal/2020/vermont")
        );
    }

    // ── index paths ───────────────────────────────────────────────────────────

    #[test]
    fn build_index_path_basic() {
        assert_eq!(
            build_index_path("official_proposal"),
            PathBuf::from("runs/official_proposal/index.json")
        );
    }

    #[test]
    fn analysis_index_path_basic() {
        assert_eq!(
            analysis_index_path("official_proposal"),
            PathBuf::from("analysis/official_proposal/index.json")
        );
    }

    #[test]
    fn report_index_path_basic() {
        assert_eq!(
            report_index_path("official_proposal"),
            PathBuf::from("reports/official_proposal/index.json")
        );
    }
}
