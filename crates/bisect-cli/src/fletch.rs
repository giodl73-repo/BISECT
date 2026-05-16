use crate::fetch::FetchItem;
use anyhow::{Context, Result};
use fletch_core::{
    adapter_handoff_report, cache_index_from_manifest, cache_index_gate_report, dry_run_flight,
    fetch_plan_with_kind, fetch_to_cache, graph_from_registry, read_cache_manifest_json,
    upsert_cache_manifest_entries, validate_registry, write_cache_manifest_json, CacheEntry,
    CacheIndexGatePolicy, CacheManifest, CachePolicy, FetchOptions, FletchDefinition,
    FletchRegistry, FreshnessPolicy, GraphNodeKind, SourceKind, SourceSpec,
    FLETCH_CACHE_INDEX_SCHEMA, FLETCH_REGISTRY_SCHEMA,
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FletchCacheIndexRow {
    pub fletch_id: String,
    pub dataset_id: String,
    pub cache_key: String,
    pub sha256: String,
    pub relative_path: String,
    pub bytes: u64,
    pub verified: bool,
    pub evidence_status: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FletchCacheIndexReport {
    pub schema_version: String,
    pub generated_by: String,
    pub source_schema: String,
    pub registry_id: String,
    pub fletch_source_count: usize,
    pub indexed_source_count: usize,
    pub missing_source_count: usize,
    pub unexpected_index_count: usize,
    pub unverified_index_count: usize,
    pub byte_count: u64,
    pub rows: Vec<FletchCacheIndexRow>,
}

pub fn fletch_registry_from_items(items: &[FetchItem]) -> FletchRegistry {
    let mut seen = BTreeSet::new();
    let mut fletches = Vec::new();
    for item in items {
        let fletch_id = fletch_id_for_item(item);
        if !seen.insert(fletch_id.clone()) {
            continue;
        }
        let (source_kind, source_url) = match &item.url {
            Some(url) => (SourceKind::Http, url.clone()),
            None => (
                SourceKind::Adapter,
                format!(
                    "bisect-release://{}/{}/{}",
                    item.state_code, item.year, item.kind
                ),
            ),
        };
        let acquisition_mode = if source_kind == SourceKind::Http {
            "generic-http-cacheline"
        } else {
            "adapter-required"
        };
        fletches.push(FletchDefinition {
            id: fletch_id,
            node_kind: GraphNodeKind::Fletch,
            shafts: vec![SourceSpec {
                kind: source_kind,
                url: source_url,
                headers: BTreeMap::new(),
            }],
            edges: Vec::new(),
            format: None,
            tags: vec![
                "bisect".to_string(),
                item.kind.clone(),
                item.year.clone(),
                item.state_code.clone(),
            ],
            metadata: BTreeMap::from([
                ("fetch_family".to_string(), item.kind.clone()),
                ("state_code".to_string(), item.state_code.clone()),
                ("year".to_string(), item.year.clone()),
                (
                    "cache_targets".to_string(),
                    item.local_path.display().to_string(),
                ),
                (
                    "done_marker".to_string(),
                    item.done_marker.display().to_string(),
                ),
                (
                    "mutation_mode".to_string(),
                    "fletch-acquires-source-bisect-owns-derived-target".to_string(),
                ),
                ("acquisition_mode".to_string(), acquisition_mode.to_string()),
                (
                    "activation_rule".to_string(),
                    "bisect fetch preserves --force, --release, local manifest, done marker, and output paths"
                        .to_string(),
                ),
                (
                    "bisect_validation_floor".to_string(),
                    "download/cache success only; BISECT build/analyze/report own admissibility claims"
                        .to_string(),
                ),
                (
                    "claim_validated_by_download".to_string(),
                    "false".to_string(),
                ),
            ]),
        });
    }
    FletchRegistry {
        schema_version: FLETCH_REGISTRY_SCHEMA.to_string(),
        generated_by: "bisect-cli".to_string(),
        registry_id: "bisect.fetch-sources".to_string(),
        fletches,
    }
}

pub fn fletch_source_handoff_report(registry: &FletchRegistry) -> FletchSourceHandoffReport {
    let requested = registry
        .fletches
        .iter()
        .map(|definition| definition.id.clone())
        .collect::<Vec<_>>();
    let validation = validate_registry(registry);
    let handoff = adapter_handoff_report(registry, &requested);
    let flight = dry_run_flight(registry, &requested);
    let graph = graph_from_registry(registry);
    let mut rows = Vec::new();
    for definition in &registry.fletches {
        let source = definition.shafts.first();
        let source_kind = source
            .map(|source| source_kind_label(&source.kind).to_string())
            .unwrap_or_else(|| "none".to_string());
        let source_url = source.map(|source| source.url.clone()).unwrap_or_default();
        let handoff_status = if validation.findings.iter().any(|finding| {
            finding
                .fletch_id
                .as_deref()
                .is_some_and(|id| id == definition.id)
        }) {
            "registry-blocked"
        } else if source.is_some_and(|source| source.kind == SourceKind::Adapter) {
            "adapter-required"
        } else {
            "generic-fetch-ready"
        };
        let validation_status = if required_metadata_present(definition)
            && definition
                .metadata
                .get("claim_validated_by_download")
                .is_some_and(|value| value == "false")
        {
            "pass"
        } else {
            "review"
        };
        rows.push(FletchSourceHandoffRow {
            fetch_family: metadata(definition, "fetch_family"),
            fletch_id: definition.id.clone(),
            state_code: metadata(definition, "state_code"),
            year: metadata(definition, "year"),
            source_kind,
            source_url,
            cache_targets: metadata(definition, "cache_targets"),
            mutation_mode: metadata(definition, "mutation_mode"),
            acquisition_mode: metadata(definition, "acquisition_mode"),
            activation_rule: metadata(definition, "activation_rule"),
            bisect_validation_floor: metadata(definition, "bisect_validation_floor"),
            handoff_status: handoff_status.to_string(),
            validation_status: validation_status.to_string(),
        });
    }
    rows.sort_by(|left, right| {
        left.year
            .cmp(&right.year)
            .then(left.state_code.cmp(&right.state_code))
            .then(left.fetch_family.cmp(&right.fetch_family))
    });
    FletchSourceHandoffReport {
        registry_id: registry.registry_id.clone(),
        registry_valid: handoff.registry_valid,
        fletch_count: handoff.fletch_count,
        source_count: handoff.source_count,
        adapter_source_count: handoff.adapter_source_count,
        graph_node_count: graph.nodes.len(),
        graph_edge_count: graph.edges.len(),
        flight_step_count: flight.steps.len(),
        validation_finding_count: handoff.validation_finding_count,
        rows,
    }
}

pub fn write_fletch_source_handoff(path: &Path, report: &FletchSourceHandoffReport) -> Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }
    let mut writer =
        csv::Writer::from_path(path).with_context(|| format!("writing {}", path.display()))?;
    for row in &report.rows {
        writer.serialize(row)?;
    }
    writer.flush()?;
    Ok(())
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

pub fn fletch_cache_manifest_path(cache_root: &Path) -> PathBuf {
    cache_root.join("cache-manifest.json")
}

pub fn read_fletch_cache_manifest(path: &Path) -> Result<CacheManifest> {
    read_cache_manifest_json(path)
        .with_context(|| format!("reading FLETCH cache manifest {}", path.display()))
}

pub fn fletch_cache_index_report(
    registry: &FletchRegistry,
    manifest: &CacheManifest,
) -> FletchCacheIndexReport {
    let expected_ids = cacheable_registry_ids(registry)
        .into_iter()
        .collect::<BTreeSet<_>>();
    let index = cache_index_from_manifest(manifest);
    let gate = cache_index_gate_report(
        &index,
        &CacheIndexGatePolicy {
            expected_dataset_ids: expected_ids.iter().cloned().collect(),
            require_verified: true,
            allow_missing_expected: true,
        },
    );
    let indexed_by_dataset = index
        .entries
        .iter()
        .map(|entry| (entry.dataset_id.clone(), entry))
        .collect::<BTreeMap<_, _>>();

    let mut rows = Vec::new();
    for fletch_id in &expected_ids {
        match indexed_by_dataset.get(fletch_id) {
            Some(entry) => rows.push(FletchCacheIndexRow {
                fletch_id: fletch_id.clone(),
                dataset_id: entry.dataset_id.clone(),
                cache_key: entry.cache_key.clone(),
                sha256: entry.sha256.clone(),
                relative_path: entry.relative_path.clone(),
                bytes: entry.bytes,
                verified: entry.verified,
                evidence_status: if entry.verified {
                    "indexed-verified"
                } else {
                    "indexed-unverified"
                }
                .to_string(),
            }),
            None => rows.push(FletchCacheIndexRow {
                fletch_id: fletch_id.clone(),
                dataset_id: String::new(),
                cache_key: String::new(),
                sha256: String::new(),
                relative_path: String::new(),
                bytes: 0,
                verified: false,
                evidence_status: "missing-index-row".to_string(),
            }),
        }
    }
    for entry in &index.entries {
        if !expected_ids.contains(&entry.dataset_id) {
            rows.push(FletchCacheIndexRow {
                fletch_id: entry.dataset_id.clone(),
                dataset_id: entry.dataset_id.clone(),
                cache_key: entry.cache_key.clone(),
                sha256: entry.sha256.clone(),
                relative_path: entry.relative_path.clone(),
                bytes: entry.bytes,
                verified: entry.verified,
                evidence_status: "unexpected-index-row".to_string(),
            });
        }
    }

    rows.sort_by(|left, right| {
        left.evidence_status
            .cmp(&right.evidence_status)
            .then(left.fletch_id.cmp(&right.fletch_id))
    });
    let indexed_source_count = rows
        .iter()
        .filter(|row| {
            matches!(
                row.evidence_status.as_str(),
                "indexed-verified" | "indexed-unverified"
            )
        })
        .count();
    let missing_source_count = rows
        .iter()
        .filter(|row| row.evidence_status == "missing-index-row")
        .count();
    let unexpected_index_count = gate.unexpected_count;
    let unverified_index_count = gate.unverified_count;
    let byte_count = rows
        .iter()
        .filter(|row| row.evidence_status == "indexed-verified")
        .map(|row| row.bytes)
        .sum();

    FletchCacheIndexReport {
        schema_version: "bisect.fletch-cache-index.v1".to_string(),
        generated_by: "bisect-cli".to_string(),
        source_schema: FLETCH_CACHE_INDEX_SCHEMA.to_string(),
        registry_id: registry.registry_id.clone(),
        fletch_source_count: expected_ids.len(),
        indexed_source_count,
        missing_source_count,
        unexpected_index_count,
        unverified_index_count,
        byte_count,
        rows,
    }
}

fn cacheable_registry_ids(registry: &FletchRegistry) -> Vec<String> {
    registry
        .fletches
        .iter()
        .filter(|definition| {
            definition
                .shafts
                .iter()
                .any(|shaft| matches!(shaft.kind, SourceKind::Http | SourceKind::File))
        })
        .map(|definition| definition.id.clone())
        .collect()
}

pub fn write_fletch_cache_index(path: &Path, report: &FletchCacheIndexReport) -> Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }
    let file =
        std::fs::File::create(path).with_context(|| format!("writing {}", path.display()))?;
    serde_json::to_writer_pretty(file, report)
        .with_context(|| format!("serializing {}", path.display()))?;
    Ok(())
}

