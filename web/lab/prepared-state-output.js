import {preparedStateRunMap} from './prepared-state-job.js';
import {preparedEnginePlanSources} from './prepared-engine-results.js';
import {encodeBytes,validateSourceFile,exactSourceText} from './package-files.js';
import {validateNationalMapBatch,validateNationalMapBatchInputs} from './national-map-batch.js';
import {stateRunOutputJobs} from './state-run-output.js';
import {stateJobScope} from './state-source-job.js';
import {tuiRunSources,tuiRunAssignmentsText} from './tui-run-files.js';
import {parseNativeMapPlanJson,inspectJsonText} from './project.js';

const sha=async text=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(text))),b=>b.toString(16).padStart(2,'0')).join('');
function same(a,b){
 if(a===b)return true;
 if(!a||!b||typeof a!=='object'||typeof b!=='object'||Array.isArray(a)!==Array.isArray(b))return false;
 const keys=Object.keys(a).sort(),other=Object.keys(b).sort();return keys.length===other.length&&keys.every((key,i)=>key===other[i]&&same(a[key],b[key]));
}
function nationalSources(states,skipped,name='prepared-completed-state-maps.json',map_type='districts'){
 const batch=validateNationalMapBatch({dpi:72,map_type,states}),bytes=new TextEncoder().encode(JSON.stringify(batch));
 if(bytes.length>128*1024*1024)throw new Error('Completed national map source JSON exceeds 128 MiB. Select fewer states.');
 const inputs={source:{name,base64:encodeBytes(bytes)}};validateNationalMapBatchInputs(inputs);return {inputs,states:Object.keys(states),skipped};
}

export function preparedStateBatchNationalSources(result){
 if(result?.schema_version!=='bisect-state-run-batch-v1'||!['completed','cancelled','interrupted'].includes(result.status))throw new Error('Finish or cancel a state batch first.');
 const states={},skipped=[];
 for(const row of result.states??[]){
  try{
   if(row.status!=='ok'||row.result?.schema_version!=='bisect-prepared-state-run-v1')throw new Error('No completed prepared engine run.');
   if(row.state_code==='DC')throw new Error('The national map supports the 50 states.');
   preparedStateRunMap(row.result);const files=preparedEnginePlanSources(row.result.prepared);
   states[row.state_code]={plan:files.auditPlanText,geometry:row.result.geometry,geometry_format:'json',analysis:null,dissolve_method:'sequential'};
  }catch(error){skipped.push({state:row.state_code,reason:error.message});}
 }
 if(!Object.keys(states).length)throw new Error('No completed prepared state has both physical export and matching display geometry.');
 return nationalSources(states,skipped);
}

