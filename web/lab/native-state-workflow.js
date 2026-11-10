import {nativeStateNameEffects,validateNativeStateNameEffects,stateWorkflowExportLabel} from './native-state-name-effects.js';
import {inspectStateCommandWithIdentity} from './native-state-identity-preview.js';
import {validateStateDiagnosticSettings} from './native-state-diagnostics.js';
import {validateNativeTargetActions,requireNativeStateTargetAuthority} from './native-state-target-authority.js';
import {stateCommandArguments,statePreparedFlagRefusals,validateStateCommandInputs} from './state-command-files.js';
import {NATIVE_STATE_BOOLEAN_FIELDS} from './native-state-boundary.js';
import {nativeOutputComponent,validateNativeStateOutputInputs,validateStreamNativeOutputSettings} from './native-state-output.js';
import {STATE_GRAPH_DEFAULTS} from './state-graph-files.js';
import {validateTuiRunWorkflowSettings} from './state-workflow-files.js';
import {nativeEngineInputRequest} from './native-adjacency-files.js';
import {validateNativeReportSnapshot,recheckNativeReportSnapshot} from './native-report-source-inputs.js';
const mapped=['version','output_dir','dpi','label','manifest','adjacency','position','debug','time_partition','state_name','population_source'];
const digest=async text=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(text))),b=>b.toString(16).padStart(2,'0')).join('');
const keys=(v,names)=>v&&typeof v==='object'&&!Array.isArray(v)&&Object.keys(v).sort().join()===names.split(',').sort().join();
export function hasStateWorkflowChoices(review,{reportSources}={}){if(review?.schema_version!=='bisect-native-state-command-v1'||review.status!=='parsed'||review.profile_filter?.allowed!==true||!Array.isArray(review.fields))return false;if(reportSources||review.fields.some(f=>f.id==='state_name'&&f.source==='command-line')){try{if(reportSources){validateNativeReportSnapshot(reportSources);if(JSON.stringify(review.arguments)!==JSON.stringify(reportSources.request.arguments))return false;}validateNativeStateNameEffects({schema_version:'bisect-native-state-name-effects-v1',preview:review.browser_identity_preview,auxiliary_sha256:null});}catch{return false;}}return !!reportSources||review.fields.some(f=>mapped.includes(f.id)&&f.source==='command-line');}
export function validateStateWorkflowBinding(v){
 if(!keys(v,'schema_version,source_command,engine_command,settings,source_sha256,engine_sha256'+(Object.hasOwn(v??{},'source_override')?',source_override':'')+(Object.hasOwn(v??{},'target_actions')?',target_actions':'')+(Object.hasOwn(v??{},'diagnostics')?',diagnostics':'')+(Object.hasOwn(v??{},'name_effects')?',name_effects':'')+(Object.hasOwn(v??{},'report_sources')?',report_sources':''))||v.schema_version!=='bisect-native-state-workflow-binding-v1'||!/^[a-f0-9]{64}$/.test(v.source_sha256)||!/^[a-f0-9]{64}$/.test(v.engine_sha256))throw new Error('Invalid native State workflow binding.');
 if(Object.hasOwn(v,'source_override')&&(!keys(v.source_override,'adjacency_path_intent,preparation_sha256')||typeof v.source_override.adjacency_path_intent!=='string'||!v.source_override.adjacency_path_intent||v.source_override.adjacency_path_intent.length>4096||v.source_override.adjacency_path_intent.includes('\0')||!/^[a-f0-9]{64}$/.test(v.source_override.preparation_sha256)))throw new Error('Invalid native State selected adjacency binding.');
 if(Object.hasOwn(v,'target_actions'))validateNativeTargetActions(v.target_actions);
 if(Object.hasOwn(v,'diagnostics'))validateStateDiagnosticSettings(v.diagnostics);
 if(Object.hasOwn(v,'name_effects'))validateNativeStateNameEffects(v.name_effects);
 if(Object.hasOwn(v,'report_sources')){validateNativeReportSnapshot(v.report_sources);if(JSON.stringify(stateCommandArguments(v.source_command))!==JSON.stringify(v.report_sources.request.arguments)||!v.name_effects||JSON.stringify(v.name_effects.preview.identity)!==JSON.stringify(v.report_sources.receipt.identity))throw Error('Report sources differ from the bound original State command/name review.');}
 validateStateCommandInputs(v.source_command);validateStateCommandInputs(v.engine_command);const s=v.settings;
 if(!keys(s,'version,output_dir_intent,label,dpi,manifest_requested,layout')||!nativeOutputComponent(s.version)||s.output_dir_intent!==null&&(typeof s.output_dir_intent!=='string'||new TextEncoder().encode(s.output_dir_intent).length>4096)||s.label!==null&&(!nativeOutputComponent(s.label)||['runs','analysis','reports','.bisect','configs'].includes(s.label))||!['72','100','150','200','300'].includes(s.dpi)||typeof s.manifest_requested!=='boolean'||s.layout!==(s.label===null?'state':'labeled')&&!(v.source_override&&s.label===null&&s.layout==='labeled'))throw new Error('Choose portable State output version/label and native DPI; paths remain recorded intent.');
 return v;
}
export async function createStateWorkflowBinding(command,review,{sourcePreparation,targetActions,identityPreview,sourceAuxiliary,reportSources}={}){
 validateStateCommandInputs(command);
 if(review?.schema_version!=='bisect-native-state-command-v1'||review.status!=='parsed'||review.profile_filter?.allowed!==true||JSON.stringify(review.arguments)!==JSON.stringify(stateCommandArguments(command)))throw new Error('Inspect these native State flags before applying workflow settings.');
 if(reportSources){validateNativeReportSnapshot(reportSources);if(JSON.stringify(stateCommandArguments(command))!==JSON.stringify(reportSources.request.arguments))throw Error('Load a report-source snapshot for these exact original State flags.');}
 const fields=review.fields,selectedMapped=[...mapped.filter(id=>id!=='population_source'||reportSources),...(targetActions?['reset','force']:[])];if(targetActions){validateNativeTargetActions(targetActions);for(const key of ['reset','force'])if((fields.find(f=>f.id===key)?.values?.[0]==='true')!==targetActions[key])throw new Error('Native target actions differ from original flags.');}if(!reportSources&&!fields.some(f=>selectedMapped.includes(f.id)&&f.source==='command-line'))throw new Error('No explicit State output/workflow flags to apply; --population-source requires a selected report-source snapshot.');
 const value=id=>fields.find(f=>f.id===id)?.values?.[0]??null;
 let source_override,custom=false;
 if(fields.some(f=>f.id==='adjacency'&&f.source==='command-line')){
  const preparation=structuredClone(sourcePreparation);if(preparation?.operation!=='prepare-native-engine-input'||!preparation.input)throw new Error('Select the native adjacency file and companions before applying --adjacency. No path is opened.');
  const {sources,...settings}=preparation.input;nativeEngineInputRequest(sources,settings);
  const resolution=({'tract':'tract','block-group':'block_group','block':'block'})[settings.unit_kind??'tract'];
  custom=['precinct','imported'].includes(settings.unit_kind);
  if(value('state')?.toUpperCase()!==settings.state||value('year')!==settings.year||!custom&&value('resolution')!==resolution||!['congressional','house','senate'].includes(value('chamber')))throw new Error('State --adjacency flags must match the selected native state/year and chamber; Census sources also require matching resolution.');
  const path=value('adjacency');if(path?.split(/[\\/]/).at(-1)!==sources.adjacency.name)throw new Error('Select the exact filename named by --adjacency; browser paths are recorded intent only.');
  source_override={adjacency_path_intent:path,preparation_sha256:await digest(JSON.stringify(preparation))};
 }
 const diagnostics=fields.some(f=>['position','debug','time_partition'].includes(f.id)&&f.source==='command-line')?validateStateDiagnosticSettings({schema_version:'bisect-native-state-diagnostic-settings-v1',position:Number(value('position')),debug_requested:value('debug')==='true',debug_behavior:'native-unused',...(value('time_partition')==='true'?{time_partition_requested:true}:{})}):null;
 const label=value('label');
 // An unlabeled custom target action must use the native State directory and
 // default export label, even without a separately supplied --state-name.
 const name_effects=reportSources||custom&&targetActions&&label===null||fields.some(f=>f.id==='state_name'&&f.source==='command-line')?await nativeStateNameEffects(identityPreview??review.browser_identity_preview,sourceAuxiliary):null;
 if(name_effects){const i=name_effects.preview.identity;if(i.requested_state_name!==value('state_name')||i.state_code!==value('state')?.toUpperCase()||i.effective_year!==value('year')||i.chamber!==value('chamber')||i.requested_label!==value('label'))throw Error('Native identity differs from the original State flags.');}
 const arguments_=[];
 for(const f of fields){if(f.source!=='command-line'||selectedMapped.includes(f.id))continue;if(!f.long||!Array.isArray(f.values))throw new Error('Invalid native State flag declaration.');
  if(NATIVE_STATE_BOOLEAN_FIELDS.includes(f.id)){if(f.values.length!==1||f.values[0]!=='true')throw new Error('Unsupported native State boolean action.');arguments_.push('--'+f.long);}
  else{if(!f.values.length)throw new Error('Native State value is missing.');for(const raw of f.values)arguments_.push('--'+f.long+'='+raw);}
 }
 const engine_command={arguments_json:JSON.stringify(arguments_)},settings={version:value('version'),output_dir_intent:value('output_dir'),label,dpi:value('dpi'),manifest_requested:value('manifest')==='true'||label!==null,layout:name_effects?name_effects.preview.identity.layout:label===null&&!custom?'state':'labeled'};
 return structuredClone(validateStateWorkflowBinding({schema_version:'bisect-native-state-workflow-binding-v1',source_command:command,engine_command,settings,source_sha256:await digest(command.arguments_json),engine_sha256:await digest(engine_command.arguments_json),...(source_override?{source_override}:{}),...(targetActions?{target_actions:structuredClone(targetActions)}:{}),...(diagnostics?{diagnostics}:{}),...(name_effects?{name_effects}:{}),...(reportSources?{report_sources:structuredClone(reportSources)}:{})}));
}
export async function recheckStateWorkflowBinding(engine,binding,command,{forExecution=false,sourcePreparation,nativeTargetAuthority,sourceAuxiliary}={}){
 validateStateWorkflowBinding(binding);if(command&&JSON.stringify(stateCommandArguments(command))!==JSON.stringify(stateCommandArguments(binding.engine_command)))throw new Error('State engine flags changed after workflow handoff. Apply again or detach the earlier workflow binding.');
 const source=await engine.execute({operation:'inspect-native-state-command',arguments:stateCommandArguments(binding.source_command)});
 const identityPreview=binding.name_effects?(await inspectStateCommandWithIdentity(engine,stateCommandArguments(binding.source_command))).browser_identity_preview:undefined;
 if(binding.report_sources)await recheckNativeReportSnapshot(engine,binding.report_sources,binding.source_command);
 const fresh=await createStateWorkflowBinding(binding.source_command,source,{sourcePreparation,targetActions:binding.target_actions,identityPreview,sourceAuxiliary,reportSources:binding.report_sources});if(JSON.stringify(fresh)!==JSON.stringify(binding))throw new Error('State workflow settings or original flags changed, or selected adjacency differs. Apply the handoff again.');
 const after=await engine.execute({operation:'inspect-native-state-command',arguments:stateCommandArguments(binding.engine_command)});
 if(after.status!=='parsed'||JSON.stringify(after.effective_algorithm)!==JSON.stringify(source.effective_algorithm)||JSON.stringify(after.profile_filter)!==JSON.stringify(source.profile_filter))throw new Error('State workflow handoff changed native algorithm or profile policy. No partition executed.');
 for(const f of source.fields.filter(f=>(!mapped.includes(f.id)||f.id==='population_source'&&!binding.report_sources)&&!(binding.target_actions&&['reset','force'].includes(f.id)))){const other=after.fields.find(g=>g.id===f.id);if(JSON.stringify([f.source,f.values])!==JSON.stringify([other?.source,other?.values]))throw new Error('State workflow handoff changed '+f.id+'. No partition executed.');}
 if(forExecution){if(binding.target_actions)await requireNativeStateTargetAuthority(nativeTargetAuthority,binding);const refused=statePreparedFlagRefusals(after);if(refused.length)throw new Error('Remaining State flags need another workflow before source preparation or acquisition: '+refused.join(', '));}
 return binding;
}
export function stateWorkflowSettingsPatch(binding,files){
 const s=validateStateWorkflowBinding(binding).settings,patch={stateWorkflowBinding:structuredClone(binding),stateCommandInputs:structuredClone(binding.engine_command),nativeStateOutputInputs:validateNativeStateOutputInputs({source:null,version:s.version,layout:s.layout}),streamNativeOutputSettings:validateStreamNativeOutputSettings({enabled:s.manifest_requested||!!files.streamNativeOutputSettings?.enabled,version:s.version,layout:s.layout}),tuiRunWorkflowSettings:validateTuiRunWorkflowSettings({...files.tuiRunWorkflowSettings??{analysis:false,map:false,report:false,dpi:'150'},dpi:s.dpi})};
 if(binding.source_override)patch.nativeStateSourceSettings={enabled:true,command_kind:'state'};
 if(binding.report_sources)patch.stateReportSourceSnapshot=null;
 const exportLabel=stateWorkflowExportLabel(binding);
 if(exportLabel!==null){const graph=files.stateGraphInputs??STATE_GRAPH_DEFAULTS,chamber=binding.report_sources?.receipt.identity.chamber;patch.stateGraphInputs={...graph,settings:{...graph.settings,label:exportLabel,...(chamber?{chamber}:{})}};if(files.preparedEngineInputs)patch.preparedEngineInputs={...files.preparedEngineInputs,settings:{...files.preparedEngineInputs.settings,label:exportLabel,...(chamber?{chamber}:{})}};}
 return structuredClone(patch);
}
export function selectedStateWorkflowBinding(files,command){
 if(files.stateReportSourceSnapshot!=null)throw Error('Apply the newly selected report-source snapshot to the original State flags before queueing.');
 const v=files.stateWorkflowBinding;if(!v)return undefined;validateStateWorkflowBinding(v);if(JSON.stringify(stateCommandArguments(command))!==JSON.stringify(stateCommandArguments(v.engine_command)))throw new Error('State flags changed after workflow handoff. Reapply or detach the earlier binding before queueing.');return structuredClone(v);
}
