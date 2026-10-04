//! Practitioner operations use the same native audit implementations in memory.
use rplan_audit::{
    AlgorithmLineage, AuditCertificate, AuditConstraint, LegalProfile, RuntimeProvenance,
};
use rplan_core::{DistrictPlan, RplanContext};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ImportMetadata { country:String, state:String, county:String, date:String, election_type:String, scope:String, status:String }

#[derive(Deserialize)]
#[serde(tag = "operation", rename_all = "kebab-case", deny_unknown_fields)]
enum ToolRequest {
    RunMultiscale { input:crate::multiscale_input::MultiscaleRequest },
    ExportEnginePlan { request:crate::engine::Request, assignments:std::collections::BTreeMap<String,u32>, label:String, chamber:String, created_at:String },
    AttachDemographicCsv { document:Value, context:Value, source_base64:String, basis:String, source_label:String },
    ImportDemographicCsv { source_base64:String, state:String, year:String, basis:String, source_label:String },
    ImportElectionCountsCsv { source_base64:String, state:String, year:String, election_year:String, source_label:String },
    ImportPartisanSharesTsv { source_base64:String, state:String, year:String, source_label:String },
    ImportCharacterCsv { source_base64:String, state:String, year:String, data_year:String, source_label:String, kind:String },
    BuildCharacterWeights { input:crate::character_input::CharacterInput, geoids:Vec<String>, edges:Vec<(usize,usize,f64)>, alpha:f64 },
    VerifyHistoryFiles { files:rhist_io::PackageFiles },
    ImportStatementCsv { source_base64:String, metadata:ImportMetadata },
    ImportNistCdfJson { source_base64:String, metadata:ImportMetadata },
    ImportRi2024Rep28Rla { audit_report_base64:String, ballot_manifest_base64:String, ballot_retrieval_base64:String },
    AggregateDistricts { files:rcount_io::PackageFiles, document:Value, context:Option<Value>, crosswalk_base64:Option<String>, contest_id:String, status:rcount_core::CountStatus, output_format:String },
    ValidateRplan {
        document: Value,
    },
    AuditPlan {
        plan: DistrictPlan,
        context: Option<RplanContext>,
        profile: LegalProfile,
        constraints: Vec<AuditConstraint>,
        generated_at_utc: String,
        lineage: Option<AlgorithmLineage>,
    },
    VerifyCertificate {
        certificate: AuditCertificate,
        plan: Option<DistrictPlan>,
        context: Option<RplanContext>,
    },
    VerifyCountFiles { files: rcount_io::PackageFiles },
    ReplayCountAudits { files: rcount_io::PackageFiles },
    VerifyCount {
        package: rcount_core::RcountPackage,
    },
}

