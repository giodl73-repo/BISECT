//! Native final-plan population retry policy shared with portable execution.
pub const MAX_BALANCE_RETRIES: u32 = 50;
pub fn balance_retry_seed(base: Option<u64>, attempt: u32) -> Option<u64> {
    if attempt == 0 { base } else { Some(base.unwrap_or(0).wrapping_add(attempt as u64)) }
}
/// Native CLI wording for the last failed final-plan population assertion.
pub fn population_failure(tolerance_fraction:f64,error:&str)->String {
    format!("population balance violation (tolerance {:.1}%): {error}",tolerance_fraction*100.0)
}
/// Native per-state result context, including the CLI's ASCII-safe diagnostics.
pub fn state_failure(state:&str,error:&str)->String {
    format!("{state}: {}",error.chars().map(|c|if c.is_ascii(){c}else{'?'}).collect::<String>())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_failure_wording_preserves_state_and_ascii_policy(){
        assert_eq!(state_failure("IA",&population_failure(0.25,"tolerance ±0.250")),"IA: population balance violation (tolerance 25.0%): tolerance ?0.250");
    }
    #[test]
    fn optional_seed_and_wraparound() {
        assert_eq!(balance_retry_seed(None,0),None);
        assert_eq!(balance_retry_seed(None,1),Some(1));
        assert_eq!(balance_retry_seed(Some(0),0),Some(0));
        assert_eq!(balance_retry_seed(Some(u64::MAX),1),Some(0));
        assert_eq!(balance_retry_seed(Some(u64::MAX),50),Some(49));
    }
}
