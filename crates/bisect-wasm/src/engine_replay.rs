//! Execute an exact selected portable run artifact and compare its assignments.
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {
    pub run_json: String,
    pub original_assignments_json: String,
    pub min_similarity: f64,
}
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
enum Run {
    Engine {
        request: crate::engine::Request,
    },
    EngineBuild {
        request: crate::engine::Request,
    },
    Multiscale {
        input: crate::multiscale_input::MultiscaleRequest,
        #[serde(default)]
        fine_edges: Option<Vec<(usize, usize, f64)>>,
    },
    MultiscaleBuild {
        input: crate::multiscale_input::MultiscaleRequest,
        #[serde(default)]
        fine_edges: Option<Vec<(usize, usize, f64)>>,
    },
    Configured {
        input: crate::config::ConfiguredRun,
        #[serde(default)]
        attempt:Option<u32>,
    },
    ConfiguredBuild {
        input: crate::config::ConfiguredRun,
    },
}
impl Run {
    fn kind(&self) -> &'static str {
        match self {
            Self::Engine { .. } => "engine",
            Self::EngineBuild { .. } => "engine-build",
            Self::Multiscale { .. } => "multiscale",
            Self::MultiscaleBuild { .. } => "multiscale-build",
            Self::Configured { .. } => "configured",
            Self::ConfiguredBuild { .. } => "configured-build",
        }
    }
    fn execute(self) -> Result<Value, String> {
        match self {
            Self::Engine { request } => crate::engine::execute(request),
            Self::EngineBuild { request } => crate::engine_build::run(request),
            Self::Multiscale { input, .. } => crate::multiscale_input::execute(input),
            Self::MultiscaleBuild { input, .. } => crate::engine_build::run_multiscale(input),
            Self::Configured { input,attempt } => crate::config::run_at(input,attempt),
            Self::ConfiguredBuild { input } => crate::config::build(input),
        }
    }
}
/// Resolve inputs for correspondence checks without executing the engine.
pub(crate) fn binding_inputs(text: &str) -> Result<Value, String> {
    let run: Run =
        serde_json::from_str(text).map_err(|e| format!("Invalid replay run artifact: {e}"))?;
    let configured_input = match &run {
        Run::Configured {input,..} | Run::ConfiguredBuild {input} => Some(serde_json::to_value(input).map_err(|e|e.to_string())?),
        _ => None,
    };
    let configured_attempt=match &run {Run::Configured{attempt,..}=>*attempt,_=>None};
    let (request, fine_ids, multiscale, chamber, fine_edges) = match run {
        Run::Engine { request } | Run::EngineBuild { request } => (request, None, None, None, None),
        Run::Configured { input,attempt } => {
            let chamber=input.chamber.clone();let (_,request)=crate::config::prepare_at(input,attempt)?;
            (request,None,None,Some(chamber),None)
        }
        Run::ConfiguredBuild { input } => {
            let chamber = input.chamber.clone();
            let (_, request) = crate::config::prepare(input)?;
            (request, None, None, Some(chamber), None)
        }
        Run::Multiscale { input, fine_edges } | Run::MultiscaleBuild { input, fine_edges } => {
            let ids = input.block_groups.as_ref().map(|g| g.geoids.clone());
            let value = serde_json::to_value(&input).map_err(|e| e.to_string())?;
            (input.request, ids, Some(value), None, fine_edges)
        }
    };
    let mut output=json!({"unit_ids":fine_ids.unwrap_or_else(||request.graph.geoids.clone()),"request":request,"multiscale_input":multiscale,"chamber":chamber,"fine_edges":fine_edges,"configured_input":configured_input});
    if let Some(attempt)=configured_attempt {output["configured_attempt"]=json!(attempt);}
    Ok(output)
}
pub(crate) fn replay(input: Input) -> Result<Value, String> {
    if input.run_json.is_empty()
        || input.original_assignments_json.is_empty()
        || input.run_json.len() > 16 * 1024 * 1024
        || input.original_assignments_json.len() > 8 * 1024 * 1024
        || input.run_json.len() + input.original_assignments_json.len() > 16 * 1024 * 1024
        || !input.min_similarity.is_finite()
    {
        return Err("Select a run artifact and assignment JSON totaling at most 16 MiB, assignments at most 8 MiB, and a finite comparison threshold.".into());
    }
    let original = crate::assignment_verification::assignments(&input.original_assignments_json)?;
    let mut kind = None;
    let mut generated = None;
    let mut agreement = None;
    let mut error: Option<String> = None;
    let mut execution_performed = false;
    let status = if original.is_empty() {
        "unavailable"
    } else {
        let outcome = (|| {
            let run: Run = serde_json::from_str(&input.run_json)
                .map_err(|e| format!("Invalid replay run artifact: {e}"))?;
            kind = Some(run.kind());
            execution_performed = true;
            let result = run.execute()?;
            let output = if result["schema_version"] == "bisect-configured-run-v1" {
                &result["result"]
            } else {
                &result
            };
            let assignments = output
                .get("assignments")
                .ok_or("Replay engine did not return assignments.")?;
            let reference = crate::assignment_verification::assignments(
                &serde_json::to_string(assignments).map_err(|e| e.to_string())?,
            )?;
            agreement = Some(
                bisect_report::assignment_verification::assignment_agreement(&original, &reference),
            );
            generated = Some(result);
            Ok(())
        })();
        match outcome {
            Ok(()) => {
                if agreement.as_ref().unwrap().similarity >= input.min_similarity {
                    "match"
                } else {
                    "different"
                }
            }
            Err(e) => {
                error = Some(e);
                "failed"
            }
        }
    };
    let output = generated.as_ref().map(|v| {
        if v["schema_version"] == "bisect-configured-run-v1" {
            &v["result"]
        } else {
            v
        }
    });
    let seed = output.map(|v| {
        v.get("options")
            .and_then(|o| o.get("seed"))
            .or_else(|| v.get("seed"))
            .or_else(|| v.get("evidence").and_then(|e| e.get("seed")))
            .cloned()
            .unwrap_or(Value::Null)
    });
    let build_report = generated
        .as_ref()
        .and_then(|v| v.get("build_report"))
        .cloned();
    let mut result = json!({"schema_version":"bisect-engine-replay-verification-v1","status":status,"comparison_exit_code":if status=="match"{0}else{1},"run_kind":kind,"execution_performed":execution_performed,"execution_completed":generated.is_some(),"error":error,"agreement":agreement,"min_similarity":input.min_similarity,"replayed_seed":seed,"build_report":build_report,"replayed_result":generated,"run_source_sha256":format!("{:x}",Sha256::digest(input.run_json.as_bytes())),"original_source_sha256":format!("{:x}",Sha256::digest(input.original_assignments_json.as_bytes())),"current_build":{"version":env!("CARGO_PKG_VERSION"),"build_commit":env!("BISECT_BUILD_COMMIT"),"rustc_version":env!("BISECT_RUSTC_VERSION")},"scope":"Actually reruns the selected portable engine/configured/multiscale artifact with all supplied graph, option and auxiliary inputs, including exact optional u64 seed. Build variants repeat the native population retry policy. Optional multiscale fine_edges are retained as selected audit-context evidence and do not change sampler execution. Assignments are compared by the shared native literal district agreement rule and selected threshold; district labels are not permuted. An empty original aborts before execution. Recorded input hashes identify selected bytes, not their author or original execution. A missing seed retains the engine's native behavior and does not establish deterministic replay. Configuration variants use this build's compositor and embedded policy; effective options and generated results are retained. No manifest sidecar checks, native CLI state discovery/fetching, binary identity attestation, external solver execution, legal certification or optimality proof is asserted."});
    if generated.is_some() && matches!(kind, Some("multiscale" | "multiscale-build")) {
        result["effective_seed"] = if result["replayed_seed"].is_null() { json!(0) } else { result["replayed_seed"].clone() };
        result["seed_policy"] = json!("Native multiscale uses the recorded accepted seed, or zero when no seed was selected. The declared seed remains null when omitted; portable sampling retains its recorded RNG.");
    }
    Ok(result)
}
