import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {runStateBatch} from '../../web/lab/state-run-batch.js';
import {stateBatchNationalSources} from '../../web/lab/prepared-state-output.js';
import {executeNationalMapBatch} from '../../web/lab/national-map-batch.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const root=path.resolve('target/state-national-analysis-handoff');await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const source=JSON.parse(await fs.readFile('target/state-run-years-workflow/browser.bisect','utf8')),jobs=JSON.parse(Buffer.from(source.files.stateRunBatchInputs.jobs.base64,'base64'));
const kinds=['districts','political','demographic','compactness'];for(const job of jobs)job.workflow.map.map_types=kinds;
const inputs={...source.files.stateRunBatchInputs,jobs:{name:'SYNTHETIC-three-years-all-map-types.json',base64:Buffer.from(JSON.stringify(jobs)).toString('base64')}};
class Adapter extends EventTarget{
 postMessage(m){setImmediate(()=>{if(this.closed)return;try{this.dispatchEvent(new MessageEvent('message',{data:m.type==='initialize'?{type:'ready',runtimeArtifact:engine.artifactIdentity}:{type:'result',id:m.id,result:engine.execute(m.request)}}));}catch(e){this.dispatchEvent(new MessageEvent('message',{data:{type:'error',id:m.id,error:e.message}}));}});}
 terminate(){this.closed=true;}
}
const result=await runStateBatch(inputs,source.files.stateBatchInputs,{createWorker:()=>new Adapter(),hardwareConcurrency:4,wasmUrl:'local'});assert.equal(result.state_summary.succeeded,3);
let workflowMatches=0;
for(const row of result.years){const request=path.join(root,'workflow-'+row.year+'.json');await fs.writeFile(request,JSON.stringify({operation:'run-tui-configured',input:jobs.find(j=>j.form.year===row.year)}));const native=spawnSync(path.resolve('target/release/examples/execute_tool_request.exe'),[request],{encoding:'utf8',maxBuffer:64*1024*1024,timeout:120000});assert.equal(native.status,0,native.stderr);assert.deepEqual(row.result.states[0].result,JSON.parse(native.stdout));workflowMatches++;}
const sha=bytes=>createHash('sha256').update(bytes).digest('hex'),requests=[],replies=[];let refusals=0;
for(const row of result.years)for(const kind of kinds){
 const before=requests.length,selected=await stateBatchNationalSources(row.result,inputs,{map_type:kind});assert.equal(requests.length,before);assert.deepEqual(selected.states,['IA']);
 const batch=JSON.parse(Buffer.from(selected.inputs.source.base64,'base64'));assert.equal(batch.map_type,kind);
 const workflow=row.result.states[0].result.workflow;
 if(kind==='districts')assert.equal(batch.states.IA.analysis,null);
 else{assert.equal(batch.states.IA.analysis.base64,workflow.files_base64['analysis/'+kind+'.json']);assert.equal(sha(Buffer.from(batch.states.IA.analysis.base64,'base64')),selected.bindings.IA.analysis_source_sha256);}
 const rendered=await executeNationalMapBatch({get artifactIdentity(){return engine.artifactIdentity;},execute(r,options){requests.push(r);const reply=engine.execute(r,options);replies.push(reply);return reply;}},batch);
 assert.equal(rendered.map_type,kind);assert.equal(String(rendered.year),row.year);
 if(kind!=='districts')assert.equal(rendered.provided_states[0].analysis_source_sha256,selected.bindings.IA.analysis_source_sha256);
 const packet=createProject({name:'SYNTHETIC '+row.year+' '+kind+' national source handoff',operation:'render-native-national-map-batch',lastOperation:'render-native-national-map-batch',constraints:[],files:{nationalMapBatchInputs:selected.inputs},result:rendered});assert.deepEqual(parseProject(JSON.stringify(packet)),packet);await fs.writeFile(path.join(root,row.year+'-'+kind+'.bisect'),JSON.stringify(packet));
 if(kind!=='districts')for(const change of [
  w=>w.files_base64['analysis/'+kind+'.json']=Buffer.from('{}').toString('base64'),
  w=>w.file_sha256['analysis/'+kind+'.json']='0'.repeat(64),
  w=>w.stages.find(s=>s.name==='map:'+kind).receipt.analysis_source_sha256='0'.repeat(64),
  w=>w.stages.find(s=>s.name==='map:'+kind).status='unavailable'
 ]){const altered=structuredClone(row.result);change(altered.states[0].result.workflow);await assert.rejects(()=>stateBatchNationalSources(altered,inputs,{map_type:kind}),/No completed state/);refusals++;}
}
await assert.rejects(()=>stateBatchNationalSources(result.years[0].result,inputs,{map_type:'rounds'}),/map type/);refusals++;
const prepared=JSON.parse(await fs.readFile('target/prepared-state-batch/batch.bisect','utf8'));await assert.rejects(()=>stateBatchNationalSources(prepared.result,prepared.files.stateRunBatchInputs,{map_type:'political'}),/no completed native political/);refusals++;
const mixedInputs={workers:2,jobs:{name:'SYNTHETIC-mixed-map-availability.json',base64:Buffer.from(JSON.stringify([jobs[0],...JSON.parse(Buffer.from(prepared.files.stateRunBatchInputs.jobs.base64,'base64')).filter(j=>j.scope.state!=='IA')])).toString('base64')}};
// Execute configured IA with prepared RI/VT; the latter have no analysis maps.
const {executePreparedStateJob}=await import('../../web/lab/prepared-state-job.js');
class MixedAdapter extends EventTarget{
 postMessage(m){setImmediate(()=>{if(this.closed)return;try{const reply=m.type==='initialize'?{type:'ready',runtimeArtifact:engine.artifactIdentity}:{type:'result',id:m.id,result:m.request.operation==='run-prepared-state-job'?executePreparedStateJob(engine,m.request.input):engine.execute(m.request)};this.dispatchEvent(new MessageEvent('message',{data:reply}));}catch(e){this.dispatchEvent(new MessageEvent('message',{data:{type:'error',id:m.id,error:e.message}}));}});}
 terminate(){this.closed=true;}
}
const mixed=await runStateBatch(mixedInputs,{year:'2020',states:['IA','RI','VT'],manifest:null},{createWorker:()=>new MixedAdapter(),hardwareConcurrency:4,wasmUrl:'local'});assert.equal(mixed.summary.succeeded,3);
for(const kind of kinds.slice(1)){const selected=await stateBatchNationalSources(mixed,mixedInputs,{map_type:kind});assert.deepEqual(selected.states,['IA']);assert.deepEqual(selected.skipped.map(r=>r.state),['RI','VT']);const batch=JSON.parse(Buffer.from(selected.inputs.source.base64,'base64'));await executeNationalMapBatch({get artifactIdentity(){return engine.artifactIdentity;},execute(r,options){requests.push(r);const reply=engine.execute(r,options);replies.push(reply);return reply;}},batch);}
const sequence=path.join(root,'native-sequence.jsonl');await fs.writeFile(sequence,requests.map(r=>JSON.stringify(r)).join('\n')+'\n');const native=spawnSync(path.resolve('target/release/examples/execute_tool_sequence.exe'),[sequence],{encoding:'utf8',maxBuffer:128*1024*1024,timeout:120000});assert.equal(native.status,0,native.stderr);const actual=native.stdout.trim().split('\n').map(JSON.parse);assert.equal(actual.length,replies.length);for(const [i,r] of actual.entries()){assert(r.ok);assert.deepEqual(r.result,replies[i]);}
await fs.writeFile(path.join(root,'browser.bisect'),JSON.stringify(createProject({...source,files:{...source.files,stateRunBatchInputs:inputs},result})));
await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({wasm_sha256:engine.artifactIdentity.sha256,native_workflow_matches:workflowMatches,native_map_protocol_matches:replies.length,map_types:kinds,years:result.years.map(r=>r.year),analysis_bytes_exact:true,source_projects_roundtrip:true,mixed_unavailable_states_omitted:true,refusals,scope:'Synthetic supplied observations reused across three vintages. Whole native workflows and national renderer match; no source authenticity, historical capacity or partition certification claim.'},null,2)+'\n');console.log(`National analysis handoff: ${workflowMatches} whole native workflows, ${replies.length} native map replies across four types/three years, exact analysis bytes and ${refusals} refusals.`);
