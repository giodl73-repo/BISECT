/// Per-state constitutional split standard lookup table.
///
/// Covers WA, CA, TX, CO explicitly. All other states get a generic disclaimer.

#[derive(Debug, Clone, serde::Serialize)]
pub struct SplitStandard {
    pub state_code: String,
    pub legal_standard: String,
    /// Template filled with actual split count at runtime.
    pub compliance_assessment_template: String,
    pub disclaimer: String,
}

/// Return the split standard for the given two-letter state code.
/// Never returns `None` — unknown states get a generic entry.
pub fn get_split_standard(state_code: &str) -> Option<SplitStandard> {
    match state_code {
        "WA" => Some(SplitStandard {
            state_code: "WA".into(),
            legal_standard: "WA Const. Art. II §43 — counties shall be preserved where possible"
                .into(),
            compliance_assessment_template:
                "{n} splits present; no binding numerical limit under WMCA".into(),
            disclaimer: "Legal compliance determination requires counsel".into(),
        }),
        "CA" => Some(SplitStandard {
            state_code: "CA".into(),
            legal_standard: "CA Const. Art. XXI §2(d) — minimize county and city splits".into(),
            compliance_assessment_template:
                "{n} county splits, {m} city splits present".into(),
            disclaimer: "Legal compliance determination requires counsel".into(),
        }),
        "TX" => Some(SplitStandard {
            state_code: "TX".into(),
            legal_standard:
                "TX Const. Art. III §26 — counties preserved where practicable".into(),
            compliance_assessment_template: "{n} county splits present".into(),
            disclaimer: "Legal compliance determination requires counsel".into(),
        }),
        "CO" => Some(SplitStandard {
            state_code: "CO".into(),
            legal_standard: "CO Const. Art. V §47 — preserve political subdivisions".into(),
            compliance_assessment_template: "{n} county splits present".into(),
            disclaimer: "Legal compliance determination requires counsel".into(),
        }),
        _ => Some(SplitStandard {
            state_code: state_code.into(),
            legal_standard: format!(
                "Consult {state_code} constitutional standards for applicable split criteria"
            ),
            compliance_assessment_template: "{n} county splits present".into(),
            disclaimer: format!(
                "Legal compliance determination requires counsel. \
                 State-specific standard not in built-in table; consult {state_code} statutes."
            ),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wa_splits_has_legal_standard_field() {
        let standard = get_split_standard("WA");
        assert!(standard.is_some());
        let s = standard.unwrap();
        assert!(
            s.legal_standard.contains("WA Const."),
            "WA standard must cite WA constitution"
        );
    }

    #[test]
    fn test_ca_splits_has_legal_standard_field() {
        let standard = get_split_standard("CA");
        assert!(standard.is_some());
        assert!(standard.unwrap().legal_standard.contains("Art. XXI"));
    }

    #[test]
    fn test_tx_splits_has_legal_standard_field() {
        let standard = get_split_standard("TX");
        assert!(standard.is_some());
    }

    #[test]
    fn test_co_splits_has_legal_standard_field() {
        let standard = get_split_standard("CO");
        assert!(standard.is_some());
    }

    #[test]
    fn test_unknown_state_returns_generic_standard() {
        let standard = get_split_standard("ND");
        assert!(standard.is_some());
        assert!(
            standard.unwrap().legal_standard.contains("consult")
                || get_split_standard("ND")
                    .unwrap()
                    .legal_standard
                    .to_lowercase()
                    .contains("consult"),
            "Generic standard must recommend consulting state standards"
        );
    }

    #[test]
    fn test_standard_always_includes_disclaimer() {
        for state in &["WA", "CA", "TX", "CO", "VT"] {
            let s = get_split_standard(state).unwrap();
            assert!(!s.disclaimer.is_empty(), "State {} must have a disclaimer field", state);
            assert!(
                s.disclaimer.contains("counsel") || s.disclaimer.contains("consult"),
                "Disclaimer must recommend legal counsel for state {}",
                state
            );
        }
    }

    #[test]
    fn test_generic_state_disclaimer_contains_consult() {
        let s = get_split_standard("AK").unwrap();
        assert!(s.disclaimer.contains("counsel") || s.disclaimer.contains("consult"));
    }
}
