import {parseSafeJson,inspectJsonText} from './project.js';
import {encodeBytes,validateSourceFile} from './package-files.js';
import {labelBatchText,labelAnalysisFiles} from './label-analysis-files.js';
import {labelReportZip} from './label-report-files.js';
const limits={draft:8*1024*1024,currentRegistry:1024*1024,upstreamIndex:1024*1024};
const sourceText=(source,kind)=>{validateSourceFile(source,{maxBytes:limits[kind]});return labelBatchText(source);};
const bytes=t=>new TextEncoder().encode(t);
const sha=async t=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes(t))),v=>v.toString(16).padStart(2,'0')).join('');
function draft(source){
 const v=parseSafeJson(sourceText(source,'draft'));
 if(!v||Object.keys(v).sort().join()!=='generated_index_json,proposed_registry_json,source_registry_json,source_registry_sha256,stage'||!['analyzed','reported'].includes(v.stage)||typeof v.source_registry_sha256!=='string'||!/^[a-f0-9]{64}$/.test(v.source_registry_sha256))throw new Error('Select a native stage reconciliation draft.');
 for(const key of ['generated_index_json','proposed_registry_json','source_registry_json']){if(typeof v[key]!=='string'||bytes(v[key]).length>1024*1024)throw new Error('Draft snapshots and indexes are limited to 1 MiB each.');parseSafeJson(v[key]);}
 return v;
}
export function validateLabelReconcileInputs(v){
 if(!v||Object.keys(v).sort().join()!=='currentRegistry,draft,upstreamIndex')throw new Error('Invalid reconciliation source fields.');
 if(v.draft!==null)draft(v.draft);
 if(v.currentRegistry!==null)parseSafeJson(sourceText(v.currentRegistry,'currentRegistry'));
 // Inspect identity/structure without converting native numeric u64 metadata.
 if(v.upstreamIndex!==null){const text=sourceText(v.upstreamIndex,'upstreamIndex');inspectJsonText(text);JSON.parse(text);}
 return v;
}
export function labelReconcileRequest(v){
 validateLabelReconcileInputs(v);if(!v.draft||!v.currentRegistry||!v.upstreamIndex)throw new Error('Select a draft, the current registry and its current upstream index.');
 return {operation:'reconcile-label-registry',input:{...draft(v.draft),current_registry_json:sourceText(v.currentRegistry,'currentRegistry'),upstream_index_json:sourceText(v.upstreamIndex,'upstreamIndex')}};
}
export async function readLabelReconcileSource(file,kind){
 if(!limits[kind]||!file||!Number.isSafeInteger(file.size)||file.size<1||file.size>limits[kind])throw new Error('Select a nonempty source within the displayed size limit.');
 const data=new Uint8Array(await file.arrayBuffer());if(data.length!==file.size)throw new Error('Source size changed while reading.');
 const source={name:file.name,base64:encodeBytes(data)};validateLabelReconcileInputs({draft:null,currentRegistry:null,upstreamIndex:null,[kind]:source});return source;
}
export async function labelReconcileFromStage(result,files){
 const analysis=result?.schema_version==='bisect-label-analysis-v1',report=result?.schema_version==='bisect-label-report-v1';
 if(!analysis&&!report||result.registry_promoted!==true)throw new Error('Run a native stage with a supplied registry first.');
 if(analysis)await labelAnalysisFiles(result);else await labelReportZip(result);
 const original=analysis?files.labelAnalysisInputs?.source:files.labelReportInputs?.source;
 if(!original)throw new Error('Original selected stage sources are unavailable.');
 const input=parseSafeJson(labelBatchText(original));if(typeof input.registry_json!=='string'||await sha(input.registry_json)!==result.registry_source_sha256)throw new Error('Original registry no longer matches the stage snapshot.');
 const value={stage:analysis?'analyzed':'reported',source_registry_json:input.registry_json,source_registry_sha256:result.registry_source_sha256,proposed_registry_json:result.registry_json,generated_index_json:result.index_json};
 const source={name:`${result.label}-${result.year}-${value.stage}-reconciliation.json`,base64:encodeBytes(bytes(JSON.stringify(value)))};
 return validateLabelReconcileInputs({draft:source,currentRegistry:null,upstreamIndex:null});
}
export async function reconciledRegistryBytes(result){
 if(result?.schema_version!=='bisect-label-registry-reconciliation-v1'||result.status!=='reconciled'||typeof result.registry_json!=='string'||bytes(result.registry_json).length>1024*1024||result.files?.['.bisect']!==result.registry_json||await sha(result.registry_json)!==result.registry_sha256)throw new Error('Reconciled registry hash or bytes do not match.');
 parseSafeJson(result.registry_json);return bytes(result.registry_json);
}