pub fn execute(input: Value) -> Result<Value, String> {
    let request: ToolRequest =
        serde_json::from_value(input).map_err(|e| format!("Invalid practitioner request: {e}"))?;
    match request {
        ToolRequest::RunMultiscale { input } => crate::multiscale_input::execute(input),
        ToolRequest::ExportEnginePlan {request,assignments,label,chamber,created_at} => crate::plan_export::export(request,assignments,label,chamber,created_at),
        ToolRequest::AttachDemographicCsv {document,context,source_base64,basis,source_label} => attach_demographic_csv(document,context,source_base64,basis,source_label),
        ToolRequest::ImportDemographicCsv {source_base64,state,year,basis,source_label} => {
            use base64::{Engine,engine::general_purpose::STANDARD};
            const LIMIT:usize=8*1024*1024;
            if source_base64.len()>LIMIT.div_ceil(3)*4 {return Err("Demographic CSV exceeds 8 MiB.".into());}
            let bytes=STANDARD.decode(source_base64).map_err(|_|"Invalid demographic source encoding.")?;
            let input=bisect_data::demographics::import_demographic_csv(&bytes,&state,&year,&basis,&source_label)?;
            serde_json::to_value(input).map_err(|e|e.to_string())
        },
        ToolRequest::ImportElectionCountsCsv {source_base64,state,year,election_year,source_label} => {
            use base64::{Engine,engine::general_purpose::STANDARD};
            const LIMIT:usize=8*1024*1024;
            if source_base64.len()>LIMIT.div_ceil(3)*4 {return Err("Election CSV exceeds 8 MiB.".into());}
            let bytes=STANDARD.decode(source_base64).map_err(|_|"Invalid election source encoding.")?;
            serde_json::to_value(crate::election_input::import_csv(&bytes,&state,&year,&election_year,&source_label)?).map_err(|e|e.to_string())
        },
        ToolRequest::ImportPartisanSharesTsv {source_base64,state,year,source_label} => {
            use base64::{Engine,engine::general_purpose::STANDARD};
            const LIMIT:usize=8*1024*1024;
            if source_base64.len()>LIMIT.div_ceil(3)*4 {return Err("Partisan TSV exceeds 8 MiB.".into());}
            let bytes=STANDARD.decode(source_base64).map_err(|_|"Invalid partisan source encoding.")?;
            serde_json::to_value(crate::partisan_input::import_tsv(&bytes,&state,&year,&source_label)?).map_err(|e|e.to_string())
        },
        ToolRequest::ImportCharacterCsv {source_base64,state,year,data_year,source_label,kind} => {
            use base64::{Engine,engine::general_purpose::STANDARD};
            const LIMIT:usize=8*1024*1024;
            if source_base64.len()>LIMIT.div_ceil(3)*4 {return Err("Character CSV exceeds 8 MiB.".into());}
            let bytes=STANDARD.decode(source_base64).map_err(|_|"Invalid character source encoding.")?;
            serde_json::to_value(crate::character_input::import_csv(&bytes,&state,&year,&data_year,&source_label,&kind)?).map_err(|e|e.to_string())
        },
        ToolRequest::BuildCharacterWeights {input,geoids,edges,alpha} => serde_json::to_value(crate::character_input::build_weights(&input,&geoids,&edges,alpha)?).map_err(|e|e.to_string()),
        ToolRequest::VerifyHistoryFiles { files } => {
            if files.len()>2000 || files.values().map(Vec::len).sum::<usize>()>8*1024*1024 { return Err("History package exceeds the portable package limit.".into()); }
            serde_json::to_value(rhist_io::verification_transcript(rhist_io::read_package_files(&files)).map_err(|e|e.to_string())?).map_err(|e|e.to_string())
        },
        ToolRequest::ImportStatementCsv { source_base64, metadata } => import_election(source_base64,metadata,false),
        ToolRequest::ImportNistCdfJson { source_base64, metadata } => import_election(source_base64,metadata,true),
        ToolRequest::ImportRi2024Rep28Rla { audit_report_base64,ballot_manifest_base64,ballot_retrieval_base64 } => import_ri_rla([audit_report_base64,ballot_manifest_base64,ballot_retrieval_base64]),
        ToolRequest::AggregateDistricts {files,document,context,crosswalk_base64,contest_id,status,output_format} => aggregate_districts(files,document,context,crosswalk_base64,contest_id,status,output_format),
        ToolRequest::ValidateRplan { document } => {
            let document =
                rplan_io::read_rplan_str(&document.to_string()).map_err(|e| e.to_string())?;
            Ok(
                json!({"document":document,"plan_hash":document.plan.plan_hash().map_err(|e|e.to_string())?}),
            )
        }
        ToolRequest::AuditPlan {
            plan,
            context,
            profile,
            constraints,
            generated_at_utc,
            lineage,
        } => {
            // Native file callers validate RCTX before auditing. Apply that
            // same boundary to typed browser requests, including array lengths
            // and context identity, before any indexed demographic traversal.
            plan.validate().map_err(|e|e.to_string())?;
            let context=context.map(|ctx|rplan_io::read_rctx_str(&serde_json::to_string(&ctx).map_err(|e|e.to_string())?).map_err(|e|e.to_string())).transpose()?;
            if let Some(ctx)=&context {
                if ctx.units.unit_ids!=plan.units.unit_ids || ctx.units.unit_kind!=plan.units.unit_kind || ctx.units.state!=plan.units.state || ctx.units.year!=plan.units.year || ctx.units.unit_universe_hash!=plan.units.unit_universe_hash || plan.assignment.len()!=ctx.units.unit_ids.len() {return Err("Plan/context unit identity or assignment length mismatch.".into());}
            }
            let runtime = RuntimeProvenance {
                binary_name: "bisect-wasm".into(),
                binary_version: env!("CARGO_PKG_VERSION").into(),
                git_commit: None,
                build_profile: Some("wasm-browser".into()),
                solver: None,
            };
            let certificate = rplan_audit::audit_plan_with_lineage(
                &plan,
                context.as_ref(),
                &profile,
                runtime,
                &constraints,
                &generated_at_utc,
                lineage,
            )
            .map_err(|e| e.to_string())?;
            serde_json::to_value(certificate).map_err(|e| e.to_string())
        }
        ToolRequest::VerifyCertificate {
            certificate,
            plan,
            context,
        } => {
            let verification = rplan_audit::verify_audit_certificate(
                &certificate,
                plan.as_ref(),
                context.as_ref(),
            )
            .map_err(|e| e.to_string())?;
            Ok(
                json!({"certificate_id":verification.certificate_id,"content_hash":verification.content_hash,"plan_hash":verification.plan_hash,"context_hash":verification.context_hash,"result":verification.result}),
            )
        }
        ToolRequest::VerifyCountFiles { files } => {
            serde_json::to_value(rcount_audit::verify_package_files(&files)).map_err(|e|e.to_string())
        }
        ToolRequest::ReplayCountAudits { files } => {
            let verification = rcount_audit::verify_package_files(&files);
            let (_,package) = rcount_io::read_package_files(&files).map_err(|e|e.to_string())?;
            let runs: Vec<_> = package.audit_algorithm_runs.iter().map(rcount_audit::replay_audit_algorithm_statistics).collect();
            let status = if verification.status == rcount_audit::VerificationStatus::Fail || runs.iter().any(|r|r.status == rcount_audit::AlgorithmReplayStatus::Fail) {"fail"}
                else if runs.is_empty() || runs.iter().any(|r|r.status == rcount_audit::AlgorithmReplayStatus::Boundary) {"boundary"} else {"pass"};
            Ok(json!({"status":status,"verification":verification,"runs":runs}))
        }
        ToolRequest::VerifyCount { package } => {
            let verification = rcount_core::verify_package(&package).map_err(|e| e.to_string())?;
            serde_json::to_value(verification).map_err(|e| e.to_string())
        }
    }
}

