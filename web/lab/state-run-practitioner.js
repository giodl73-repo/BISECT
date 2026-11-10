import {nativeStateFollowupSources} from './native-state-followup.js';
import {stateRunOutputJobs,stateOutputSha} from './state-run-output.js';
import {stateJobScope} from './state-source-job.js';
import {tuiRunSources,tuiRunAssignmentsText,tuiRunImprovementInputs,tuiRunCompareSide} from './tui-run-files.js';
import {preparedEnginePlanSources,preparedEngineMapDocument,preparedEngineImprovementInputs,preparedEngineReplayInputs} from './prepared-engine-results.js';
import {engineReplayFromConfigured} from './engine-replay-files.js';
import {parseNativeMapPlanJson,inspectJsonText} from './project.js';
import {validateDoctorInputs} from './doctor-files.js';
import {TUI_COMPARE_DEFAULTS,validateTuiCompareInputs} from './tui-compare-files.js';
import {TEXT_COMPARISON_DEFAULTS,validateTextComparisonInputs} from './text-comparison-files.js';
import {encodeBytes,exactSourceText,exactContextText} from './package-files.js';
import {runReproducibilityInputs} from './run-reproducibility.js';
import {validateReproducibilityInputs,reproducibilityRequest} from './reproducibility-files.js';
function same(a,b){if(a===b)return true;if(!a||!b||typeof a!=='object'||typeof b!=='object'||Array.isArray(a)!==Array.isArray(b))return false;const x=Object.keys(a).sort(),y=Object.keys(b).sort();return x.length===y.length&&x.every((k,i)=>k===y[i]&&same(a[k],b[k]));}
const sameComparisonUnits=(a,b)=>['unit_ids','unit_kind','state','year','unit_universe_hash'].every(key=>same(a?.[key],b?.[key]));
export async function stateRunPractitionerInputs(batch,inputs,state,action,files={}){
 if(!['audit','doctor','improve','replay','reproducibility','compare-a','compare-b'].includes(action))throw new Error('Choose audit, Doctor, improvement, explicit replay, reproducibility package or a comparison side.');
 const compare=action.startsWith('compare-');
 const {jobs,year}=await stateRunOutputJobs(batch,inputs),row=batch.states.find(r=>r.state_code===state&&r.status==='ok'),job=jobs.find(j=>{const s=stateJobScope(j);return s.state===state&&s.year===year;});
 if(!row||!job)throw new Error('Select a completed state with matching retained jobs.');
 const run=row.result,prepared=run?.schema_version==='bisect-prepared-state-run-v1',scope=stateJobScope(job),districts=batch.selection.states.find(r=>r.state_code===state)?.districts;
 let sources,document,replay;
 if(prepared){
  if(run.scope?.state!==state||run.scope?.year!==year||run.scope?.districts!==districts)throw new Error('Prepared state scope differs from native selection.');
  const map=preparedEngineMapDocument(run.prepared);
  if(map.plan.units.state!==state||String(map.plan.units.year)!==year||map.plan.k!==districts)throw new Error('Prepared accepted assignments differ from State scope.');
  if(action==='replay')replay=await preparedEngineReplayInputs(run.prepared);
  else if(!compare||run.prepared.export&&!run.prepared.export_error)sources=preparedEnginePlanSources(run.prepared);
 }else{
  tuiRunAssignmentsText(run);sources=tuiRunSources(run);const output=run.practitioner_export;
  document=parseNativeMapPlanJson(output.document_json);inspectJsonText(output.context_json,4000000);const context=JSON.parse(output.context_json),request=run.configured_run.engine_request;
  if(run.form.location.toUpperCase()!==state||run.form.year!==year||document.plan.units.state!==state||String(document.plan.units.year)!==year||document.plan.k!==districts||scope.districts!==null&&scope.districts!==districts||!same(document,output.document)||!same(context,output.context)||!same(context.units,document.plan.units)||context.context_hash!==output.context_hash||!same(document.plan.units.unit_ids,request.graph.geoids)||request.graph.state!==state||String(request.graph.year)!==year||request.options.districts!==districts)throw new Error('Accepted native plan/context and configured State scope differ.');
  if(run.workflow){const w=run.workflow,p=await stateOutputSha(output.document_json),c=await stateOutputSha(output.context_json);if(w.schema_version!=='bisect-state-workflow-v1'||!['completed','partial'].includes(w.status)||w.accepted_plan_source_sha256!==p||w.accepted_context_source_sha256!==c||w.file_sha256?.['plan.rplan']!==p||w.file_sha256?.['context.rctx']!==c)throw new Error('Workflow accepted plan/context bytes changed.');}
  if(action==='replay')replay=engineReplayFromConfigured(run.configured_run);
 }
 if(compare){
  const side=action.slice(-1),comparison=prepared?{label:run.prepared.export?.document?.metadata?.label??state+' '+year,sources:{'data/final_assignments.json':{name:'final_assignments.json',base64:encodeBytes(new TextEncoder().encode(JSON.stringify(run.prepared.result.assignments,null,2)+'\n'))}}}:await tuiRunCompareSide(run);
  if(prepared&&run.workflow){const retained=await nativeStateFollowupSources(run);for(const name of ['compactness','summary','splits','contiguity'])if(retained.analysis[name+'.json'])comparison.sources['analysis/'+name+'.json']=retained.analysis[name+'.json'];}
  // Labels identify the declared scope; full original labels remain in retained
  // native plans. Users can edit the comparison labels before explicit Run.
  let label=state+' '+year+' · '+comparison.label;while(new TextEncoder().encode(label).length>200)label=Array.from(label).slice(0,-1).join('');comparison.label=label;
  const tuiCompareInputs=validateTuiCompareInputs({...TUI_COMPARE_DEFAULTS,...files.tuiCompareInputs,[side]:comparison});
  const exact=validateTextComparisonInputs({...TEXT_COMPARISON_DEFAULTS,...files.textComparisonInputs,[side]:sources?.auditPlanText??null});let exactCompatible=false;
  // Never carry a previous, unrelated context into a newly selected pair.
  if(exact.a&&exact.b){const a=parseNativeMapPlanJson(exactSourceText(exact.a)),b=parseNativeMapPlanJson(exactSourceText(exact.b));exactCompatible=sameComparisonUnits(a.plan.units,b.plan.units);}
  const priorContext=exact.context;exact.context=null;
  if(exactCompatible||Boolean(exact.a)!==Boolean(exact.b)){const units=parseNativeMapPlanJson(exactSourceText(exact.a??exact.b)).plan.units;for(const source of [priorContext,sources?.auditContextText]){if(!source)continue;const context=JSON.parse(exactContextText(source));if(sameComparisonUnits(context.units,units)){exact.context=source;break;}}}
  return {operation:'compare-tui-plans',files:{...files,tuiCompareInputs,textComparisonInputs:validateTextComparisonInputs(exact)},state,year,exactCompatible};
 }
 if(action==='replay')return {operation:'replay-engine-run',files:{...files,engineReplayInputs:replay},state,year};
 document??=parseNativeMapPlanJson(new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(Uint8Array.from(atob(sources.auditPlanText.base64),c=>c.charCodeAt(0))));
 const next={...files,...sources};delete next.auditCertificateText;delete next.auditLineageText;
 if(action==='reproducibility'){
  const originalLabel=prepared?document.metadata.label:run.form.label,label=files.reproducibilityInputs?.label??(/^[A-Za-z0-9_-]{1,120}$/.test(originalLabel)?originalLabel:state+'_'+year+'_plan');
  // Replace prior package sources completely; retain only checked outputs
  // from this accepted State workflow. Missing follow-ups remain absent.
  const selected=prepared?{inputs:validateReproducibilityInputs({label,...await nativeStateFollowupSources(run)})}:await runReproducibilityInputs(run,label);
  next.reproducibilityInputs=selected.inputs;reproducibilityRequest(next,selected.inputs);
  return {operation:'export-reproducibility-package',files:next,state,year};
 }
 if(action==='improve'){const selected=prepared?preparedEngineImprovementInputs(run.prepared,next):tuiRunImprovementInputs(run,next);return {operation:'improve-rplan-text',files:selected,state,year};}
 if(action==='doctor')next.doctorInputs=validateDoctorInputs({settings:{state,year,chamber:document.metadata.chamber,resolution:document.plan.units.unit_kind},policy:files.doctorInputs?.policy??null,context:sources.auditContextText});
 return {operation:action==='doctor'?'diagnose-location-inputs':'audit-rplan-text',files:next,state,year};
}
