use std::collections::BTreeMap;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let request: serde_json::Value = serde_json::from_slice(&std::fs::read(std::env::args().nth(1).ok_or("request path")?)?)?;
    let manifest: fletch_core::CacheManifest = serde_json::from_str(request["manifest_json"].as_str().ok_or("manifest text")?)?;
    fletch_core::validate_cache_manifest(&manifest)?;
    let ids: Vec<String> = serde_json::from_value(request["expected_ids"].clone())?;
    let registry = fletch_core::FletchRegistry {
        schema_version: fletch_core::FLETCH_REGISTRY_SCHEMA.into(), generated_by: "bisect-cli".into(),
        registry_id: "bisect.fetch-sources".into(),
        fletches: ids.into_iter().map(|id| fletch_core::FletchDefinition {
            id, node_kind: fletch_core::GraphNodeKind::Fletch,
            shafts: vec![fletch_core::SourceSpec {kind: fletch_core::SourceKind::Http, url: "https://example.test/source".into(),headers:BTreeMap::new()}],
            edges:vec![],format:None,tags:vec![],metadata:BTreeMap::new(),
        }).collect(),
    };
    println!("{}",serde_json::to_string_pretty(&bisect_cli::fletch::fletch_cache_index_report(&registry,&manifest))?);
    Ok(())
}
