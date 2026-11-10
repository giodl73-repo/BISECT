use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {
    pub location: String,
    pub chamber: String,
    pub year: String,
    pub policy_name: Option<String>,
    pub policy_json: Option<String>,
    pub manifest_name: Option<String>,
    pub manifest_json: Option<String>,
}
pub(crate) fn inspect(input: Input) -> Result<Value, String> {
    if [&input.location, &input.chamber, &input.year]
        .iter()
        .any(|s| s.len() > 200 || s.chars().any(char::is_control))
    {
        return Err("TUI Doctor fields support 200 UTF-8 bytes without controls.".into());
    }
    let (policy, policy_name, policy_kind, policy_hash) = crate::location_policy::database(
        input.policy_name.as_deref(),
        input.policy_json.as_deref(),
    )?;
    let (manifest_text, manifest_name, manifest_kind) =
        match (&input.manifest_name, &input.manifest_json) {
            (None, None) => (
                include_str!("../../../data/manifest.json"),
                "manifest.json",
                "embedded",
            ),
            (Some(name), Some(text))
                if !name.is_empty() && name.len() <= 200 && text.len() <= 512 * 1024 =>
            {
                (text.as_str(), name.as_str(), "local-override")
            }
            _ => {
                return Err("Supply both manifest name and text up to 512 KiB, or neither.".into())
            }
        };
    let manifest =
        serde_json::from_str::<bisect_core::fetch_plan::ManifestWithDistricts<u64>>(manifest_text);
    let registry = bisect_analysis::location_registry::PolicyRegistry::new(&policy);
    let policy_count = registry.chamber_districts(&input.location, &input.chamber, &input.year);
    // Native manifest lookup uses the original code, including case, and only
    // substitutes positive congressional counts when the policy has no count.
    let fallback = if input.chamber == "congressional" {
        manifest
            .as_ref()
            .ok()
            .and_then(|m| m.states.get(&input.location))
            .and_then(|s| s.districts.get(&input.year))
            .copied()
            .filter(|n| *n > 0)
    } else {
        None
    };
    let count = policy_count.or(fallback);
    let tolerance = bisect_analysis::location_registry::chamber_balance_tolerance(
        &policy,
        &input.location,
        &input.chamber,
    );
    let checks = bisect_report::tui_doctor::checks(
        &policy,
        &input.location,
        &input.chamber,
        &input.year,
        count,
        tolerance,
    );
    if checks
        .iter()
        .any(|c| c.message.len() > 8192 || c.hint.as_ref().is_some_and(|h| h.len() > 8192))
    {
        return Err("Policy produces oversized Doctor messages.".into());
    }
    let status = if checks
        .iter()
        .any(|c| c.status == bisect_report::tui_doctor::CheckStatus::Fail)
    {
        "failed"
    } else if checks
        .iter()
        .any(|c| c.status == bisect_report::tui_doctor::CheckStatus::Warn)
    {
        "warnings"
    } else {
        "passed"
    };
    Ok(
        json!({"schema_version":"bisect-tui-doctor-v1","status":status,"location":input.location,"chamber":input.chamber,"year":input.year,"checks":checks,"districts":count.map(|n|n.to_string()),"district_count_source":if policy_count.is_some(){"policy"}else if fallback.is_some(){"manifest"}else{"none"},"policy_name":policy_name,"policy_kind":policy_kind,"policy_sha256":policy_hash,"manifest_name":manifest_name,"manifest_kind":manifest_kind,"manifest_sha256":format!("{:x}",Sha256::digest(manifest_text.as_bytes())),"manifest_parse_error":manifest.err().map(|e|e.to_string()),"execution_performed":false,"scope":"Shared native TUI Doctor messages, statuses, hints and ordering. Tract granularity is checked as in the native screen. Congressional counts use the supplied or embedded native fetch manifest only if policy has no count; invalid manifests produce no fallback count and retain a diagnostic. Original location case is retained for native tolerance and manifest lookup. Policy/manifest paths and URLs are recorded text, never opened or fetched. Explicit Run replaces native live-on-render execution. These reference policy checks do not generate a plan or certify legal compliance; data preparation, RCTX diagnosis and artifact verification are separate tools. Project Open is inert."}),
    )
}
