use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {
    pub form: bisect_report::tui_run::Form,
    pub run: crate::config::ConfiguredRun,
    pub created_at: String,
    #[serde(default)]pub target:Option<crate::run_target::Input>,
    #[serde(default)]
    pub workflow: Option<crate::state_workflow::Options>,
}
pub(crate) fn run(mut input: Input) -> Result<Value, String> {
    if let Some(options)=&input.workflow {options.validate()?;}
    let f = &input.form;
    if [
        &f.location,
        &f.chamber,
        &f.year,
        &f.resolution,
        &f.seed,
        &f.label,
        &f.version,
        &f.balance_tol,
    ]
    .iter()
    .any(|s| s.len() > 200 || s.chars().any(char::is_control))
        || input.created_at.is_empty()
        || input.created_at.len() > 200
        || input.created_at.chars().any(char::is_control)
    {
        return Err(
            "Run form fields and export timestamp support 200 UTF-8 bytes without controls.".into(),
        );
    }
    let resolution=match input.run.graph.effective_unit_kind() {
        rplan_core::UnitKind::Tract=>"tract",
        rplan_core::UnitKind::County=>"county",
        rplan_core::UnitKind::BlockGroup=>"block_group",
        rplan_core::UnitKind::Block=>"block",
        rplan_core::UnitKind::Precinct=>"precinct",
        rplan_core::UnitKind::Imported=>"imported",
    };
    if f.resolution!=resolution && !(resolution=="block_group"&&f.resolution=="bg") {
        return Err("Run resolution differs from the selected prepared graph unit kind. Other resolutions require their native input/export adapters; no relabeling or resolution substitution occurred.".into());
    }
    if f.location.to_uppercase() != input.run.graph.state
        || f.year != input.run.graph.year
        || f.chamber != input.run.chamber
    {
        return Err(
            "Run form location/year/chamber differs from selected configured input.".into(),
        );
    }
    bisect_report::label_rename::validate_name(&f.label)?;
    if f.version.is_empty()
        || f.version.len() > 100
        || !f
            .version
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
    {
        return Err("Recorded native output version requires a portable plain name.".into());
    }
    let target= input.target.take().map(|t|crate::run_target::check(t,&f.version,&f.year,&f.label)).transpose()?;
    let seed = if f.seed.is_empty() {
        None
    } else {
        Some(
            f.seed
                .parse::<u64>()
                .map_err(|_| "Run seed must be an exact u64 decimal integer.")?,
        )
    };
    let tolerance = if f.balance_tol.is_empty() {
        None
    } else {
        Some(
            f.balance_tol
                .parse::<f64>()
                .map_err(|_| "Run tolerance must be a native percent number.")?,
        )
    };
    let electoral=input.run.electoral.clone();
    let original_yaml = input.run.yaml_text.clone();
    let effective = bisect_runner::config::run_form_yaml(&original_yaml, seed, tolerance)?;
    input.run.yaml_text = effective.clone();
    crate::progress::emit(
        &json!({"schema_version":"bisect-tui-run-progress-v1","stage":"engine-build","message":"Running native configured population retries"}),
    )?;
    let configured = crate::config::build(input.run.clone())?;
    crate::progress::emit(
        &json!({"schema_version":"bisect-tui-run-progress-v1","stage":"plan-export","message":"Exporting accepted native plan and context"}),
    )?;
    let assignments = serde_json::from_value(configured["result"]["assignments"].clone())
        .map_err(|e| e.to_string())?;
    let attempt = configured["build_report"]["accepted_attempt"]
        .as_u64()
        .map(|n| n as u32);
    let exported = crate::config::export(
        input.run,
        assignments,
        f.label.clone(),
        input.created_at,
        attempt,
    )?;
    let document = exported["document_json"]
        .as_str()
        .ok_or("Missing native plan export.")?;
    let context = exported["context_json"]
        .as_str()
        .ok_or("Missing native context export.")?;
    let manifest = crate::plan_audit_text::report_manifest(document, context)?;
    if input.workflow.is_none(){
    crate::progress::emit(
        &json!({"schema_version":"bisect-tui-run-progress-v1","stage":"complete","message":"Native plan, context and report manifest prepared"}),
    )?;
    }
    let mut output=
        json!({"schema_version":"bisect-tui-run-v1","status":"completed","form":input.form,"native_state_arguments":bisect_report::tui_run::state_arguments(f),"original_yaml_sha256":format!("{:x}",Sha256::digest(original_yaml.as_bytes())),"effective_yaml":effective,"effective_yaml_sha256":format!("{:x}",Sha256::digest(effective.as_bytes())),"configured_run":configured,"practitioner_export":exported,"report_manifest":manifest,"recorded_output_namespace":format!("outputs/{}/{}/plans/{}",f.version,f.year,f.label),"execution_performed":true,"scope":if ["precinct","imported"].contains(&resolution){"Configured native-core Run on an explicitly supplied precinct/imported graph with aligned county identities. Unit IDs, county labels, populations, edges, declared unit year and selected options bind the exported RPLAN/RCTX. Recorded native form arguments describe selections, not a filesystem acquisition command for these units. No data acquisition, precinct geography preparation or source authentication occurs; broader resolution adapters and command orchestration remain unfinished. Open stays inert."}else{"Shared native TUI Run arguments and explicit form seed/tolerance overrides applied through the native YAML schema. All selected compositor options are retained; an explicit supported engine is required. Runs native configured population retries on the selected prepared Census graph, exports accepted RPLAN/RCTX and constructs the native report manifest. Effective and original YAML identities remain separate. Native version/output namespace is recorded text, not a filesystem installation. Fetching, source preparation for additional resolutions, precinct/imported identity adapters, all-state scheduling, automatic analysis/maps, external scripts and full native State orchestration remain unfinished. Progress reports real wrapper stages without inventing per-depth engine progress. Save/Open is inert; analysis/audit/map tools require separate explicit handoffs and runs."}});
    if let Some(evidence)=target {if evidence["force"]!=true {output["native_state_arguments"].as_array_mut().ok_or("Missing native State arguments")?.retain(|arg|arg!="--force");}output["target_preflight"]=evidence;}
    if let Some(settings)=electoral {
        let args=output["native_state_arguments"].as_array_mut().ok_or("Missing native State arguments.")?;
        args.push(json!("--seats-per-district"));args.push(json!(settings.seats_per_district.to_string()));
        if let Some(total)=settings.total_seats {args.push(json!("--total-seats"));args.push(json!(total.to_string()));}
    }
    if let Some(options)=input.workflow {output["workflow"]=crate::state_workflow::run(&output,options)?;crate::progress::emit(&json!({"schema_version":"bisect-state-workflow-progress-v1","stage":"complete","message":"Selected follow-up stages finished; inspect their outcomes"}))?;}
    Ok(output)
}
