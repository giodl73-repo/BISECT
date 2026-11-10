import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {nativeEngineInputRequest} from '../../web/lab/native-adjacency-files.js';
import {preparedStateJobFromResult,executePreparedStateJob,preparedStateRunZip} from '../../web/lab/prepared-state-job.js';
import {preparedEnginePlanSources,preparedEngineReplayInputs} from '../../web/lab/prepared-engine-results.js';
import {bindNativeReportExport} from '../../web/lab/native-report-export-binding.js';
import {createWorkbenchArchive,parseWorkbenchArchive} from '../../web/lab/workbench-archive.js';
import {nativeReportReference} from './fixtures/native-report-reference.mjs';

const root=await fs.mkdtemp(path.resolve('target/state-report-binding/node-')),sha=b=>createHash('sha256').update(b).digest('hex');
const wasm=process.env.BISECT_TEST_WASM??'target/state-report-binding/default.wasm',wasmBytes=await fs.readFile(wasm),engine=await instantiateEngine(wasmBytes);
let partitions=0;const facade={artifactIdentity:engine.artifactIdentity,execute(r,o){if(r.operation==='run-prepared-engine'&&!r.input.inspect_only)partitions++;return engine.execute(r,o);}};
const profile=process.env.BISECT_REPORT_PROFILE??'default',references=await nativeReportReference(profile),{sourceRoot,native}=references,sourceEvidence={native_reference:references.reference};
const fixture=JSON.parse(await fs.readFile('scripts/web/fixtures/native-state-source.json','utf8')),source=engine.execute(nativeEngineInputRequest(fixture.nativeAdjacencyInputs,fixture.nativeAdjacencySettings));
const args=['--state','RI','--year','2020','--districts','2','--metis-engine','metis-core','--structure','nway','--search','single','--seed','18446744073709551615','--balance-tolerance','5'];
async function run(label){
 const settings={label,chamber:'congressional',created_at:'2026-10-27T00:00:00Z',inspect_only:true};
 const compiled=engine.execute({operation:'prepare-native-state-graph-run',input:{arguments:args,graph_json:source.graph_json,graph_source_name:'native-fixture.json',label,chamber:settings.chamber,created_at:settings.created_at}});
 const inputs={source:{name:'native-compiled.json',text:compiled.prepared_request_json},settings};
 const outer=executePreparedStateJob(facade,await preparedStateJobFromResult(inputs,compiled.prepared_review));
 return {outer,inputs};
}
const outcomes=[];let saved;
for(const row of native.outcomes){
 const snapshot=JSON.parse(await fs.readFile(path.join(sourceRoot,row.name+'.json'),'utf8')),label=snapshot.receipt.identity.effective_label;
 const {outer,inputs}=await run(label),accepted=outer.prepared,before=JSON.stringify(accepted),count=partitions;
 const bound=await bindNativeReportExport(facade,accepted,snapshot);assert.equal(partitions,count);assert.equal(JSON.stringify(accepted),before);
 assert.deepEqual(bound.executed_request,accepted.executed_request);assert.deepEqual(bound.result,accepted.result);assert.equal(bound.replay_artifact_json,accepted.replay_artifact_json);assert.equal(bound.replay_artifact_sha256,accepted.replay_artifact_sha256);
 if(row.expected_native_failure){assert.equal(bound.export,null);assert(bound.export_error.includes('Native report source failed'));assert.throws(()=>preparedEnginePlanSources(bound),/no physical/);}
 else{
  const original=JSON.parse(accepted.export.context_json),context=JSON.parse(bound.export.context_json),plan=JSON.parse(bound.export.document_json),expected=JSON.parse(await fs.readFile(row.files.find(f=>f.path.endsWith('/context.rctx')).path,'utf8'));
  for(const field of ['units','graph','populations','subdivisions','geometry'])assert.deepEqual(context[field],original[field],row.name+' '+field);
  assert.deepEqual(plan.plan,accepted.export.document.plan);assert.deepEqual(context.demographics??null,expected.demographics??null);assert.deepEqual(context.populations,expected.populations);
  assert.notEqual(context.context_hash,original.context_hash);assert.equal(bound.report_source_binding.receipt.graph_binding_verified,true);
  const manifest=engine.execute({operation:'prepare-plan-report-manifest',document_json:bound.export.document_json,context_json:bound.export.context_json});
  assert.match(manifest.manifest_json,/"engine_seed":\s*18446744073709551615\b/);const metadata=JSON.parse(manifest.manifest_json);assert.equal(metadata.population_source,row.requested);assert.equal(metadata.context_hash,context.context_hash);
  await fs.writeFile(path.join(root,row.name+'-manifest.json'),manifest.manifest_json);await fs.writeFile(path.join(root,row.name+'-context.rctx'),bound.export.context_json);await fs.writeFile(path.join(root,row.name+'-plan.rplan'),bound.export.document_json);
  const replay=await preparedEngineReplayInputs(bound);assert.equal(replay.minSimilarity,1);
  await fs.writeFile(path.join(root,row.name+'.zip'),preparedStateRunZip({...outer,prepared:bound}));
 }
 const archive=createWorkbenchArchive({name:row.name,files:{preparedEngineInputs:inputs},operation:'run-prepared-engine',result:bound,lastOperation:'run-prepared-engine'}),opened=parseWorkbenchArchive(archive);assert.deepEqual(opened.result,bound);
 await fs.writeFile(path.join(root,row.name+'.bisect'),JSON.stringify(archive));await fs.writeFile(path.join(root,row.name+'-binding.json'),JSON.stringify(bound.report_source_binding));
 outcomes.push({name:row.name,native_report_counts_or_failure_match:true,accepted_generation_and_replay_unchanged:true,input_unchanged:true,additional_partitions:partitions-count,archive_result_roundtrip:true,expected_native_failure:row.expected_native_failure});
 if(row.name==='total')saved={accepted,snapshot,bound};
}
const {accepted,snapshot,bound}=saved;
const forged=structuredClone(snapshot);forged.receipt.identity.policy_sha256='sha256:'+'0'.repeat(64);await assert.rejects(()=>bindNativeReportExport(facade,accepted,forged),/differs/);
const modified=structuredClone(snapshot),entry=modified.request.observations.find(r=>r.kind==='missing');entry.kind='file';entry.source_base64=Buffer.from('GEOID,total_vap,minority_vap\n').toString('base64');await assert.rejects(()=>bindNativeReportExport(facade,accepted,modified),/differs/);
const wrong=await run('wrong_label');await assert.rejects(()=>bindNativeReportExport(facade,wrong.outer.prepared,snapshot),/scope differs/);
const changedDocument=structuredClone(bound.export.document);changedDocument.provenance.producer.native_report_source_binding.context_hash='stale';assert.throws(()=>engine.execute({operation:'prepare-plan-report-manifest',document_json:JSON.stringify(changedDocument),context_json:bound.export.context_json}),/provenance differs/);
const rebound=await bindNativeReportExport(facade,bound,snapshot);assert.deepEqual(rebound.export.context.populations,accepted.export.context.populations);assert.equal(rebound.export.context_hash,bound.export.context_hash);
const cancel=new AbortController();await assert.rejects(()=>bindNativeReportExport({async execute(r){const response=engine.execute(r);cancel.abort(Error('Cancelled binding'));return response;}},accepted,snapshot,{signal:cancel.signal}),/Cancelled binding/);
let current=true;await assert.rejects(()=>bindNativeReportExport({async execute(r){const response=engine.execute(r);current=false;return response;}},accepted,snapshot,{isCurrent:()=>current}),/request changed/);
const mutable=structuredClone(snapshot),captured=structuredClone(snapshot);const isolated=await bindNativeReportExport({async execute(r){const response=engine.execute(r);mutable.request.observations.length=0;return response;}},accepted,mutable);assert.deepEqual(isolated.report_source_binding.source_snapshot,captured);
const evidence={status:'passed-accepted-export-binding',profile,wasm:{path:wasm,sha256:sha(wasmBytes),byte_length:wasmBytes.length},source_preparation:references.sourcePreparation,native_reference:sourceEvidence.native_reference,outcomes,source_or_policy_changes_refused:true,wrong_label_refused:true,stale_manifest_provenance_refused:true,explicit_rebind_preserves_graph:true,post_await_cancel_and_generation_change_refused:true,source_snapshot_isolated_from_caller_mutation:true,automated_state_flag_workflow:false,actual_browser_ui:false,scope:'Actual accepted prepared nway runs followed by report-source binding. Eleven retained native source outcomes compare counts/failures; successful bound exports preserve graph/populations/assignments and exact seed text, regenerate matching research manifests and retain replay/result-only archive/ZIP artifacts. Counts binding executes no further partition. Original State population-source flag handoff, automatic jobs, input adoption, actual UI and full package/release acceptance remain open.'};
await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify(evidence,null,2));console.log(JSON.stringify({root,status:evidence.status,cases:outcomes.length}));
