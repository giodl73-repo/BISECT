import {validateFreshNativeStateSession} from './fresh-native-state-session.js';
import{validateRecordedReplaySettings,validateRecordedReplayReceiptSource}from'./native-recorded-replay.js';
import {validateNativeReplaySources} from './native-replay-sources.js';
import {validateConfigPublicationSettings} from './config-publication.js';
import {validateCeremonyStartInputs} from './ceremony-start-sources.js';
import {validateElectionSourceFormat} from './election-source-format.js';
import {validateFetchArchivePackageSettings} from './fetch-archive-package.js';
import {validateNativeOutputVerificationSettings} from './native-output-verification.js';
import {validateNativeSelectedStateSourceSettings} from './native-selected-state-source.js';
import {validateNativeStateOutputInputs,validateStreamNativeOutputSettings} from './native-state-output.js';
import {validateNativeStateSourceSettings} from './native-state-sources.js';
import {validateStateGraphInputs} from './state-graph-files.js';
import {validateStateCommandInputs} from './state-command-files.js';
import {validateStateWorkflowBinding} from './native-state-workflow.js';
import {validateNativeReportSnapshot} from './native-report-source-inputs.js';
import {validateContextEngineInputs,validateContextEngineOrigin} from './context-engine-files.js';
import {validateNationalMapDirectory} from './national-map-directory.js';
import {validateWorkerFailure} from './worker-failure.js';
import {validateMigrationBaseInputs} from './migration-base-files.js';
import {validateNativePlanImportInputs} from './native-plan-import-files.js';
import {validateExportFolderInputs} from './export-folder-files.js';
import {validateNativeFolderInputs} from './native-folder-files.js';
import {validateChamberCountInputs} from './chamber-count-files.js';
import {validatePreparedGraphInputs} from './prepared-graph-files.js';
import {validatePreparedCensusTemplateInputs} from './prepared-census-job.js';
import {validatePreparedEngineInputs} from './prepared-engine-files.js';
import {validateNativeCharacterEngineInputs} from './native-character-engine-files.js';
import {validateNativeCharacterWeightInputs} from './native-character-weight-files.js';
import {nativeCharacterFallbackInfo} from './character-fallback-source.js';
import {validateNativeCharacterInputs} from './native-character-files.js';
import {validateCharacterRecheck} from './character-recheck.js';
import {preparedCharacterInput,validateCharacterPreparation} from './character-handoff.js';
import {validateIlpInspectionInputs} from './ilp-inspection-files.js';
import {validatePartitionDiagnosticInputs} from './partition-diagnostic-files.js';
import {validateMultiSummaryInputs} from './multi-summary-files.js';
import {validateNativeAggregateInputs} from './native-aggregate-files.js';
import {validateNativeAggregateWorkspaceSettings} from './native-aggregate-workspace.js';
import {validateNativeNarrativeInputs} from './native-comparison-narrative-files.js';
import {validateDirectAssignmentInputs} from './native-assignment-compare-files.js';
import {validateTextComparisonInputs} from './text-comparison-files.js';
import {validateProofModelInputs} from './proof-model-files.js';
import {validateCertifiedContextSettings,validateCertifiedDiscoverySettings} from './certified-context-files.js';
import {validatePlanHubInputs} from './plan-hub-files.js';
import {validateLabelImportInputs,validateLabelImportContext} from './label-import-files.js';
import {validateLabelImportBaseline} from './label-import-application.js';
import {validateLabelBuildApplicationBaseline} from './label-build-application.js';
import {validateNativeSummaryBaseline} from './native-summary-application.js';
import {validateStateRunBatchInputs} from './state-run-batch.js';
import {validateTuiRunWorkflowSettings} from './state-workflow-files.js';
import {validateTuiRunSettings} from './tui-run-files.js';
import {validateTuiVerifySettings} from './tui-verify-files.js';
import {validateTuiDoctorInputs} from './tui-doctor-files.js';
import {validateTuiCompareInputs} from './tui-compare-files.js';
import {validateTuiSessionInputs} from './tui-session-files.js';
import {validateTuiLibraryInputs} from './tui-library-files.js';
import {validateNationalMapBatchInputs} from './national-map-batch.js';
import {validateNativeMapInputs,validateNativeNationalMapSource} from './map-export-files.js';
import {validateStateBatchInputs} from './state-batch-files.js';
import {validateNativeAdjacencyInputs,validateNativeAdjacencySettings} from './native-adjacency-files.js';
import {validateFetchArchiveSource} from './fetch-acquisition.js';
import {validateFletchStageInputs,resolveFletchStageSource} from './fletch-stage-files.js';
import {validateFletchCacheUpdates} from './fletch-merge-files.js';
import {validateFletchSourceSettings} from './fletch-source-files.js';
import {validateFletchCacheManifestSource,validateFletchCacheSettings} from './fletch-cache-files.js';
import {validateTractPreparationSettings} from './tract-input-files.js';
import {validateTigerInputs} from './tiger-files.js?v=tiger-large-v1';
import {validatePl94Inputs} from './pl94-files.js?v=pl94-large-v1';
import {validateFetchDataSettings} from './fetch-data-files.js';
import {validateFetchAcquisition} from './fetch-acquisition.js?v=census-acquire-zip-v1';
import {validateFetchPlanInputs} from './fetch-plan-files.js';
import {validateLodesInputs} from './lodes-files.js?v=tract-preparation-v1';
import {validateAcsHousingInputs} from './acs-housing-files.js';
import {validateSweepSettings} from './sweep-files.js';
import {validateVerifyAssignmentsSettings} from './verify-assignments-files.js';
import {validateAssignmentDiscoveryInputs} from './assignment-discovery-files.js';
import {validateEngineReplayInputs} from './engine-replay-files.js';
import {validateAssignmentVerificationInputs} from './assignment-verification-files.js';
import {validatePackageVerificationInputs} from './package-verification-files.js';
import {validateIlpAuditInputs,ilpAuditScopeSettings,ilpAuditScopeSelection} from './ilp-audit-files.js';
import {validateLabelPackageMergeInputs} from './label-package-merge-files.js';
import {validateLabelReconcileInputs} from './label-reconcile-files.js';
import {validateLabelReportInputs} from './label-report-files.js';
import {validateLabelAnalysisInputs} from './label-analysis-files.js';
import {validateAnalysisCollection} from './analysis-collection.js';
import {validateLabelBuildInputs} from './label-build-files.js?v=label-build-checkpoint-v1';
import {validateLabelRenameInputs} from './label-rename-files.js';
import {validateLabelInventoryInputs,validateLabelCompareInputs} from './label-inventory-files.js';
import {validateLabelChainInputs} from './label-chain-files.js';
import {validateMigrationInputs} from './migration-files.js';
import {validateCandidateInputs} from './candidate-files.js';
import {validateCivicManifestSources,validateCivicInputs,validateCivicIngestSettings,validateNativeCivicConflictInputs} from './civic-files.js';
import {validateSuiteAssemblyInputs} from './suite-assembly-files.js';
import {validatePlanDoctorInputs} from './plan-doctor-files.js';
import {validateTutorialInputs} from './tutorial-files.js';
import {validateManifestSource} from './manifest-files.js';
import {validateDoctorInputs} from './doctor-files.js';
import {validatePolicyInputs} from './policy-files.js';
import {validateConfigRunInputs,validateConfigRunSettings} from './config-run.js';
import {validateConfigTemplate} from './config-template.js';
import {validateConfigSource,validateConfigSettings} from './config-files.js';
import {validateCommissionInputs,validateCommissionAuditSource,validateCommissionFallbackSettings,validateCommissionOpaqueSource,validateCommissionDeliverySettings} from './commission-files.js';
import {validateNationalInputs} from './national-files.js';
import {validateBlocInputs} from './bloc-files.js';
import {validateCompactnessInputs} from './compactness-files.js?v=native-build-v2';
import {validateStatisticsInputs} from './statistics-files.js';
import {validateElectionAnalysisSettings} from './election-analysis-settings.js';
import {validateStabilityInputs} from './stability-files.js';
import {validateSuiteInputs} from './suite-files.js';
import {validateResearchInputs} from './research-files.js';
import {validateReproducibilityInputs} from './reproducibility-files.js';
import {validateReportOutput} from './report-output.js';
// Portable project snapshots. Browser storage is a convenience; downloaded files are backups.
import {validatePlanImportSettings,validateParetoSelection,validateParetoSettings,validateSmcSelection,validateSmcSettings,validateImprovementSettings,validateTextAuditSettings,validateColumnSettings,validateCeremonySettings,validatePackageFiles,validateSourceFile,validateImportSettings,validateRiSources,validateAggregationSettings,validateDemographicSettings,validateReportFiles,validateReportSettings,validateReportComparisonSettings} from './package-files.js?v=native-build-v2';
export const PROJECT_SCHEMA = 'bisect-project-v1';
export const MAX_PROJECT_BYTES = 25 * 1024 * 1024;
export const WORKBENCH_ARCHIVE_SCHEMA='bisect-workbench-archive-v1';
export const MAX_WORKBENCH_ARCHIVE_BYTES=128*1024*1024;
export const MAX_WORKBENCH_EXPANDED_BYTES=256*1024*1024;
// Dense full national receipts exceed four million values; ordinary JSON remains capped.
export const MAX_WORKBENCH_VALUES=8000000;
const fields = ['schema_version','name','saved_at_utc','files','operation','constraints','result','lastOperation'];
import {validateConfiguredRuntimeArtifacts} from './configured-runtime-artifacts.js';
const inputs = ['freshNativeStateSession','nativeRecordedReplayReceipt','nativeRecordedReplaySettings','nativeReplaySources','configPublicationSettings','labelBuildApplicationBaseline','fetchArchivePackageSettings','nativeOutputVerificationSettings','nativeOutputVerificationArchive','nativeSelectedStateSourceSettings','stateReportSourceSnapshot','stateWorkflowBinding','runCommandInputs','statesCommandInputs','streamNativeOutputSettings','nativeStateOutputInputs','nativeStateSourceSettings','stateGraphInputs','stateCommandInputs','contextEngineInputs','contextEngineOrigin','nativeNationalDirectoryInputs','workerFailureReceipt','migrationBaseInputs','nativePlanImportInputs','exportFolderInputs','nativeFolderInputs','nativeWorkspaceArchive','chamberCountInputs','preparedGraphInputs','preparedCensusTemplateInputs','preparedEngineInputs','nativeCharacterEngineInputs','nativeCharacterWeightInputs','characterFallbackSource','nativeCharacterInputs','characterPreparationRecheck','characterPreparation','ilpInspectionInputs','partitionDiagnosticInputs','configuredRuntimeArtifacts','multiSummaryInputs','nativeAggregateInputs','nativeAggregateWorkspaceSettings','nativeNarrativeInputs','directAssignmentInputs','textComparisonInputs','proofModelInputs','certifiedDiscoverySettings','certifiedContextSettings','nationalMapBatchInputs','planHubInputs','labelImportApplicationBaseline','nativeSummaryApplicationBaseline','labelImportContext','labelImportInputs','labelCompareInputs','stateRunBatchInputs','tuiRunWorkflowSettings','tuiRunSettings','tuiVerifySettings','tuiDoctorInputs','tuiCompareInputs','tuiSessionInputs','tuiLibraryInputs','nativeNationalMapSource','nativeMapInputs','stateBatchInputs','nativeAdjacencySettings','nativeAdjacencyInputs','fetchArchiveSource','fletchStageInputs','fletchCacheUpdates','fletchSourceSettings','fletchCacheSettings','fletchCacheManifest','tractPreparationSettings','tigerInputs','pl94Inputs','fetchDataPackageSettings','fetchAcquisitionReceipt','fetchPlanInputs','lodesInputs','acsHousingInputs','sweepSettings','verifyAssignmentsSettings','assignmentDiscoveryInputs','engineReplayInputs','assignmentVerificationInputs','packageVerificationInputs','ilpAuditInputs','ilpAuditScopeSettings','labelPackageMergeInputs','labelReconcileInputs','labelReportInputs','labelAnalysisInputs','splitsGeography','analysisCollection','commissionDeliverySettings','commissionFallbackSettings','commissionAuditOpaqueSource','commissionAuditSource','commissionInputs','labelBuildInputs','labelRenameInputs','labelInventoryInputs','labelChainInputs','migrationInputs','candidateInputs','civicManifestSources','nativeCivicConflictInputs','civicIngestSettings','civicInputs','suiteAssemblyInputs','nativeValidationSettings','planDoctorInputs','tutorialInputs','manifestSource','doctorInputs','policyInputs','configRunInputs','configRunSettings','configTemplate','configSettings','configSource','nationalInputs','blocInputs','compactnessInputs','statisticsInputs','electionAnalysisSettings','electionCsvFormat','electionAnalysisSource','stabilityInputs','suiteInputs','researchInputs','reproducibilityInputs','planImportCentroids','planImportSource','planImportSettings','paretoFrontier','paretoSelection','paretoSettings','planGeometry','smcEnsemble','smcSelection','smcSettings','improvementSettings','auditPlanText','auditContextText','auditProfileText','auditCertificateText','textAuditSettings','auditLineageText','columnSettings','ceremonyTree','ceremonyConfig','ceremonyTranscript','ceremonySettings','ceremonyStartInputs','exactSource','reportPlanSettings','comparisonReportFiles','reportComparisonSettings','reportFiles','reportSettings','plan','comparisonPlan', 'context', 'profile', 'certificate', 'package','packageFiles','sourceFile','importSettings','riSources','crosswalkFile','aggregationSettings','nativeDemographicReceipt','demographicSource','demographicSettings'];
const operations = ['prepare-fetch-archive-package','verify-native-output-workspace','run-native-selected-state-source','inspect-native-run-command','prepare-native-run-run','prepare-native-run-graph-run','run-native-run-graph','run-native-run-template','run-native-run-job','inspect-native-states-command','prepare-native-states-run','prepare-native-states-graph-run','run-native-states-graph','run-native-states-template','run-native-states-job','prepare-native-state-output','run-accepted-state-workflow','prepare-context-engine-input','render-native-national-directory','inspect-native-migration-base','stage-native-migration-base','import-native-plan','export-native-plan-folder','inspect-native-input-folder','resolve-native-chamber-count','bind-prepared-engine-graph','run-prepared-engine','run-native-character-engine','evaluate-native-character-weights','prepare-native-character-source','summarize-plan-labels','aggregate-native-analysis','render-native-comparison-narrative','compare-native-assignment-files','compare-rplan-text','compile-certified-proof-models','solve-certified-discovery-context','solve-certified-canonical-context','solve-certified-tree-context','inspect-plan-hub','prepare-label-import-plan','import-label-assignments','inspect-label-comparison','run-state-batch','prepare-census-engine-input','run-tui-configured','verify-tui-manifest','diagnose-tui-location','compare-tui-plans','inspect-tui-session','inspect-tui-plan-library','render-native-national-map-batch','render-native-national-map','render-native-district-map','prepare-native-state-graph-run','prepare-native-state-run','inspect-native-state-command','inspect-state-batch-selection','prepare-native-engine-input','inspect-native-adjacency','stage-fletch-cache-source','merge-fletch-cache-manifest','verify-fletch-cache-files','inspect-fletch-sources','inspect-fletch-cache','prepare-tract-engine-input','prepare-native-map-tiger','prepare-tiger-source','prepare-pl94-source','prepare-fetch-data-package','plan-fetch-sources','prepare-lodes-source','prepare-acs-housing-source','plan-sweep','verify-assignments-only','discover-original-assignments','verify-manifest-replay','replay-engine-run','verify-manifest-prechecks','inspect-native-replay-recipe','inspect-native-replay-sources','prepare-native-recorded-replay','run-native-recorded-replay','compare-verify-assignments','verify-manifest-certificate','verify-rplan-package','audit-ilp-artifacts','reconcile-label-package','reconcile-label-registry','run-label-report','run-label-analysis','analyze-plan-package','analyze-plan-splits','prepare-plan-report-manifest','analyze-plan-contiguity','assemble-commission-audit','assemble-commission-report','run-label-build','rename-label-package','inspect-label-registry','verify-label-chain','migrate-legacy-plan','ingest-candidate-race','inspect-civic-manifests','inspect-native-civic-conflicts','ingest-civic-coi','inspect-civic-coi','assemble-plan-suite','validate-native-rplan','diagnose-plan-artifacts','verify-tutorial-files','verify-run-manifest','diagnose-location-inputs','inspect-location-policy','run-algorithm-config','create-algorithm-config','validate-algorithm-config','inspect-algorithm-config','aggregate-national-analysis','analyze-bloc-voting','analyze-plan-compactness','analyze-plan-statistics','analyze-plan-election','analyze-suite-stability','validate-suite-nesting','inspect-native-ilp-run','diagnose-partition-traces','validate-research-ensemble','export-reproducibility-package','import-plan-interchange','export-rplan-interchange','select-pareto-plan','sample-pareto-context','validate-rplan-text','select-smc-plan','sample-smc-context','improve-rplan-text','audit-rplan-text','verify-audit-text','solve-branch-price-context','solve-branch-price','start-ceremony-from-sources','start-ceremony','announce-ceremony','halt-ceremony','verify-ceremony','inspect-ceremony-prefix','solve-certified-canonical','verify-certified-canonical','solve-certified-single','verify-certified-single','solve-certified-split','verify-certified-split','solve-certified-tree','verify-certified-tree','render-report-comparison','inspect-report-side','compare-plans','inspect-native-demographic-csv','attach-demographic-csv','verify-history-files', 'validate-rplan', 'audit-plan', 'verify-certificate', 'verify-count','verify-count-files','replay-count-audits','import-statement-csv','import-nist-cdf-json','import-ri2024-rep28-rla','aggregate-districts'];
const checks = ['plan-shape', 'population', 'contiguity', 'splits', 'vra', 'geometry'];
export function createProject({name, files, operation, constraints, result, lastOperation}) {
  return parseProject(JSON.stringify({schema_version: PROJECT_SCHEMA,
    name: name || 'Untitled project', saved_at_utc: new Date().toISOString(),
    files, operation, constraints, result: result ?? null, lastOperation: lastOperation ?? null}));
}
export function validateProject(project) {
  return validateProjectEnvelope(project,PROJECT_SCHEMA,MAX_PROJECT_BYTES);
}
export function validateWorkbenchArchive(project){
  return validateProjectEnvelope(project,WORKBENCH_ARCHIVE_SCHEMA,MAX_WORKBENCH_ARCHIVE_BYTES,MAX_WORKBENCH_VALUES);
}
export function validateExpandedWorkbenchArchive(project){
  return validateProjectEnvelope(project,WORKBENCH_ARCHIVE_SCHEMA,MAX_WORKBENCH_EXPANDED_BYTES,MAX_WORKBENCH_VALUES);
}
function validateProjectEnvelope(project,schema,maxStringBytes,maxValues=500000) {
  validateJsonTree(project,maxValues,maxStringBytes);
  if (!isRecord(project) || Object.keys(project).length !== fields.length || fields.some(key => !Object.hasOwn(project,key)) || Object.keys(project).some(key => !fields.includes(key))) throw new Error('Invalid project fields.');
  if (!project || project.schema_version !== schema) throw new Error('Unsupported project format.');
  if (typeof project.name !== 'string' || project.name.length > 200) throw new Error('Invalid project name.');
  if (typeof project.saved_at_utc !== 'string' || project.saved_at_utc.length > 40 || !Number.isFinite(Date.parse(project.saved_at_utc))) throw new Error('Invalid project save date.');
  if (!operations.includes(project.operation)) throw new Error('Unsupported project operation.');
  if (!Array.isArray(project.constraints) || project.constraints.length > checks.length || new Set(project.constraints).size !== project.constraints.length || project.constraints.some(c => !checks.includes(c))) throw new Error('Invalid project checks.');
  if (!isRecord(project.files)) throw new Error('Invalid project inputs.');
  if(project.files.commissionAuditSource&&project.files.commissionAuditOpaqueSource)throw new Error('Supply one commission audit manifest representation.');
  for (const [key, value] of Object.entries(project.files)) {
    // Null means the pending selection was moved into a checked workflow binding.
    if(key==='stateReportSourceSnapshot'&&value===null)continue;
    if (!inputs.includes(key) || !isRecord(value)) throw new Error('Invalid project input: ' + key);
    if(key==='configPublicationSettings')validateConfigPublicationSettings(value);
    if(key==='configuredRuntimeArtifacts')validateConfiguredRuntimeArtifacts(value);
    if(key==='tuiSessionInputs')validateTuiSessionInputs(value);
    if(key==='multiSummaryInputs')validateMultiSummaryInputs(value);
    if(key==='nativeAggregateInputs')validateNativeAggregateInputs(value);
    if(key==='nativeAggregateWorkspaceSettings')validateNativeAggregateWorkspaceSettings(value);
    if(key==='nativeNarrativeInputs')validateNativeNarrativeInputs(value);
    if(key==='directAssignmentInputs')validateDirectAssignmentInputs(value);
    if(key==='textComparisonInputs')validateTextComparisonInputs(value);
    if(key==='tuiCompareInputs')validateTuiCompareInputs(value);
    if(key==='tuiDoctorInputs')validateTuiDoctorInputs(value);
    if(key==='nativeOutputVerificationSettings')validateNativeOutputVerificationSettings(value);
    if(key==='tuiVerifySettings')validateTuiVerifySettings(value);
    if(key==='tuiRunSettings')validateTuiRunSettings(value);
    if(key==='nativeStateOutputInputs')validateNativeStateOutputInputs(value);
    if(key==='tuiRunWorkflowSettings')validateTuiRunWorkflowSettings(value);
    if(key==='tuiLibraryInputs')validateTuiLibraryInputs(value);
    if(key==='suiteAssemblyInputs')validateSuiteAssemblyInputs(value);
    if(key==='fetchArchivePackageSettings')validateFetchArchivePackageSettings(value);
    if(key==='fetchDataPackageSettings')validateFetchDataSettings(value);
    if(key==='fetchArchiveSource')validateFetchArchiveSource(value);
    if(key==='fetchAcquisitionReceipt')validateFetchAcquisition(value);
    if(key==='fletchSourceSettings')validateFletchSourceSettings(value);
    if(key==='fletchCacheSettings')validateFletchCacheSettings(value);
    if(key==='nativeNationalDirectoryInputs')validateNationalMapDirectory(value);
    if(key==='nationalMapBatchInputs')validateNationalMapBatchInputs(value);
    if(key==='nativeNationalMapSource')validateNativeNationalMapSource(value);
    if(key==='nativeMapInputs')validateNativeMapInputs(value);
    if(key==='stateRunBatchInputs')validateStateRunBatchInputs(value);
    if(key==='streamNativeOutputSettings')validateStreamNativeOutputSettings(value);
    if(key==='nativeStateSourceSettings')validateNativeStateSourceSettings(value);
    if(key==='stateGraphInputs')validateStateGraphInputs(value);
    if(key==='runCommandInputs'||key==='statesCommandInputs')validateStateCommandInputs(value);
    if(key==='stateCommandInputs')validateStateCommandInputs(value);
    if(key==='nativeSelectedStateSourceSettings')validateNativeSelectedStateSourceSettings(value);
    if(key==='stateWorkflowBinding')validateStateWorkflowBinding(value);
    if(key==='stateReportSourceSnapshot'&&value!==null)validateNativeReportSnapshot(value);
    if(key==='stateBatchInputs')validateStateBatchInputs(value);
    if(key==='certifiedContextSettings')validateCertifiedContextSettings(value);
    if(key==='proofModelInputs')validateProofModelInputs(value);
    if(key==='certifiedDiscoverySettings')validateCertifiedDiscoverySettings(value);
    if(key==='contextEngineInputs')validateContextEngineInputs(value);
    if(key==='contextEngineOrigin')validateContextEngineOrigin(value);
    if(key==='nativeAdjacencySettings')validateNativeAdjacencySettings(value);
    if(key==='nativeAdjacencyInputs')validateNativeAdjacencyInputs(value);
    if(key==='fletchStageInputs'){validateFletchStageInputs(value);if(value.source_ref)resolveFletchStageSource(value,project.files);}
    if(key==='fletchCacheUpdates')validateFletchCacheUpdates(value);
    if(key==='fletchCacheManifest')validateFletchCacheManifestSource(value);
    if(key==='fetchPlanInputs')validateFetchPlanInputs(value);
    if(key==='tractPreparationSettings')validateTractPreparationSettings(value);
    if(key==='tigerInputs')validateTigerInputs(value);
    if(key==='pl94Inputs')validatePl94Inputs(value);
    if(key==='lodesInputs')validateLodesInputs(value);
    if(key==='preparedCensusTemplateInputs')validatePreparedCensusTemplateInputs(value);
    if(key==='migrationBaseInputs')validateMigrationBaseInputs(value);
    if(key==='nativePlanImportInputs')validateNativePlanImportInputs(value);
    if(key==='exportFolderInputs')validateExportFolderInputs(value);
    if(key==='nativeFolderInputs')validateNativeFolderInputs(value);
    if(key==='nativeWorkspaceArchive')validateSourceFile(value,{maxBytes:64*1024*1024});
    if(key==='chamberCountInputs')validateChamberCountInputs(value);
    if(key==='preparedGraphInputs')validatePreparedGraphInputs(value);
    if(key==='preparedEngineInputs')validatePreparedEngineInputs(value);
    if(key==='nativeCharacterEngineInputs')validateNativeCharacterEngineInputs(value);
    if(key==='nativeCharacterWeightInputs')validateNativeCharacterWeightInputs(value);
    if(key==='nativeCharacterInputs')validateNativeCharacterInputs(value);
    if(key==='acsHousingInputs')validateAcsHousingInputs(value);
    if(key==='characterPreparation'){preparedCharacterInput(value);const input=project.files.plan?.provenance?.producer?.character_input??project.files.plan?.provenance?.producer?.multiscale_input?.request?.character;if(input)validateCharacterPreparation(value,input);}
    if(key==='characterFallbackSource'){const info=nativeCharacterFallbackInfo(value);if(project.files.characterPreparation)throw new Error('Observed and fallback source companions conflict.');const producer=project.files.plan?.provenance?.producer;if(project.files.plan){const request=producer?.multiscale_input?.request,options=request?.options??producer?.engine_options;if(project.files.plan.metadata?.jurisdiction!==info.state||options?.character_absent_file_fallback!==true||options.weights!==info.kind+'-character'||producer?.character_input||request?.character)throw new Error('Plan producer does not record the selected character fallback.');}}
    if(key==='characterPreparationRecheck'){const source=project.files.characterPreparation??project.files.characterFallbackSource;if(!source)throw new Error('Recheck receipt requires retained preparation sources.');validateCharacterRecheck(value,source);}
    if(key==='sweepSettings')validateSweepSettings(value);
    if(key==='verifyAssignmentsSettings')validateVerifyAssignmentsSettings(value);
    if(key==='assignmentDiscoveryInputs')validateAssignmentDiscoveryInputs(value);
    if(key==='engineReplayInputs')validateEngineReplayInputs(value);
    if(key==='assignmentVerificationInputs')validateAssignmentVerificationInputs(value);
    if(key==='nativeRecordedReplaySettings')validateRecordedReplaySettings(value);
    if(key==='freshNativeStateSession')validateFreshNativeStateSession(value);
    if(key==='nativeRecordedReplayReceipt')validateRecordedReplayReceiptSource(value);
    if(key==='nativeReplaySources')validateNativeReplaySources(value);
    if(key==='packageVerificationInputs')validatePackageVerificationInputs(value);
    if(key==='ilpAuditInputs')validateIlpAuditInputs(value);
    if(key==='ilpAuditScopeSettings'){ilpAuditScopeSettings(value);if(value.mode==='summary-declared')ilpAuditScopeSelection(project.files.ilpAuditInputs,value);}
    if(key==='labelPackageMergeInputs')validateLabelPackageMergeInputs(value);
    if(key==='labelReconcileInputs')validateLabelReconcileInputs(value);
    if(key==='labelReportInputs')validateLabelReportInputs(value);
    if(key==='labelAnalysisInputs')validateLabelAnalysisInputs(value);
    if(key==='labelBuildApplicationBaseline')validateLabelBuildApplicationBaseline(value);
    if(key==='labelBuildInputs')validateLabelBuildInputs(value);
    if(key==='labelRenameInputs')validateLabelRenameInputs(value);
    if(key==='planHubInputs')validatePlanHubInputs(value);
    if(key==='labelImportContext')validateLabelImportContext(value);
    if(key==='labelImportInputs')validateLabelImportInputs(value);
    if(key==='labelImportApplicationBaseline')validateLabelImportBaseline(value);
    if(key==='nativeSummaryApplicationBaseline')validateNativeSummaryBaseline(value);
    if(key==='labelCompareInputs')validateLabelCompareInputs(value);
    if(key==='labelInventoryInputs')validateLabelInventoryInputs(value);
    if(key==='labelChainInputs')validateLabelChainInputs(value);
    if(key==='migrationInputs')validateMigrationInputs(value);
    if(key==='candidateInputs')validateCandidateInputs(value);
    if(key==='civicManifestSources')validateCivicManifestSources(value);
    if(key==='nativeCivicConflictInputs')validateNativeCivicConflictInputs(value);
    if(key==='civicIngestSettings')validateCivicIngestSettings(value);
    if(key==='civicInputs')validateCivicInputs(value);
    if(key==='nativeValidationSettings'&&(!value||Object.keys(value).join()!=='strict'||typeof value.strict!=='boolean'))throw new Error('Invalid native validation settings.');
    if(key==='workerFailureReceipt')validateWorkerFailure(value);
    if(key==='planDoctorInputs')validatePlanDoctorInputs(value);
    if(key==='tutorialInputs')validateTutorialInputs(value);
    if(key==='manifestSource')validateManifestSource(value);
    if(key==='doctorInputs')validateDoctorInputs(value);
    if(key==='policyInputs')validatePolicyInputs(value);
    if(key==='reportPlanSettings'&&(Object.keys(value).length!==1||typeof value.include_plan_measurements!=='boolean'))throw new Error('Invalid report plan measurement settings.');
    if(key==='comparisonReportFiles')validateReportFiles(value);
    if(key==='reportComparisonSettings')validateReportComparisonSettings(value);
    if(key==='blocInputs')validateBlocInputs(value);
    if(key==='compactnessInputs')validateCompactnessInputs(value);
    if(key==='statisticsInputs')validateStatisticsInputs(value);
    if(key==='electionCsvFormat')validateElectionSourceFormat(value);
    if(key==='electionAnalysisSettings')validateElectionAnalysisSettings(value);
    if(key==='electionAnalysisSource'){validateSourceFile(value);if(value.base64.length>4*1024*1024)throw new Error('Election analysis source exceeds 3 MiB.');}
    if(key==='configRunInputs')validateConfigRunInputs(value);
    if(key==='configRunSettings')validateConfigRunSettings(value);
    if(key==='configTemplate')validateConfigTemplate(value);
    if(key==='configSettings')validateConfigSettings(value);
    if(key==='configSource')validateConfigSource(value);
    if(key==='analysisCollection')validateAnalysisCollection(value);
    if(key==='commissionInputs')validateCommissionInputs(value);
    if(key==='commissionAuditSource')validateCommissionAuditSource(value);
    if(key==='commissionAuditOpaqueSource')validateCommissionOpaqueSource(value);
    if(key==='commissionFallbackSettings')validateCommissionFallbackSettings(value);
    if(key==='commissionDeliverySettings')validateCommissionDeliverySettings(value,{requireComplete:false});
    if(key==='nationalInputs')validateNationalInputs(value);
    if(key==='stabilityInputs')validateStabilityInputs(value);
    if(key==='suiteInputs')validateSuiteInputs(value);
    if(key==='ilpInspectionInputs')validateIlpInspectionInputs(value);
    if(key==='partitionDiagnosticInputs')validatePartitionDiagnosticInputs(value);
    if(key==='researchInputs')validateResearchInputs(value);
    if(key==='reproducibilityInputs')validateReproducibilityInputs(value);
    if(key==='reportFiles')validateReportFiles(value);
    if(key==='reportSettings')validateReportSettings(value);
    if(key==='packageFiles')validatePackageFiles(value);
    // This shared chooser also retains native RCTX for column benchmarks. Keep
    // it when switching tools; individual requests enforce their own text cap.
    if(key==='exactSource')validateSourceFile(value,{maxBytes:8*1024*1024});
    if(['ceremonyTree','ceremonyConfig','ceremonyTranscript'].includes(key)){validateSourceFile(value);if(value.base64.length>4*1024*1024)throw new Error('Ceremony artifact exceeds 3 MiB.');}
    if(key==='splitsGeography')validateSourceFile(value,{maxBytes:3*1024*1024});
    if(key==='auditContextText')validateSourceFile(value,{maxBytes:8*1024*1024});
    if(key==='auditPlanText')validateSourceFile(value,{maxBytes:8*1024*1024});
    if(['auditProfileText','auditCertificateText','auditLineageText'].includes(key)){validateSourceFile(value);if(value.base64.length>4*1024*1024)throw new Error('Text audit source exceeds 3 MiB.');}
    if(['planImportSource','planImportCentroids'].includes(key)){validateSourceFile(value);if(value.base64.length>4*1024*1024)throw new Error('Plan import exceeds 3 MiB.');}
    if(key==='planImportSettings')validatePlanImportSettings(value);
    if(key==='planGeometry')validateSourceFile(value,{maxBytes:64*1024*1024});
    if(key==='smcEnsemble')validateSourceFile(value);
    if(key==='smcSelection')validateSmcSelection(value);
    if(key==='paretoFrontier')validateSourceFile(value);
    if(key==='paretoSelection')validateParetoSelection(value);
    if(key==='paretoSettings')validateParetoSettings(value);
    if(key==='smcSettings')validateSmcSettings(value);
    if(key==='improvementSettings')validateImprovementSettings(value);
    if(key==='textAuditSettings')validateTextAuditSettings(value);
    if(key==='columnSettings')validateColumnSettings(value);
    if(key==='ceremonySettings')validateCeremonySettings(value);
    if(key==='ceremonyStartInputs')validateCeremonyStartInputs(value);
    if(key==='sourceFile')validateSourceFile(value);
    if(key==='importSettings')validateImportSettings(value,{requireComplete:false});
    if(key==='riSources')validateRiSources(value,{requireComplete:false});
    if(key==='crosswalkFile')validateSourceFile(value);
    if(key==='aggregationSettings')validateAggregationSettings(value,{requireComplete:false});
    if(key==='demographicSource'||key==='nativeDemographicReceipt')validateSourceFile(value);
    if(key==='demographicSettings')validateDemographicSettings(value,{requireComplete:false,allowTotalPopulation:project.operation==='inspect-native-demographic-csv'||project.lastOperation==='inspect-native-demographic-csv'||Object.hasOwn(project.files,'nativeDemographicReceipt')});
  }
  if (project.lastOperation !== null && !operations.includes(project.lastOperation)) throw new Error('Invalid result operation.');
  if (project.result !== null && (typeof project.result !== 'object' || Array.isArray(project.result) || !project.lastOperation)) throw new Error('Invalid project result.');
  if (project.result?.schema_version === 'bisect-election-import-v1') validatePackageFiles(project.result.package_files);
  if (project.result?.schema_version === 'bisect-district-aggregation-v1') validateSourceFile({name:'aggregation.json',base64:project.result.transcript_base64});
  if(project.result?.schema_version==='bisect-imported-report-comparison-v1')validateReportOutput(project.result);
  return project;
}
export function parseProject(text) {
  return validateProject(parseSafeJson(text));
}
export function parseSafeJson(text) {
  if (typeof text !== 'string' || text.length > MAX_PROJECT_BYTES || new TextEncoder().encode(text).byteLength > MAX_PROJECT_BYTES) throw new Error('Project exceeds the 25 MB import limit.');
  inspectJsonText(text);
  const value = JSON.parse(text); validateJsonTree(value); return value;
}
// Native exports retain an exact numeric u64 engine seed in provenance. Only
// that metadata leaf becomes a decimal string in this display view; original
// source text is still passed unchanged to Rust. Other unsafe integers reject.
export function parseNativeMapPlanJson(text){
 const cap=3*1024*1024;
 if(typeof text!=='string'||text.length>cap||new TextEncoder().encode(text).byteLength>cap)throw new Error('Native map RPLAN exceeds 3 MiB.');
 inspectJsonText(text);const large=[];
 const value=JSON.parse(text,(_key,item,context)=>{
  if(typeof item!=='number'||!Number.isInteger(item)||Number.isSafeInteger(item))return item;
  const raw=context?.source;
  if(typeof raw!=='string'||!/^(0|[1-9][0-9]{0,19})$/.test(raw)||BigInt(raw)>18446744073709551615n)throw new Error('Native map integer exceeds the supported exact range.');
  const marker={decimal:raw};large.push(marker);return marker;
 });
 if(large.length){const options=value?.provenance?.producer?.engine_options;if(large.length!==1||options?.seed!==large[0])throw new Error('Only the native engine provenance seed may exceed the JavaScript integer range.');options.seed=large[0].decimal;}
 validateJsonTree(value);return value;
}
// Geometry has many coordinate values; project snapshots retain these as bounded bytes.
// The streamed package holds compressed per-state sources; individual geometry
// expansion remains bounded separately and only occurs on explicit Run.
export function parseNationalMapBatchJson(text){
 const cap=128*1024*1024;
 if(typeof text!=='string'||text.length>cap||new TextEncoder().encode(text).byteLength>cap)throw new Error('National map source package exceeds 128 MiB.');
 inspectJsonText(text);const value=JSON.parse(text);validateJsonTree(value,500000,cap);return value;
}
export function parseNativeNationalMapJson(text){
 if(typeof text!=="string"||text.length>64*1024*1024||new TextEncoder().encode(text).byteLength>64*1024*1024)throw new Error("National map bundle exceeds 64 MiB.");
 inspectJsonText(text);const value=JSON.parse(text);validateJsonTree(value,500000,64*1024*1024);return value;
}
export function parseGeometryJson(text){
 const large=typeof text==='string'&&/"geometry_profile"\s*:\s*"(?:source-exact-large|display-topology)"/.test(text),cap=(large?64:16)*1024*1024;
 if(typeof text!=='string'||text.length>cap||new TextEncoder().encode(text).byteLength>cap)throw new Error(large?'Large source geometry exceeds 64 MiB.':'Display geometry exceeds 16 MiB.');
 const values=large?8000000:4000000;inspectJsonText(text,values);const value=JSON.parse(text);validateJsonTree(value,values,cap);
 if(large&&!['source-exact-large','display-topology'].includes(value.geometry_profile))throw new Error('Large geometry requires the explicit root source profile.');return value;
}
// Engine sources have dense adjacency/edge arrays; keep general imports capped.
export function parsePreparedGraphJson(text){
  const cap=8*1024*1024;
  if(typeof text!=='string'||text.length>cap||new TextEncoder().encode(text).byteLength>cap)throw new Error('Prepared graph exceeds 8 MiB.');
  inspectJsonText(text,2000000);const value=JSON.parse(text);validateJsonTree(value,2000000);return value;
}
// Prepared build batches contain many graph values but remain bounded source bytes.
// Keep ordinary projects and other JSON imports at their existing complexity limit.
export function parsePreparedJobsJson(text){
  if(typeof text!=='string'||text.length>16*1024*1024||new TextEncoder().encode(text).byteLength>16*1024*1024)throw new Error('Prepared jobs exceed 16 MiB.');
  inspectJsonText(text,4000000);const value=JSON.parse(text);validateJsonTree(value,4000000);return value;
}
export function parseBuildArchiveJson(text){
  return parseArchiveJson(text,MAX_WORKBENCH_ARCHIVE_BYTES,'Build archive exceeds 128 MiB.');
}
// Only the bounded gzip archive reader uses this decoded-text allowance.
export function parseCompressedWorkbenchJson(text){
  return parseArchiveJson(text,MAX_WORKBENCH_EXPANDED_BYTES,'Compressed archive decoded JSON exceeds 256 MiB.');
}
function parseArchiveJson(text,cap,message){
  if(typeof text!=='string'||text.length>cap||new TextEncoder().encode(text).byteLength>cap)throw new Error(message);
  inspectJsonText(text,MAX_WORKBENCH_VALUES);const value=JSON.parse(text);validateJsonTree(value,MAX_WORKBENCH_VALUES,cap);return value;
}
// JSON.parse accepts duplicate fields by keeping the last one. Reject that
// ambiguity, including escaped keys. Bound depth and values before JSON.parse
// allocates an object tree; syntax validation still belongs to JSON.parse.
export function inspectJsonText(text, maxValues=500000) {
  const containers=[];let nodes=0;
  function value(depth) {
    if(++nodes>maxValues || depth>64)throw new Error(`JSON is too complex (maximum 64 levels and ${maxValues.toLocaleString('en-US')} values).`);
  }
  for(let i=0;i<text.length;i++) {
    const char=text[i];
    if(char==='{' || char==='[') { value(containers.length);containers.push(char==='{'?new Set():null); }
    else if(char==='}' || char===']')containers.pop();
    else if(char==='"') {
      const start=i;
      while(++i<text.length) { if(text[i]==='\\')i++; else if(text[i]==='"')break; }
      let next=i+1;while(/\s/.test(text[next]||'') && next<text.length)next++;
      if(text[next]===':') {
        const key=JSON.parse(text.slice(start,i+1)),keys=containers[containers.length-1];
        if(keys instanceof Set) {
          if(['__proto__','constructor','prototype'].includes(key))throw new Error('Unsafe project property.');
          if(keys.has(key))throw new Error('Duplicate JSON property.');
          keys.add(key);
        }
      } else value(containers.length);
    } else if(!/[\s,:]/.test(char)) {
      value(containers.length);
      while(i+1<text.length && !/[\s,\[\]{}:"]/.test(text[i+1]))i++;
    }
  }
}
export async function readFileTextStrict(file) {
  if(!file || !Number.isSafeInteger(file.size) || file.size<0 || file.size>MAX_PROJECT_BYTES)throw new Error('Project exceeds the 25 MB import limit.');
  const bytes=await file.arrayBuffer();
  if(bytes.byteLength>MAX_PROJECT_BYTES)throw new Error('Project exceeds the 25 MB import limit.');
  try { return new TextDecoder('utf-8',{fatal:true}).decode(bytes); }
  catch { throw new Error('File is not valid UTF-8.'); }
}
export function readJsonFile(file, options) {
  return readValidatedFile(file,new URL('./json-worker.js',import.meta.url),options);
}
function isRecord(value) {
  return value !== null && typeof value === 'object' && (Object.getPrototypeOf(value) === Object.prototype || Object.getPrototypeOf(value) === null);
}
export function validateJsonTree(root, maxValues=500000, maxStringBytes=MAX_PROJECT_BYTES) {
  const stack = [[root,0]], seen = new Set();
  let nodes = 0, stringBytes = 0;
  while (stack.length) {
    const [value,depth] = stack.pop();
    if (++nodes > maxValues || depth > 64) throw new Error(`JSON is too complex (maximum 64 levels and ${maxValues.toLocaleString('en-US')} values).`);
    if (typeof value === 'string') { stringBytes += new TextEncoder().encode(value).byteLength; }
    else if (typeof value === 'number') {
      if (!Number.isFinite(value)) throw new Error('Invalid project number.');
      // JSON.parse rounds larger integers before schema or certificate checks.
      // Reject them rather than silently verifying different imported data.
      if (Number.isInteger(value) && !Number.isSafeInteger(value)) throw new Error('Project integer exceeds the exact JavaScript range.');
    }
    else if (value !== null && typeof value === 'object') {
      if (seen.has(value)) throw new Error('Repeated or cyclic project object.');
      seen.add(value);
      if (!Array.isArray(value) && !isRecord(value)) throw new Error('Project contains a non-JSON object.');
      const keys = Object.keys(value);
      if (keys.length > maxValues - nodes - stack.length) throw new Error('Project has too many values.');
      for (const key of keys) {
        if (['__proto__','constructor','prototype'].includes(key)) throw new Error('Unsafe project property.');
        const property = Object.getOwnPropertyDescriptor(value,key);
        if (!property || !Object.hasOwn(property,'value')) throw new Error('Project contains an accessor.');
        stringBytes += new TextEncoder().encode(key).byteLength;
        stack.push([property.value,depth+1]);
      }
    } else if (value !== null && typeof value !== 'boolean') throw new Error('Project contains a non-JSON value.');
    if (stringBytes > maxStringBytes) throw new Error(maxStringBytes===MAX_PROJECT_BYTES?'Project exceeds the 25 MB import limit.':'JSON string bytes exceed the configured import limit.');
  }
}

// Parse off the UI thread. Untrusted files never replace current state until validated.
export function readProjectFile(file, {signal} = {}) {
  return readValidatedFile(file, new URL('./project-worker.js?v=census-acquire-zip-v1',import.meta.url), {signal});
}
export function readValidatedFile(file, workerUrl, {signal,maxBytes=MAX_PROJECT_BYTES,timeoutMs=10000} = {}) {
  if(!Number.isSafeInteger(maxBytes)||maxBytes<1||maxBytes>128*1024*1024||!Number.isSafeInteger(timeoutMs)||timeoutMs<1||timeoutMs>60000)return Promise.reject(new Error('Invalid import limits.'));
  if (!file || !Number.isSafeInteger(file.size) || file.size < 0 || file.size > maxBytes) return Promise.reject(new Error(maxBytes===MAX_PROJECT_BYTES?'Project exceeds the 25 MB import limit.':'File exceeds the configured import limit.'));
  if (signal?.aborted) return Promise.reject(new Error('Project import cancelled. Current project was kept.'));
  return new Promise((resolve,reject) => {
    const worker = new Worker(workerUrl,{type:'module'});
    const timer = setTimeout(() => finish(new Error('Project import timed out. Current project was kept.')),timeoutMs);
    let settled=false;
    function abort() { finish(new Error('Project import cancelled. Current project was kept.')); }
    function finish(error,project) {
      if(settled)return;settled=true;
      clearTimeout(timer);signal?.removeEventListener('abort',abort);worker.terminate();error ? reject(error) : resolve(project);
    }
    signal?.addEventListener('abort',abort,{once:true});
    worker.onmessage = ({data}) => {
      if(data?.ok === true && Object.hasOwn(data,'project')) finish(null,data.project);
      else finish(new Error(typeof data?.error === 'string' ? data.error : 'Project import failed. Current project was kept.'));
    };
    worker.onerror = () => finish(new Error('Project import failed. Current project was kept.'));
    worker.onmessageerror = () => finish(new Error('Project import failed. Current project was kept.'));
    try { worker.postMessage(file); } catch { finish(new Error('Project import failed. Current project was kept.')); }
  });
}
export async function writeAutosave(project) {
  return withStore('readwrite', store => store.put(validateProject(project), 'current'));
}
export async function readAutosave() {
  const project = await withStore('readonly', store => store.get('current'));
  return project ? validateProject(project) : null;
}
function withStore(mode, action) {
  return new Promise((resolve, reject) => {
    const open = indexedDB.open('bisect-workbench', 1);
    open.onupgradeneeded = () => open.result.createObjectStore('projects');
    open.onerror = () => reject(open.error);
    open.onsuccess = () => {
      const db = open.result;
      const transaction = db.transaction('projects', mode);
      const request = action(transaction.objectStore('projects'));
      transaction.oncomplete = () => { db.close(); resolve(request.result); };
      transaction.onabort = transaction.onerror = () => { db.close(); reject(transaction.error || request.error); };
    };
  });
}
