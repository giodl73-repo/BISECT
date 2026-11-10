import {validatePackageVerificationInputs,packageVerificationRequest} from './package-verification-files.js';
import {validateEngineReplayInputs,engineReplayFromConfigured} from './engine-replay-files.js';
import {preparedEngineReplayInputs} from './prepared-engine-results.js';
import {parseBuildArchiveJson,parseSafeJson} from './project.js';
import {validateSourceFile,encodeBytes} from './package-files.js';
import {nativeOutputHash} from './native-state-output.js';
import {nativeTargetHash} from './native-state-target-authority.js';
import {decodeNativeWorkspaceArchive} from './native-workspace-source.js';

export const NATIVE_OUTPUT_VERIFICATION_DEFAULTS={mode:'inspect',minSimilarity:1};
export function nativeOutputVerificationSummary(result,fresh=false){
 const stages=result?.stages&&typeof result.stages==='object'?result.stages:{};
 const lines=[(fresh?'Computed on this device: ':'Saved record · unverified on Open: ')+String(result?.status??'unavailable'),
  'Recorded accepted seed: '+(stages.replay_source?.recorded_seed??'not-recorded'),
  'Original graph sources: '+(stages.original_sources?.status??'not-run'),
  'Manifest/run binding: '+(stages.manifest_and_replay?.manifest_to_run_binding??'not-run'),
  'Partition performed: '+(result?.execution_performed===true)+'; completed: '+(result?.execution_completed===true)+'; replay matched: '+(result?.replay_matched===true),
  'Requested similarity threshold: '+(result?.requested_min_similarity??'not-recorded'),
  'Historical producer claim: '+(stages.recorded_output_runtime?.status??'not-run')];
 const failed=Object.entries(stages).find(([,stage])=>stage&&typeof stage==='object'&&stage.error);
 if(failed)lines.push('Detail ('+failed[0].replaceAll('_',' ')+'): '+String(failed[1].error).slice(0,1024));
 lines.push('Source authenticity and historical execution are not authenticated.');return lines.join('\n');
}
export function validateNativeOutputVerificationSettings(value){if(!value||Object.keys(value).sort().join()!=='minSimilarity,mode'||!['inspect','replay'].includes(value.mode)||!Number.isFinite(value.minSimilarity))throw new Error('Choose source inspection or explicit replay and a finite native similarity threshold.');return value;}
const bytes=source=>Uint8Array.from(atob(source.base64),c=>c.charCodeAt(0)),text=source=>new TextDecoder('utf-8',{fatal:true}).decode(bytes(source));
const source=(name,value)=>({name,base64:encodeBytes(new TextEncoder().encode(value))});
export async function nativeOutputPackageFromArchive(data,{signal}={}){
 const decoded=await decodeNativeWorkspaceArchive(data,{signal,layout:'output-package'}),manifests=decoded.files.filter(f=>f.path==='manifest.json'||f.path.endsWith('/manifest.json'));
 if(manifests.length!==1)throw new Error('Select an archive containing exactly one native output target manifest.');
 const prefix=manifests[0].path.slice(0,-'manifest.json'.length);if(decoded.files.some(f=>!f.path.startsWith(prefix)))throw new Error('Output archive contains files outside its single target.');
 const files=Object.fromEntries(decoded.files.filter(f=>f!==manifests[0]).map(f=>[f.path.slice(prefix.length),{name:f.path.split('/').at(-1),base64:encodeBytes(f.bytes)}]));
 const packageInputs={manifest:{name:'manifest.json',base64:encodeBytes(manifests[0].bytes)},files};validatePackageVerificationInputs(packageInputs);return {packageInputs,archive_sha256:decoded.archive_sha256,prefix};
}
export function nativeOutputVerificationRequest(files){
 const package_inputs=structuredClone(files.packageVerificationInputs??{manifest:null,files:{}}),settings=structuredClone(validateNativeOutputVerificationSettings(files.nativeOutputVerificationSettings??NATIVE_OUTPUT_VERIFICATION_DEFAULTS));packageVerificationRequest(package_inputs,'verify-manifest-prechecks');
 const archive=files.nativeOutputVerificationArchive??null;if(archive)validateSourceFile(archive,{maxBytes:64*1024*1024});
 const selected_run=files.engineReplayInputs?.run??null;if(selected_run)validateEngineReplayInputs({run:selected_run,original:null,minSimilarity:settings.minSimilarity});
 return {operation:'verify-native-output-workspace',input:structuredClone({package_inputs,settings,archive,selected_run})};
}
function equivalentInventory(left,right){if(left.manifest.base64!==right.manifest.base64)return false;const a=Object.keys(left.files).sort(),b=Object.keys(right.files).sort();return JSON.stringify(a)===JSON.stringify(b)&&a.every(p=>left.files[p].base64===right.files[p].base64);}
function originalAssignments(raw,ids,k){
 const original=parseSafeJson(raw);if(!original||Array.isArray(original)||typeof original!=='object')throw new Error('Native assignment file must contain an object.');
 const keys=Object.keys(original);if(!keys.length)return null;const universe=new Set(ids);if(!Array.isArray(ids)||!ids.length||universe.size!==ids.length||ids.some(id=>typeof id!=='string')||keys.length!==ids.length||!Number.isSafeInteger(k)||k<1||k>500)throw new Error('Select an exact complete unit order and supported district count for the native assignment file.');
 const byUnit=keys.every(id=>universe.has(id)),byIndex=keys.every(id=>/^(0|[1-9][0-9]*)$/.test(id)&&Number(id)<ids.length);
 if(!byUnit&&!byIndex)throw new Error('Native assignments differ from the selected unit universe or zero-based index convention.');
 if(byUnit&&byIndex&&ids.some((id,i)=>id!==String(i)))throw new Error('Ambiguous native assignment keys: numeric unit IDs overlap zero-based indices with a different ordering. Supply an explicit unambiguous assignment convention.');
 const values=ids.map((id,i)=>original[byUnit?id:String(i)]);if(values.some(v=>!Number.isSafeInteger(v)||v<1||v>k))throw new Error('Native original assignment district is outside 1..=district count.');
 return {assignments:Object.fromEntries(ids.map((id,i)=>[id,values[i]])),convention:byUnit?'unit-ids':'zero-based-unit-indices'};
}
export async function executeNativeOutputVerification(engine,input,{onProgress=()=>{}}={}){
 if(!input||Object.keys(input).sort().join()!=='archive,package_inputs,selected_run,settings')throw new Error('Invalid native output verification input.');
 const request=nativeOutputVerificationRequest({packageVerificationInputs:input?.package_inputs,nativeOutputVerificationSettings:input?.settings,nativeOutputVerificationArchive:input?.archive,engineReplayInputs:{run:input?.selected_run}});input=request.input;
 const pkg=input.package_inputs,precheckRequest=packageVerificationRequest(pkg,'verify-manifest-prechecks'),stages={},result={schema_version:'bisect-native-output-verification-v1',status:'inspection-partial',mode:input.settings.mode,requested_min_similarity:input.settings.minSimilarity,stages,execution_performed:false,execution_completed:false,replay_matched:false,loaded_runtime_artifacts:engine.runtimeArtifacts??null,loaded_wasm:engine.artifactIdentity,source_authenticity_verified:false,filesystem_writes_performed:false,network_requests_performed:false,scope:'Explicit selected native target inspection/source regeneration and optional manifest-bound portable replay. Stage consistency is not authenticated historical execution, source publication, legal validity or certification. No reset, source acquisition or output installation is performed.'};
 const fail=(stage,error)=>{stages[stage]={status:'failed',error:String(error.message??error)};result.status='failed';return result;};
 const selected={manifest:await nativeOutputHash(bytes(pkg.manifest)),files:Object.fromEntries(await Promise.all(Object.entries(pkg.files).map(async([p,s])=>[p,await nativeOutputHash(bytes(s))])))};result.selected_sha256=selected;
 onProgress({stage:'native-output-prechecks',message:'Checking recorded manifest sidecars before source or replay work'});
 stages.prechecks=engine.execute(precheckRequest);if(stages.prechecks.native_exit_code!==0){result.status='precheck-failed';return result;}
 stages.producer=engine.execute({operation:'verify-run-manifest',source_name:pkg.manifest.name,manifest_json:precheckRequest.input.manifest_json,runtime_artifact:engine.artifactIdentity});
 if(pkg.files['browser/runtime-artifact.json'])try{const recorded=parseSafeJson(text(pkg.files['browser/runtime-artifact.json']));stages.recorded_output_runtime={status:'unverified-recorded-claim',recorded,original_execution_authenticated:false};}catch(error){stages.recorded_output_runtime={status:'invalid-recorded-claim',error:error.message};}else stages.recorded_output_runtime={status:'not-recorded'};
 if(input.archive)try{const decoded=await nativeOutputPackageFromArchive(bytes(input.archive));if(!equivalentInventory(pkg,decoded.packageInputs))throw new Error('Selected output files differ from the retained ZIP archive.');stages.archive={status:'verified',source_sha256:decoded.archive_sha256,prefix:decoded.prefix};}catch(error){return fail('archive',error);}else stages.archive={status:'not-selected'};
 let accepted=null,replayInputs,artifact,ids,sourceGraph=null;
 try{
  if(pkg.files['browser/accepted-run.json'])accepted=parseBuildArchiveJson(text(pkg.files['browser/accepted-run.json']));
  if(input.selected_run)replayInputs={run:input.selected_run,original:null,minSimilarity:input.settings.minSimilarity};
  else if(accepted){const prepared=accepted.schema_version==='bisect-prepared-state-run-v1'?accepted.prepared:accepted;replayInputs=accepted.schema_version==='bisect-tui-run-v1'?engineReplayFromConfigured(accepted.configured_run):await preparedEngineReplayInputs(prepared);}
  if(!replayInputs){stages.replay_source={status:'not-recorded',message:'Select a portable replay artifact or a browser output containing its accepted Run receipt.'};return result;}
  artifact=parseBuildArchiveJson(text(replayInputs.run));const seed=artifact.kind?.startsWith('engine')?artifact.request?.options?.seed:artifact.kind?.startsWith('multiscale')?artifact.input?.request?.options?.seed:undefined;stages.replay_source={status:'selected',sha256:await nativeOutputHash(bytes(replayInputs.run)),origin:input.selected_run?'explicit-file':'selected-browser-run',recorded_seed:typeof seed==='string'||seed===null?seed:Number.isSafeInteger(seed)?String(seed):'See exact original replay artifact'};
  const index=pkg.files['unit-index.json']?parseSafeJson(text(pkg.files['unit-index.json'])):null;
  ids=index?.unit_ids??(artifact.kind?.startsWith('engine')?artifact.request?.graph?.geoids:artifact.kind?.startsWith('multiscale')?(artifact.input?.fine_level==='bg'?artifact.input.block_groups?.geoids:artifact.input?.request?.graph?.geoids):null);
  if(index&&index.schema_version!=='bisect-native-state-unit-index-v1')throw new Error('Unknown native unit-index schema.');
  const original=pkg.files['data/final_assignments.json'];
  if(!original){stages.original_assignments={status:'not-recorded'};return result;}
  const mapped=originalAssignments(text(original),ids,500);if(!mapped){stages.original_assignments={status:'unavailable'};result.status=input.settings.mode==='replay'?'replay-unavailable':'inspection-partial';return result;}
  if(pkg.files['assignments-by-unit.json']&&JSON.stringify(Object.entries(parseSafeJson(text(pkg.files['assignments-by-unit.json']))).sort())!==JSON.stringify(Object.entries(mapped.assignments).sort()))throw new Error('Indexed and by-unit original assignment files disagree.');
  replayInputs.original=source('data/final-assignments-by-unit.json',JSON.stringify(mapped.assignments));replayInputs.minSimilarity=input.settings.minSimilarity;validateEngineReplayInputs(replayInputs);stages.original_assignments={status:'selected',source_path:'data/final_assignments.json',source_sha256:selected.files['data/final_assignments.json'],unit_order:index?'unit-index.json':'selected-replay-graph',convention:mapped.convention,units:ids.length,district_count_check:'Exact manifest district range is checked by native binding before replay'};
 }catch(error){return fail('original_and_replay_selection',error);}
 try{
  const native=accepted?.native_state??accepted?.native_run??accepted?.native_states,preparation=native?.recipe?.source_inputs?.preparation;
  if(preparation){
   if(!['prepare-native-engine-input','prepare-census-engine-input','prepare-tract-engine-input'].includes(preparation.operation))throw new Error('Unsupported retained original-source preparation.');
   if(await nativeTargetHash(native.recipe)!==native.source_sha256)throw new Error('Retained original-source recipe changed.');
   onProgress({stage:'native-output-source-check',message:'Regenerating retained source graph without partitioning'});const prepared=engine.execute(preparation);
   if(prepared.execution_performed!==false||typeof prepared.graph_json!=='string'||await nativeOutputHash(new TextEncoder().encode(prepared.graph_json))!==prepared.graph_sha256)throw new Error('Regenerated source graph has inconsistent byte evidence.');
   const recorded=native.compilation?.state_graph_source?.graph_sha256??native.compilation?.native_adjacency_source?.graph_sha256;if(recorded&&recorded!==prepared.graph_sha256)throw new Error('Regenerated source graph differs from its recorded preparation.');
   sourceGraph=prepared.graph_json;stages.original_sources={status:'regenerated',graph_sha256:prepared.graph_sha256,operation:preparation.operation,source_recipe_sha256:native.source_sha256,scope:'Original graph source regeneration; auxiliary correspondence remains in native manifest/run binding. No upstream source authenticity.'};
  }else stages.original_sources={status:'not-recorded',message:'Prepared replay graph is selected; upstream source bytes are not independently regenerated.'};
 }catch(error){return fail('original_sources',error);}
 try{
  const reply=engine.execute({operation:'verify-manifest-replay',runtime_artifact:engine.artifactIdentity,input:{prechecks:precheckRequest.input,replay:{run_json:text(replayInputs.run),original_assignments_json:text(replayInputs.original),min_similarity:input.settings.minSimilarity},inspect_only:input.settings.mode==='inspect',...(sourceGraph?{source_graph_json:sourceGraph}:{})}});
  stages.manifest_and_replay=reply;if(sourceGraph&&reply.source_graph_binding?.status==='verified')stages.original_sources.status='verified';
  result.execution_performed=reply.execution_performed===true;result.execution_completed=reply.execution_completed===true;result.replay_matched=reply.status==='match';
  result.status=input.settings.mode==='inspect'?(reply.inspection_exit_code===0?(sourceGraph&&reply.manifest_to_run_binding==='verified'?'inspection-passed':'inspection-partial'):'failed'):reply.status==='match'?'replay-matched':reply.status==='different'?'replay-different':reply.status==='unavailable'?'replay-unavailable':'replay-failed';
  return result;
 }catch(error){return fail('manifest_and_replay',error);}
}
