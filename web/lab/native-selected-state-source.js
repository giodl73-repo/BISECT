import {createNativeStateSourceJob,nativeStatePreparationScope} from './native-state-sources.js';
import {validateNativeStateJob,executeNativeStateJob} from './native-state-job.js';
import {stateCommandRequest,statePreparedFlagRefusals} from './state-command-files.js';
import {chamberCountRequest} from './chamber-count-files.js';

export const NATIVE_SELECTED_STATE_SOURCE_DEFAULTS={kind:'native'};
export function validateNativeSelectedStateSourceSettings(v){
 if(!v||Object.keys(v).join()!=='kind'||!['native','census','tract'].includes(v.kind))throw new Error('Choose selected native binary, Census or tract State sources.');return v;
}
export async function nativeSelectedStateSourceRequest(files){
 const snapshot=structuredClone(files),settings=validateNativeSelectedStateSourceSettings(snapshot.nativeSelectedStateSourceSettings??NATIVE_SELECTED_STATE_SOURCE_DEFAULTS);
 if(!snapshot.nativeStateSourceSettings?.enabled||['run','states'].includes(snapshot.nativeStateSourceSettings.command_kind))throw new Error('Single selected-source execution requires native State flags. Run/States retain their own queue policies.');
 const input=await createNativeStateSourceJob(snapshot,settings.kind,{selectedScope:true});validateNativeStateJob(input);
 return {operation:'run-native-selected-state-source',input};
}
export async function reviewNativeSelectedStateSource(engine,value){
 const job=structuredClone(validateNativeStateJob(value)),recipe=job.native_state;
 if(!recipe.source_inputs||recipe.command_kind)throw new Error('Select one original State source recipe for this Run.');
 const digest=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(JSON.stringify(recipe)))),b=>b.toString(16).padStart(2,'0')).join('');
 if(digest!==job.source_sha256)throw new Error('Selected State flags or source bytes changed. Select them again before Run.');
 if(recipe.source_scope_policy==='match-native-adjacency-flags')return {job,custom:true};
 const scope=nativeStatePreparationScope(recipe.source_inputs.preparation),review=engine.execute(stateCommandRequest(recipe.command)),field=id=>review.fields?.find(f=>f.id===id)?.values?.[0]??null;
 if(review.status!=='parsed'||review.profile_filter?.allowed!==true||statePreparedFlagRefusals(review).length)throw new Error('Review or apply the State source/output flags before this single-state Run. No source prepared or partition executed.');
 if(review.state?.toUpperCase()!==scope.state||review.year!==scope.year||review.resolution!==scope.resolution||field('chamber')!==job.scope.chamber)throw new Error('Single State flags must match the selected source state, year, resolution and chamber. This Run never changes the selected scope.');
 let count;
 if(field('districts')!==null){const raw=field('districts');if(!/^[0-9]+$/.test(raw))throw new Error('Native State district count must be an integer.');count={districts:Number(raw),allocation_source:'explicit-native-flag'};}
 else{count=engine.execute(chamberCountRequest({state:scope.state,year:scope.year,chamber:job.scope.chamber,policy:null,manifest:null}));if(count.schema_version!=='bisect-native-chamber-count-v1'||count.execution_performed!==false||count.state!==scope.state||count.year!==scope.year||count.chamber!==job.scope.chamber)throw new Error('Native chamber allocation differs from the selected State scope. No source prepared or partition executed.');}
 if(!Number.isSafeInteger(count.districts)||count.districts<1||count.districts>500||job.scope.districts!==null&&job.scope.districts!==count.districts)throw new Error('Selected State district count must match the native flags or native chamber allocation and be within 1–500.');
 job.scope.districts=count.districts;
 return {job,scope,count,custom:false};
}
export async function executeNativeSelectedStateSource(engine,value,{onProgress,nativeTargetAuthority}={}){
 const {job,scope,count,custom}=await reviewNativeSelectedStateSource(engine,value),recipe=job.native_state;
 if(recipe.source_scope_policy==='match-native-adjacency-flags'){
  const output=await executeNativeStateJob(engine,job,{onProgress,nativeTargetAuthority}),scope=output.result.scope;
  return {...output.result,source_preparation:output.source_preparation,single_source_selection:{schema_version:'bisect-native-selected-state-source-v1',scope:structuredClone(scope),resolution:null,unit_kind:recipe.source_inputs.preparation.input.unit_kind,district_count:structuredClone(output.result.native_state.compilation.district_count),scope_changed:false,acquisition_performed:false}};
 }
 const output=await executeNativeStateJob(engine,job,{onProgress,nativeTargetAuthority});
 // Original bytes remain in the recipe. Avoid retaining duplicate graph and
 // display JSON strings alongside the already accepted request and geometry.
 return {...output.result,source_preparation:Object.fromEntries(Object.entries(output.source_preparation).filter(([key])=>!['graph_json','geometry_json'].includes(key))),single_source_selection:{schema_version:'bisect-native-selected-state-source-v1',scope:structuredClone(job.scope),resolution:scope.resolution,district_count:count,scope_changed:false,acquisition_performed:false}};
}
export function nativeCommandOutputChoices(run){
 const recipe=(run.native_state??run.native_states??run.native_run)?.recipe,review=(run.native_state??run.native_states??run.native_run)?.compilation.native_command;
 const value=id=>review?.fields?.find(f=>f.id===id)?.values?.[0];
 return {version:recipe?.workflow_binding?.settings.version??review?.version??value('version')??'v1',dpi:Number(recipe?.workflow_binding?.settings.dpi??value('dpi')??150),layout:recipe?.workflow_binding?.settings.layout??(recipe?.source_scope_policy==='match-native-adjacency-flags'?'labeled':'state')};
}
