/// Per-state constitutional split standard lookup table.
///
/// Covers WA, CA, TX, CO, VA, NV, LA, CT explicitly. All other states get a generic
/// disclaimer. The `subdivision_term` and `subdivision_term_plural` fields record the
/// correct local name for county-level jurisdictions (e.g. "parish" for LA, "town" for CT).

#[derive(Debug, Clone, serde::Serialize)]
pub struct SplitStandard {
    pub state_code: String,
    /// Singular term for county-level subdivisions, e.g. "parish", "borough", "town".
    pub subdivision_term: String,
    /// Plural term for county-level subdivisions, e.g. "parishes", "boroughs", "towns".
    pub subdivision_term_plural: String,
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
            subdivision_term: "county".into(),
            subdivision_term_plural: "counties".into(),
            legal_standard: "WA Const. Art. II §43 — counties shall be preserved where possible"
                .into(),
            compliance_assessment_template:
                "{n} splits present; no binding numerical limit under WMCA".into(),
            disclaimer: "Legal compliance determination requires counsel".into(),
        }),
        "CA" => Some(SplitStandard {
            state_code: "CA".into(),
            subdivision_term: "county".into(),
            subdivision_term_plural: "counties".into(),
            legal_standard: "CA Const. Art. XXI §2(d) — minimize county and city splits".into(),
            compliance_assessment_template:
                "{n} county splits, {m} city splits present".into(),
            disclaimer: "Legal compliance determination requires counsel".into(),
        }),
        "TX" => Some(SplitStandard {
            state_code: "TX".into(),
            subdivision_term: "county".into(),
            subdivision_term_plural: "counties".into(),
            legal_standard:
                "TX Const. Art. III §26 — counties preserved where practicable".into(),
            compliance_assessment_template: "{n} county splits present".into(),
            disclaimer: "Legal compliance determination requires counsel".into(),
        }),
        "CO" => Some(SplitStandard {
            state_code: "CO".into(),
            subdivision_term: "county".into(),
            subdivision_term_plural: "counties".into(),
            legal_standard: "CO Const. Art. V §47 — preserve political subdivisions".into(),
            compliance_assessment_template: "{n} county splits present".into(),
            disclaimer: "Legal compliance determination requires counsel".into(),
        }),
        "VA" => Some(SplitStandard {
            state_code: "VA".into(),
            subdivision_term: "county or independent city".into(),
            subdivision_term_plural: "counties and independent cities".into(),
            legal_standard: "VA Const. Art. II §6 — counties and cities shall be preserved \
                where practical. IMPORTANT: Virginia has 95 counties AND 38 independent cities \
                that are legally separate from any county. Both are tracked as separate entities \
                in this analysis (133 total). Independent cities have FIPS codes 51510–51840 \
                and appear alongside counties in the split count. This is automatic — Census \
                GEOIDs encode independent cities with their own FIPS codes."
                .into(),
            compliance_assessment_template:
                "{n} splits present across counties and independent cities (133 total entities). \
                 VA Const. requires preserving both."
                .into(),
            disclaimer: "Legal compliance determination requires counsel. \
                Virginia's independent cities are distinct jurisdictions with no \
                county affiliation — they must be preserved on equal footing with counties."
                .into(),
        }),
        "NV" => Some(SplitStandard {
            state_code: "NV".into(),
            subdivision_term: "county".into(),
            subdivision_term_plural: "counties".into(),
            legal_standard: "NV Const. Art. 15 §13 — county boundaries shall be preserved \
                where possible. Clark County contains ~71% of Nevada's population and \
                necessarily spans multiple legislative districts."
                .into(),
            compliance_assessment_template: "{n} county splits present".into(),
            disclaimer: "Legal compliance determination requires counsel. \
                Clark County splits are expected given its dominant population share."
                .into(),
        }),
        "LA" => Some(SplitStandard {
            state_code: "LA".into(),
            subdivision_term: "parish".into(),
            subdivision_term_plural: "parishes".into(),
            legal_standard: "LA Const. Art. III §6 — parish (county) boundaries shall be \
                preserved where possible. Louisiana uses parishes rather than counties. \
                VRA Section 2 requires majority-minority districts for Black voters (~33%)."
                .into(),
            compliance_assessment_template: "{n} parish splits present".into(),
            disclaimer: "Legal compliance determination requires counsel. \
                Bayou geography and island tracts may make some parish splits unavoidable."
                .into(),
        }),
        "CT" => Some(SplitStandard {
            state_code: "CT".into(),
            subdivision_term: "town".into(),
            subdivision_term_plural: "towns".into(),
            legal_standard: "CT Const. Art. III §5 — town boundaries shall be preserved \
                where possible. Connecticut abolished counties as administrative units in 1960; \
                towns are the primary political subdivision."
                .into(),
            compliance_assessment_template: "{n} town splits present".into(),
            disclaimer: "Legal compliance determination requires counsel. \
                Connecticut uses towns, not counties, as the primary unit of preservation."
                .into(),
        }),
        _ => Some(SplitStandard {
            state_code: state_code.into(),
            subdivision_term: "county".into(),
            subdivision_term_plural: "counties".into(),
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

    #[test]
    fn test_la_subdivision_term_is_parish() {
        let s = get_split_standard("LA").unwrap();
        assert_eq!(s.subdivision_term, "parish");
        assert_eq!(s.subdivision_term_plural, "parishes");
    }

    #[test]
    fn test_wa_subdivision_term_is_county() {
        let s = get_split_standard("WA").unwrap();
        assert_eq!(s.subdivision_term, "county");
        assert_eq!(s.subdivision_term_plural, "counties");
    }

    #[test]
    fn test_va_subdivision_term_contains_city() {
        let s = get_split_standard("VA").unwrap();
        assert!(
            s.subdivision_term.contains("county") || s.subdivision_term.contains("city"),
            "VA subdivision_term must mention county or city, got: {}",
            s.subdivision_term
        );
    }

    #[test]
    fn test_ct_subdivision_term_is_town() {
        let s = get_split_standard("CT").unwrap();
        assert_eq!(s.subdivision_term, "town");
        assert_eq!(s.subdivision_term_plural, "towns");
    }

    #[test]
    fn test_generic_state_subdivision_term_defaults_to_county() {
        let s = get_split_standard("ND").unwrap();
        assert_eq!(s.subdivision_term, "county");
        assert_eq!(s.subdivision_term_plural, "counties");
    }
}
