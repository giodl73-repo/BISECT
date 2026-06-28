use crate::*;

pub(crate) fn canonicalize_value(value: &Value) -> Value {
    match value {
        Value::Array(values) => Value::Array(values.iter().map(canonicalize_value).collect()),
        Value::Object(map) => {
            let mut sorted = Map::new();
            let mut keys: Vec<_> = map.keys().collect();
            keys.sort();
            for key in keys {
                sorted.insert(key.clone(), canonicalize_value(&map[key]));
            }
            Value::Object(sorted)
        }
        other => other.clone(),
    }
}

pub(crate) fn validate_contest(contest: &Contest) -> Result<(), RcountCoreError> {
    let mut seen = BTreeSet::new();
    for selection in contest.selections.iter() {
        if !seen.insert(selection.selection_id.as_str()) {
            return Err(RcountCoreError::DuplicateSelectionId {
                contest_id: contest.contest_id.clone(),
                selection_id: selection.selection_id.clone(),
            });
        }
    }
    Ok(())
}

pub(crate) fn ensure_non_negative(value: i64) -> Result<(), RcountCoreError> {
    if value < 0 {
        return Err(RcountCoreError::NegativeCount);
    }
    Ok(())
}

pub(crate) fn is_sha256_hash(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64 && hex.bytes().all(|byte| byte.is_ascii_hexdigit())
}

pub(crate) fn check_residual(
    contest_id: &str,
    field: &str,
    declared: i64,
    computed: i64,
) -> Result<(), RcountCoreError> {
    if declared != computed {
        return Err(RcountCoreError::JurisdictionResidualMismatch {
            contest_id: contest_id.to_string(),
            field: field.to_string(),
            declared,
            computed,
        });
    }
    Ok(())
}

pub(crate) fn check_cvr_field(
    contest_id: &str,
    reporting_unit_id: &str,
    field: &str,
    summary: i64,
    cvr: i64,
) -> Result<(), RcountCoreError> {
    if summary != cvr {
        return Err(RcountCoreError::CvrSummaryMismatch {
            contest_id: contest_id.to_string(),
            reporting_unit_id: reporting_unit_id.to_string(),
            field: field.to_string(),
            summary,
            cvr,
        });
    }
    Ok(())
}

pub(crate) fn is_supported_audit_algorithm_method(method_id: &str) -> bool {
    matches!(
        method_id,
        BRAVO_BALLOT_POLLING_METHOD_ID
            | MINERVA_BALLOT_POLLING_METHOD_ID
            | ATHENA_BALLOT_POLLING_METHOD_ID
            | KAPLAN_MARKOV_COMPARISON_METHOD_ID
            | ALPHA_MARTINGALE_METHOD_ID
            | SHANGRLA_ASSORTER_METHOD_ID
            | STRATIFIED_HYBRID_RLA_METHOD_ID
            | BATCH_COMPARISON_METHOD_ID
            | RAIRE_IRV_METHOD_ID
            | AWAIRE_IRV_METHOD_ID
            | BAYESIAN_TABULATION_AUDIT_METHOD_ID
            | SOBA_OBSERVABLE_BALLOT_AUDIT_METHOD_ID
    )
}

pub(crate) fn is_positive_rational(value: RationalValue) -> bool {
    value.denominator > 0 && value.numerator > 0
}

pub(crate) fn is_non_negative_rational(value: RationalValue) -> bool {
    value.denominator > 0 && value.numerator >= 0
}

pub(crate) fn has_positive_denominator(value: RationalValue) -> bool {
    value.denominator > 0
}

pub(crate) fn rational_gt(lhs: RationalValue, rhs: RationalValue) -> bool {
    (lhs.numerator as i128) * (rhs.denominator as i128)
        > (rhs.numerator as i128) * (lhs.denominator as i128)
}

pub(crate) fn rational_eq(lhs: RationalValue, rhs: RationalValue) -> bool {
    lhs.denominator > 0
        && rhs.denominator > 0
        && (lhs.numerator as i128) * (rhs.denominator as i128)
            == (rhs.numerator as i128) * (lhs.denominator as i128)
}
