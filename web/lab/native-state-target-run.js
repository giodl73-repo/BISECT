import {stateWorkflowExportLabel} from './native-state-name-effects.js';
import {inspectStateCommandWithIdentity} from './native-state-identity-preview.js';
import {stateCommandArguments,statePreparedFlagRefusals} from './state-command-files.js';
import {createStateWorkflowBinding,recheckStateWorkflowBinding} from './native-state-workflow.js';
import {nativeTargetHash,authorizeNativeStateTarget,requireNativeStateTargetAuthority} from './native-state-target-authority.js';
import {validateNativeStateJob} from './native-state-job.js';
import {executeNativeSelectedStateSource,reviewNativeSelectedStateSource} from './native-selected-state-source.js';
import {nativeOutputComponent} from './native-state-output.js';
import {chamberCountRequest} from './chamber-count-files.js';
export async function prepareNativeStateTargetRun(engine,value){
 const original=structuredClone(validateNativeStateJob(value)),recipe=original.native_state;
 if(await nativeTargetHash(recipe)!==original.source_sha256)throw new Error('State source recipe changed before target preflight. No files touched.');
 if(!recipe.source_inputs||recipe.command_kind||original.acquisition)throw new Error('Reviewed target actions require a single selected State source.');
 const command=recipe.workflow_binding?.source_command??recipe.command;
 const review=engine.execute({operation:'inspect-native-state-command',arguments:stateCommandArguments(command)});
 if(review.status!=='parsed'||review.profile_filter?.allowed!==true)throw new Error('Review valid State flags/profile before selecting a target.');
 const field=id=>review.fields?.find(f=>f.id===id)?.values?.[0]??null,actions={reset:field('reset')==='true',force:field('force')==='true'};
 if(!actions.reset&&!actions.force)return null;
 if(recipe.workflow_binding)await recheckStateWorkflowBinding(engine,recipe.workflow_binding,recipe.command,{sourcePreparation:recipe.source_inputs.preparation,sourceAuxiliary:recipe.source_inputs.auxiliary});
 const customUnlabeled=recipe.source_scope_policy==='match-native-adjacency-flags'&&field('label')===null;
 const identityPreview=customUnlabeled||recipe.workflow_binding?.report_sources||review.fields.some(f=>f.id==='state_name'&&f.source==='command-line')?(await inspectStateCommandWithIdentity(engine,stateCommandArguments(command))).browser_identity_preview:undefined;
 const binding=await createStateWorkflowBinding(command,review,{sourcePreparation:recipe.source_inputs.preparation,targetActions:actions,identityPreview,sourceAuxiliary:recipe.source_inputs.auxiliary,reportSources:recipe.workflow_binding?.report_sources});
 const derived=structuredClone(original);derived.native_state.command=binding.engine_command;derived.native_state.workflow_binding=binding;
 if(stateWorkflowExportLabel(binding)!==null)derived.native_state.source_inputs.settings.label=stateWorkflowExportLabel(binding);
 derived.source_sha256=await nativeTargetHash(derived.native_state);
 validateNativeStateJob(derived);
 if(field('districts')!==null&&(!Number.isSafeInteger(Number(field('districts')))||Number(field('districts'))<1||Number(field('districts'))>500))throw new Error('Selected State target Runs support 1–500 districts.');
 const selected=await reviewNativeSelectedStateSource(engine,derived);derived.scope=selected.job.scope;
 const districts=selected.count?.districts??(field('districts')!==null?Number(field('districts')):engine.execute(chamberCountRequest({state:original.scope.state,year:original.scope.year,chamber:original.scope.chamber,policy:null,manifest:null})).districts);
 if(!Number.isSafeInteger(districts)||districts<1||districts>500||original.scope.districts!==null&&original.scope.districts!==districts)throw new Error('Selected State district count must match the native flags or allocation and be within 1–500. No files touched.');
 derived.scope.districts=districts;
 // Match prepare_resolved_with_override in native_state_run.rs: these four
 // split methods and a one-district request do not consume a METIS backend.
 const usesMetis=districts>1&&!['moving-knife','spectral','regionalization','flow-construction'].includes(review.effective_algorithm?.structure),backend=review.effective_algorithm?.metis?.engine;
 if(usesMetis&&backend==='c-ffi'&&!engine.runtimeArtifacts?.metis_sidecar)throw new Error('Load the verified C runtime or select metis-core before reviewing destructive target actions.');
 if(usesMetis&&backend==='gpmetis')throw new Error('The gpmetis stub cannot run after target actions. Select an available engine first.');
 const engineReview=engine.execute({operation:'inspect-native-state-command',arguments:stateCommandArguments(binding.engine_command)});
 if(statePreparedFlagRefusals(engineReview).length)throw new Error('Other State flags still require their workflow: '+statePreparedFlagRefusals(engineReview).join(', '));
 if(review.state?.toUpperCase()!==original.scope.state||review.year!==original.scope.year||field('chamber')!==original.scope.chamber)throw new Error('Original State flags differ from the selected source scope.');
 const version=binding.settings.version,label=binding.settings.label;
 if(!nativeOutputComponent(version)||label!==null&&!nativeOutputComponent(label))throw new Error('Choose a portable target version and label.');
 if(customUnlabeled&&(!binding.name_effects||binding.settings.layout!=='state'))throw new Error('Review the original native State name and folder before taking an unlabeled custom target action.');
 const empty={source_name:'native-before-run-target.json',folder_json:JSON.stringify({version,paths:[],manifests:{}}),force:actions.force};
 const scope=engine.execute(binding.name_effects?{operation:'inspect-native-named-state-run-target',version,arguments:stateCommandArguments(binding.source_command),input:empty}:{operation:label===null?'inspect-native-state-run-target':'inspect-native-run-target',version,year:original.scope.year,...(label===null?{state:original.scope.state}:{label}),input:empty});
 if(scope.schema_version!=='bisect-native-run-target-v1'||!scope.target_namespace?.startsWith('outputs/'))throw new Error('Invalid native target scope.');
 const intent={schema_version:'bisect-native-state-target-intent-v1',version,year:original.scope.year,state:original.scope.state,label,layout:label===null?'state':'labeled',...(label===null?{state_slug:scope.state_slug}:{}),target_namespace:scope.target_namespace.slice(8),...(binding.name_effects?{state_name_command:binding.source_command,native_name_effects:binding.name_effects}:{}),...actions,request_sha256:await nativeTargetHash(original),derived_source_sha256:derived.source_sha256,binding_sha256:await nativeTargetHash(binding),runtime_artifact:engine.artifactIdentity,output_dir_intent:binding.settings.output_dir_intent,source_arguments_json:command.arguments_json,engine_arguments_json:binding.engine_command.arguments_json};
 return {original,derived,binding,intent};
}
export async function executeReviewedNativeSelectedStateSource(engine,input,{performTargetAction,onProgress}={}){
 const prepared=await prepareNativeStateTargetRun(engine,input);if(!prepared)return executeNativeSelectedStateSource(engine,input,{onProgress});
 const authority=await authorizeNativeStateTarget(prepared.intent,performTargetAction);
 const receipt=await requireNativeStateTargetAuthority(authority,prepared.binding);
 let result;try{result=await executeNativeSelectedStateSource(engine,prepared.derived,{onProgress,nativeTargetAuthority:authority});}
 catch(error){error.native_target_actions={intent:prepared.intent,receipt,source_request_sha256:prepared.intent.request_sha256,action_completed:true};throw error;}
 result.native_state.target_actions={intent:prepared.intent,receipt,original_source_sha256:prepared.original.source_sha256,original_arguments_json:prepared.intent.source_arguments_json,engine_arguments_json:prepared.intent.engine_arguments_json,action_completed:true};return result;
}
