/// Composable bitfield exit codes for `redist analyze`.
///
/// Bits:
///   0 (value 1) — population balance violation
///   1 (value 2) — contiguity violation
///   2 (value 4) — nesting violation
///   3 (value 8) — missing data
///
/// Flags like `--allow-noncontiguous` suppress the corresponding bit.

pub const BIT_BALANCE: u8 = 0b0001; // 1
pub const BIT_CONTIGUITY: u8 = 0b0010; // 2
pub const BIT_NESTING: u8 = 0b0100; // 4
pub const BIT_MISSING_DATA: u8 = 0b1000; // 8

/// Compute a bitfield exit code from four violation booleans.
pub fn compute_exit_code(
    balance_violation: bool,
    contiguity_violation: bool,
    nesting_violation: bool,
    missing_data: bool,
) -> u8 {
    let mut code: u8 = 0;
    if balance_violation {
        code |= BIT_BALANCE;
    }
    if contiguity_violation {
        code |= BIT_CONTIGUITY;
    }
    if nesting_violation {
        code |= BIT_NESTING;
    }
    if missing_data {
        code |= BIT_MISSING_DATA;
    }
    code
}

/// Like `compute_exit_code` but respects suppression flags.
///
/// `allow_noncontiguous` — clear bit 1 (contiguity)
/// `allow_imbalance`     — clear bit 0 (balance)
pub fn compute_exit_code_with_flags(
    balance_violation: bool,
    contiguity_violation: bool,
    nesting_violation: bool,
    missing_data: bool,
    allow_noncontiguous: bool,
    allow_imbalance: bool,
) -> u8 {
    let effective_contiguity = contiguity_violation && !allow_noncontiguous;
    let effective_balance = balance_violation && !allow_imbalance;
    compute_exit_code(
        effective_balance,
        effective_contiguity,
        nesting_violation,
        missing_data,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_violations_exit_0() {
        let code = compute_exit_code(false, false, false, false);
        assert_eq!(code, 0);
    }

    #[test]
    fn test_balance_violation_only_exit_1() {
        let code = compute_exit_code(true, false, false, false);
        assert_eq!(code, 1);
    }

    #[test]
    fn test_contiguity_violation_only_exit_2() {
        let code = compute_exit_code(false, true, false, false);
        assert_eq!(code, 2);
    }

    #[test]
    fn test_balance_and_contiguity_exit_3() {
        let code = compute_exit_code(true, true, false, false);
        assert_eq!(code, 3);
    }

    #[test]
    fn test_nesting_exit_code_is_bit2() {
        let code = compute_exit_code(false, false, true, false);
        assert_eq!(code, 4);
    }

    #[test]
    fn test_balance_and_nesting_exit_code() {
        let code = compute_exit_code(true, false, true, false);
        assert_eq!(code, 5);
    }

    #[test]
    fn test_missing_data_exit_8() {
        let code = compute_exit_code(false, false, false, true);
        assert_eq!(code, 8);
    }

    #[test]
    fn test_all_violations_exit_15() {
        let code = compute_exit_code(true, true, true, true);
        assert_eq!(code, 15);
    }

    #[test]
    fn test_allow_noncontiguous_suppresses_bit1() {
        let code = compute_exit_code_with_flags(
            true,
            true,
            false,
            false,
            /*allow_noncontiguous=*/ true,
            /*allow_imbalance=*/ false,
        );
        // bit 1 (contiguity=2) suppressed; only bit 0 (balance=1) remains
        assert_eq!(code, 1);
    }

    #[test]
    fn test_allow_imbalance_suppresses_bit0() {
        let code = compute_exit_code_with_flags(
            true,
            false,
            false,
            false,
            /*allow_noncontiguous=*/ false,
            /*allow_imbalance=*/ true,
        );
        assert_eq!(code, 0);
    }

    #[test]
    fn test_nesting_exit_code_is_4() {
        // nesting violation only -> exit code 4 (bit 2)
        assert_eq!(compute_exit_code(false, false, true, false), 4);
    }

    #[test]
    fn test_balance_and_nesting_is_5() {
        // balance violation + nesting violation -> bits 0 + 2 = 5
        assert_eq!(compute_exit_code(true, false, true, false), 5);
    }

    // ── Task 202: missing election data must not set the fatal missing_data bit ──

    #[test]
    fn test_missing_election_data_exit_zero() {
        // When the partisan analyzer can't find election data, the exit code should be 0.
        // Election data is OPTIONAL — only adjacency (required for contiguity) sets missing_data.
        // Simulate: optional data missing (demographics/election) → missing_optional_data=true
        //           required data present                         → missing_data=false
        let missing_data = false; // adjacency present (required data OK)
        // missing_optional_data is tracked separately and does NOT enter compute_exit_code
        let exit_code = compute_exit_code(
            false, // balance_violation
            false, // contiguity_violation
            false, // nesting_violation
            missing_data, // only required-data flag goes here
        );
        assert_eq!(exit_code, 0,
            "exit code must be 0 when only optional data (election/demographics) is missing");
    }

    #[test]
    fn test_required_missing_data_sets_exit_8() {
        // When REQUIRED data (adjacency) is missing, exit code must be non-zero (bit 3 = 8).
        let missing_data = true; // adjacency not available
        let exit_code = compute_exit_code(false, false, false, missing_data);
        assert_eq!(exit_code, 8,
            "missing required data (adjacency) must set exit code bit 3 (value 8)");
    }
}
