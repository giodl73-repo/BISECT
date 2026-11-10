import {preparedEnginePlanSources,preparedEngineAuditInputs} from './prepared-engine-results.js';
import {preparedEngineDownload} from './prepared-engine-files.js';
import {encodeBytes,validateSourceFile} from './package-files.js';
import {NATIVE_MAP_DEFAULTS,validateNativeMapInputs} from './map-export-files.js';

function equal(a,b){
 if(a===b)return true;
 if(!a||!b||typeof a!=='object'||typeof b!=='object'||Array.isArray(a)!==Array.isArray(b))return false;
 const x=Object.keys(a).sort(),y=Object.keys(b).sort();return x.length===y.length&&x.every((k,i)=>k===y[i]&&equal(a[k],b[k]));
}
// These checks establish consistency only. The UI must separately require its
// current computed result; imported receipts/results confer no execution authority.
export function recordedReplayAcceptedRun(result){
 if(result?.schema_version!=='bisect-native-recorded-replay-v1'||result.status!=='match'||result.execution_performed!==true||result.recorded_paths_opened!==false||result.error!==null||result.source_check?.status!=='sources-and-graph-checked'||result.source_check.error!==null)throw Error('Complete a fresh matching recorded replay first.');
 const comparison=result.comparison,run=result.replay;
 if(comparison?.literal_assignments_equal!==true||comparison.district_relabelling_performed!==false||!equal(comparison.reference_assignments,comparison.regenerated_assignments)||!equal(comparison.regenerated_assignments,run?.result?.assignments)||result.execution_comparison!=null&&(result.execution_comparison.bound_to_recipe!==true||['equal','requested_seed_equal','accepted_seed_equal','retry_attempt_equal','sampler_equal'].some(key=>result.execution_comparison[key]!==true)))throw Error('Recorded replay assignments or execution comparison do not match.');
 preparedEnginePlanSources(run);
 return run;
}
export function recordedReplayDownload(result,kind){
 const run=recordedReplayAcceptedRun(result);
 if(kind==='evidence')return {name:'recorded-replay-evidence.json',text:JSON.stringify(result,null,2)+'\n'};
 const file=preparedEngineDownload(run,kind);
 return {...file,name:kind==='plan'?'recorded-replay.rplan':'recorded-replay.rctx'};
}
const digest=async text=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(text))),b=>b.toString(16).padStart(2,'0')).join('');
export async function recordedReplayHandoff(result,files={},action){
 const run=recordedReplayAcceptedRun(result),sources=preparedEnginePlanSources(run);
 if(!['audit','map'].includes(action))throw Error('Choose replay audit or map handoff.');
 const receipt={schema_version:'bisect-recorded-replay-handoff-v1',status:result.status,comparison:result.comparison,execution_comparison:result.execution_comparison,source_check:result.source_check,backend_resolution:result.backend_resolution,runtime_artifacts:result.runtime_artifacts??null,request_sha256:run.request_sha256,replay_artifact_sha256:run.replay_artifact_sha256,plan_hash:run.export.plan_hash,context_hash:run.export.context_hash,document_sha256:await digest(run.export.document_json),context_sha256:await digest(run.export.context_json),scope:'Retained replay comparison and exact exported-byte bindings. Imported receipt is untrusted data; no filesystem access, automatic execution, generation authentication or certification authority is restored.'};
 const retained={name:'recorded-replay-receipt.json',base64:encodeBytes(new TextEncoder().encode(JSON.stringify(receipt,null,2)+'\n'))};validateSourceFile(retained,{maxBytes:8*1024*1024});
 if(action==='audit')return {...preparedEngineAuditInputs(run,files),nativeRecordedReplayReceipt:retained};
 const inputs=validateNativeMapInputs({...NATIVE_MAP_DEFAULTS,plan:sources.auditPlanText,geometry:files.nativeMapInputs?.geometry??files.planGeometry??null,dpi:files.nativeMapInputs?.dpi??150});
 return {...files,nativeMapInputs:inputs,nativeRecordedReplayReceipt:retained};
}
