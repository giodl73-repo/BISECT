use serde::{Deserialize, Serialize};
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserves_native_console_parser() {
        let r = parse_output(
            "manifest.json",
            "Jaccard similarity: 0.9987\n",
            "[PASS] verified\n",
        );
        assert!(r.passed);
        assert_eq!(r.jaccard, 0.9987);
        assert_eq!(r.manifest_path_used, "manifest.json");
        let r = parse_output("manifest.json", "", "[FAIL] changed\n");
        assert!(!r.passed);
        assert_eq!(r.fail_reason.as_deref(), Some("[FAIL] changed"));
    }
    #[test]
    fn console_substrings_are_not_execution_evidence() {
        assert!(
            parse_output(
                "manifest.json",
                "Equivalent command: --label PASS_label\n",
                "[DRY RUN] Command not executed.\n"
            )
            .passed
        );
        let dry = parse_output("manifest.json", "", "[DRY RUN] Command not executed.\n");
        assert!(!dry.passed);
        assert_eq!(dry.likely_causes.len(), 3);
    }
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct VerifyResult {
    pub passed: bool,
    pub jaccard: f64,
    pub label: String,
    pub state_code: String,
    pub year: String,
    pub binary_match: bool,
    pub metis_version: String,
    pub adjacency_match: bool,
    pub tiger_match: bool,
    pub seed_recorded: bool,
    pub fail_reason: Option<String>,
    pub likely_causes: Vec<String>,
    pub manifest_path_used: String,
}

/// Legacy TUI console parser, retained for native compatibility, not trusted browser evidence.
pub fn parse_output(path: &str, stdout: &str, stderr: &str) -> VerifyResult {
    let mut vr = VerifyResult::default();
    vr.manifest_path_used = path.to_string();
    let combined = format!("{stdout}{stderr}");
    vr.passed = combined.contains("PASS") && !combined.contains("FAIL");

    // Parse Jaccard score: "Jaccard similarity: 0.9987"
    for line in combined.lines() {
        if line.contains("Jaccard similarity") || line.contains("Jaccard:") {
            if let Some(val) = line
                .split_whitespace()
                .last()
                .and_then(|s| s.parse::<f64>().ok())
            {
                vr.jaccard = val;
            }
        }
        if line.contains("FAIL") && !line.contains("PASS") {
            vr.passed = false;
            vr.fail_reason = Some(line.trim().to_string());
        }
    }

    if !vr.passed && vr.fail_reason.is_none() {
        vr.fail_reason = Some("Verification failed — check manifest path and binary".to_string());
        vr.likely_causes = vec![
            "METIS version differs from manifest".to_string(),
            "Different adjacency file (check sha256)".to_string(),
            "Run without --seed (add seed to reproduce)".to_string(),
        ];
    }
    vr
}
