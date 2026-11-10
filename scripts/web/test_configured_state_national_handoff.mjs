import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {stateBatchNationalSources,preparedStateBatchNationalSources} from '../../web/lab/prepared-state-output.js';
import {executeNationalMapBatch} from '../../web/lab/national-map-batch.js';
import {executePreparedStateJob} from '../../web/lab/prepared-state-job.js';
import {runStateBatch} from '../../web/lab/state-run-batch.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const root=path.resolve('target/configured-state-national-handoff');await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const sha=value=>createHash('sha256').update(value).digest('hex');
const workflow=JSON.parse(await fs.readFile('target/state-run-years-workflow/browser.bisect','utf8'));
const requests=[],results=[],records=[];
const render=async selected=>{
 const batch=JSON.parse(Buffer.from(selected.inputs.source.base64,'base64'));
 return executeNationalMapBatch({get artifactIdentity(){return engine.artifactIdentity;},execute(request,options){requests.push(request);const result=engine.execute(request,options);results.push(result);return result;}},batch);
};
for(const row of workflow.result.years){
 const selected=await stateBatchNationalSources(row.result,workflow.files.stateRunBatchInputs);
 assert.deepEqual(selected.states,['IA']);assert.deepEqual(selected.skipped,[]);
 const job=JSON.parse(Buffer.from(workflow.files.stateRunBatchInputs.jobs.base64,'base64')).find(j=>j.form.year===row.year);
 const batch=JSON.parse(Buffer.from(selected.inputs.source.base64,'base64'));
 assert.equal(Buffer.from(batch.states.IA.geometry.base64,'base64').toString(),job.workflow.map.unit_geometry_json);
 assert.equal(Buffer.from(batch.states.IA.plan.base64,'base64').toString(),row.result.states[0].result.practitioner_export.document_json);
 assert.equal(selected.bindings.IA.geometry_source_sha256,sha(job.workflow.map.unit_geometry_json));
 const map=await render(selected);assert(map);assert.equal(String(map.year),row.year);
 assert.equal(map.provided_states[0].geometry_source_sha256,selected.bindings.IA.geometry_source_sha256);
 records.push({year:row.year,states:selected.states,bindings:selected.bindings,png_sha256:map.png_sha256});
 const packet=createProject({name:'SYNTHETIC configured native national '+row.year,operation:'render-native-national-map-batch',lastOperation:'render-native-national-map-batch',constraints:[],files:{nationalMapBatchInputs:selected.inputs},result:map});
 assert.deepEqual(parseProject(JSON.stringify(packet)),packet);await fs.writeFile(path.join(root,row.year+'.bisect'),JSON.stringify(packet));
}
const sourceBatch=JSON.parse(await fs.readFile('target/state-source-batch/browser.bisect','utf8'));
const census=await stateBatchNationalSources(sourceBatch.result,sourceBatch.files.stateRunBatchInputs);
assert.deepEqual(census.states,['RI']);assert.equal(census.bindings.RI.geometry_source_sha256,sha(sourceBatch.result.states[0].source_preparation.geometry_json));
await render(census);
const prepared=JSON.parse(await fs.readFile('target/prepared-state-batch/batch.bisect','utf8'));
const plain=await stateBatchNationalSources(prepared.result,prepared.files.stateRunBatchInputs);
assert.deepEqual(plain.inputs,preparedStateBatchNationalSources(prepared.result).inputs);
const directJobs=JSON.parse(Buffer.from(workflow.files.stateRunBatchInputs.jobs.base64,'base64')),configured=directJobs.find(j=>j.form.year==='2020');
const preparedJobs=JSON.parse(Buffer.from(prepared.files.stateRunBatchInputs.jobs.base64,'base64')).filter(j=>j.scope.state!=='IA');
const mixedInputs={workers:2,jobs:{name:'mixed-state-maps.json',base64:Buffer.from(JSON.stringify([configured,...preparedJobs])).toString('base64')}};
class Adapter extends EventTarget{
 postMessage(m){setImmediate(()=>{if(this.closed)return;try{const result=m.type==='initialize'?null:m.request.operation==='run-prepared-state-job'?executePreparedStateJob(engine,m.request.input):engine.execute(m.request);this.dispatchEvent(new MessageEvent('message',{data:m.type==='initialize'?{type:'ready',runtimeArtifact:engine.artifactIdentity}:{type:'result',id:m.id,result}}));}catch(e){this.dispatchEvent(new MessageEvent('message',{data:{type:'error',id:m.id,error:e.message}}));}});}
 terminate(){this.closed=true;}
}
const mixed=await runStateBatch(mixedInputs,{year:'2020',states:['IA','RI','VT'],manifest:null},{createWorker:()=>new Adapter(),hardwareConcurrency:4,wasmUrl:'local'});
const mixedSources=await stateBatchNationalSources(mixed,mixedInputs);assert.deepEqual(mixedSources.states,['IA','RI','VT']);assert.deepEqual(mixedSources.skipped,[]);await render(mixedSources);
let refusals=0;
const first=workflow.result.years[0].result,inputs=workflow.files.stateRunBatchInputs;
for(const change of [
 r=>r.states[0].result.practitioner_export.document_json+=' ',
 r=>r.states[0].result.practitioner_export.context_json+=' ',
 r=>{const id=Object.keys(r.states[0].result.configured_run.result.assignments)[0];r.states[0].result.configured_run.result.assignments[id]=99;},
 r=>r.states[0].result.workflow.stages.find(s=>s.name==='map:districts').receipt.geometry_source_sha256='0'.repeat(64),
 r=>r.states[0].result.workflow.stages.find(s=>s.name==='map:districts').receipt.plan_hash='sha256:'+'0'.repeat(64),
 r=>r.states[0].result.workflow.stages.find(s=>s.name==='map:districts').receipt.year=2010,
 r=>r.states[0].result.workflow.stages.find(s=>s.name==='map:districts').receipt.dissolve_method='batch',
 r=>r.states[0].result.workflow.stages.find(s=>s.name==='map:districts').receipt.dpi=300,
 r=>r.states[0].result.workflow.stages.find(s=>s.name==='map:districts').status='failed',
 r=>r.states[0].result.workflow.accepted_context_source_sha256='0'.repeat(64)
]){const value=structuredClone(first);change(value);await assert.rejects(()=>stateBatchNationalSources(value,inputs),/No completed state/);refusals++;}
const changedInputs=structuredClone(inputs),changedJobs=JSON.parse(Buffer.from(changedInputs.jobs.base64,'base64'));changedJobs[0].form.label='Changed';changedInputs.jobs.base64=Buffer.from(JSON.stringify(changedJobs)).toString('base64');
await assert.rejects(()=>stateBatchNationalSources(first,changedInputs),/job bytes differ/);refusals++;
const wrongName=structuredClone(mixedInputs);wrongName.jobs.name='different.json';await assert.rejects(()=>stateBatchNationalSources(mixed,wrongName),/job bytes differ/);refusals++;
const partial=structuredClone(mixed);partial.states.find(row=>row.state_code==='IA').result.workflow.stages.find(s=>s.name==='map:districts').receipt.geometry_source_sha256='0'.repeat(64);
const partialSources=await stateBatchNationalSources(partial,mixedInputs);assert.deepEqual(partialSources.states,['RI','VT']);assert.equal(partialSources.skipped[0].state,'IA');
const sequence=path.join(root,'native-sequence.jsonl');await fs.writeFile(sequence,requests.map(r=>JSON.stringify(r)).join('\n')+'\n');
const native=spawnSync(path.resolve('target/release/examples/execute_tool_sequence.exe'),[sequence],{encoding:'utf8',maxBuffer:128*1024*1024,timeout:120000});assert.equal(native.status,0,native.stderr);
const replies=native.stdout.trim().split('\n').map(JSON.parse);assert.equal(replies.length,results.length);for(const [i,r]of replies.entries()){assert(r.ok);assert.deepEqual(r.result,results[i]);}
await fs.writeFile(path.join(root,'mixed.bisect'),JSON.stringify(createProject({name:'SYNTHETIC mixed configured and prepared state maps',operation:'run-state-batch',lastOperation:'run-state-batch',constraints:[],files:{stateRunBatchInputs:mixedInputs,stateBatchInputs:{year:'2020',states:['IA','RI','VT'],manifest:null}},result:mixed})));
await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({wasm_sha256:engine.artifactIdentity.sha256,native_sequence_sha256:sha(await fs.readFile('target/release/examples/execute_tool_sequence.exe')),native_map_replies:results.length,configured_years:3,census_source_geometry:true,mixed_configured_prepared:true,legacy_prepared_package_unchanged:true,refusals,partial_bad_state_omitted:true,exact_plan_and_geometry_bytes:true,no_engine_or_render_call_during_source_selection:true,records,scope:'Native-rendered synthetic configured and prepared/source snapshots; actual native national renderer comparisons. No source authenticity, historical original-data capacity or partition certification.'},null,2)+'\n');
console.log(`Configured national handoff:three years, Census and mixed snapshots; ${results.length} exact native map replies, ${refusals} binding refusals, exact sources and legacy prepared compatibility passed.`);

