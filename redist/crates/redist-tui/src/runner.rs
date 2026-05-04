/// STATUS subprocess integration — parse progress messages from redist subprocesses.
///
/// The Python pipeline emits STATUS:{position}:{message} lines to stdout.
/// This module parses those lines and maps them to RunProgress updates.

use crate::app::RunProgress;

/// Parse a STATUS:{position}:{message} line.
/// Returns Some(message) if the line matches the protocol, None otherwise.
pub fn parse_status_line(line: &str) -> Option<String> {
    // Line must start with "STATUS:"
    let rest = line.strip_prefix("STATUS:")?;
    // Find the second colon separating position from message
    let colon_pos = rest.find(':')?;
    // Position must be parseable as an integer (but we don't validate the value further)
    let pos_str = &rest[..colon_pos];
    let _position: i64 = pos_str.parse().ok()?;
    // Message is everything after position:
    let message = &rest[colon_pos + 1..];
    Some(message.to_string())
}

/// Update RunProgress from a parsed STATUS message.
///
/// Recognizes patterns emitted by run_single_state:
/// - "loading adjacency" → no numeric change, just marks activity
/// - "bisect depth {n}: {done}/{total}" → updates depths vec
/// - "districts assigned: {n}/{total}" → updates assigned counts
/// - "balance OK" → sets balance_ok = true
/// - "elapsed: {secs}s" → updates elapsed_secs
pub fn update_progress_from_message(progress: &mut RunProgress, msg: &str) {
    let msg_lower = msg.to_lowercase();

    // Bisection depth progress: "bisect depth 2: 4/8"
    if msg_lower.contains("bisect depth") {
        if let Some((done, total)) = parse_fraction_in_msg(msg) {
            if let Some(depth) = parse_depth_index(msg) {
                // Grow depths vec as needed
                while progress.depths.len() <= depth {
                    progress.depths.push((0, 0));
                }
                progress.depths[depth] = (done, total);
            }
        }
        return;
    }

    // Districts assigned: "districts assigned: 218/435"
    if msg_lower.contains("districts assigned") || msg_lower.contains("assigned") {
        if let Some((done, total)) = parse_fraction_in_msg(msg) {
            progress.districts_assigned = done;
            progress.districts_total = total;
        }
        return;
    }

    // Balance: "balance OK"
    if msg_lower.contains("balance ok") || msg_lower.contains("balance_ok") {
        progress.balance_ok = true;
        return;
    }

    // Elapsed time: "elapsed: 42s"
    if msg_lower.contains("elapsed") {
        if let Some(secs) = parse_elapsed_secs(msg) {
            progress.elapsed_secs = secs;
        }
    }
}

/// Parse "N/M" fraction anywhere in a message string.
fn parse_fraction_in_msg(msg: &str) -> Option<(usize, usize)> {
    for token in msg.split_whitespace() {
        if let Some(slash) = token.find('/') {
            let lhs = token[..slash].trim_matches(|c: char| !c.is_ascii_digit());
            let rhs = token[slash + 1..].trim_matches(|c: char| !c.is_ascii_digit());
            if let (Ok(done), Ok(total)) = (lhs.parse::<usize>(), rhs.parse::<usize>()) {
                return Some((done, total));
            }
        }
    }
    None
}

/// Parse "depth N" from message — returns zero-based depth index.
fn parse_depth_index(msg: &str) -> Option<usize> {
    let lower = msg.to_lowercase();
    let depth_pos = lower.find("depth")?;
    let after = &msg[depth_pos + 5..].trim_start();
    // Read digits
    let end = after.find(|c: char| !c.is_ascii_digit()).unwrap_or(after.len());
    let num_str = &after[..end];
    let n: usize = num_str.parse().ok()?;
    Some(n.saturating_sub(1)) // convert 1-based to 0-based
}

