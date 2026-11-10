//! In-memory verification instructions. No commands execute here.
use serde::{Deserialize, Serialize};
/// Record of an external analyzer script used during analysis.
/// Stored in manifest.json to support independent reproducibility checks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalAnalyzerRecord {
    /// Relative path or basename of the script
    pub script: String,
    /// SHA-256 of the script file at time of run
    pub sha256: String,
    /// Command template with {assignments_json} and {output_dir} placeholders
    pub command: String,
}

/// Generate the machine-readable verification command from a plan manifest.
///
/// Board amendments:
/// - No local file paths — only relative paths and BISECT CLI flags
/// - Must include --seed for reproducibility
/// - Must include sha256sum step for external analyzer scripts (board amendment R3)
/// - Format: a shell script fragment that can be embedded in audit.json
pub fn generate_verification_command(
    label: &str,
    state_code: &str,
    year: &str,
    seed: Option<i64>,
    chamber: &str,
    num_districts: impl std::fmt::Display,
    adjacency_file: &str,
    adjacency_sha256: &str,
    tiger_source_url: &str,
    tiger_sha256: Option<&str>,
    external_analyzers: &[ExternalAnalyzerRecord],
) -> String {
    let seed_str = seed
        .map(|s| format!(" --seed {}", s))
        .unwrap_or_else(|| " --seed '<unknown: seed not recorded in manifest>'".to_string());

    let tiger_check = if let Some(sha) = tiger_sha256 {
        format!(
            "\n# Verify TIGER source shapefile (download from Census.gov):\n# wget '{}'\n# sha256sum <downloaded_file>  # expected: {}",
            comment_text(tiger_source_url), comment_text(sha)
        )
    } else {
        format!("\n# Download TIGER source from:\n# {}", comment_text(tiger_source_url))
    };

    let adj_check = format!(
        "\n# Verify adjacency file:\n# sha256sum {}  # expected: {}\
        \n# If not present locally: bisect fetch --type adjacency --states {} --year {}",
        comment_text(adjacency_file), comment_text(adjacency_sha256), comment_text(state_code), comment_text(year)
    );

    let external_checks: String = external_analyzers
        .iter()
        .map(|rec| {
            format!(
                "\n# Verify external analyzer script:\n# sha256sum {}  # expected: {}\n# {}",
                comment_text(&rec.script), comment_text(&rec.sha256), comment_text(&rec.command)
            )
        })
        .collect();

    format!(
        "# Verification command for plan: {comment_label}\n\
        # Re-run with identical parameters to reproduce:\n\
        bisect state --state {state} --year {year} --chamber {chamber} --districts {num_districts} --label {label}{seed}\n\
        {tiger_check}\n\
        {adj_check}{external_checks}",
        label = shell_argument(label),
        comment_label = comment_text(label),
        state = shell_argument(state_code),
        year = shell_argument(year),
        chamber = shell_argument(chamber),
        num_districts = num_districts,
        seed = seed_str,
        tiger_check = tiger_check,
        adj_check = adj_check,
        external_checks = external_checks,
    )
}

/// Generate ONLY the executable lines from the verification command (no comment lines).
///
/// Returns a shell script fragment containing only the commands that can be run directly
/// (lines that do not start with `#` and are not empty).
///
/// Used by `BISECT export --format reproducibility-package` to write `verify.sh`.
pub fn generate_verification_script<C: std::fmt::Display>(manifest: &crate::manifest_schema::PlanManifestWithCounts<C>) -> String {
    if manifest.source_tool.as_deref()==Some("bisect-wasm") {return "# Replay requires the saved browser project with its exact configuration and prepared graph; no native CLI replay command is asserted.".into();}
    let Some(seed) = manifest.seed else {
        return "# Verification unavailable: seed not recorded in supplied manifest".into();
    };
    format!("bisect state --state {} --year {} --chamber {} --districts {} --label {} --seed {}",
        shell_argument(&manifest.state_code), shell_argument(&manifest.year),
        shell_argument(&manifest.chamber), manifest.num_districts,
        shell_argument(&manifest.label), seed)

}

/// High-level convenience wrapper using a PlanManifest.
pub fn generate_verification_command_from_manifest<C: std::fmt::Display>(
    manifest: &crate::manifest_schema::PlanManifestWithCounts<C>,
) -> String {
    if manifest.source_tool.as_deref()==Some("bisect-wasm") {return generate_verification_script(manifest);}
    generate_verification_command(
        &manifest.label,
        &manifest.state_code,
        &manifest.year,
        manifest.seed,
        &manifest.chamber,
        &manifest.num_districts,
        &manifest.adjacency_file,
        &manifest.adjacency_sha256,
        &manifest.tiger_source_url,
        manifest.tiger_sha256.as_deref(),
        &[],
    )
}


/// POSIX shell literal; embedded quotes and newlines remain within one argument.
fn shell_argument(value: &str) -> String {
    format!("'{}'", value.replace("'", "'\"'\"'"))
}

fn comment_text(value: &str) -> String {
    value.replace('\r', "\\r").replace('\n', "\\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn supplied_shell_syntax_is_one_literal_argument() {
        let manifest = crate::PlanManifest { seed:Some(42), label:"x'; $(touch bad)\n# embedded".into(), state_code:"WA".into(), year:"2020".into(), chamber:"house".into(), num_districts:2, ..Default::default() };
        assert_eq!(generate_verification_script(&manifest), "bisect state --state 'WA' --year '2020' --chamber 'house' --districts 2 --label 'x'\"'\"'; $(touch bad)\n# embedded' --seed 42");
    }
    #[test]
    fn missing_seed_never_generates_executable_guess() {
        assert_eq!(generate_verification_script(&crate::PlanManifest::default()), "# Verification unavailable: seed not recorded in supplied manifest");
    }
}