pub fn fletch_cache_index_gate_failures(report: &FletchCacheIndexReport) -> Vec<String> {
    let mut failures = Vec::new();
    if report.unverified_index_count > 0 {
        failures.push(format!(
            "{} indexed source row(s) are unverified",
            report.unverified_index_count
        ));
    }
    if report.unexpected_index_count > 0 {
        failures.push(format!(
            "{} cache index row(s) do not map to the BISECT FLETCH registry",
            report.unexpected_index_count
        ));
    }
    failures
}

pub fn fetch_item_to_fletch(
    item: &FetchItem,
    cache_root: &Path,
    force: bool,
) -> Result<fletch_core::FetchOutcome, String> {
    let url = item
        .url
        .as_ref()
        .ok_or_else(|| format!("{} has no generic URL", item.kind))?;
    let mut plan = fetch_plan_with_kind(fletch_id_for_item(item), url.clone(), SourceKind::Http)
        .map_err(|e| e.to_string())?;
    plan.version = Some(item.year.clone());
    plan.cache_policy = CachePolicy {
        freshness: FreshnessPolicy::Immutable,
        allow_offline: true,
        resumable: true,
    };
    plan.metadata
        .insert("bisect_kind".to_string(), item.kind.clone());
    plan.metadata
        .insert("state_code".to_string(), item.state_code.clone());
    plan.metadata.insert(
        "bisect_cache_target".to_string(),
        item.local_path.display().to_string(),
    );
    let outcome = fetch_to_cache(
        &plan,
        FetchOptions::new(PathBuf::from(cache_root)).with_force(force),
    )
    .map_err(|e| e.to_string())?;
    upsert_fletch_cache_manifest_entries(cache_root, [outcome.entry.clone()])
        .map_err(|e| e.to_string())?;
    Ok(outcome)
}