// Source hashes bind the exact selected job snapshot and native completed map;
// they establish correspondence, not source authenticity or historical execution.
export async function stateBatchNationalSources(result,selectedInputs,{map_type='districts'}={}){
 if(!['districts','political','demographic','compactness'].includes(map_type))throw new Error('Choose a completed national map type.');
 if(result?.schema_version!=='bisect-state-run-batch-v1'||!['completed','cancelled','interrupted'].includes(result.status)||!Array.isArray(result.states)||result.states.length>51)throw new Error('Finish or cancel a state batch first.');
 const {jobs,year}=await stateRunOutputJobs(result,selectedInputs);
 const states={},skipped=[],bindings={};let configured=0;
 for(const row of result.states){
  try{
   if(row.status!=='ok')throw new Error('This state has no completed Run.');
   if(Object.hasOwn(states,row.state_code))throw new Error('Duplicate completed state.');
   if(row.state_code==='DC')throw new Error('The national map supports the 50 states.');
   const run=row.result,prepared=run?.schema_version==='bisect-prepared-state-run-v1',job=jobs.find(j=>{const s=stateJobScope(j);return s.state===row.state_code&&s.year===year;});
   if(prepared&&!run.workflow?.stages?.some(s=>s.name==='map:'+map_type&&s.status==='completed')){
    if(map_type!=='districts')throw new Error('Prepared engine output has no completed native '+map_type+' map analysis.');
    if(run.scope?.year!==year||run.scope?.state!==row.state_code)throw new Error('Prepared state differs from the selected year.');
    preparedStateRunMap(run);const files=preparedEnginePlanSources(run.prepared);
    states[row.state_code]={plan:files.auditPlanText,geometry:run.geometry,geometry_format:'json',analysis:null,dissolve_method:'sequential'};continue;
   }
   if(!job||!(prepared?run.scope?.state===row.state_code&&run.scope?.year===year:run?.schema_version==='bisect-tui-run-v1'&&run.form?.location?.toUpperCase()===row.state_code&&run.form.year===year))throw new Error('No matching completed State Run and selected job.');
   const files=prepared?preparedEnginePlanSources(run.prepared):tuiRunSources(run);if(!prepared)tuiRunAssignmentsText(run);
   const exported=prepared?run.prepared.export:run.practitioner_export;
   const plan=exported.document_json,context=exported.context_json,document=parseNativeMapPlanJson(plan);
   inspectJsonText(context,4000000);const ctx=JSON.parse(context),workflow=run.workflow;
   if(document.plan.units.state!==row.state_code||String(document.plan.units.year)!==year||!same(ctx.units,document.plan.units)||ctx.context_hash!==exported.context_hash)throw new Error('Native plan/context scope differs.');
   if(workflow?.schema_version!=='bisect-state-workflow-v1'||!['completed','partial'].includes(workflow.status))throw new Error('No retained native map workflow.');
   const planSha=await sha(plan),contextSha=await sha(context);
   if(workflow.accepted_plan_source_sha256!==planSha||workflow.accepted_context_source_sha256!==contextSha||workflow.file_sha256?.['plan.rplan']!==planSha||workflow.file_sha256?.['context.rctx']!==contextSha)throw new Error('Accepted workflow plan/context bytes changed.');
   const maps=(workflow.stages||[]).filter(s=>s.name?.startsWith('map:')&&s.status==='completed');
   if(!maps.length)throw new Error('No completed native state map receipt.');
   const selectedMap=maps.find(s=>s.name==='map:'+map_type);
   if(!selectedMap)throw new Error('No completed native '+map_type+' state map receipt.');
   const receipt=selectedMap.receipt;
   const options=job.workflow?.map,settings=(job.native_state?.followup??job.followup)?.settings,method=options?.dissolve_method??settings?.dissolve_method??'sequential',dpi=Number(options?.dpi??settings?.dpi),kinds=options?.map_types??settings?.map_types??['districts'];
   for(const stage of maps){const r=stage.receipt,kind=stage.name.slice(4);if(r?.schema_version!=='bisect-native-district-map-v1'||r.plan_source_sha256!==planSha||r.plan_hash!==exported.plan_hash||r.state!==row.state_code||String(r.year)!==year||r.districts!==document.plan.k||r.units!==document.plan.units.unit_ids.length||r.geometry_source_sha256!==receipt?.geometry_source_sha256||r.dissolve_method!==method||r.dpi!==dpi||r.map_type!==kind||!kinds.includes(kind)||workflow.file_sha256?.['maps/'+kind+'.svg']!==r.svg_sha256||workflow.file_sha256?.['maps/'+kind+'.png']!==r.png_sha256)throw new Error('Completed map receipt differs from the accepted State Run.');}
   let analysis=null;
   if(map_type!=='districts'){
    const path='analysis/'+map_type+'.json',base64=workflow.files_base64?.[path];
    analysis={name:row.state_code+'-'+year+'-'+map_type+'.json',base64};
    validateSourceFile(analysis,{maxBytes:3*1024*1024});
    const binary=atob(base64),bytes=new Uint8Array(binary.length);for(let i=0;i<binary.length;i++)bytes[i]=binary.charCodeAt(i);
    const hash=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>b.toString(16).padStart(2,'0')).join('');
    if(hash!==receipt.analysis_source_sha256||hash!==workflow.file_sha256?.[path])throw new Error('Completed '+map_type+' analysis bytes differ from their native map receipt.');
    inspectJsonText(new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(bytes),4000000);
   }
   const candidates=[...(prepared&&run.geometry?[exactSourceText(run.geometry)]:[]),row.source_preparation?.geometry_json,job.workflow?.map?.unit_geometry_json,job.workflow?.analysis?.compactness?.unit_geometry_json].filter(v=>typeof v==='string');
   let geometry;
   for(const value of candidates){if(value.length>64*1024*1024)continue;const bytes=new TextEncoder().encode(value);if(bytes.length>64*1024*1024)continue;if(await sha(value)===receipt.geometry_source_sha256){geometry={name:'completed-'+row.state_code+'-'+year+'.geojson',base64:encodeBytes(bytes)};break;}}
   if(!geometry)throw new Error('Original map geometry snapshot is unavailable or differs from its native receipt.');
   validateSourceFile(geometry,{maxBytes:64*1024*1024});
   states[row.state_code]={plan:files.auditPlanText,geometry,geometry_format:'json',analysis,dissolve_method:receipt.dissolve_method};
   bindings[row.state_code]={year,plan_source_sha256:planSha,context_source_sha256:contextSha,geometry_source_sha256:receipt.geometry_source_sha256};configured++;
   if(analysis)bindings[row.state_code].analysis_source_sha256=receipt.analysis_source_sha256;
  }catch(error){skipped.push({state:row.state_code,reason:error.message});}
 }
 if(!Object.keys(states).length)throw new Error('No completed state has bound plan/map sources. '+skipped.slice(0,5).map(s=>s.state+': '+s.reason).join('; '));
 return {...nationalSources(states,skipped,configured?'completed-state-maps.json':undefined,map_type),bindings};
}
