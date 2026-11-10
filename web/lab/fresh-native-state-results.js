import {encodeBytes,validateSourceFile,validateImprovementSettings} from './package-files.js';
import {inspectJsonText,parseNativeMapPlanJson} from './project.js';

function equal(a,b){
 if(a===b)return true;
 if(!a||!b||typeof a!=='object'||typeof b!=='object'||Array.isArray(a)!==Array.isArray(b))return false;
 const x=Object.keys(a).sort(),y=Object.keys(b).sort();
 return x.length===y.length&&x.every((key,i)=>key===y[i]&&equal(a[key],b[key]));
}
function source(name,text,maxBytes){
 if(typeof text!=='string')throw Error('This fresh Run has no completed native plan/context export.');
 const file={name,base64:encodeBytes(new TextEncoder().encode(text))};
 validateSourceFile(file,{maxBytes});return file;
}

// Accept only an in-memory completed fresh Run. Callers must separately enforce
// the live review/result grant; saved JSON is never such a grant.
export function freshNativeStatePlanSources(result){
 if(result?.schema_version!=='bisect-fresh-native-state-run-v1'||result.execution_performed!==true||result.recorded_paths_opened!==false||!/^[a-f0-9]{64}$/.test(result.input_sha256??''))throw Error('Complete a fresh native State Run first.');
 const output=result.export,input=result.executed_request,request=input?.request??input;
 if(!output||result.export_error!=null)throw Error('This fresh Run has no completed native plan/context export.');
 const sources={auditPlanText:source('fresh-native-state.rplan',output.document_json,3*1024*1024),auditContextText:source('fresh-native-state.rctx',output.context_json,8*1024*1024)};
 const document=parseNativeMapPlanJson(output.document_json);
 inspectJsonText(output.context_json,4000000);const context=JSON.parse(output.context_json);
 const plan=document.plan,k=request?.options?.districts;
 const ids=input?.fine_level==='bg'?input.block_groups?.geoids:request?.graph?.geoids;
 const assignments=result.result?.assignments;
 if(!Array.isArray(ids)||!ids.length||new Set(ids).size!==ids.length||!assignments||Object.keys(assignments).length!==ids.length||!Number.isSafeInteger(k)||k<1||k>500||!equal(plan?.units?.unit_ids,ids)||plan.k!==k||!Array.isArray(plan.assignment)||plan.assignment.length!==ids.length)throw Error('Fresh native output does not cover the accepted unit universe.');
 for(let i=0;i<ids.length;i++){
  const district=assignments[ids[i]];
  if(!Object.hasOwn(assignments,ids[i])||!Number.isSafeInteger(district)||district<1||district>k||plan.assignment[i]!==district-1)throw Error('Fresh native output differs from accepted assignments.');
 }
 if(!equal(document,output.document)||!equal(context,output.context)||!equal(context.units,plan.units)||plan.units.state!==request.graph.state||String(plan.units.year)!==String(request.graph.year)||document.metadata?.jurisdiction!==request.graph.state||context.context_hash!==output.context_hash||output.fresh_native_state_binding?.input_sha256!==result.input_sha256||output.fresh_native_state_binding.original_context_loaded!==false)throw Error('Fresh native text, scope or context binding differs from the completed Run.');
 return sources;
}

export function freshNativeStateFollowupInputs(result,action,files={}){
 if(!['audit','analyze','improve'].includes(action))throw Error('Select an available fresh State follow-up.');
 const next={...files,...freshNativeStatePlanSources(result)};
 delete next.auditCertificateText;delete next.auditLineageText;
 if(action==='improve'){
  const request=result.executed_request.request??result.executed_request;
  next.improvementSettings=validateImprovementSettings(files.improvementSettings??{tolerance_percent:request.options.balance_tolerance,label:null,generated_at_utc:''});
 }
 return next;
}
