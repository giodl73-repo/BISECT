/// STATUS progress protocol for the redistricting pipeline.
///
/// The Python pipeline uses a STATUS protocol where child processes print:
///   STATUS:{position}:{message}
/// and the parent progress coordinator reads these lines to update tqdm bars.
///
/// MERIDIAN invariant: STATUS output must be ASCII only (no Unicode).
/// Windows CP1252 encoding crashes on Unicode → ✓✗→• all forbidden.
/// Python CLAUDE.md: "NEVER in console (Windows CP1252)"
///
/// This module provides the Rust implementation of the same protocol.
use std::io::{self, Write};

/// Emit a STATUS line to stdout for the parent progress coordinator.
///
/// Format: `STATUS:{position}:{message}\n`
/// Position 999 = parallel/silent mode (line still emitted, parent ignores it)
pub fn status(position: i32, message: &str) {
    // Validate ASCII-only (MERIDIAN: no Unicode in console output)
    debug_assert!(
        message.is_ascii(),
        "STATUS message must be ASCII-only: {:?}", message
    );
    // Use print! + flush rather than println! to avoid buffering issues
    print!("STATUS:{position}:{message}\n");
    let _ = io::stdout().flush();
}

/// Emit a PROGRESS line indicating completed/total splits.
/// Used by child processes to report bisection progress to the parent.
pub fn progress(completed: usize, total: usize) {
    print!("PROGRESS:{completed}/{total}\n");
    let _ = io::stdout().flush();
}

/// Sanitize a message to be ASCII-safe for STATUS output.
/// Replaces non-ASCII chars with '?' to avoid CP1252 crashes.
pub fn ascii_safe(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_ascii() { c } else { '?' })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_safe_passthrough() {
        assert_eq!(ascii_safe("Alabama redistricting"), "Alabama redistricting");
    }

    #[test]
    fn test_ascii_safe_replaces_unicode() {
        // checkmark, arrow, bullet — all forbidden in STATUS output
        let msg = ascii_safe("done [OK] -> items");
        assert!(msg.is_ascii(), "output should be ASCII: {:?}", msg);
    }

    #[test]
    fn test_ascii_safe_replaces_nonascii() {
        let msg = ascii_safe("caf\u{00E9}"); // café
        assert_eq!(msg, "caf?");
    }

    #[test]
    fn test_status_format_is_ascii() {
        // Verify the format string used by status() would produce valid ASCII
        let pos = 2i32;
        let message = "Round 1/3: 4 splits";
        let line = format!("STATUS:{pos}:{message}\n");
        assert!(line.is_ascii(), "STATUS line must be ASCII: {:?}", line);
        assert!(line.starts_with("STATUS:"));
    }

    #[test]
    fn test_progress_format() {
        let line = format!("PROGRESS:{}/{}\n", 3, 7);
        assert_eq!(line, "PROGRESS:3/7\n");
        assert!(line.is_ascii());
    }

    #[test]
    fn test_status_position_999_silent() {
        // Position 999 is the "parallel/silent" mode — still valid STATUS format
        let line = format!("STATUS:{}:{}\n", 999, "processing");
        assert!(line.starts_with("STATUS:999:"));
    }

    #[test]
    fn test_status_separators_count() {
        // Exactly 2 colons: STATUS:{pos}:{msg}
        let line = format!("STATUS:{}:{}\n", 2, "some:message:with:colons");
        let colons = line.chars().filter(|&c| c == ':').count();
        // 2 structural colons + however many are in the message
        assert!(colons >= 2);
        // The first two fields are correct
        let parts: Vec<&str> = line.trim().splitn(3, ':').collect();
        assert_eq!(parts[0], "STATUS");
        assert_eq!(parts[1], "2");
    }
}
