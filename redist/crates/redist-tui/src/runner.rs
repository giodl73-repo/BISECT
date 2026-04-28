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
}
