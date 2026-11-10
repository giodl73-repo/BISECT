//! Shared native source report projection, metadata and CSV encoding.
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FletchSourceHandoffRow {
    pub fetch_family: String,
    pub fletch_id: String,
    pub state_code: String,
    pub year: String,
    pub source_kind: String,
    pub source_url: String,
    pub cache_targets: String,
    pub mutation_mode: String,
    pub acquisition_mode: String,
    pub activation_rule: String,
    pub bisect_validation_floor: String,
    pub handoff_status: String,
    pub validation_status: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FletchSourceHandoffReport {
    pub registry_id: String,
    pub registry_valid: bool,
    pub fletch_count: usize,
    pub source_count: usize,
    pub adapter_source_count: usize,
    pub graph_node_count: usize,
    pub graph_edge_count: usize,
    pub flight_step_count: usize,
    pub validation_finding_count: usize,
    pub rows: Vec<FletchSourceHandoffRow>,
}

pub fn fletch_source_handoff_gate_failures(report: &FletchSourceHandoffReport) -> Vec<String> {
    let mut failures = Vec::new();
    if !report.registry_valid {
        failures.push("registry validation failed".to_string());
    }
    if report.validation_finding_count > 0 {
        failures.push(format!(
            "{} registry validation findings",
            report.validation_finding_count
        ));
    }
    for row in &report.rows {
        if row.validation_status != "pass" {
            failures.push(format!(
                "{} {} {} validation_status={}",
                row.state_code, row.year, row.fetch_family, row.validation_status
            ));
        }
        if row.handoff_status == "registry-blocked" {
            failures.push(format!("{} is registry-blocked", row.fletch_id));
        }
    }
    failures
}

#[derive(Debug, Clone)]
pub struct SourceHandoffDefinition {
    pub id: String,
    pub source_kind: Option<String>,
    pub source_url: Option<String>,
    pub metadata: BTreeMap<String, String>,
    pub registry_blocked: bool,
}
#[derive(Debug, Clone)]
pub struct SourceHandoffCounts {
    pub registry_valid: bool,
    pub fletch_count: usize,
    pub source_count: usize,
    pub adapter_source_count: usize,
    pub graph_node_count: usize,
    pub graph_edge_count: usize,
    pub flight_step_count: usize,
    pub validation_finding_count: usize,
}
pub fn project_source_handoff(
    registry_id: &str,
    definitions: &[SourceHandoffDefinition],
    counts: SourceHandoffCounts,
) -> FletchSourceHandoffReport {
    let mut rows = vec![];
    for d in definitions {
        let m = |k: &str| d.metadata.get(k).cloned().unwrap_or_default();
        let present = [
            "fetch_family",
            "state_code",
            "year",
            "cache_targets",
            "mutation_mode",
            "acquisition_mode",
            "activation_rule",
            "bisect_validation_floor",
        ]
        .iter()
        .all(|k| d.metadata.get(*k).is_some_and(|v| !v.is_empty()));
        rows.push(FletchSourceHandoffRow {
            fetch_family: m("fetch_family"),
            fletch_id: d.id.clone(),
            state_code: m("state_code"),
            year: m("year"),
            source_kind: d.source_kind.clone().unwrap_or_else(|| "none".into()),
            source_url: d.source_url.clone().unwrap_or_default(),
            cache_targets: m("cache_targets"),
            mutation_mode: m("mutation_mode"),
            acquisition_mode: m("acquisition_mode"),
            activation_rule: m("activation_rule"),
            bisect_validation_floor: m("bisect_validation_floor"),
            handoff_status: if d.registry_blocked {
                "registry-blocked"
            } else if d.source_kind.as_deref() == Some("adapter") {
                "adapter-required"
            } else {
                "generic-fetch-ready"
            }
            .into(),
            validation_status: if present && m("claim_validated_by_download") == "false" {
                "pass"
            } else {
                "review"
            }
            .into(),
        });
    }
    rows.sort_by(|a, b| {
        a.year
            .cmp(&b.year)
            .then(a.state_code.cmp(&b.state_code))
            .then(a.fetch_family.cmp(&b.fetch_family))
    });
    FletchSourceHandoffReport {
        registry_id: registry_id.into(),
        registry_valid: counts.registry_valid,
        fletch_count: counts.fletch_count,
        source_count: counts.source_count,
        adapter_source_count: counts.adapter_source_count,
        graph_node_count: counts.graph_node_count,
        graph_edge_count: counts.graph_edge_count,
        flight_step_count: counts.flight_step_count,
        validation_finding_count: counts.validation_finding_count,
        rows,
    }
}
pub fn source_handoff_csv(report: &FletchSourceHandoffReport) -> Result<String, String> {
    let mut writer = csv::Writer::from_writer(Vec::new());
    for row in &report.rows {
        writer.serialize(row).map_err(|e| e.to_string())?;
    }
    String::from_utf8(writer.into_inner().map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}
pub fn source_id(year: &str, state: &str, kind: &str) -> String {
    let safe = |s: &str| {
        s.chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                    c.to_ascii_lowercase()
                } else {
                    '-'
                }
            })
            .collect::<String>()
    };
    format!(
        "bisect.{}.{}.{}",
        safe(year),
        safe(&state.to_lowercase()),
        safe(kind)
    )
}
#[derive(Debug, Clone)]
pub struct GeneratedFetchSource {
    pub state_code: String,
    pub year: String,
    pub kind: String,
    pub url: Option<String>,
    pub cache_targets: String,
    pub done_marker: String,
}
pub fn generated_source_definition(item: &GeneratedFetchSource) -> SourceHandoffDefinition {
    let adapter = item.url.is_none();
    SourceHandoffDefinition{id:source_id(&item.year,&item.state_code,&item.kind),source_kind:Some(if adapter{"adapter"}else{"http"}.into()),source_url:Some(item.url.clone().unwrap_or_else(||format!("bisect-release://{}/{}/{}",item.state_code,item.year,item.kind))),registry_blocked:false,
 metadata:BTreeMap::from([("fetch_family".into(),item.kind.clone()),("state_code".into(),item.state_code.clone()),("year".into(),item.year.clone()),("cache_targets".into(),item.cache_targets.clone()),("done_marker".into(),item.done_marker.clone()),("mutation_mode".into(),"fletch-acquires-source-bisect-owns-derived-target".into()),("acquisition_mode".into(),if adapter{"adapter-required"}else{"generic-http-cacheline"}.into()),("activation_rule".into(),"bisect fetch preserves --force, --release, local manifest, done marker, and output paths".into()),("bisect_validation_floor".into(),"download/cache success only; BISECT build/analyze/report own admissibility claims".into()),("claim_validated_by_download".into(),"false".into())])}
}
/// Only BISECT's generated registry: one shaft per deduplicated ID, no edges,
/// pinned valid schema. Upstream validation checks schema, IDs and shafts.
pub fn generated_source_handoff(items: &[GeneratedFetchSource]) -> FletchSourceHandoffReport {
    let mut seen = BTreeSet::new();
    let mut definitions = vec![];
    for item in items {
        let d = generated_source_definition(item);
        if seen.insert(d.id.clone()) {
            definitions.push(d);
        }
    }
    let n = definitions.len();
    let adapters = definitions
        .iter()
        .filter(|d| d.source_kind.as_deref() == Some("adapter"))
        .count();
    project_source_handoff(
        "bisect.fetch-sources",
        &definitions,
        SourceHandoffCounts {
            registry_valid: true,
            fletch_count: n,
            source_count: n,
            adapter_source_count: adapters,
            graph_node_count: 2 * n,
            graph_edge_count: n,
            flight_step_count: n,
            validation_finding_count: 0,
        },
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    fn item() -> GeneratedFetchSource {
        GeneratedFetchSource {
            state_code: "RI".into(),
            year: "2020".into(),
            kind: "tiger".into(),
            url: Some("https://example.test/a".into()),
            cache_targets: "data/a,\"b\"\nc".into(),
            done_marker: "data/a.done".into(),
        }
    }
    #[test]
    fn generated_sources_deduplicate_first_and_keep_csv_evidence() {
        assert_eq!(source_id("2020", "İ", "school"), "bisect.2020.i-.school");
        let first = item();
        let mut duplicate = first.clone();
        duplicate.url = Some("other".into());
        let mut adapter = first.clone();
        adapter.kind = "adjacency".into();
        adapter.url = None;
        let report = generated_source_handoff(&[first.clone(), duplicate, adapter]);
        assert_eq!(report.fletch_count, 2);
        assert_eq!(report.graph_node_count, 4);
        assert_eq!(report.adapter_source_count, 1);
        assert_eq!(report.rows[1].source_url, first.url.unwrap());
        assert!(fletch_source_handoff_gate_failures(&report).is_empty());
        let csv = source_handoff_csv(&report).unwrap();
        let mut reader = csv::Reader::from_reader(csv.as_bytes());
        let records = reader.records().collect::<Result<Vec<_>, _>>().unwrap();
        assert_eq!(records[1].get(6), Some(first.cache_targets.as_str()));
    }
    #[test]
    fn blocked_and_missing_metadata_remain_gate_failures() {
        let mut d = generated_source_definition(&item());
        d.registry_blocked = true;
        d.metadata.remove("cache_targets");
        d.source_kind = None;
        d.source_url = None;
        let report = project_source_handoff(
            "registry",
            &[d],
            SourceHandoffCounts {
                registry_valid: false,
                fletch_count: 1,
                source_count: 0,
                adapter_source_count: 0,
                graph_node_count: 1,
                graph_edge_count: 0,
                flight_step_count: 1,
                validation_finding_count: 1,
            },
        );
        assert_eq!(report.rows[0].source_kind, "none");
        assert_eq!(report.rows[0].validation_status, "review");
        assert_eq!(
            fletch_source_handoff_gate_failures(&report),
            vec![
                "registry validation failed",
                "1 registry validation findings",
                "RI 2020 tiger validation_status=review",
                "bisect.2020.ri.tiger is registry-blocked"
            ]
        );
    }
}