fn attach_demographic_csv(document:Value, context:Value, source_base64:String, basis:String, source_label:String) -> Result<Value,String> {
    use base64::{Engine,engine::general_purpose::STANDARD};
    use sha2::{Digest,Sha256};
    const LIMIT:usize=8*1024*1024;
    if !["voting-age-population","citizen-voting-age-population"].contains(&basis.as_str()) {return Err("VRA context requires explicit VAP or CVAP counts, not a total-population proxy.".into());}
    if source_base64.len()>LIMIT.div_ceil(3)*4 {return Err("Demographic CSV exceeds 8 MiB.".into());}
    let bytes=STANDARD.decode(source_base64).map_err(|_|"Invalid demographic source encoding.")?;
    let document=rplan_io::read_rplan_str(&document.to_string()).map_err(|e|e.to_string())?;
    if document.plan.k>500 {return Err("Portable demographic reporting supports at most 500 districts.".into());}
    let mut context=rplan_io::read_rctx_str(&context.to_string()).map_err(|e|e.to_string())?;
    let units=&document.plan.units;
    if units.unit_kind!=rplan_core::UnitKind::Tract || context.units.unit_kind!=units.unit_kind || context.units.unit_ids!=units.unit_ids || context.units.state!=units.state || context.units.year!=units.year || context.units.unit_universe_hash!=units.unit_universe_hash {return Err("Demographic CSV attachment requires matching tract plan/context unit order and scope.".into());}
    let state=units.state.as_deref().ok_or("Plan needs an explicit state for demographic import.")?;
    let year=units.year.ok_or("Plan needs an explicit year for demographic import.")?.to_string();
    let input=bisect_data::demographics::import_demographic_csv(&bytes,state,&year,&basis,&source_label)?;
    if input.counts.len()!=units.unit_ids.len() || units.unit_ids.iter().any(|id|!input.counts.contains_key(id)) {return Err("Demographic CSV must cover every context tract exactly; missing counts are not zero filled.".into());}
    let total_vap=units.unit_ids.iter().map(|id|input.counts[id].total).collect::<Vec<_>>();
    let minority_vap=units.unit_ids.iter().map(|id|input.counts[id].minority).collect::<Vec<_>>();
    let previous_context_hash=context.context_hash.clone();
    context.demographics=Some(rplan_core::DemographicContext{total_vap:Some(total_vap.clone()),minority_vap:Some(minority_vap.clone())});
    let source_sha256=format!("sha256:{:x}",Sha256::digest(&bytes));
    let demographics_sha256=rplan_core::canonical_sha256(&serde_json::to_value(&input).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
    context.source_hashes.entries.retain(|key,_|!key.starts_with("bisect.demographics."));
    context.source_hashes.entries.insert(format!("bisect.demographics.csv.{basis}"),source_sha256.clone());
    context.source_hashes.entries.insert("bisect.demographics.counts".into(),demographics_sha256.clone());
    context.validate().map_err(|e|e.to_string())?;
    context.context_hash=context.compute_context_hash().map_err(|e|e.to_string())?;
    let mut totals=vec![0.0;document.plan.k];let mut minorities=vec![0.0;document.plan.k];
    for (i,district) in document.plan.assignment.iter().enumerate(){totals[*district as usize]+=total_vap[i];minorities[*district as usize]+=minority_vap[i];}
    if totals.iter().any(|v|!v.is_finite()||*v>9_007_199_254_740_991.0) {return Err("District demographic total exceeds the portable browser range.".into());}
    let district_totals=(0..document.plan.k).map(|i|json!({"district_id":i,"total":totals[i],"minority":minorities[i],"share":if totals[i]==0.0 {None}else{Some(minorities[i]/totals[i])}})).collect::<Vec<_>>();
    Ok(json!({"schema_version":"bisect-demographic-context-v1","context":context,"basis":basis,"source_label":source_label,"source_sha256":source_sha256,"demographics_sha256":demographics_sha256,"previous_context_hash":previous_context_hash,"context_hash":context.context_hash,"plan_hash":document.plan.plan_hash().map_err(|e|e.to_string())?,"unit_count":units.unit_ids.len(),"district_totals":district_totals,"source_boundary":"Counts aligned to the existing context unit order and included in its native hash. Original CSV bytes remain in the project. Source provenance and legal compliance are not authenticated; native RCTX field names remain total_vap/minority_vap for either explicitly selected count basis."}))
}

fn aggregate_districts(files:rcount_io::PackageFiles, document:Value, context:Option<Value>, crosswalk_base64:Option<String>, contest_id:String, status:rcount_core::CountStatus, output_format:String) -> Result<Value,String> {
    use base64::{Engine,engine::general_purpose::STANDARD};
    const LIMIT:usize=8*1024*1024;
    if files.len()>2000||files.values().map(Vec::len).sum::<usize>()>LIMIT {return Err("Aggregation package exceeds the portable package limit.".into());}
    if contest_id.trim().is_empty()||contest_id.len()>200 {return Err("Supply a contest ID of 1–200 bytes.".into());}
    let (_,package)=rcount_io::read_package_files(&files).map_err(|e|e.to_string())?;
    let plan=rplan_io::read_rplan_str(&document.to_string()).map_err(|e|e.to_string())?;
    if plan.plan.k>500 {return Err("Browser aggregation supports at most 500 districts.".into());}
    let context=context.map(|value|rplan_io::read_rctx_str(&value.to_string())).transpose().map_err(|e|e.to_string())?;
    let crosswalk=crosswalk_base64.map(|source|{if source.len()>LIMIT.div_ceil(3)*4 {return Err("Crosswalk exceeds 8 MiB.".to_string());}let bytes=STANDARD.decode(source).map_err(|e|e.to_string())?;if bytes.is_empty()||bytes.len()>LIMIT{return Err("Select a nonempty crosswalk of at most 8 MiB.".into());}Ok(bytes)}).transpose()?;
    let transcript=rcount_district::aggregate_package_districts_bytes(&package,&plan.plan,context.as_ref(),crosswalk.as_deref(),&contest_id,status).map_err(|e|e.to_string())?;
    let exact=match output_format.as_str(){"json"=>serde_json::to_vec(&transcript),"pretty-json"=>serde_json::to_vec_pretty(&transcript),_=>return Err("Select json or pretty-json transcript output.".into())}.map_err(|e|e.to_string())?;
    if exact.len()>LIMIT {return Err("Aggregation transcript exceeds 8 MiB.".into());}
    let totals:Vec<_>=transcript.district_totals.iter().map(|district|json!({"district_id":district.district_id,"district_label":district.district_label,"source_reporting_unit_count":district.source_reporting_unit_ids.len(),"counted_ballots":district.summary.counted_ballots.to_string(),"undervotes":district.summary.undervotes.to_string(),"overvotes":district.summary.overvotes.to_string(),"blank_contests":district.summary.blank_contests.to_string(),"totals":district.summary.totals.iter().map(|total|json!({"selection_id":total.selection_id,"votes":total.votes.to_string()})).collect::<Vec<_>>()})).collect();
    let verification=rcount_audit::verify_package_files(&files);
    Ok(json!({"schema_version":"bisect-district-aggregation-v1","status":verification.status,"verification":verification,"contest_id":contest_id,"count_status":status,"district_totals":totals,"aggregation_checks":transcript.checks,"rplan_plan_hash":transcript.rplan_plan_hash,"rcount_package_content_hash":transcript.rcount_package_content_hash,"rctx_context_hash":transcript.rctx_context_hash,"rctx_crosswalk_hash":transcript.rctx_crosswalk_hash,"transcript_base64":STANDARD.encode(exact),"output_format":output_format,"source_boundary":"Shared native aggregation, including explicit integral crosswalk allocation. Counts are decimal strings in this view; the downloaded native transcript retains exact integer JSON. Package/source checks do not certify an election or authenticate imported provenance."}))
}

fn import_ri_rla(sources:[String;3]) -> Result<Value,String> {
    use base64::{Engine,engine::general_purpose::STANDARD};
    const LIMIT:usize=8*1024*1024;
    if sources.iter().map(String::len).sum::<usize>()>LIMIT.div_ceil(3)*4+8 {return Err("Combined RI audit sources exceed 8 MiB.".into());}
    let bytes=sources.iter().map(|source|STANDARD.decode(source).map_err(|e|format!("Invalid source encoding: {e}"))).collect::<Result<Vec<_>,_>>()?;
    if bytes.iter().any(Vec::is_empty)||bytes.iter().map(Vec::len).sum::<usize>()>LIMIT {return Err("Select three nonempty RI audit sources totaling at most 8 MiB.".into());}
    let package=rcount_io::import_ri_2024_rep28_ballot_polling_audit_bytes(&bytes[0],&bytes[1],&bytes[2]).map_err(|e|e.to_string())?;
    let mut manifest=rcount_io::ri_2024_rep28_manifest(&package).map_err(|e|e.to_string())?;
    manifest.created_by=rcount_io::CreatedBy{tool:"bisect-wasm-ri-rla-import".into(),version:env!("CARGO_PKG_VERSION").into()};
    let files=rcount_io::write_ri_2024_rep28_package_files(&bytes[0],&bytes[1],&bytes[2],&manifest,&package).map_err(|e|e.to_string())?;
    if files.values().map(Vec::len).sum::<usize>()>LIMIT {return Err("Generated RI audit package exceeds 8 MiB.".into());}
    let verification=rcount_audit::verify_package_files(&files);
    let encoded:std::collections::BTreeMap<_,_>=files.iter().map(|(path,bytes)|(path.clone(),STANDARD.encode(bytes))).collect();
    let summary=rcount_io::ri_2024_rep28_source_summary_bytes(&bytes[0],&bytes[2]).map_err(|e|e.to_string())?;
    Ok(json!({"schema_version":"bisect-election-import-v1","adapter":"ri-2024-rep28-rla","status":verification.status,"manifest":manifest,"package_files":encoded,"verification":verification,"source_summary":summary,
        "source_boundary":"RI 2024 Representative 28 adapter only. Three original sources are retained and hashed. Sample keys are cross-checked; recorded risk calculations and human observations are not independently verified. Replay exposes the native boundary status."}))
}

fn import_election(source_base64:String, metadata:ImportMetadata, nist:bool) -> Result<Value,String> {
    use base64::{Engine,engine::general_purpose::STANDARD};
    const LIMIT:usize=8*1024*1024;
    if source_base64.len()>LIMIT.div_ceil(3)*4 {return Err("Election source exceeds 8 MiB.".into());}
    let bytes=STANDARD.decode(source_base64).map_err(|e|format!("Invalid source encoding: {e}"))?;
    if bytes.is_empty()||bytes.len()>LIMIT {return Err("Select a nonempty election source of at most 8 MiB.".into());}
    for value in [&metadata.country,&metadata.state,&metadata.county,&metadata.date,&metadata.election_type,&metadata.scope,&metadata.status] {
        if value.trim().is_empty()||value.len()>200 {return Err("Supply explicit election and jurisdiction metadata (1–200 bytes per field).".into());}
    }
    let package=if nist {rcount_io::import_nist_cdf_json_bytes(&bytes)} else {rcount_io::import_statement_csv_bytes(&bytes)}.map_err(|e|e.to_string())?;
    let manifest=rcount_io::RcountManifest {
        rcount_version:rcount_core::RCOUNT_VERSION.into(),
        jurisdiction:rcount_io::Jurisdiction {country:metadata.country,state:metadata.state,county:metadata.county},
        election:rcount_io::Election {date:metadata.date,election_type:metadata.election_type,scope:metadata.scope},
        status:metadata.status,hash_algorithm:"sha256".into(),content_hash:rcount_core::package_content_hash(&package).map_err(|e|e.to_string())?,
        created_by:rcount_io::CreatedBy {tool:"bisect-wasm-browser-import".into(),version:env!("CARGO_PKG_VERSION").into()}
    };
    let files=if nist {rcount_io::write_nist_cdf_package_files(&bytes,&manifest,&package)} else {rcount_io::write_statement_csv_package_files(&bytes,&manifest,&package)}.map_err(|e|e.to_string())?;
    if files.values().map(Vec::len).sum::<usize>()>LIMIT {return Err("Generated election package exceeds the 8 MiB portable-package limit.".into());}
    let verification=rcount_audit::verify_package_files(&files);
    let encoded:std::collections::BTreeMap<_,_>=files.iter().map(|(path,bytes)|(path.clone(),STANDARD.encode(bytes))).collect();
    Ok(json!({"schema_version":"bisect-election-import-v1","adapter":if nist {"nist-cdf-json"} else {"statement-csv"},"status":verification.status,
        "manifest":manifest,"package_files":encoded,"verification":verification,
        "source_boundary":"Native adapter subset; original source bytes retained. Verification checks the package and recorded equations, not election certification."}))
}
