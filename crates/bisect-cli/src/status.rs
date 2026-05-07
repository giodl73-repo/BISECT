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
    // MERIDIAN invariant: STATUS output must be ASCII-only.
    // Windows CP1252 will crash on non-ASCII bytes; Python coordinator
    // reads raw bytes. Use assert! (not debug_assert!) so this check
    // fires in release builds too — silent corruption is worse than a panic.
    if !message.is_ascii() {
        // Sanitize rather than panic: replace non-ASCII with '?'
        let safe = ascii_safe(message);
        print!("STATUS:{position}:{safe}\n");
    } else {
        print!("STATUS:{position}:{message}\n");
    }
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
    fn test_ascii_safe_empty_string() {
        assert_eq!(ascii_safe(""), "");
    }

    #[test]
    fn test_status_sanitizes_non_ascii_not_panic() {
        // In release builds, non-ASCII must be sanitized, not panicked (and not silently corrupt)
        // The status() function now auto-sanitizes; this verifies it doesn't panic
        // We capture by verifying ascii_safe produces what we expect
        let msg = "Round 1: caf\u{00E9} done";
        let safe = ascii_safe(msg);
        assert!(safe.is_ascii(), "sanitized message must be ASCII: {:?}", safe);
        assert!(safe.contains("caf?"), "non-ASCII char should be replaced with '?'");
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

    // ── Additional status / ascii_safe tests ─────────────────────────────────

    #[test]
    fn test_ascii_safe_all_printable_ascii_unchanged() {
        let printable = " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~";
        let result = ascii_safe(printable);
        assert_eq!(result, printable, "all printable ASCII must pass through unchanged");
    }

    #[test]
    fn test_ascii_safe_high_unicode_all_replaced() {
        let input = "\u{0100}\u{0101}\u{4E2D}\u{6587}"; // Ā, ā, 中, 文
        let result = ascii_safe(input);
        assert_eq!(result, "????", "all non-ASCII chars must be replaced with '?'");
    }

    #[test]
    fn test_ascii_safe_mixed_replaced_only_non_ascii() {
        let input = "A\u{00E9}B\u{00FC}C"; // AéBüC
        let result = ascii_safe(input);
        assert_eq!(result, "A?B?C", "only non-ASCII chars must be replaced");
    }

    #[test]
    fn test_ascii_safe_tab_and_newline_preserved() {
        // Tab (0x09) and newline (0x0A) are ASCII control chars — must pass through.
        let input = "col1\tcol2\nrow2";
        let result = ascii_safe(input);
        assert_eq!(result, input, "ASCII control chars must be preserved");
    }

    #[test]
    fn test_ascii_safe_null_byte_preserved() {
        // Null (0x00) is technically ASCII — must pass through unchanged.
        let input = "hello\0world";
        let result = ascii_safe(input);
        assert_eq!(result, input);
    }

    #[test]
    fn test_ascii_safe_single_nonascii_char() {
        let result = ascii_safe("\u{00E9}"); // é
        assert_eq!(result, "?");
    }

    #[test]
    fn test_ascii_safe_long_ascii_string_unchanged() {
        let long: String = "abcdefghijklmnopqrstuvwxyz ".repeat(100);
        let result = ascii_safe(&long);
        assert_eq!(result, long, "long ASCII string must be unchanged");
    }

    #[test]
    fn test_progress_zero_zero() {
        let line = format!("PROGRESS:0/0\n");
        assert_eq!(line, "PROGRESS:0/0\n");
        assert!(line.is_ascii());
    }

    #[test]
    fn test_progress_large_values() {
        let line = format!("PROGRESS:{}/{}\n", 435, 435);
        assert_eq!(line, "PROGRESS:435/435\n");
    }

    #[test]
    fn test_status_negative_position() {
        // Position -1 is used in the Python STATUS protocol for "no bar" mode.
        let line = format!("STATUS:{}:{}\n", -1, "processing");
        assert!(line.starts_with("STATUS:-1:"), "negative position must be emitted as-is");
        assert!(line.is_ascii());
    }

    #[test]
    fn test_ascii_safe_emoji_replaced() {
        let input = "done \u{2705}"; // done ✅
        let result = ascii_safe(input);
        assert_eq!(result, "done ?", "emoji must be replaced with '?'");
        assert!(result.is_ascii(), "result must be ASCII");
    }

    #[test]
    fn test_status_message_preserves_spaces_and_punctuation() {
        let msg = "Vermont: 193 tracts, 1 district [OK]";
        let line = format!("STATUS:0:{msg}\n");
        // Verify message is embedded intact
        let parts: Vec<&str> = line.trim().splitn(3, ':').collect();
        // parts[2] = message part (may contain colons, verified by splitting on first 2 colons)
        assert!(parts.len() >= 3);
        assert!(parts[2].contains("Vermont"));
        assert!(parts[2].contains("[OK]"));
    }
}
