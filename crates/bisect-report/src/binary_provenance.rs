//! Advisory executable hash comparison used by native Verify.
use serde::Serialize;

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Status { NotRecorded, Skipped, Unavailable, Match, Mismatch }

pub fn assess(recorded: &str, skip: bool, current: Option<&str>) -> Status {
    if recorded.is_empty() || recorded.starts_with("(not") { return Status::NotRecorded; }
    if skip { return Status::Skipped; }
    match current { None => Status::Unavailable, Some(value) if value == recorded => Status::Match, Some(_) => Status::Mismatch }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_sentinels_precede_skip_and_comparison_is_literal() {
        assert_eq!(assess("", true, None), Status::NotRecorded);
        assert_eq!(assess("(not available)", true, Some("abc")), Status::NotRecorded);
        assert_eq!(assess("abc", true, Some("abc")), Status::Skipped);
        assert_eq!(assess("abc", false, None), Status::Unavailable);
        assert_eq!(assess("abc", false, Some("abc")), Status::Match);
        assert_eq!(assess("ABC", false, Some("abc")), Status::Mismatch);
    }
}
