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
    InspectNativeStateCommand {arguments:Vec<String>},
    InspectNativeStateIdentity {arguments:Vec<String>},
    InspectNativeStateReportSources {arguments:Vec<String>, #[serde(default)] observations:Option<Vec<crate::state_report_sources::Observation>>, #[serde(default)] unit_ids:Option<Vec<String>>, #[serde(default)] observation_origin:Option<String>},
    BindNativeStateReportSources {input:crate::state_report_binding::Input},
    InspectNativeStatesCommand {arguments:Vec<String>},
    InspectNativeRunCommand {arguments:Vec<String>},
    PrepareNativeRunGraphRun {input:crate::native_state_run::GraphInput},
    PrepareNativeRunRun {input:crate::native_state_run::Input},
    PrepareNativeStatesGraphRun {input:crate::native_state_run::GraphInput},
    PrepareNativeStatesRun {input:crate::native_state_run::Input},
    ScopeNativeStateCommand {arguments:Vec<String>,state:String,year:String,resolution:String,districts:usize},
    PrepareNativeStateRun {input:crate::native_state_run::Input},
    PrepareNativeStateGraphRun {input:crate::native_state_run::GraphInput},
    PrepareNativeAdjacencyStateRun {input:crate::native_state_run::AdjacencyInput},
    PrepareFreshNativeStateRun {input:crate::fresh_native_state::Input},
    RunFreshNativeState {input:crate::fresh_native_state::Input},
    InspectNativeMigrationBase {input:crate::migration_base::Selection},
    StageNativeMigrationBase {input:crate::migration_base::Input},
    ImportNativePlan {input:crate::native_import::Input},
    ExportNativePlanFolder {input:crate::export_folder::Input},
    AssembleCommissionAudit {#[serde(default)] manifest_json:Option<String>, #[serde(default)] manifest_bytes:Option<Vec<u8>>, #[serde(default)] fallback:Option<bisect_report::commission_manifest::ReportFallback>},
    AssembleCommissionReport {files:std::collections::BTreeMap<String,Vec<u8>>,generated_at:String, #[serde(default)] fallback:Option<bisect_report::commission_manifest::ReportFallback>, #[serde(default)] delivery:Option<bisect_report::report_delivery::ReportDeliverySettings>},
    ImportLabelAssignments {input:crate::label_import::Input},
    InspectPlanHub {input:crate::plan_hub::Input},
    PrepareLabelImportPlan {input:crate::label_import::PlanInput},
    RunLabelBuild {input:crate::label_build::Input},
    PlanLabelBuild {input:crate::label_build::Input,dry_run:bool},
    AssembleLabelBuildIndex {input:crate::build_index::Input},
    InspectLabelComparison {registry_json:Option<String>,label_a:String,label_b:String,year:String},
    InspectLabelRegistry {registry_json:Option<String>,label:Option<String>},
    RunTuiConfigured {input:crate::tui_run::Input},
    VerifyTuiManifest {input:crate::tui_verify::Input,runtime_artifact:Option<crate::manifest::RuntimeArtifact>},
    DiagnoseTuiLocation {input:crate::tui_doctor::Input},
    CompareTuiPlans {input:crate::tui_compare::Input},
    InspectTuiPlanLibrary {input:crate::tui_library::Input},
    InspectTuiSession {source_name:Option<String>,session_toml:Option<String>},
    ExportTuiSession {settings:crate::tui_session::Settings},
    RenameLabelPackage {registry_json:String,files_base64:std::collections::BTreeMap<String,String>,directories:Vec<String>,source_label:String,destination_label:String,force:bool},
    VerifyLabelChain {registry_json:Option<String>,files_base64:std::collections::BTreeMap<String,String>,label:String,year_filter:Option<String>},
    MigrateLegacyPlan {files_base64:std::collections::BTreeMap<String,String>,settings:crate::migration::Settings},
    IngestCandidateRace {csv_base64:String,source_name:String,documents:std::collections::BTreeMap<String,String>,staged_path:String,settings:crate::candidate::Settings},
    InspectCivicManifests {inputs:Vec<crate::civic::ManifestInput>},
    IngestCivicCoi {input:crate::civic::Input,state:String,year:String,validate_mode:String,settings:crate::civic::IngestSettings},
    InspectCivicCoi {inputs:Vec<crate::civic::Input>,state:String,year:String,validate_mode:String},
    InspectNativeCivicConflicts {inputs:Vec<crate::native_civic_conflicts::NormalizedInput>},
    AssemblePlanSuite {house_json:String,senate_json:String,congressional_json:Option<String>,suite_name:String,nest_mode:String,required_ratio:Option<usize>,policy_source_name:Option<String>,policy_json:Option<String>},
    ValidateNativeRplan {source_name:String,document_json:String,strict:bool},
    DiagnosePlanArtifacts {plans:Vec<crate::plan_doctor::PlanInput>},
    VerifyTutorialData {tutorial:String,source_name:String,checksums_json:String,files:std::collections::BTreeMap<String,crate::tutorial::FileDigest>},
    VerifyRunManifest {source_name:String,manifest_json:String,#[serde(default)] runtime_artifact:Option<crate::manifest::RuntimeArtifact>},
    DiagnoseLocationInputs {settings:crate::doctor::Settings,source_name:Option<String>,policy_json:Option<String>,context_source_name:Option<String>,context_json:Option<String>},
    DiagnoseNativeLocationWorkspace {input:crate::doctor_workspace::Input},
    InspectLocationPolicy {query:String,source_name:Option<String>,policy_json:Option<String>,#[serde(default)] fallback:Option<crate::location_policy::Fallback>},
    ExportConfiguredPlan {input:crate::config::ConfiguredRun,assignments:std::collections::BTreeMap<String,u32>,label:String,created_at:String,#[serde(default)] attempt:Option<u32>},
    RunAlgorithmBuild {input:crate::config::ConfiguredRun},
    RunEngineBuild {request:crate::engine::Request},
    RunAlgorithmConfig {input:crate::config::ConfiguredRun},
    InspectAlgorithmConfig {yaml_text:String,source_name:String},
    CreateAlgorithmConfig {settings:bisect_runner::config::TemplateConfig},
    ValidateAlgorithmConfig {yaml_text:String,source_name:String,profile:String},
    AggregateNationalAnalysis {reports_json:std::collections::BTreeMap<String,String>},
    AggregateNativeAnalysis {input:crate::native_aggregate::Input},
    AnalyzeBlocVoting {precincts_tsv:String,annotation_csv:String,annotation_source_name:String,attestation_files:std::collections::BTreeMap<String,String>,settings:crate::bloc_analysis::Settings},
    AnalyzePlanCompactness {document_json:String,unit_geometry_json:String,coordinate_basis:bisect_analysis::district_geometry::CoordinateBasis,crs_label:String,#[serde(default)] include_exact_reock:bool,#[serde(default)] population_centroids_csv:Option<String>,#[serde(default)] dissolve_method:bisect_analysis::district_geometry::DissolveMethod},
    ImportPlanInterchange {source_text:String,centroids_json:Option<String>,context_json:Option<String>,settings:crate::interchange::ImportSettings},
    DiagnosePartitionTraces {source_json:String,max_lag:usize},
    InspectNativeIlpRun {source_json:String},
    ValidateResearchEnsemble {chains_jsonl:std::collections::BTreeMap<String,String>,enacted_json:Option<String>,rhat_threshold:f64,ess_min:f64,#[serde(default)] strict:bool},
    AnalyzePlanStatistics {document_json:String,demographic_csv:Option<String>,urban_csv:Option<String>,election_csv:Option<String>,tolerance_percent:f64},
    AnalyzePlanElection {document_json:String,election_csv:String,settings:Option<crate::election_analysis::Settings>},
    AnalyzeSuiteStability {plans_json:std::collections::BTreeMap<String,String>},
    ValidateSuiteNesting {house_json:String,senate_json:String,suite_name:String,required_ratio:Option<usize>},
    ExportRplanInterchange {document_json:String},
    ExportReproducibilityPackage {document_json:String,manifest_json:Option<String>,analysis_files:std::collections::BTreeMap<String,String>,label:String},
    SelectParetoPlan {context_json:String,frontier_ndjson:String,settings:crate::pareto::Settings,selection:crate::pareto::Selection},
    SampleParetoContext {context_json:String,settings:crate::pareto::Settings},
    PreparePlanReportManifest {document_json:String,context_json:String},
    PrepareNamedStateReportManifest {document_json:String,context_json:String,arguments:Vec<String>,adjacency_source_name:String,adjacency_sha256:String},
    AnalyzePlanPackage {input:crate::analysis_package::Input},
    RunLabelAnalysis {input:crate::label_analysis::Input},
    RunLabelAnalysisYears {input:crate::label_analysis_years::Input},
    RunLabelReportYears {input:crate::label_report_years::Input},
    ResumeLabelAnalysisYears {input:crate::label_analysis_years::Input,checkpoint:Value},
    ResumeLabelReportYears {input:crate::label_report_years::Input,checkpoint:Value},
    RunLabelReport {input:crate::label_report::Input},
    RunNativeLabelSummary {input:crate::native_label_summary::Input},
    ReconcileLabelRegistry {input:crate::label_reconcile::Input},
    ReconcileLabelPackage {input:crate::label_package_merge::Input},
    AuditIlpArtifacts {input:crate::ilp_audit::Input},
    VerifyRplanPackage {input:crate::package_verification::Input},
    VerifyManifestCertificate {input:crate::package_verification::Input},
    CompareVerifyAssignments {input:crate::assignment_verification::Input},
    DiscoverOriginalAssignments {input:crate::assignment_discovery::Input},
    VerifyAssignmentsOnly {input:crate::verify_assignments::Input},
    PlanSweep {settings:bisect_runner::sweep::Settings},
    PrepareFetchDataPackage {input:crate::fetch_data_package::Input},
    PlanFletchCacheVerification {cache_manifest_json:String},
    ResolveFletchCacheRequest {input:crate::fletch_request::Input},
    PlanFletchCachePrune {cache_manifest_json:String,files:std::collections::BTreeMap<String,String>},
    VerifyFletchCacheEvidence {cache_manifest_json:String,files:std::collections::BTreeMap<String,crate::fletch_verify::Observation>},
    InspectFletchSources {input:crate::fetch_plan::Input},
    InspectFletchCache {input:crate::fletch_cache::Input},
    MergeFletchCacheManifest {cache_manifest_json:String,entries_json:String},
    StageFletchCacheSource {input:crate::fletch_stage::Input},
    PlanFetchSources {input:crate::fetch_plan::Input},
    RenderNativeNationalMap {input:crate::map_export::NationalInput},
    BeginNativeNationalMap {input:crate::national_map_stream::Begin},
    AppendNativeNationalMap {input:crate::national_map_stream::Append},
    AppendNativeNationalMapStep {input:crate::national_map_stream::Append},
    AppendNativeNationalRawMap {input:crate::national_map_stream::RawAppend},
    AppendNativeNationalRawMapStep {input:crate::national_map_stream::RawAppend},
    FinishNativeNationalMap {session:u32},
    AbortNativeNationalMap {session:u32},
    RenderNativeDistrictMap {input:crate::map_export::Input},
    RenderNativeAssignmentMap {input:crate::native_assignment_map::Input},
    RenderNativeDistrictMapStep {input:crate::map_export::Input},
    InspectStateBatchSelection {input:crate::state_catalog::Input},
    InspectNativeAdjacency {input:crate::native_adjacency::Input},
    PrepareNativeEngineInput {input:crate::native_adjacency::PrepareInput},
    PrepareContextEngineInput {input:crate::context_engine_input::Input},
    PrepareTractEngineInput {input:crate::tract_engine_input::Input},
    PrepareCensusEngineInput {input:crate::tract_engine_input::Input},
    PrepareTigerSource {input:crate::tiger_source::Input},
    PrepareNativeMapTiger {input:crate::native_map_tiger::Input},
    PrepareNativeMapTigerStep {input:crate::native_map_tiger::Input},
    AppendNativeNationalTigerMapStep {input:crate::national_map_stream::TigerAppend},
    RenderNativeTigerAssignmentMap {input:crate::native_tiger_assignment_map::Input},
    PreparePl94Source {input:crate::pl94_source::Input},
    PrepareLodesSource {input:crate::lodes_source::Input},
    PrepareAcsHousingSource {input:crate::acs_housing::Input},
    VerifyManifestPrechecks {input:crate::manifest_prechecks::Input},
    InspectNativeReplayRecipe {input:crate::manifest_prechecks::Input},
    InspectNativeReplaySources {input:crate::native_recipe_sources::Input},
    PrepareNativeRecordedReplay {input:crate::native_recorded_replay::Input},
    RunNativeRecordedReplay {input:crate::native_recorded_replay::Input},
    ReplayEngineRun {input:crate::engine_replay::Input},
    VerifyManifestReplay {input:crate::verify_replay::Input,#[serde(default)] runtime_artifact:Option<crate::manifest::RuntimeArtifact>,#[serde(default)] skip_binary_check:bool},
    AnalyzePlanSplits {document_json:String,geography_json:Option<String>,context_json:Option<String>},
    AnalyzePlanContiguity {document_json:String,context_json:String},
    ValidateRplanText {document_json:String},
    SelectSmcPlan {context_json:String,ensemble_ndjson:String,settings:crate::smc::SelectionSettings},
    SampleSmcContext {context_json:String,settings:crate::smc::Settings},
    ImproveRplanText {document_json:String, context_json:String, settings:crate::improve::Settings},
    AuditRplanText {document_json:String, context_json:Option<String>, profile_json:String, constraints:Vec<AuditConstraint>, generated_at_utc:String, lineage_json:Option<String>},
    VerifyAuditText {certificate_json:String, document_json:Option<String>, context_json:Option<String>},
    SolveBranchPriceContext { context_json:String, settings:crate::column::ContextSettings },
    SolveBranchPrice { instance_json:String },
    StartCeremony { tree_json:String, config_json:String },
    AnnounceCeremony { tree_json:String, transcript_json:String, published_unix_seconds:String, witness_receipts:Vec<String> },
    HaltCeremony { tree_json:String, transcript_json:String, reason:String },
    VerifyCeremony { transcript_json:String, tree_json:Option<String> },
    InspectCeremonyPrefix { transcript_json:String },
    SolveCertifiedCanonical { instance_json:String },
    VerifyCertifiedCanonical { artifacts_json:String },
    SolveCertifiedSingle { instance_json:String },
    VerifyCertifiedSingle { artifacts_json:String },
    SolveCertifiedSplit { instance_json:String },
    VerifyCertifiedSplit { artifacts_json:String },
    SolveCertifiedTree { instance_json:String },
    CompileCertifiedProofModels {instance_json:String,discovery_json:String},
    SolveCertifiedDiscoveryInstance {instance_json:String,settings:crate::discovery_instance::Settings},
    SolveCertifiedDiscoveryContext {context_json:String,settings:crate::discovery_context::Settings},
    SolveCertifiedCanonicalContext {context_json:String,settings:crate::certified_context::Settings},
    SolveCertifiedTreeContext { context_json:String,settings:crate::certified_context::Settings },
    VerifyCertifiedTree { tree_json:String },
    RenderReportComparison { files_a:std::collections::BTreeMap<String,Vec<u8>>, files_b:std::collections::BTreeMap<String,Vec<u8>>, label_a:String, label_b:String, leaning_threshold:f64, close_call_band:f64, generated_at:String, plan_a:Option<Value>, plan_b:Option<Value>, context:Option<Value> },
    InspectReportSide { files:std::collections::BTreeMap<String,Vec<u8>>, label:String, leaning_threshold:f64 },
    ComparePlans { plan_a:Value, plan_b:Value, context:Option<Value> },
    CompareNativeAssignmentFiles {input:crate::native_assignment_compare::Input},
    RenderNativeComparisonNarrative {input:crate::native_comparison_narrative::Input},
    SummarizePlanLabels {input:crate::multi_plan_summary::Input},
    CompareRplanText {plan_a_json:String,plan_b_json:String,context_json:Option<String>},
    RunMultiscale { input:crate::multiscale_input::MultiscaleRequest },
    RunMultiscaleBuild { input:crate::multiscale_input::MultiscaleRequest },
    ExportMultiscalePlan { input:crate::multiscale_input::MultiscaleRequest, fine_edges:Option<Vec<(usize,usize,f64)>>, assignments:std::collections::BTreeMap<String,u32>, label:String, chamber:String, created_at:String },
    ExportEnginePlan { request:crate::engine::Request, assignments:std::collections::BTreeMap<String,u32>, label:String, chamber:String, created_at:String },
    AttachDemographicCsv { document:Value, context:Value, source_base64:String, basis:String, source_label:String },
    ImportDemographicCsv { source_base64:String, state:String, year:String, basis:String, source_label:String },
    InspectNativeDemographicCsv { source_base64:String, basis:String, source_label:String },
    ImportElectionCountsCsv { source_base64:String, state:String, year:String, election_year:String, source_label:String },
    ImportPartisanSharesTsv { source_base64:String, state:String, year:String, source_label:String },
    ImportCharacterCsv { source_base64:String, state:String, year:String, data_year:String, source_label:String, kind:String },
    BuildCharacterWeights { input:crate::character_input::CharacterInput, geoids:Vec<String>, edges:Vec<(usize,usize,f64)>, alpha:f64 },
    AlignCharacterInput { input:crate::character_input::CharacterInput, geoids:Vec<String> },
    InspectNativeInputFolder {input:crate::native_input_folder::Input},
    ResolveNativeChamberCount {input:crate::chamber_count::Input},
    BindPreparedEngineGraph {input:crate::prepared_graph_binding::Input},
    RunPreparedEngine {input:crate::prepared_engine::Input},
    RunPreparedEngineTimed {input:crate::prepared_engine::Input},
    RunAcceptedStateWorkflow {input:crate::state_workflow::AcceptedInput},
    InspectNativeRunTarget {input:crate::run_target::Input,version:String,year:String,label:String},
    InspectNativeStateRunTarget {input:crate::run_target::Input,version:String,year:String,state:String},
    InspectNativeNamedStateRunTarget {input:crate::run_target::Input,version:String,arguments:Vec<String>},
    RunNativeCharacterEngine {input:crate::native_character_engine::Input},
    EvaluateNativeCharacterWeights {input:crate::native_character_weights::Input},
    PrepareNativeCharacterSource { input:crate::native_character_source::Input },
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
        ToolRequest::VerifyTutorialData {tutorial,source_name,checksums_json,files} => crate::tutorial::inspect(&tutorial,&source_name,&checksums_json,files),
        ToolRequest::VerifyRunManifest {source_name,manifest_json,runtime_artifact} => crate::manifest::inspect(&source_name,&manifest_json,runtime_artifact),
        ToolRequest::DiagnosePlanArtifacts {plans} => crate::plan_doctor::inspect(plans),
        ToolRequest::ValidateNativeRplan {source_name,document_json,strict} => crate::format_validation::inspect(&source_name,&document_json,strict),
        ToolRequest::InspectCivicCoi {inputs,state,year,validate_mode} => crate::civic::inspect(inputs,&state,&year,&validate_mode),
        ToolRequest::InspectNativeCivicConflicts {inputs} => crate::native_civic_conflicts::inspect(inputs),
        ToolRequest::IngestCivicCoi {input,state,year,validate_mode,settings} => crate::civic::ingest(input,&state,&year,&validate_mode,settings),
        ToolRequest::InspectCivicManifests {inputs} => crate::civic::inventory(inputs),
        ToolRequest::InspectLabelComparison {registry_json,label_a,label_b,year} => crate::label_inventory::compare(registry_json,label_a,label_b,year),
        ToolRequest::InspectLabelRegistry {registry_json,label} => crate::label_inventory::inspect(registry_json,label),
        ToolRequest::RunTuiConfigured {input} => crate::tui_run::run(input),
        ToolRequest::VerifyTuiManifest {input,runtime_artifact} => crate::tui_verify::verify(input,runtime_artifact),
        ToolRequest::DiagnoseTuiLocation {input} => crate::tui_doctor::inspect(input),
        ToolRequest::CompareTuiPlans {input} => crate::tui_compare::compare(input),
        ToolRequest::InspectTuiPlanLibrary {input} => crate::tui_library::inspect(input),
        ToolRequest::InspectTuiSession {source_name,session_toml} => crate::tui_session::inspect(source_name,session_toml),
        ToolRequest::ExportTuiSession {settings} => crate::tui_session::export(settings),
        ToolRequest::AssembleLabelBuildIndex {input} => crate::build_index::assemble(input),
        ToolRequest::ImportLabelAssignments {input} => crate::label_import::run(input),
        ToolRequest::InspectPlanHub {input} => crate::plan_hub::inspect(input),
        ToolRequest::PrepareLabelImportPlan {input} => crate::label_import::prepare_plan(input),
        ToolRequest::RunLabelBuild {input} => crate::label_build::run(input),
        ToolRequest::PlanLabelBuild {input,dry_run} => crate::label_build::plan(input,dry_run),
        ToolRequest::AssembleCommissionAudit {manifest_json,manifest_bytes,fallback}=>{
            use sha2::Digest;
            if manifest_json.is_some() && manifest_bytes.is_some() {return Err("Supply one manifest source representation.".into());}
            let raw=manifest_bytes.or_else(||manifest_json.map(String::into_bytes));
            let resolved=bisect_report::commission_manifest::resolve_report_manifest(raw.as_deref(),fallback.as_ref()).map_err(|e|e.to_string())?;
            let audit_json=bisect_report::commission_audit::build_commission_audit_json(&resolved.manifest).map_err(|e|e.to_string())?;
            Ok(json!({"schema_version":"bisect-commission-audit-v1","audit_json":audit_json,"label":resolved.manifest.label,"manifest_sha256":raw.as_ref().map(|b|format!("{:x}",sha2::Sha256::digest(b))),"manifest_resolution":{"fallback_used":resolved.fallback_used,"source_status":resolved.source_status},"source_boundary":"Native report audit-only document from supplied manifest metadata or explicit native minimal fallback label/year. Fallback does not recover provenance. No analysis, context, certificate or map is required or read. Recorded hashes and commands are not recomputed, authenticated or executed; this is not a generated plan audit certificate."}))
        }
        ToolRequest::AssembleCommissionReport {files,generated_at,fallback,delivery}=>{
            use sha2::{Digest,Sha256};
            if let Some(settings)=&delivery {settings.validate().map_err(|e|e.to_string())?;}
            let report=bisect_report::report_package::assemble_report_package_with_fallback(&files,generated_at,fallback.as_ref()).map_err(|e|e.to_string())?;
            let report_json=serde_json::to_string_pretty(&report).map_err(|e|e.to_string())?;
            let report_html=if delivery.as_ref().is_none_or(|s|s.wants_html()) {Some(bisect_report::render_html_report(&report).map_err(|e|e.to_string())?)} else {None};
            let resolved=bisect_report::commission_manifest::resolve_report_manifest(files.get("manifest.json").map(Vec::as_slice),fallback.as_ref()).map_err(|e|e.to_string())?;
            let manifest=&resolved.manifest;
            let audit_json=bisect_report::commission_audit::build_commission_audit_json(manifest).map_err(|e|e.to_string())?;
            let report_files=delivery.as_ref().map(|s|bisect_report::report_delivery::report_delivery_files(s,&report_json,report_html.as_deref(),&audit_json)).transpose().map_err(|e|e.to_string())?;
            let report_file_sha256=report_files.as_ref().map(|f|f.iter().map(|(p,t)|(p.clone(),format!("{:x}",Sha256::digest(t.as_bytes())))).collect::<std::collections::BTreeMap<_,_>>());
            let hashes=files.iter().map(|(p,b)|(p.clone(),format!("{:x}",Sha256::digest(b)))).collect::<std::collections::BTreeMap<_,_>>();
            Ok(json!({"schema_version":"bisect-commission-report-v1","report_json":report_json,"report_html":report_html,"audit_json":audit_json,"report_files":report_files,"report_file_sha256":report_file_sha256,"delivery_settings":delivery,"manifest_resolution":{"fallback_used":resolved.fallback_used,"source_status":resolved.source_status},"label":report.label,"state":report.state,"year":report.year,"source_sha256":hashes,"source_boundary":"Shared native ten-section JSON and self-contained HTML report assembled from supplied manifest, analysis, context, certificate and map bytes. Missing optional analysis remains unavailable. Source hashes record supplied-byte integrity; analyses, audit claims and map images are not recomputed or authenticated. Explicit host timestamp replaces the native clock. Explicit minimal fallback label/year may replace missing or invalid manifest metadata; this does not recover provenance. PDF delivery and full native report orchestration remain incomplete."}))
        },
        ToolRequest::RenameLabelPackage {registry_json,files_base64,directories,source_label,destination_label,force} => crate::label_rename::rename(registry_json,files_base64,directories,source_label,destination_label,force),
        ToolRequest::VerifyLabelChain {registry_json,files_base64,label,year_filter} => crate::label_chain::verify(registry_json,files_base64,label,year_filter),
        ToolRequest::MigrateLegacyPlan {files_base64,settings} => crate::migration::migrate(files_base64,settings),
        ToolRequest::IngestCandidateRace {csv_base64,source_name,documents,staged_path,settings} => crate::candidate::ingest(csv_base64,source_name,documents,staged_path,settings),
        ToolRequest::AssemblePlanSuite {house_json,senate_json,congressional_json,suite_name,nest_mode,required_ratio,policy_source_name,policy_json} => crate::suite::assemble(&house_json,&senate_json,congressional_json.as_deref(),&suite_name,&nest_mode,required_ratio,policy_source_name.as_deref(),policy_json.as_deref()),
        ToolRequest::DiagnoseLocationInputs {settings,source_name,policy_json,context_source_name,context_json}=>crate::doctor::inspect(settings,source_name.as_deref(),policy_json.as_deref(),context_source_name.as_deref(),context_json.as_deref()),
        ToolRequest::DiagnoseNativeLocationWorkspace {input}=>crate::doctor_workspace::inspect(input),
        ToolRequest::InspectLocationPolicy {query,source_name,policy_json,fallback}=>match fallback {
            Some(input) if source_name.is_none()&&policy_json.is_none()=>crate::location_policy::inspect_fallback(&query,input),
            Some(_)=>Err("Choose explicit inspection or native fallback inputs, not both.".into()),
            None=>crate::location_policy::inspect(&query,source_name.as_deref(),policy_json.as_deref()),
        },
        ToolRequest::ExportConfiguredPlan {input,assignments,label,created_at,attempt}=>crate::config::export(input,assignments,label,created_at,attempt),
        ToolRequest::RunAlgorithmBuild {input}=>crate::config::build(input),
        ToolRequest::RunEngineBuild {request}=>crate::engine_build::run(request),
        ToolRequest::RunAlgorithmConfig {input}=>crate::config::run(input),
        ToolRequest::AnalyzeBlocVoting {precincts_tsv,annotation_csv,annotation_source_name,attestation_files,settings}=>crate::bloc_analysis::analyze(&precincts_tsv,&annotation_csv,&annotation_source_name,attestation_files,settings),
        ToolRequest::InspectAlgorithmConfig {yaml_text,source_name}=>crate::config::inspect(&yaml_text,&source_name),
        ToolRequest::CreateAlgorithmConfig {settings}=>crate::config::create(&settings),
        ToolRequest::ValidateAlgorithmConfig {yaml_text,source_name,profile}=>crate::config::validate(&yaml_text,&source_name,&profile),
        ToolRequest::AggregateNationalAnalysis {reports_json}=>crate::national::aggregate(&reports_json),
        ToolRequest::AggregateNativeAnalysis {input}=>crate::native_aggregate::run(input),
        ToolRequest::AnalyzePlanCompactness {document_json,unit_geometry_json,coordinate_basis,crs_label,include_exact_reock,population_centroids_csv,dissolve_method} => crate::plan_compactness::analyze(&document_json,&unit_geometry_json,coordinate_basis,&crs_label,include_exact_reock,population_centroids_csv.as_deref(),dissolve_method),
        ToolRequest::SelectParetoPlan {context_json,frontier_ndjson,settings,selection} => crate::pareto::select(&context_json,&frontier_ndjson,settings,selection),
        ToolRequest::SampleParetoContext {context_json,settings} => crate::pareto::run(&context_json,settings),
        ToolRequest::PreparePlanReportManifest {document_json,context_json} => crate::plan_audit_text::report_manifest(&document_json,&context_json),
        ToolRequest::PrepareNamedStateReportManifest {document_json,context_json,arguments,adjacency_source_name,adjacency_sha256} => crate::plan_audit_text::named_state_manifest(&document_json,&context_json,arguments,&adjacency_source_name,&adjacency_sha256),
        ToolRequest::AnalyzePlanPackage {input} => crate::analysis_package::run(input),
        ToolRequest::RunLabelAnalysis {input} => crate::label_analysis::run(input),
        ToolRequest::RunLabelAnalysisYears {input} => crate::label_analysis_years::run(input),
        ToolRequest::RunLabelReportYears {input} => crate::label_report_years::run(input),
        ToolRequest::ResumeLabelAnalysisYears {input,checkpoint} => crate::label_analysis_years::resume(input,checkpoint),
        ToolRequest::ResumeLabelReportYears {input,checkpoint} => crate::label_report_years::resume(input,checkpoint),
        ToolRequest::RunLabelReport {input} => crate::label_report::run(input),
        ToolRequest::RunNativeLabelSummary {input} => crate::native_label_summary::run(input),
        ToolRequest::ReconcileLabelRegistry {input} => crate::label_reconcile::reconcile(input),
        ToolRequest::ReconcileLabelPackage {input} => crate::label_package_merge::merge(input),
        ToolRequest::AuditIlpArtifacts {input} => crate::ilp_audit::audit(input),
        ToolRequest::VerifyRplanPackage {input} => crate::package_verification::verify(input),
        ToolRequest::VerifyManifestCertificate {input} => crate::manifest_certificate::verify(input),
        ToolRequest::CompareVerifyAssignments {input} => crate::assignment_verification::compare(input),
        ToolRequest::DiscoverOriginalAssignments {input} => crate::assignment_discovery::discover(input),
        ToolRequest::VerifyAssignmentsOnly {input} => crate::verify_assignments::verify(input),
        ToolRequest::PlanSweep {settings} => crate::sweep::plan(settings),
        ToolRequest::PrepareFetchDataPackage {input} => crate::fetch_data_package::prepare(input),
        ToolRequest::ResolveFletchCacheRequest {input} => crate::fletch_request::resolve(input),
        ToolRequest::PlanFletchCachePrune {cache_manifest_json,files} => crate::fletch_prune::plan(cache_manifest_json,files),
        ToolRequest::PlanFletchCacheVerification {cache_manifest_json} => crate::fletch_verify::plan(cache_manifest_json),
        ToolRequest::VerifyFletchCacheEvidence {cache_manifest_json,files} => crate::fletch_verify::verify(cache_manifest_json,files),
        ToolRequest::InspectFletchSources {input} => crate::fletch_sources::inspect(input),
        ToolRequest::InspectFletchCache {input} => crate::fletch_cache::inspect(input),
        ToolRequest::MergeFletchCacheManifest {cache_manifest_json,entries_json} => crate::fletch_cache::merge(cache_manifest_json,entries_json),
        ToolRequest::StageFletchCacheSource {input} => crate::fletch_stage::stage(input),
        ToolRequest::PlanFetchSources {input} => crate::fetch_plan::plan(input),
        ToolRequest::RenderNativeNationalMap {input} => crate::map_export::render_national(input),
        ToolRequest::BeginNativeNationalMap {input} => crate::national_map_stream::begin(input),
        ToolRequest::AppendNativeNationalMap {input} => crate::national_map_stream::append(input),
        ToolRequest::AppendNativeNationalMapStep {input} => crate::national_map_stream::append_step(input),
        ToolRequest::AppendNativeNationalRawMap {input} => crate::national_map_stream::append_raw(input),
        ToolRequest::AppendNativeNationalRawMapStep {input} => crate::national_map_stream::append_raw_step(input),
        ToolRequest::FinishNativeNationalMap {session} => crate::national_map_stream::finish(session),
        ToolRequest::AbortNativeNationalMap {session} => crate::national_map_stream::abort(session),
        ToolRequest::RenderNativeDistrictMap {input} => crate::map_export::render(input),
        ToolRequest::RenderNativeAssignmentMap {input} => crate::native_assignment_map::render(input),
        ToolRequest::RenderNativeDistrictMapStep {input} => crate::map_export::render_step(input),
        ToolRequest::InspectStateBatchSelection {input} => crate::state_catalog::inspect(input),
        ToolRequest::InspectNativeAdjacency {input} => crate::native_adjacency::inspect(input),
        ToolRequest::PrepareNativeEngineInput {input} => crate::native_adjacency::prepare(input),
        ToolRequest::PrepareContextEngineInput {input} => crate::context_engine_input::prepare(input),
        ToolRequest::PrepareCensusEngineInput {input} => crate::tract_engine_input::prepare_census(input),
        ToolRequest::PrepareTractEngineInput {input} => crate::tract_engine_input::prepare(input),
        ToolRequest::PrepareTigerSource {input} => crate::tiger_source::prepare(input),
        ToolRequest::RenderNativeTigerAssignmentMap {input} => crate::native_tiger_assignment_map::render(input),
        ToolRequest::AppendNativeNationalTigerMapStep {input} => crate::national_map_stream::append_tiger_step(input),
        ToolRequest::PrepareNativeMapTiger {input} => crate::native_map_tiger::prepare(input),
        ToolRequest::PrepareNativeMapTigerStep {input} => Ok(match crate::native_map_tiger::prepare(input) { Ok(prepared)=>serde_json::json!({"schema_version":"bisect-native-map-tiger-step-v1","status":"prepared","prepared":prepared,"error":null}), Err(error)=>serde_json::json!({"schema_version":"bisect-native-map-tiger-step-v1","status":"geometry-error","prepared":null,"error":error}) }),
        ToolRequest::PreparePl94Source {input} => crate::pl94_source::prepare(input),
        ToolRequest::PrepareLodesSource {input} => crate::lodes_source::prepare(input),
        ToolRequest::PrepareAcsHousingSource {input} => crate::acs_housing::prepare(input),
        ToolRequest::VerifyManifestPrechecks {input} => crate::manifest_prechecks::verify(input),
        ToolRequest::InspectNativeReplayRecipe {input} => crate::native_recipe_binding::inspect(input),
        ToolRequest::InspectNativeReplaySources {input} => crate::native_recipe_sources::inspect(input),
        ToolRequest::PrepareNativeRecordedReplay {input} => crate::native_recorded_replay::replay(input,false),
        ToolRequest::RunNativeRecordedReplay {input} => crate::native_recorded_replay::replay(input,true),
        ToolRequest::ReplayEngineRun {input} => crate::engine_replay::replay(input),
        ToolRequest::VerifyManifestReplay {input,runtime_artifact,skip_binary_check} => crate::verify_replay::verify(input,runtime_artifact,skip_binary_check),
        ToolRequest::AnalyzePlanSplits {document_json,geography_json,context_json} => crate::plan_splits::analyze(&document_json,geography_json.as_deref(),context_json.as_deref()),
        ToolRequest::AnalyzePlanContiguity {document_json,context_json} => crate::plan_audit_text::contiguity(&document_json,&context_json),
        ToolRequest::ValidateRplanText {document_json} => crate::plan_audit_text::validate(&document_json),
        ToolRequest::DiagnosePartitionTraces {source_json,max_lag} => crate::partition_diagnostics::execute(&source_json,max_lag),
        ToolRequest::InspectNativeIlpRun {source_json} => crate::ilp_inspection::inspect(&source_json),
        ToolRequest::ValidateResearchEnsemble {chains_jsonl,enacted_json,rhat_threshold,ess_min,strict} => crate::research::validate(&chains_jsonl,enacted_json.as_deref(),rhat_threshold,ess_min,strict),
        ToolRequest::AnalyzePlanStatistics {document_json,demographic_csv,urban_csv,election_csv,tolerance_percent} => crate::plan_statistics::analyze(&document_json,demographic_csv.as_deref(),urban_csv.as_deref(),election_csv.as_deref(),tolerance_percent),
        ToolRequest::AnalyzePlanElection {document_json,election_csv,settings} => crate::election_analysis::analyze(&document_json,&election_csv,settings),
        ToolRequest::AnalyzeSuiteStability {plans_json} => crate::suite::stability(&plans_json),
        ToolRequest::ValidateSuiteNesting {house_json,senate_json,suite_name,required_ratio} => crate::suite::validate(&house_json,&senate_json,&suite_name,required_ratio),
        ToolRequest::InspectNativeMigrationBase {input} => crate::migration_base::inspect(input),
        ToolRequest::StageNativeMigrationBase {input} => crate::migration_base::stage(input),
        ToolRequest::ImportNativePlan {input} => crate::native_import::import(input),
        ToolRequest::ExportNativePlanFolder {input} => crate::export_folder::export(input),
        ToolRequest::ExportRplanInterchange {document_json} => crate::interchange::export(&document_json),
        ToolRequest::ExportReproducibilityPackage {document_json,manifest_json,analysis_files,label} => crate::interchange::reproducibility(&document_json,manifest_json.as_deref(),&analysis_files,&label),
        ToolRequest::ImportPlanInterchange {source_text,centroids_json,context_json,settings} => crate::interchange::import(&source_text,centroids_json.as_deref(),context_json.as_deref(),settings),
        ToolRequest::SelectSmcPlan {context_json,ensemble_ndjson,settings} => crate::smc::select(&context_json,&ensemble_ndjson,settings),
        ToolRequest::SampleSmcContext {context_json,settings} => crate::smc::run(&context_json,settings),
        ToolRequest::ImproveRplanText {document_json,context_json,settings} => crate::improve::run(&document_json,&context_json,settings),
        ToolRequest::AuditRplanText {document_json,context_json,profile_json,constraints,generated_at_utc,lineage_json} => crate::plan_audit_text::audit(&document_json,context_json.as_deref(),&profile_json,constraints,generated_at_utc,lineage_json.as_deref()),
        ToolRequest::VerifyAuditText {certificate_json,document_json,context_json} => crate::plan_audit_text::verify(&certificate_json,document_json.as_deref(),context_json.as_deref()),
        ToolRequest::SolveBranchPriceContext {context_json,settings} => crate::column::solve_context(&context_json,settings),
        ToolRequest::SolveBranchPrice {instance_json} => crate::column::solve(&instance_json),
        ToolRequest::StartCeremony {tree_json,config_json} => crate::ceremony::start(&tree_json,&config_json),
        ToolRequest::AnnounceCeremony {tree_json,transcript_json,published_unix_seconds,witness_receipts} => crate::ceremony::announce(&tree_json,&transcript_json,&published_unix_seconds,witness_receipts),
        ToolRequest::HaltCeremony {tree_json,transcript_json,reason} => crate::ceremony::halt(&tree_json,&transcript_json,reason),
        ToolRequest::VerifyCeremony {transcript_json,tree_json} => crate::ceremony::verify(&transcript_json,tree_json.as_deref()),
        ToolRequest::InspectCeremonyPrefix {transcript_json} => crate::ceremony::inspect_prefix(&transcript_json),
        ToolRequest::SolveCertifiedCanonical {instance_json} => crate::certified_exact::solve_canonical(&instance_json),
        ToolRequest::VerifyCertifiedCanonical {artifacts_json} => crate::certified_exact::verify_canonical(&artifacts_json),
        ToolRequest::SolveCertifiedSingle {instance_json} => crate::certified_exact::solve_single(&instance_json),
        ToolRequest::VerifyCertifiedSingle {artifacts_json} => crate::certified_exact::verify_single(&artifacts_json),
        ToolRequest::SolveCertifiedSplit {instance_json} => crate::certified_exact::solve_split(&instance_json),
        ToolRequest::VerifyCertifiedSplit {artifacts_json} => crate::certified_exact::verify_split(&artifacts_json),
        ToolRequest::CompileCertifiedProofModels {instance_json,discovery_json}=>crate::proof_models::compile(&instance_json,&discovery_json),
        ToolRequest::SolveCertifiedDiscoveryInstance {instance_json,settings}=>crate::discovery_instance::solve(&instance_json,settings),
        ToolRequest::SolveCertifiedDiscoveryContext {context_json,settings}=>crate::discovery_context::solve(&context_json,settings),
        ToolRequest::SolveCertifiedCanonicalContext {context_json,settings}=>crate::canonical_context::solve(&context_json,settings),
        ToolRequest::SolveCertifiedTreeContext {context_json,settings}=>crate::certified_context::solve(&context_json,settings),
        ToolRequest::SolveCertifiedTree {instance_json} => crate::certified_exact::solve_tree(&instance_json),
        ToolRequest::VerifyCertifiedTree {tree_json} => crate::certified_exact::verify_tree(&tree_json),
        ToolRequest::InspectReportSide {files,label,leaning_threshold} => {
            let (side,usable)=read_report_side(&files,&label,leaning_threshold)?;
            Ok(json!({"schema_version":"bisect-report-side-inspection-v1","native_report_side":side,"usable_analysis":usable,"leaning_threshold":leaning_threshold,"supplied_analysis":{"partisan":files.contains_key("analysis/partisan.json"),"demographic":files.contains_key("analysis/vra_analysis.json"),"compactness":files.contains_key("analysis/compactness.json")},"source_boundary":"Native report reader output from supplied bytes; original-byte hashes establish integrity only. Imported analysis is not recomputed or authenticated. Native missing-analysis defaults (zero seats/counts and null compactness) are retained raw values, not measurements. Native total_population is a placeholder zero in this reader."}))
        },
        ToolRequest::RenderReportComparison {files_a,files_b,label_a,label_b,leaning_threshold,close_call_band,generated_at,plan_a,plan_b,context} => {
            if !close_call_band.is_finite() || !(0.0..=1.0).contains(&close_call_band) || generated_at.len()>100 {
                return Err("Invalid report close-call band or generation timestamp.".into());
            }
            let (a,availability_a)=read_report_side(&files_a,&label_a,leaning_threshold)?;
            let (b,availability_b)=read_report_side(&files_b,&label_b,leaning_threshold)?;
            let evidence=report_plan_evidence(plan_a,plan_b,context,&a,&b,&files_a,&files_b)?;
            let evidence_json=evidence.as_ref().map(|e|json!({"unit_kind":e.unit_kind,"changed_units":e.changed_units,"district_ids_with_changes":e.district_ids_with_changes,"population_in_changed_units":e.population_in_changed_units.map(|n|n.to_string()),"plan_a_total_population":e.total_population_a.map(|n|n.to_string()),"plan_b_total_population":e.total_population_b.map(|n|n.to_string()),"plan_a_hash":e.plan_a_hash,"plan_b_hash":e.plan_b_hash,"context_hash":e.context_hash,"scope":"Measurements computed from separately supplied plans/context. Matching district counts do not authenticate imported analysis to plans. District IDs are zero based; label renumbering counts as change; no population movement is implied."}));
            let report=bisect_report::comparison::ComparisonReport::from_loaded(a,b,None,Default::default());
            let cfg=bisect_report::narrative::NarrativeConfig {leaning_threshold,close_call_band,..Default::default()};
            let markdown=bisect_report::narrative::render_imported_narrative_with_evidence(&report,&cfg,availability_a,availability_b,evidence.as_ref(),true);
            let ctx=bisect_report::html_comparison::HtmlComparisonContext {narrative_markdown:bisect_report::narrative::render_imported_narrative_with_evidence(&report,&cfg,availability_a,availability_b,evidence.as_ref(),false),template_sha256:String::new(),build_commit_short:String::new(),version:env!("CARGO_PKG_VERSION").into(),approved_at:generated_at.clone()};
            let html=bisect_report::html_comparison::render_imported_comparison_html_with_evidence(&report,&cfg,&ctx,availability_a,availability_b,evidence.as_ref());
            let mut report_json=serde_json::to_value(&report).map_err(|e|e.to_string())?;
            // Native imported-reader placeholders are not measured report values.
            report_json["diff"]=Value::Null;
            report_json["plan_a"]["total_population"]=json!(evidence.as_ref().and_then(|e|e.total_population_a).map(|n|n.to_string()));
            report_json["plan_b"]["total_population"]=json!(evidence.as_ref().and_then(|e|e.total_population_b).map(|n|n.to_string()));
            use sha2::{Digest,Sha256};
            let hash=|bytes:&[u8]|format!("{:x}",Sha256::digest(bytes));
            Ok(json!({"schema_version":"bisect-imported-report-comparison-v1","draft":true,"generated_at_supplied":generated_at,"leaning_threshold":leaning_threshold,"close_call_band":close_call_band,"report":report_json,"plan_evidence":evidence_json,"availability":{"plan_a":availability_a,"plan_b":availability_b},"markdown":markdown,"html":html,"receipt":{"schema_version":"bisect-report-byte-receipt-v1","html_sha256":hash(html.as_bytes()),"markdown_sha256":hash(markdown.as_bytes()),"plan_a_manifest_sha256":report.plan_a.manifest_sha256,"plan_b_manifest_sha256":report.plan_b.manifest_sha256,"plan_a_analysis_sha256":report.plan_a.analysis_sha256,"plan_b_analysis_sha256":report.plan_b.analysis_sha256,"plan_evidence":evidence_json,"bisect_version":env!("CARGO_PKG_VERSION")},"source_boundary":if evidence.is_some(){"Draft report: assignment/population measurements are computed from separately supplied plans/context. Imported partisan, demographic and compactness analysis is not recomputed or authenticated to those plans. Receipt hashes bind supplied input and generated output bytes; they do not certify source truth, execution provenance, approval or legal compliance."}else{"Draft comparison of imported analysis. Measurements are not recomputed or authenticated. Assignment differences and population totals are unavailable. Receipt hashes bind supplied input and generated output bytes; they do not certify source truth, execution provenance, approval or legal compliance."}}))
        },
        ToolRequest::ComparePlans {plan_a,plan_b,context} => crate::plan_comparison::compare(plan_a,plan_b,context),
        ToolRequest::CompareNativeAssignmentFiles {input} => crate::native_assignment_compare::run(input),
        ToolRequest::RenderNativeComparisonNarrative {input} => crate::native_comparison_narrative::run(input),
        ToolRequest::SummarizePlanLabels {input} => crate::multi_plan_summary::run(input),
        ToolRequest::CompareRplanText {plan_a_json,plan_b_json,context_json} => crate::plan_comparison::compare_text(&plan_a_json,&plan_b_json,context_json.as_deref()),
        ToolRequest::RunMultiscale { input } => crate::multiscale_input::execute(input),
        ToolRequest::RunMultiscaleBuild { input } => crate::engine_build::run_multiscale(input),
        ToolRequest::ExportMultiscalePlan {input,fine_edges,assignments,label,chamber,created_at} => crate::plan_export::export_multiscale(input,fine_edges,assignments,label,chamber,created_at),
        ToolRequest::ExportEnginePlan {request,assignments,label,chamber,created_at} => crate::plan_export::export(request,assignments,label,chamber,created_at),
        ToolRequest::AttachDemographicCsv {document,context,source_base64,basis,source_label} => attach_demographic_csv(document,context,source_base64,basis,source_label),
        ToolRequest::InspectNativeDemographicCsv {source_base64,basis,source_label} => {
            use base64::{Engine,engine::general_purpose::STANDARD};
            use sha2::{Digest,Sha256};
            const LIMIT:usize=8*1024*1024;
            if source_label.trim().is_empty()||source_label.len()>200{return Err("Supply a source label of 1–200 bytes.".into());}
            if source_base64.len()>LIMIT.div_ceil(3)*4{return Err("Native demographic CSV exceeds 8 MiB.".into());}
            let bytes=STANDARD.decode(source_base64).map_err(|_|"Invalid demographic source encoding.")?;
            if bytes.is_empty()||bytes.len()>LIMIT{return Err("Select a nonempty native demographic CSV of at most 8 MiB.".into());}
            let text=std::str::from_utf8(&bytes).map_err(|_|"Native demographic CSV is not UTF-8.")?;
            let mut result=bisect_data::native_demographics::inspect_native_demographics(text,&basis)?;
            result["source_label"]=json!(source_label);result["source_sha256"]=json!(format!("{:x}",Sha256::digest(&bytes)));
            Ok(result)
        },
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
        ToolRequest::AlignCharacterInput {input,geoids} => serde_json::to_value(crate::character_input::align(&input,&geoids)?).map_err(|e|e.to_string()),
        ToolRequest::InspectNativeInputFolder{input}=>crate::native_input_folder::inspect(input),
        ToolRequest::ResolveNativeChamberCount{input}=>crate::chamber_count::inspect(input),
        ToolRequest::BindPreparedEngineGraph {input} => crate::prepared_graph_binding::bind(input),
        ToolRequest::PrepareNativeStateRun {input} => crate::native_state_run::prepare(input),
        ToolRequest::PrepareNativeStateGraphRun {input} => crate::native_state_run::prepare_graph(input),
        ToolRequest::PrepareNativeAdjacencyStateRun {input} => crate::native_state_run::prepare_adjacency(input),
        ToolRequest::PrepareFreshNativeStateRun {input} => crate::fresh_native_state::prepare(input),
        ToolRequest::RunFreshNativeState {input} => crate::fresh_native_state::run(input),
        ToolRequest::InspectNativeStateCommand {arguments} => bisect_runner::state_command::inspect(arguments),
        ToolRequest::InspectNativeStateIdentity {arguments} => crate::state_identity::inspect(arguments),
        ToolRequest::InspectNativeStateReportSources {arguments,observations,unit_ids,observation_origin} => crate::state_report_sources::inspect(arguments,observations,unit_ids,observation_origin),
        ToolRequest::BindNativeStateReportSources {input} => crate::state_report_binding::bind(input),
        ToolRequest::InspectNativeStatesCommand {arguments}=>bisect_runner::states_config::inspect(arguments),
        ToolRequest::InspectNativeRunCommand {arguments}=>bisect_runner::run_command::inspect(arguments),
        ToolRequest::PrepareNativeRunGraphRun {input}=>crate::native_state_run::prepare_run_graph(input),
        ToolRequest::PrepareNativeRunRun {input}=>crate::native_state_run::prepare_run(input),
        ToolRequest::PrepareNativeStatesGraphRun {input}=>crate::native_state_run::prepare_states_graph(input),
        ToolRequest::PrepareNativeStatesRun {input}=>crate::native_state_run::prepare_states(input),
        ToolRequest::ScopeNativeStateCommand {arguments,state,year,resolution,districts} => bisect_runner::state_command::scope(arguments,state,year,resolution,districts),
        ToolRequest::RunPreparedEngine {input} => crate::prepared_engine::run(input),
        ToolRequest::RunPreparedEngineTimed {input} => crate::prepared_engine::run_with_partition_timing(input),
        ToolRequest::RunAcceptedStateWorkflow {input} => crate::state_workflow::run_accepted(input),
        ToolRequest::InspectNativeRunTarget {input,version,year,label} => crate::run_target::inspect(input,version,year,label),
        ToolRequest::InspectNativeStateRunTarget {input,version,year,state} => crate::run_target::inspect_state(input,version,year,state),
        ToolRequest::InspectNativeNamedStateRunTarget {input,version,arguments} => crate::run_target::inspect_named_state(input,version,arguments),
        ToolRequest::RunNativeCharacterEngine {input} => crate::native_character_engine::run(input),
        ToolRequest::EvaluateNativeCharacterWeights {input} => crate::native_character_weights::evaluate(input),
        ToolRequest::PrepareNativeCharacterSource {input} => crate::native_character_source::prepare(input),
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
            let certificate = audit_native(&plan,context.as_ref(),&profile,&constraints,&generated_at_utc,lineage)?;
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
    if contest_id.trim().is_empty()||contest_id.len()>200 {return Err("Supply a contest ID of 1â€“200 bytes.".into());}
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
        if value.trim().is_empty()||value.len()>200 {return Err("Supply explicit election and jurisdiction metadata (1â€“200 bytes per field).".into());}
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

fn read_report_side(files:&std::collections::BTreeMap<String,Vec<u8>>,label:&str,leaning_threshold:f64)->Result<(bisect_report::comparison::PlanSide,bisect_report::comparison::ReportAvailability),String> {
            const ALLOWED:[&str;4]=["manifest.json","analysis/partisan.json","analysis/vra_analysis.json","analysis/compactness.json"];
            if files.len()>4 || files.keys().any(|key| !ALLOWED.contains(&key.as_str())) {
                return Err("Report inspection accepts only manifest.json and the three named analysis files.".into());
            }
            if files.values().map(Vec::len).sum::<usize>()>4*1024*1024 || label.len()>200 {
                return Err("Report inspection exceeds the portable input limit.".into());
            }
            if !leaning_threshold.is_finite() || !(0.0..=1.0).contains(&leaning_threshold) {
                return Err("Leaning threshold must be a finite fraction between zero and one.".into());
            }
            let side=bisect_report::comparison::load_plan_side_from_files(files,label,leaning_threshold).map_err(|e|e.to_string())?;
            if side.label.len()>200 || side.submitted_by.as_ref().is_some_and(|x|x.len()>200) || side.submitted_at.as_ref().is_some_and(|x|x.len()>200) {
                return Err("Imported report metadata exceeds 200 UTF-8 bytes per field.".into());
            }
            if side.n_districts>500 || side.per_district_dem_share.len()>500 {
                return Err("Report inspection exceeds the portable district limit.".into());
            }

    let demographic=files.get("analysis/vra_analysis.json").and_then(|bytes|serde_json::from_slice::<Value>(bytes).ok()).and_then(|v|v.get("mm_count").and_then(Value::as_u64)).is_some() && side.mm_count<=side.n_districts;
    let compactness=files.get("analysis/compactness.json").and_then(|bytes|serde_json::from_slice::<Value>(bytes).ok()).is_some_and(|v| {
        if let Some(mean)=v.get("mean_polsby_popper").and_then(Value::as_f64) { return (0.0..=1.0).contains(&mean); }
        v.get("districts").and_then(Value::as_array).is_some_and(|rows| rows.len()==side.n_districts && rows.iter().all(|row|row.get("polsby_popper").and_then(Value::as_f64).is_some_and(|x|(0.0..=1.0).contains(&x))))
    });
    let available=bisect_report::comparison::ReportAvailability {partisan:side.per_district_dem_share.len()==side.n_districts,demographic,compactness};
    Ok((side,available))
}

fn report_plan_evidence(
    plan_a:Option<Value>,plan_b:Option<Value>,context:Option<Value>,
    a:&bisect_report::comparison::PlanSide,b:&bisect_report::comparison::PlanSide,
    files_a:&std::collections::BTreeMap<String,Vec<u8>>,files_b:&std::collections::BTreeMap<String,Vec<u8>>,
)->Result<Option<bisect_report::comparison::ReportPlanEvidence>,String> {
    let (plan_a,plan_b)=match (plan_a,plan_b) {
        (Some(a),Some(b))=>(a,b),
        (None,None) if context.is_none()=>return Ok(None),
        _=>return Err("Supply both plans for report measurements; a context cannot be used without them.".into()),
    };
    let measured=crate::plan_comparison::compare(plan_a,plan_b,context)?;
    for (side,files,key) in [(a,files_a,"plan_a"),(b,files_b,"plan_b")] {
        if measured[key]["districts"].as_u64()!=Some(side.n_districts as u64) {return Err("Imported report district count does not match its supplied plan.".into());}
        let manifest:Value=serde_json::from_slice(&files["manifest.json"]).map_err(|e|e.to_string())?;
        for name in ["state","year"] {
            if let Some(declared)=manifest.get(name) {
                let supplied=measured[name].as_str().map(str::to_owned).or_else(||measured[name].as_u64().map(|n|n.to_string())).ok_or("Invalid plan scope.")?;
                let declared=declared.as_str().map(str::to_owned).or_else(||declared.as_u64().map(|n|n.to_string())).ok_or("Invalid imported report scope.")?;
                if declared!=supplied {return Err("Imported report state/year does not match its supplied plan.".into());}
            }
        }
    }
    let population=|value:&Value|->Result<Option<u64>,String>{
        if value.is_null(){Ok(None)}else{value.as_str().ok_or("Invalid population measurement.")?.parse::<u64>().map(Some).map_err(|e|e.to_string())}
    };
    let changes=&measured["assignment_change"];
    Ok(Some(bisect_report::comparison::ReportPlanEvidence {
        unit_kind:measured["unit_kind"].as_str().ok_or("Invalid unit kind.")?.into(),
        changed_units:changes["changed_units"].as_u64().ok_or("Invalid assignment measurement.")? as usize,
        district_ids_with_changes:changes["district_ids_with_changes"].as_array().ok_or("Invalid touched districts.")?.iter().map(|n|n.as_u64().map(|n|n as usize).ok_or_else(||"Invalid district ID.".into())).collect::<Result<Vec<_>,String>>()?,
        population_in_changed_units:population(&changes["population_in_changed_units"] )?,
        total_population_a:population(&measured["plan_a"]["total_population"] )?,
        total_population_b:population(&measured["plan_b"]["total_population"] )?,
        plan_a_hash:measured["plan_a"]["plan_hash"].as_str().ok_or("Missing plan A hash.")?.into(),
        plan_b_hash:measured["plan_b"]["plan_hash"].as_str().ok_or("Missing plan B hash.")?.into(),
        context_hash:measured["context_hash"].as_str().map(str::to_owned),
    }))
}

pub(crate) fn audit_native(plan:&DistrictPlan,context:Option<&RplanContext>,profile:&LegalProfile,constraints:&[AuditConstraint],generated_at_utc:&str,lineage:Option<AlgorithmLineage>)->Result<AuditCertificate,String>{
    let runtime=RuntimeProvenance {binary_name:"bisect-wasm".into(),binary_version:env!("CARGO_PKG_VERSION").into(),git_commit:None,build_profile:Some("wasm-browser".into()),solver:None};
    rplan_audit::audit_plan_with_lineage(plan,context,profile,runtime,constraints,generated_at_utc,lineage).map_err(|e|e.to_string())
}
