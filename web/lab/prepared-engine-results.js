import {encodeBytes,validateSourceFile,validateImprovementSettings} from './package-files.js';
import {inspectJsonText,parseNativeMapPlanJson} from './project.js';
import {validateEngineReplayInputs} from './engine-replay-files.js';

function run(result){
 if(result?.schema_version!=='bisect-prepared-engine-v1'||result.execution_performed!==true||!['ordinary','multiscale'].includes(result.run_kind))throw new Error('Complete a prepared engine Run first.');
 const input=result.executed_request,request=result.run_kind==='ordinary'?input:input?.request;
 if(!request?.graph||!request.options||!result.result?.assignments)throw new Error('Prepared result has no accepted request and assignments.');
 return {input,request};
}
function equal(a,b){
 if(a===b)return true;
 if(!a||!b||typeof a!=='object'||typeof b!=='object'||Array.isArray(a)!==Array.isArray(b))return false;
 const x=Object.keys(a).sort(),y=Object.keys(b).sort();return x.length===y.length&&x.every((k,i)=>k===y[i]&&equal(a[k],b[k]));
}
function source(name,text,maxBytes){
 if(typeof text!=='string')throw new Error('Prepared result has no physical plan/context export.');
 const value={name,base64:encodeBytes(new TextEncoder().encode(text))};validateSourceFile(value,{maxBytes});return value;
}
export function preparedEngineMapDocument(result){
 const {input,request}=run(result),g=request.graph,k=request.options.districts,fine=result.run_kind==='multiscale'&&input.fine_level==='bg';
 const ids=fine?input.block_groups?.geoids:g.geoids,assignments=result.result.assignments;
 if(!Array.isArray(ids)||!ids.length||new Set(ids).size!==ids.length||Object.keys(assignments).length!==ids.length||!Number.isSafeInteger(k)||k<1||k>500)throw new Error('Prepared result has incomplete target assignments.');
 const assignment=ids.map(id=>{const value=assignments[id];if(typeof id!=='string'||!Object.hasOwn(assignments,id)||!Number.isSafeInteger(value)||value<1||value>k)throw new Error('Prepared result has invalid unit IDs or district labels.');return value-1;});
 return {plan:{k,units:{unit_ids:[...ids],state:g.state,year:g.year,unit_kind:fine?'block-group':g.unit_kind??'tract'},assignment}};
}
export function preparedEnginePlanSources(result){
 const map=preparedEngineMapDocument(result),output=result.export;
 if(!output||result.export_error!=null)throw new Error('This Run has no physical plan/context export.');
 const sources={auditPlanText:source('prepared-engine.rplan',output.document_json,3*1024*1024),auditContextText:source('prepared-engine.rctx',output.context_json,8*1024*1024)};
 // Full state adjacency can exceed the ordinary small-JSON node budget. Keep
 // the existing context byte cap and use the bounded workbench tree budget.
 const document=parseNativeMapPlanJson(output.document_json);inspectJsonText(output.context_json,4000000);const context=JSON.parse(output.context_json);
 if(!equal(document,output.document)||!equal(context,output.context)||document.metadata?.jurisdiction!==map.plan.units.state||document.plan?.k!==map.plan.k||String(document.plan?.units?.year)!==String(map.plan.units.year)||document.plan.units.unit_kind!==map.plan.units.unit_kind||!equal(document.plan.units.unit_ids,map.plan.units.unit_ids)||!equal(document.plan.assignment,map.plan.assignment)||!equal(context.units,document.plan.units)||context.context_hash!==output.context_hash)throw new Error('Prepared export disagrees with accepted assignments, scope or retained native text.');
 return sources;
}
export function preparedEngineAuditInputs(result,files={}){
 const next={...files,...preparedEnginePlanSources(result)};delete next.auditCertificateText;delete next.auditLineageText;return next;
}
export function preparedEngineImprovementInputs(result,files={}){
 const next=preparedEngineAuditInputs(result,files),{request}=run(result);
 next.improvementSettings=validateImprovementSettings(files.improvementSettings??{tolerance_percent:request.options.balance_tolerance,label:null,generated_at_utc:''});return next;
}
export async function preparedEngineReplayInputs(result){
 const {input}=run(result);preparedEngineMapDocument(result);
 const text=result.replay_artifact_json;if(typeof text!=='string'||!/^[a-f0-9]{64}$/.test(result.replay_artifact_sha256??''))throw new Error('Run again with an engine that records the accepted replay artifact.');
 const sha=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(text))),b=>b.toString(16).padStart(2,'0')).join('');
 if(sha!==result.replay_artifact_sha256)throw new Error('Accepted replay artifact bytes changed.');
 // Accepted graph artifacts use the same tree budget as saved workbenches.
 inspectJsonText(text,4000000);const artifact=JSON.parse(text),ordinary=result.run_kind==='ordinary';
 if(artifact.kind!==(ordinary?'engine':'multiscale')||!equal(ordinary?artifact.request:artifact.input,input))throw new Error('Replay artifact disagrees with accepted engine inputs.');
 const inputs={run:source('accepted-prepared-run.json',text,16*1024*1024),original:source('accepted-final-assignments.json',JSON.stringify(result.result.assignments,null,2)+'\n',8*1024*1024),minSimilarity:1};
 validateEngineReplayInputs(inputs);return inputs;
}