fn upsert_fletch_cache_manifest_entries(
    cache_root: &Path,
    entries: impl IntoIterator<Item = CacheEntry>,
) -> Result<CacheManifest> {
    let manifest_path = fletch_cache_manifest_path(cache_root);
    let entries = entries.into_iter().collect::<Vec<_>>();
    if entries.is_empty() {
        return if manifest_path.exists() {
            read_fletch_cache_manifest(&manifest_path)
        } else {
            fletch_core::cache_manifest(cache_root.display().to_string(), Vec::new())
                .context("creating empty FLETCH cache manifest")
        };
    }
    let manifest = if manifest_path.exists() {
        read_fletch_cache_manifest(&manifest_path)?
    } else {
        fletch_core::cache_manifest(cache_root.display().to_string(), Vec::new())
            .context("creating empty FLETCH cache manifest")?
    };
    let manifest = upsert_cache_manifest_entries(manifest, entries)
        .context("upserting FLETCH cache manifest entries")?;
    write_cache_manifest_json(&manifest_path, &manifest)
        .with_context(|| format!("writing FLETCH cache manifest {}", manifest_path.display()))?;
    Ok(manifest)
}

pub fn fletch_id_for_item(item: &FetchItem) -> String {
    format!(
        "bisect.{}.{}.{}",
        safe_id(&item.year),
        safe_id(&item.state_code.to_lowercase()),
        safe_id(&item.kind)
    )
}