/// Parse elapsed seconds from "elapsed: 42s" or similar.
fn parse_elapsed_secs(msg: &str) -> Option<u64> {
    // Look for a number followed by optional 's'
    for token in msg.split_whitespace() {
        let digits: String = token.chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        if !digits.is_empty() {
            if let Ok(n) = digits.parse::<u64>() {
                return Some(n);
            }
        }
    }
    None
}

/// Spawn `redist state` subprocess in a background thread.
/// Returns immediately. Progress is written to `log_lines` and `done`.
/// done: None = still running, Some(true) = success, Some(false) = failed.
pub fn spawn_redist_state(
    form: &crate::app::RunForm,
    log_lines: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
    done: std::sync::Arc<std::sync::Mutex<Option<bool>>>,
) {
    // Find the redist binary alongside us
    let redist_bin = std::env::current_exe()
        .map(|mut p| { p.set_file_name(if cfg!(windows) { "redist.exe" } else { "redist" }); p })
        .unwrap_or_else(|_| std::path::PathBuf::from("redist"));

    let mut args = vec![
        "state".to_string(),
        "--state".to_string(), form.location.clone(),
        "--chamber".to_string(), form.chamber.clone(),
        "--year".to_string(), form.year.clone(),
        "--resolution".to_string(), form.resolution.clone(),
        "--version".to_string(), form.version.clone(),
        "--force".to_string(),
    ];
    if !form.seed.is_empty() {
        args.push("--seed".to_string());
        args.push(form.seed.clone());
    }
    if !form.label.is_empty() {
        args.push("--label".to_string());
        args.push(form.label.clone());
    }
    if !form.balance_tol.is_empty() {
        args.push("--balance-tolerance".to_string());
        args.push(form.balance_tol.clone());
    }

    std::thread::spawn(move || {
        use std::io::BufRead;
        let result = std::process::Command::new(&redist_bin)
            .args(&args)
            .stderr(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn();

        match result {
            Err(e) => {
                let mut lines = log_lines.lock().unwrap();
                lines.push(format!("ERROR: failed to launch redist: {e}"));
                *done.lock().unwrap() = Some(false);
            }
            Ok(mut child) => {
                // Read stderr (STATUS: lines come here)
                if let Some(stderr) = child.stderr.take() {
                    for line in std::io::BufReader::new(stderr).lines().map_while(Result::ok) {
                        log_lines.lock().unwrap().push(line);
                    }
                }
                let success = child.wait().map(|s| s.success()).unwrap_or(false);
                *done.lock().unwrap() = Some(success);
            }
        }
    });
}

/// Run `redist verify --manifest {path}` and return VerifyResult.
/// This blocks — call from a background thread.
pub fn run_verify_subprocess(manifest_path: &str) -> crate::app::VerifyResult {
    let redist_bin = std::env::current_exe()
        .map(|mut p| { p.set_file_name(if cfg!(windows) { "redist.exe" } else { "redist" }); p })
        .unwrap_or_else(|_| std::path::PathBuf::from("redist"));

    let result = std::process::Command::new(&redist_bin)
        .args(["verify", "--manifest", manifest_path, "--dry-run"])
        .stderr(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .output();

    let mut vr = crate::app::VerifyResult::default();
    vr.manifest_path_used = manifest_path.to_string();

    match result {
        Err(e) => {
            vr.fail_reason = Some(format!("Failed to launch redist: {e}"));
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let combined = format!("{stdout}{stderr}");

            vr.passed = combined.contains("PASS") && !combined.contains("FAIL");

            // Parse Jaccard score: "Jaccard similarity: 0.9987"
            for line in combined.lines() {
                if line.contains("Jaccard similarity") || line.contains("Jaccard:") {
                    if let Some(val) = line.split_whitespace().last()
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
        }
    }
    vr
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_status_line_valid() {
        let result = parse_status_line("STATUS:0:loading adjacency");
        assert_eq!(result, Some("loading adjacency".to_string()));
    }

    #[test]
    fn test_parse_status_line_not_status() {
        let result = parse_status_line("some other log line");
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_status_line_empty() {
        let result = parse_status_line("");
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_status_line_position_999() {
        let result = parse_status_line("STATUS:999:balance OK");
        assert_eq!(result, Some("balance OK".to_string()));
    }

    #[test]
    fn test_parse_status_line_colons_in_message() {
        // Message itself may contain colons — only the first colon after position is separator
        let result = parse_status_line("STATUS:2:WA: bisecting into 10 districts");
        assert_eq!(result, Some("WA: bisecting into 10 districts".to_string()));
    }

    #[test]
    fn test_update_progress_balance_ok() {
        let mut progress = RunProgress::default();
        update_progress_from_message(&mut progress, "balance OK");
        assert!(progress.balance_ok);
    }

    #[test]
    fn test_update_progress_districts_assigned() {
        let mut progress = RunProgress::default();
        update_progress_from_message(&mut progress, "districts assigned: 218/435");
        assert_eq!(progress.districts_assigned, 218);
        assert_eq!(progress.districts_total, 435);
    }

    // ── Additional runner tests ───────────────────────────────────────────────

    #[test]
    fn test_parse_status_line_negative_position_rejected() {
        // Negative position is parseable as i64, so it returns a message
        let result = parse_status_line("STATUS:-1:some message");
        assert_eq!(result, Some("some message".to_string()));
    }

    #[test]
    fn test_parse_status_line_non_numeric_position_rejected() {
        let result = parse_status_line("STATUS:abc:some message");
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_status_line_only_prefix_no_colon() {
        let result = parse_status_line("STATUS:");
        // No second colon → None
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_status_line_message_can_be_empty() {
        let result = parse_status_line("STATUS:0:");
        assert_eq!(result, Some(String::new()));
    }

    #[test]
    fn test_update_progress_balance_ok_case_insensitive() {
        let mut progress = RunProgress::default();
        update_progress_from_message(&mut progress, "BALANCE OK — convergence reached");
        assert!(progress.balance_ok, "balance_ok must be set regardless of case");
    }

    #[test]
    fn test_update_progress_elapsed_parses_number() {
        let mut progress = RunProgress::default();
        update_progress_from_message(&mut progress, "elapsed: 87s");
        assert_eq!(progress.elapsed_secs, 87);
    }

    #[test]
    fn test_update_progress_elapsed_no_trailing_s() {
        let mut progress = RunProgress::default();
        update_progress_from_message(&mut progress, "elapsed 120");
        assert_eq!(progress.elapsed_secs, 120);
    }

    #[test]
    fn test_update_progress_bisect_depth_grows_vec() {
        let mut progress = RunProgress::default();
        update_progress_from_message(&mut progress, "bisect depth 3: 4/8");
        // depth index 2 (1-based 3 → 0-based 2) — vec grows to at least 3
        assert!(progress.depths.len() >= 3);
        assert_eq!(progress.depths[2], (4, 8));
    }

    #[test]
    fn test_update_progress_bisect_depth_1_is_index_0() {
        let mut progress = RunProgress::default();
        update_progress_from_message(&mut progress, "bisect depth 1: 1/2");
        assert!(!progress.depths.is_empty());
        assert_eq!(progress.depths[0], (1, 2));
    }

    #[test]
    fn test_update_progress_unrecognised_message_is_noop() {
        let mut progress = RunProgress::default();
        update_progress_from_message(&mut progress, "some random log output");
        // No fields should have changed
        assert!(!progress.balance_ok);
        assert_eq!(progress.districts_assigned, 0);
        assert_eq!(progress.elapsed_secs, 0);
    }

    #[test]
    fn test_update_progress_districts_one_each() {
        let mut progress = RunProgress::default();
        update_progress_from_message(&mut progress, "assigned 1/1");
        assert_eq!(progress.districts_assigned, 1);
        assert_eq!(progress.districts_total, 1);
    }

    #[test]
    fn test_parse_status_preserves_internal_colons() {
        // Multiple colons in message
        let result = parse_status_line("STATUS:5:a:b:c");
        assert_eq!(result, Some("a:b:c".to_string()));
    }
}
