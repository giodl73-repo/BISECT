//! Static public catalog: shared geometry, exact effective settings and public fields only.
use crate::{
    data::Paths,
    model::{supported_searches, Experiment, Job},
};
use anyhow::{ensure, Context, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub fn effective_config(c: &Experiment, k: usize) -> Value {
    let budget = match c.search.as_str() {
        "single" => 0,
        "multi" | "percentile" | "parallel-tempering" => c.seeds,
        "short-burst" | "short-burst-forest" | "short-burst-merge-split" => {
            c.steps.div_ceil(20) * 20
        }
        _ => c.steps,
    };
    json!({"year":c.year,"chamber":c.chamber,"district_count":k,"structure":c.structure,
        "weights":c.weights,"search":c.search,"seed":c.seed,"budget":budget,
        "percentile":if ["single","multi"].contains(&c.search.as_str()) {0.0} else {c.percentile},
        "alpha_county":if c.weights=="county" {c.alpha_county.max(1.0)} else {0.0},
        "balance_tolerance":c.balance_tolerance,"iterations":c.iterations,
        "area_swing":if c.structure=="ratio-optimal-area" {c.area_swing} else {0.0}})
}

fn asset(out: &Path, bytes: &[u8], assets: &mut BTreeMap<String, String>) -> Result<String> {
    let sha = digest(bytes);
    let name = format!("assets/{sha}.json");
    if !assets.contains_key(&name) {
        std::fs::write(out.join(&name), bytes)?;
        assets.insert(name.clone(), sha);
    }
    Ok(name)
}

pub fn public_metrics(metrics: &Value) -> Value {
    let fields = [
        "district_count",
        "units",
        "population",
        "ideal_population",
        "max_deviation_percent",
        "districts",
        "contiguous",
        "graph_boundary_m",
        "weighted_boundary",
        "cut_edges",
        "split_counties",
        "balance_passed",
        "graph_sha256",
        "geoid_join_sha256",
        "preparation_seconds",
        "engine_seconds",
        "engine_sha256",
        "boundary_includes_synthetic_bridges",
    ];
    let mut public = serde_json::Map::new();
    for key in fields {
        if let Some(value) = metrics.get(key) {
            public.insert(key.into(), value.clone());
        }
    }
    let mut provenance = serde_json::Map::new();
    for key in [
        "bisect_build_date",
        "build_commit",
        "rustc_version",
        "version",
    ] {
        if let Some(value) = metrics.get("engine_provenance").and_then(|p| p.get(key)) {
            provenance.insert(key.into(), value.clone());
        }
    }
    public.insert("engine_provenance".into(), Value::Object(provenance));
    if !public.contains_key("engine_sha256") {
        public.insert("engine_sha256".into(), Value::Null);
    }
    if let Some(districts) = metrics["districts"].as_array() {
        public.insert(
            "districts".into(),
            json!(districts
                .iter()
                .map(|district| {
                    let mut result = serde_json::Map::new();
                    for key in [
                        "district",
                        "population",
                        "units",
                        "deviation_percent",
                        "components",
                    ] {
                        if let Some(value) = district.get(key) {
                            result.insert(key.into(), value.clone());
                        }
                    }
                    Value::Object(result)
                })
                .collect::<Vec<_>>()),
        );
    }
    public.insert("execution".into(), json!("heuristic"));
    public.insert("optimality".into(), json!("unproved"));
    Value::Object(public)
}

pub fn validate_join(geometry: &Value, assignments: &Value, k: usize) -> Result<()> {
    ensure!(
        geometry["type"] == "FeatureCollection",
        "Invalid geometry collection."
    );
    let assignment = assignments
        .as_object()
        .context("Assignments must be a GEOID object.")?;
    ensure!(!assignment.is_empty(), "Empty assignments.");
    let mut geoids = BTreeSet::new();
    for feature in geometry["features"]
        .as_array()
        .context("Missing geometry features.")?
    {
        let geoid = feature["properties"]["geoid"]
            .as_str()
            .context("Missing GEOID.")?;
        ensure!(
            geoid.len() == 11
                && geoid.bytes().all(|b| b.is_ascii_digit())
                && geoids.insert(geoid.to_owned()),
            "Invalid or duplicate tract GEOID."
        );
    }
    for (geoid, district) in assignment {
        ensure!(
            geoids.contains(geoid),
            "Assignment GEOID absent from geometry: {geoid}"
        );
        let d = district.as_u64().context("District must be an integer.")?;
        ensure!(d >= 1 && d <= k as u64, "Invalid district assignment.");
    }
    // Unassigned display tracts are allowed and visibly remain neutral, as in local mode.
    Ok(())
}

pub fn build(paths: &Paths, jobs: &[Job], out: &Path, max_bytes: u64) -> Result<Value> {
    ensure!(
        !out.exists() || std::fs::read_dir(out)?.next().is_none(),
        "Output directory must be new or empty; export into a fresh directory."
    );
    std::fs::create_dir_all(out.join("assets"))?;
    let mut assets = BTreeMap::new();
    let mut geometries = BTreeMap::<String, String>::new();
    let mut runs = Vec::new();
    let mut success = 0;
    for job in jobs {
        if !supported_searches(&job.config.structure).contains(&job.config.search.as_str()) {
            eprintln!(
                "Excluded {}: requested search is not executed by its structure",
                job.id
            );
            continue;
        }
        job.config.validate(&paths.codes())?;
        let mut states = Vec::new();
        for state in &job.states {
            let mut record = json!({"code":state.code,"status":state.status,"error":null,"elapsed_seconds":state.elapsed_seconds,"command":[],"metrics":null});
            if state.status == "completed" {
                let metrics = state
                    .metrics
                    .as_ref()
                    .context("Completed state lacks metrics.")?;
                let k = metrics["district_count"]
                    .as_u64()
                    .context("Missing district count.")? as usize;
                let dir = paths.store.join("jobs").join(&job.id);
                let mut geometry: Value = serde_json::from_slice(&std::fs::read(
                    dir.join(format!("{}_map.json", state.code)),
                )?)?;
                let assignments: Value = serde_json::from_slice(&std::fs::read(
                    dir.join(format!("{}_assignments.json", state.code)),
                )?)?;
                for feature in geometry["features"]
                    .as_array_mut()
                    .context("Missing features.")?
                {
                    feature["properties"]
                        .as_object_mut()
                        .context("Missing properties.")?
                        .remove("district");
                }
                geometry = json!({"type":"FeatureCollection","state":geometry["state"],"year":geometry["year"],"features":geometry["features"].as_array().unwrap().iter().map(|feature|json!({"type":"Feature","geometry":feature["geometry"],"properties":{"geoid":feature["properties"]["geoid"],"state":feature["properties"]["state"],"county":feature["properties"]["county"]}})).collect::<Vec<_>>()});
                validate_join(&geometry, &assignments, k)?;
                ensure!(
                    assignments.as_object().unwrap().len() as u64
                        == metrics["units"].as_u64().context("Missing unit count.")?,
                    "Assignment count differs from metrics."
                );
                ensure!(
                    geometry["state"] == state.code && geometry["year"] == job.config.year,
                    "Geometry identity differs from run."
                );
                let geometry_bytes = serde_json::to_vec(&geometry)?;
                let geometry_ref = asset(out, &geometry_bytes, &mut assets)?;
                let geometry_key = format!("{}:{}", state.code, job.config.year);
                if let Some(previous) = geometries.insert(geometry_key, geometry_ref.clone()) {
                    ensure!(
                        previous == geometry_ref,
                        "Conflicting display geometry for the same state and year."
                    );
                }
                let assignments_ref = asset(out, &serde_json::to_vec(&assignments)?, &mut assets)?;
                record["metrics"] = public_metrics(metrics);
                record["geometry_ref"] = json!(geometry_ref);
                record["assignments_ref"] = json!(assignments_ref);
                record["effective_config"] = effective_config(&job.config, k);
                success += 1;
            } else {
                record["status"] = json!("unavailable");
                record["error"]=json!(format!("Saved state result is {}. Detailed diagnostics are retained in the local laboratory.",state.status));
            }
            states.push(record);
        }
        runs.push(json!({"id":job.id,"config":job.config,"created_unix":job.created_unix,
            "status":job.status,"states":states,"logs":["Precomputed catalog. No engine was executed by this browser."]}));
    }
    ensure!(
        success > 0,
        "Catalog needs at least one successful state result."
    );
    let mut catalog = paths.catalog();
    for key in ["data_root", "store", "token"] {
        catalog.as_object_mut().unwrap().remove(key);
    }
    catalog["engine_available"] = json!(false);
    catalog["execution"] = json!("precomputed");
    for state in catalog["states"].as_array_mut().unwrap() {
        let code = state["code"].as_str().unwrap().to_owned();
        for year in state["years"].as_array_mut().unwrap() {
            let available =
                geometries.contains_key(&format!("{code}:{}", year["year"].as_str().unwrap()));
            year["available"] = json!(available);
            year["geometry"] = json!(available);
            year["cached"] = json!(available);
        }
    }
    let manifest = json!({"schema_version":1,"catalog":catalog,"runs":runs,"geometries":geometries,"assets":assets,
        "coverage":{"successful_state_results":success,"states":geometries.keys().map(|s|s.split(':').next().unwrap()).collect::<BTreeSet<_>>().len()},
        "integrity":"SHA-256 of asset bytes; not a signature or legal certification"});
    let html = include_str!("../../../web/lab/index.html")
        .replace("<body>", "<body data-backend=\"catalog\">");
    std::fs::write(out.join("index.html"), html)?;
    std::fs::write(
        out.join("lab.css"),
        include_str!("../../../web/lab/lab.css"),
    )?;
    std::fs::write(out.join("lab.js"), include_str!("../../../web/lab/lab.js"))?;
    std::fs::write(
        out.join("static.js"),
        include_str!("../../../web/lab/static.js"),
    )?;
    std::fs::write(out.join(".nojekyll"), "")?;
    let bytes = serde_json::to_vec(&manifest)?;
    let total = std::fs::read_dir(out.join("assets"))?.try_fold(0u64, |sum, e| {
        Ok::<_, std::io::Error>(sum + e?.metadata()?.len())
    })? + ["index.html", "lab.css", "lab.js", "static.js"]
        .iter()
        .try_fold(0u64, |sum, p| {
            Ok::<_, std::io::Error>(sum + std::fs::metadata(out.join(p))?.len())
        })?
        + bytes.len() as u64;
    ensure!(
        total <= max_bytes,
        "Catalog exceeds the configured artifact size budget."
    );
    std::fs::write(out.join("catalog.json"), bytes)?;
    Ok(
        json!({"output":out,"bytes":total,"coverage":manifest["coverage"],"experiments":manifest["runs"].as_array().unwrap().len()}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn full_export_shares_geometry_and_omits_private_context() {
        let temp = tempfile::tempdir().unwrap();
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let paths = Paths::new(root, temp.path().join("store"), "missing-engine".into()).unwrap();
        let mut jobs = Vec::new();
        for id in ["a", "b"] {
            let mut config = crate::model::fixture();
            config.districts = Some(1);
            if id == "b" {
                config.seed += 1;
            }
            let dir = paths.store.join("jobs").join(id);
            std::fs::create_dir_all(&dir).unwrap();
            let geometry = json!({"type":"FeatureCollection","state":"RI","year":"2020","private":"secret","features":[{"type":"Feature","properties":{"geoid":"44001000100","state":"RI","county":"44001","district":1,"private":"secret"},"geometry":{"type":"MultiPolygon","coordinates":[]}}]});
            crate::data::write_json(&dir.join("RI_map.json"), &geometry).unwrap();
            crate::data::write_json(&dir.join("RI_assignments.json"), &json!({"44001000100":1}))
                .unwrap();
            jobs.push(Job{id:id.into(),config,status:"completed".into(),created_unix:0,logs:vec!["secret".into()],states:vec![crate::model::StateRun{code:"RI".into(),status:"completed".into(),error:None,elapsed_seconds:1.0,command:vec!["secret".into()],metrics:Some(json!({"district_count":1,"units":1,"graph_path":"secret","districts":[{"district":1,"private":"secret"}]}))}]});
        }
        let out = temp.path().join("site");
        build(&paths, &jobs, &out, 1_000_000).unwrap();
        let manifest: Value =
            serde_json::from_slice(&std::fs::read(out.join("catalog.json")).unwrap()).unwrap();
        assert_eq!(manifest["assets"].as_object().unwrap().len(), 2);
        assert_eq!(manifest["geometries"].as_object().unwrap().len(), 1);
        assert!(!manifest.to_string().contains("secret"));
        for file in std::fs::read_dir(out.join("assets")).unwrap() {
            assert!(!std::fs::read_to_string(file.unwrap().path())
                .unwrap()
                .contains("secret"));
        }
        assert!(build(&paths, &jobs, &out, 1_000_000).is_err());
        let oversized = temp.path().join("too-small");
        assert!(build(&paths, &jobs, &oversized, 1).is_err());
        assert!(!oversized.join("catalog.json").exists());
    }
    #[test]
    fn public_record_omits_process_context() {
        let public = public_metrics(
            &json!({"district_count":2,"graph_path":"C:/private","token":"secret","engine_provenance":{"version":"1","private_path":"secret"}}),
        );
        assert!(public.get("graph_path").is_none());
        assert!(!public.to_string().contains("secret"));
        assert_eq!(public["engine_provenance"]["version"], "1");
    }
    #[test]
    fn joins_preserve_geoid_strings_and_reject_duplicates() {
        let feature = json!({"properties":{"geoid":"01001000100"}});
        let mut geometry = json!({"type":"FeatureCollection","features":[feature.clone()]});
        validate_join(&geometry, &json!({"01001000100":1}), 2).unwrap();
        assert!(validate_join(&geometry, &json!({"44001000100":1}), 2).is_err());
        geometry["features"].as_array_mut().unwrap().push(feature);
        assert!(validate_join(&geometry, &json!({"01001000100":1}), 2).is_err());
    }
    #[test]
    fn matching_ignores_inactive_budgets_but_retains_active_dimensions() {
        let a = crate::model::fixture();
        let mut b = a.clone();
        b.steps = 999;
        b.seeds = 999;
        b.name = "Other".into();
        assert_eq!(effective_config(&a, 2), effective_config(&b, 2));
        b.seed += 1;
        assert_ne!(effective_config(&a, 2), effective_config(&b, 2));
        b = a.clone();
        b.search = "percentile".into();
        let mut c = b.clone();
        c.seeds += 1;
        assert_ne!(effective_config(&b, 2), effective_config(&c, 2));
    }
    #[test]
    fn published_matrices_use_supported_configurations() {
        for (source, experiments, states) in [
            (include_str!("../../../web/lab/matrices/across-scales-2020.json"), 6, 3),
            (include_str!("../../../web/lab/matrices/national-2020.json"), 66, 50),
        ] {
            let matrix: Vec<crate::model::Experiment> = serde_json::from_str(source).unwrap();
            assert_eq!(matrix.len(), experiments);
            for config in matrix {
                assert_eq!(config.states.len(), states);
                config.validate(&config.states).unwrap();
            }
        }
    }
}
