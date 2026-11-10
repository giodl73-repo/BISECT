import {validateNativeAggregateInputs,nativeAggregateRequest,resolvedNativeAggregateTypes} from './native-aggregate-files.js';
import {aggregateWorkspaceVersion} from './native-aggregate-workspace.js';
const encode=text=>new TextEncoder().encode(text);
const hash=async bytes=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>b.toString(16).padStart(2,'0')).join('');
const identity=async value=>hash(encode(JSON.stringify(value)));
const check=options=>{options.signal?.throwIfAborted();options.assertCurrent?.();};
const MAX_BYTES=50*1024*1024;

// This is a neutral preparation record, never a live directory grant or receipt.
// Operations describe successful final-file close order. Native temporary writes,
// renames/error effects and browser interruption/recovery still need a writer.
export async function prepareNativeAggregateApplicationPlan(execute,input,{version='v1',...options}={}){
 if(typeof execute!=='function')throw Error('Aggregate execution is required.');
 aggregateWorkspaceVersion(version);
 check(options);const retained=structuredClone(validateNativeAggregateInputs(input));
 const request=nativeAggregateRequest(retained),year=request.input.year,target=version+'/'+year+'/national';
 const expected=await execute(structuredClone(request));check(options);
 if(expected?.schema_version!=='bisect-native-aggregate-v1'||expected.year!==year)throw Error('Invalid full aggregate preparation reply.');
 if(!expected.files||!expected.file_sha256||JSON.stringify(Object.keys(expected.files).sort())!==JSON.stringify(Object.keys(expected.file_sha256).sort()))throw Error('Invalid full aggregate file identities.');
 const types=resolvedNativeAggregateTypes(request.input.types),force=request.input.completion?.force??false;
 const existing=new Set(request.input.completion?.existing_outputs||[]);
 const frames=[],operations=[],blobs=Object.create(null),finalFiles=Object.create(null),finalHashes=Object.create(null);let uniqueBytes=0;
 for(const [ordinal,type]of types.entries()){
  check(options);
  const step={operation:request.operation,input:{...request.input,types:[type],reports:Object.fromEntries(Object.entries(request.input.reports).map(([state,reports])=>[state,Object.hasOwn(reports,type)?{[type]:reports[type]}:{}])),completion:{year,force,existing_outputs:[...existing]}}};
  const result=await execute(step);check(options);
  if(result?.schema_version!=='bisect-native-aggregate-v1'||result.year!==year||!Array.isArray(result.summaries)||result.summaries.length!==1||result.summaries[0].analyzer!==type)throw Error('Invalid per-analyzer aggregate reply.');
  const summary=result.summaries[0],status=summary.status;
  if(!['aggregated','no-data','skipped-existing'].includes(status))throw Error('Unknown aggregate completion status.');
  const names=status==='aggregated'?['us_'+type+'.json',...(request.input.csv||request.input.format==='csv'?['us_'+type+'.csv']:[]),...(request.input.format==='json'?['us_'+type+'_'+(type==='proportionality'?'states':'districts')+'.json']:[])]:[];
  if(!result.files||!result.file_sha256||JSON.stringify(Object.keys(result.files).sort())!==JSON.stringify([...names].sort())||JSON.stringify(Object.keys(result.file_sha256).sort())!==JSON.stringify([...names].sort()))throw Error('Aggregate file set differs from native format rules.');
  const writes=[];
  for(const name of names){
   const text=result.files[name];if(typeof text!=='string')throw Error('Aggregate output bytes must be text.');
   const bytes=encode(text);if(bytes.length>25*1024*1024)throw Error('Aggregate output exceeds 25 MiB per file.');const digest=await hash(bytes);check(options);
   if(digest!==result.file_sha256[name])throw Error('Aggregate output hash mismatch.');
   if(!Object.hasOwn(blobs,digest)){if((uniqueBytes+=bytes.length)>MAX_BYTES)throw Error('Aggregate preparation exceeds 50 MiB of distinct output bytes.');blobs[digest]={text,byte_length:bytes.length};}
   const effect={frame:ordinal,ordinal:operations.length,path:target+'/'+name,sha256:digest,byte_length:bytes.length};operations.push(effect);writes.push(effect.ordinal);finalFiles[name]=text;finalHashes[effect.path]=digest;
  }
  frames.push({ordinal,analyzer:type,summary:structuredClone(summary),skipped_sources:structuredClone(result.skipped),source_sha256:structuredClone(result.source_sha256),step_request_sha256:await identity(step),write_ordinals:writes});
  if(status==='aggregated')existing.add('us_'+type+'.json');
  check(options);await options.onProgress?.({completed_frames:frames.length,write_operations:operations.length});check(options);
 }
 if(JSON.stringify(frames.map(f=>f.summary))!==JSON.stringify(expected.summaries)||JSON.stringify(Object.keys(finalFiles).sort())!==JSON.stringify(Object.keys(expected.files).sort()))throw Error('Ordered preparation differs from full native aggregate output.');
 for(const [name,text]of Object.entries(finalFiles))if(expected.files[name]!==text||expected.file_sha256[name]!==finalHashes[target+'/'+name])throw Error('Ordered preparation changed aggregate file bytes.');
 const plan={schema_version:'bisect-native-aggregate-application-plan-v1',version,year,target_namespace:target,input:retained,input_sha256:await identity(retained),result_sha256:await identity(expected),frames,operations,blobs,final_file_sha256:finalHashes,directory_preparation:{path:target,before_analyzer_processing:true},scope:'Preparation only over bounded selected source snapshots and a portable version component. Ordered successful final-file effects are retained, including repeated Force writes. No handles, writes, native temporary-file/failure-prefix equivalence, installation receipt, recovery or source authenticity authority.'};
 check(options);return plan;
}