fn metadata(definition: &FletchDefinition, key: &str) -> String {
    definition.metadata.get(key).cloned().unwrap_or_default()
}

fn required_metadata_present(definition: &FletchDefinition) -> bool {
    [
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
    .all(|key| {
        definition
            .metadata
            .get(*key)
            .is_some_and(|value| !value.is_empty())
    })
}

fn source_kind_label(kind: &SourceKind) -> &'static str {
    match kind {
        SourceKind::Http => "http",
        SourceKind::File => "file",
        SourceKind::Adapter => "adapter",
    }
}

fn safe_id(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cache_entry(dataset_id: &str, verified: bool) -> CacheEntry {
        CacheEntry {
            dataset_id: dataset_id.to_string(),
            version: Some("2020".to_string()),
            source_url: format!("https://example.test/{dataset_id}.zip"),
            cache_key: format!("sha256:{}", "a".repeat(64)),
            relative_path: "objects/sha256/aa".to_string(),
            sha256: format!("sha256:{}", "b".repeat(64)),
            bytes: 42,
            fetched_at_ms: 1,
            verified,
            fetch_attempts: 1,
            retry_count: 0,
            last_retryable_error: None,
        }
    }

    fn fetch_item(kind: &str, url: Option<&str>) -> FetchItem {
        FetchItem {
            state_code: "VT".to_string(),
            year: "2020".to_string(),
            kind: kind.to_string(),
            url: url.map(str::to_string),
            local_path: PathBuf::from("data")
                .join("2020")
                .join(kind)
                .join("target.dat"),
            done_marker: PathBuf::from("data")
                .join("2020")
                .join(kind)
                .join("target.done"),
            available_locally: false,
        }
    }

    #[test]
    fn fletch_registry_marks_http_and_adapter_sources() {
        let registry = fletch_registry_from_items(&[
            fetch_item("tiger", Some("https://example.test/tiger.zip")),
            fetch_item("adjacency", None),
        ]);
        let report = fletch_source_handoff_report(&registry);
        assert_eq!(report.fletch_count, 2);
        assert_eq!(report.adapter_source_count, 1);
        assert!(fletch_source_handoff_gate_failures(&report).is_empty());
        assert!(report.rows.iter().any(|row| {
            row.fetch_family == "tiger" && row.handoff_status == "generic-fetch-ready"
        }));
        assert!(report.rows.iter().any(|row| {
            row.fetch_family == "adjacency" && row.handoff_status == "adapter-required"
        }));
    }

    #[test]
    fn fletch_id_for_item_is_stable_and_path_safe() {
        let item = fetch_item("school-districts", Some("https://example.test/school.zip"));
        assert_eq!(fletch_id_for_item(&item), "bisect.2020.vt.school-districts");
    }

    #[test]
    fn fletch_cache_index_report_maps_manifest_to_registry_sources() {
        let item = fetch_item("tiger", Some("https://example.test/tiger.zip"));
        let registry = fletch_registry_from_items(&[item]);
        let manifest =
            fletch_core::cache_manifest("cache", vec![cache_entry("bisect.2020.vt.tiger", true)])
                .unwrap();

        let report = fletch_cache_index_report(&registry, &manifest);

        assert_eq!(report.source_schema, FLETCH_CACHE_INDEX_SCHEMA);
        assert_eq!(report.indexed_source_count, 1);
        assert_eq!(report.missing_source_count, 0);
        assert_eq!(report.unexpected_index_count, 0);
        assert!(fletch_cache_index_gate_failures(&report).is_empty());
    }
}
