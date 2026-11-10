import {validateJsonTree} from './project.js';
import {verifyNativeStateVra} from './native-state-vra.js';
import {storedZipBytes} from './reproducibility-files.js';
// Native State serializes a BTreeMap<String, usize>: decimal indices sort
// lexicographically. JSON.stringify on an object would reorder numeric keys.
export function nativeStateAssignmentText(ids,assignments,k){
 validateJsonTree(assignments);
 if(!Array.isArray(ids)||!ids.length||ids.length>1000000||new Set(ids).size!==ids.length||!Number.isSafeInteger(k)||k<1||k>500||!assignments||Array.isArray(assignments)||Object.keys(assignments).length!==ids.length)throw new Error('Native State files require complete ordered units and assignments.');
 let bytes=0;const rows=ids.map((id,i)=>{if(typeof id!=='string'||!id.length||id.length>200||!Object.hasOwn(assignments,id)||!Number.isSafeInteger(assignments[id])||assignments[id]<1||assignments[id]>k)throw new Error('Invalid native State assignment or unit identity.');bytes+=new TextEncoder().encode(id).length;if(bytes>32*1024*1024)throw new Error('Native State unit identity export exceeds 32 MiB.');return [String(i),assignments[id]];});rows.sort(([a],[b])=>a<b?-1:a>b?1:0);
 return '{\n'+rows.map(([index,district])=>'  '+JSON.stringify(index)+': '+district).join(',\n')+'\n}';
}
export function nativeStateDataEntries(request,result,{fine=null,multiscale=null}={}){
 const graph=request?.graph,options=result?.options,assignments=result?.assignments,ids=fine?.geoids??graph?.geoids,k=options?.districts;
 if(!graph||!options||!request.options||typeof graph.state!=='string'||typeof graph.year!=='string'||options.structure!==request.options?.structure||options.weights!==request.options?.weights||k!==request.options.districts||!/^[A-Z]{2}$/.test(graph.state)||!/^\d{4}$/.test(graph.year))throw new Error('Native State output differs from its engine request.');
 const assignmentText=nativeStateAssignmentText(ids,assignments,k),vra=result.metrics?.native_state_vra;
 if(vra!==undefined){if(fine)throw new Error('Parent minority summaries cannot describe fine-unit State output.');verifyNativeStateVra(vra,graph,{...options,demographics:{[graph.state]:request.demographics}},assignments,k);if(vra.demographics_sha256!==result.metrics?.weighting_evidence?.demographics_sha256)throw new Error('Native State summary source identity differs from the Run.');}
 const files=[['data/final_assignments.json',assignmentText],['unit-index.json',JSON.stringify({schema_version:'bisect-native-state-unit-index-v1',state:graph.state,year:graph.year,unit_kind:fine?'block-group':graph.unit_kind??'tract',unit_ids:ids},null,2)+'\n'],['assignments-by-unit.json',JSON.stringify(assignments,null,2)+'\n'],['engine-options.json',JSON.stringify(options,null,2)+'\n']];
 if(vra)files.push(['data/vra_analysis.json',vra.analysis_json]);
 if(multiscale)files.push(['multiscale-settings.json',JSON.stringify({schema_version:'bisect-native-state-multiscale-settings-v1',fine_level:multiscale.fine_level,coarse_level:multiscale.coarse_level,total_steps:multiscale.total_steps,alpha:multiscale.alpha,percentile:multiscale.percentile,adaptive:multiscale.adaptive??null,parent_engine_options:request.options},null,2)+'\n']);
 const balanced=result.metrics?.balance_passed??result.metrics?.within_requested_tolerance,connected=result.metrics?.contiguous;
 files.push(['receipt.json',JSON.stringify({schema_version:'bisect-native-state-data-v1',state:graph.state,year:graph.year,units:ids.length,districts:k,population_balance_valid:typeof balanced==='boolean'?balanced:null,contiguous:typeof connected==='boolean'?connected:null,assignment_labels:'one-based',assignment_keys:'zero-based-unit-indices',unit_order:'unit-index.json',files:files.map(([name])=>name),scope:'Native State assignment format and optional descriptive minority summary, bound to the retained engine unit order. Original run provenance and source inputs remain in the separate run/project receipt. No filesystem installation or native executable provenance is asserted.'},null,2)+'\n']);
 return files.map(([name,text])=>['native-state/'+name,new TextEncoder().encode(text)]);
}
export function nativeStateResultEntries(output){
 if(output?.schema_version==='bisect-prepared-state-run-v1')output=output.prepared;
 let request,result,fine=null,multiscaleSettings=null;
 if(['bisect-prepared-engine-v1','bisect-native-character-engine-v1'].includes(output?.schema_version)&&output.execution_performed===true){
  const input=output.executed_request,multiscale=output.run_kind==='multiscale';if(!['ordinary','multiscale'].includes(output.run_kind))throw new Error('Unknown prepared Run kind.');request=multiscale?input?.request:input;fine=multiscale&&input.fine_level==='bg'?input.block_groups:null;
  result=multiscale?{...output.result,options:request?.options}:output.result;
  if(multiscale)multiscaleSettings=input;
 }else if(output?.schema_version==='bisect-tui-run-v1'&&output.status==='completed'){request=output.configured_run?.engine_request;result=output.configured_run?.result;}
 else throw new Error('Complete a fresh prepared engine or native TUI Run before exporting State data.');
 return nativeStateDataEntries(request,result,{fine,multiscale:multiscaleSettings});
}

export function nativeStateResultZip(output){return storedZipBytes(nativeStateResultEntries(output),{maxSourceBytes:128*1024*1024});}
