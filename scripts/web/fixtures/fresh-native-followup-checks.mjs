import assert from 'node:assert/strict';
import {freshNativeStatePlanSources,freshNativeStateFollowupInputs} from '../../../web/lab/fresh-native-state-results.js';

export function verifyFreshNativeFollowups(run){
 const original=JSON.stringify(run),sources=freshNativeStatePlanSources(run);
 assert.equal(Buffer.from(sources.auditPlanText.base64,'base64').toString('utf8'),run.export.document_json);
 assert.equal(Buffer.from(sources.auditContextText.base64,'base64').toString('utf8'),run.export.context_json);
 for(const action of ['audit','analyze','improve']){
  const selected=freshNativeStateFollowupInputs(run,action,{auditCertificateText:{name:'stale'},auditLineageText:{name:'stale'},unrelated:'retained'});
  assert.equal(selected.unrelated,'retained');assert.equal(selected.auditCertificateText,undefined);assert.equal(selected.auditLineageText,undefined);
  assert.deepEqual(selected.auditPlanText,sources.auditPlanText);assert.deepEqual(selected.auditContextText,sources.auditContextText);
  if(action==='improve')assert.equal(selected.improvementSettings.tolerance_percent,(run.executed_request.request??run.executed_request).options.balance_tolerance);
 }
 for(const change of [
  r=>r.execution_performed=false,
  r=>r.recorded_paths_opened=true,
  r=>r.export_error='Native export failed',
  r=>r.input_sha256='0'.repeat(64),
  r=>r.export.context_json=r.export.context_json.replace('context_hash','altered_hash'),
  r=>r.result.assignments[Object.keys(r.result.assignments)[0]]=999,
  r=>r.export.document.plan.units.unit_kind='invented-kind',
  r=>{const d=JSON.parse(r.export.document_json);d.plan.assignment[0]=d.plan.assignment[0]===0?1:0;r.export.document=d;r.export.document_json=JSON.stringify(d);},
 ]){const changed=structuredClone(run);change(changed);assert.throws(()=>freshNativeStatePlanSources(changed));}
 assert.throws(()=>freshNativeStateFollowupInputs(run,'replay'));
 assert.equal(JSON.stringify(run),original);
 return {actions:3,refusals:9,exact_native_plan_context_bytes:true};
}
