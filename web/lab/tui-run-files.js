import {nativeStateDataEntries} from './native-state-files.js';
import {validateRunTargetSettings,runTargetRequest} from './run-target-files.js';
import {parseNativeMapPlanJson,parseGeometryJson} from './project.js';
import {planMapModel} from './plan-map.js';
import {configRunRequest} from './config-run.js';
import {encodeBytes,validateSourceFile,validateImprovementSettings} from './package-files.js';
import {storedZipBytes} from './reproducibility-files.js';
import {TUI_COMPARE_DEFAULTS,validateTuiCompareInputs} from './tui-compare-files.js';

export function tuiRunAssignmentsText(result){
 tuiRunSources(result);const document=parseNativeMapPlanJson(result.practitioner_export.document_json),ids=document.plan.units.unit_ids,assignment=document.plan.assignment;
 if(!Array.isArray(ids)||!Array.isArray(assignment)||ids.length!==assignment.length||new Set(ids).size!==ids.length||!Number.isSafeInteger(document.plan.k)||assignment.some(d=>!Number.isSafeInteger(d)||d<0||d>=document.plan.k))throw new Error('Invalid accepted native plan assignments.');
 const assignments=Object.fromEntries(ids.map((id,index)=>[id,assignment[index]+1]));
 if(JSON.stringify(Object.entries(assignments).sort())!==JSON.stringify(Object.entries(result.configured_run?.result?.assignments||{}).sort()))throw new Error('Accepted Run assignments disagree with its native plan export.');
 return JSON.stringify(result.configured_run.result.assignments,null,2)+'\n';
}
const sha=async bytes=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>b.toString(16).padStart(2,'0')).join('');
export async function tuiRunCompareSide(result){
 const assignment=tuiRunAssignmentsText(result),sources={'data/final_assignments.json':{name:'final_assignments.json',base64:encodeBytes(new TextEncoder().encode(assignment))}};
 const workflow=result.workflow;
 if(workflow){
  if(workflow.schema_version!=='bisect-state-workflow-v1'||!['completed','partial'].includes(workflow.status)||workflow.accepted_plan_source_sha256!==await sha(new TextEncoder().encode(result.practitioner_export.document_json))||workflow.accepted_context_source_sha256!==await sha(new TextEncoder().encode(result.practitioner_export.context_json)))throw new Error('Workflow sources differ from the accepted Run.');
  for(const name of ['compactness','summary','splits','contiguity']){
   const path='analysis/'+name+'.json',base64=workflow.files_base64?.[path];if(base64===undefined)continue;
   const source={name:name+'.json',base64};validateSourceFile(source,{maxBytes:8*1024*1024});const bytes=Uint8Array.from(atob(base64),c=>c.charCodeAt(0));
   if(await sha(bytes)!==workflow.file_sha256?.[path])throw new Error('Workflow comparison analysis hash mismatch.');sources[path]=source;
  }
 }
 const side={label:result.form.label,sources};validateTuiCompareInputs({a:side,b:TUI_COMPARE_DEFAULTS.b});return side;
}
export const TUI_RUN_FORM_DEFAULTS={location:'VT',chamber:'congressional',year:'2020',resolution:'tract',seed:'',label:'browser_state_run',version:'v1',balance_tol:''};
export function validateTuiRunSettings(v){if(!v||Object.keys(v).sort().join()!==(Object.hasOwn(v,'target')?'created_at,form,target':'created_at,form')||!v.form||Object.keys(v.form).sort().join()!==Object.keys(TUI_RUN_FORM_DEFAULTS).sort().join()||Object.values(v.form).some(s=>typeof s!=='string'||new TextEncoder().encode(s).length>200||/[\u0000-\u001f\u007f-\u009f]/.test(s))||typeof v.created_at!=='string'||new TextEncoder().encode(v.created_at).length>200||/[\u0000-\u001f\u007f-\u009f]/.test(v.created_at))throw new Error('Invalid bounded native Run form.');if(v.form.seed!==''&&(!/^(0|[1-9][0-9]{0,19})$/.test(v.form.seed)||BigInt(v.form.seed)>18446744073709551615n))throw new Error('Run seed must be an exact decimal u64 integer.');if(v.form.balance_tol!==''&&(!v.form.balance_tol.trim()||!Number.isFinite(Number(v.form.balance_tol))))throw new Error('Run tolerance must be a finite native percent.');if(Object.hasOwn(v,'target'))validateRunTargetSettings(v.target);return v;}
export function tuiRunRequest(settings,source,profile,inputs,runSettings){validateTuiRunSettings(settings);if(!settings.created_at)throw new Error('Record a native export timestamp.');const r=configRunRequest(source,profile,inputs,{...runSettings,chamber:settings.form.chamber,native_build:true});return {operation:'run-tui-configured',input:{form:settings.form,run:r.input,created_at:settings.created_at,...(settings.target?{target:runTargetRequest(settings.target)}:{})}};}
export function tuiRunSources(result){if(result?.schema_version!=='bisect-tui-run-v1'||result.status!=='completed')throw new Error('Complete a fresh native Run first.');const output=result.practitioner_export,sources={};for(const [key,name,text]of [['auditPlanText','tui-run.rplan',output?.document_json],['auditContextText','tui-run.rctx',output?.context_json]]){if(typeof text!=='string')throw new Error('Missing native Run export.');const source={name,base64:encodeBytes(new TextEncoder().encode(text))};validateSourceFile(source);sources[key]=source;}return sources;}
export function tuiRunAuditInputs(result,files={}){
 const sources=tuiRunSources(result);validateSourceFile(sources.auditPlanText,{maxBytes:3*1024*1024});validateSourceFile(sources.auditContextText,{maxBytes:8*1024*1024});
 const next={...files,...sources};delete next.auditCertificateText;delete next.auditLineageText;return next;
}
export function tuiRunImprovementInputs(result,files={}){
 const next=tuiRunAuditInputs(result,files);
 next.improvementSettings=validateImprovementSettings(files.improvementSettings||{tolerance_percent:result.configured_run.configuration.execution_tolerance_percent,label:null,generated_at_utc:''});return next;
}
export function tuiRunZip(result){tuiRunSources(result);const native=result.practitioner_export,manifest=result.report_manifest?.manifest_json,assignmentText=JSON.stringify(result.configured_run.result.assignments,null,2)+'\n';if(typeof result.effective_yaml!=='string'||typeof manifest!=='string')throw new Error('Missing effective configuration or native report manifest.');return storedZipBytes(Object.entries({'plan.rplan':native.document_json,'context.rctx':native.context_json,'manifest.json':manifest,'effective-config.yml':result.effective_yaml,'final_assignments.json':assignmentText,'data/final_assignments.json':assignmentText,'run-receipt.json':JSON.stringify(result,null,2)+'\n'}).map(([name,text])=>[name,new TextEncoder().encode(text)]).concat(result.configured_run.result.metrics?.native_state_vra?nativeStateDataEntries(result.configured_run.engine_request,result.configured_run.result):[]),{maxSourceBytes:128*1024*1024});}

export function tuiRunGeometry(result,source){
 if(source==null)return null;
 tuiRunSources(result);validateSourceFile(source,{maxBytes:64*1024*1024});
 const geometry=parseGeometryJson(new TextDecoder('utf-8',{fatal:true}).decode(Uint8Array.from(atob(source.base64),c=>c.charCodeAt(0))));
 planMapModel(parseNativeMapPlanJson(result.practitioner_export.document_json),geometry);
 return source;
}

export function tuiRunApplyPreferences(current,preferences){
 if(!preferences||Object.keys(preferences).sort().join()!=='chamber,location,resolution,version,year')throw new Error('Invalid native Run preference fields.');
 return validateTuiRunSettings({form:{...validateTuiRunSettings(current).form,...preferences},created_at:current.created_at,...(current.target?{target:current.target}:{})});
}
