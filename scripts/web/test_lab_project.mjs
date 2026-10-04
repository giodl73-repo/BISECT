import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
import {createLabProject,parseLabProject,validateLabConfig} from '../../web/lab/laboratory-project.js';
const root=process.argv[2]||'dist/wasm-national';
const manifest=JSON.parse(await fs.readFile(path.join(root,'catalog.json'),'utf8'));
const fetcher=async name=>{const bytes=await fs.readFile(path.join(root,name));return{ok:true,arrayBuffer:async()=>bytes.buffer.slice(bytes.byteOffset,bytes.byteOffset+bytes.length)};};
const engine=await instantiateEngine(await fs.readFile(path.join(root,'bisect_wasm.wasm')));
let executions=0;
class Worker {
  postMessage(message){setImmediate(()=>{if(this.terminated)return;try{if(message.type==='initialize')this.onmessage({data:{type:'ready'}});else{executions++;this.onmessage({data:{type:'result',id:message.id,result:engine.execute(message.request),memoryBytes:engine.memoryBytes()}});}}catch(error){this.onmessage({data:{type:'error',id:message.id,error:error.message}});}});}
  terminate(){this.terminated=true;}
}
const testYear=process.argv[4]||'2020', testStates=(process.argv[3]||'RI,IA').split(',').slice(0,2), primary=testStates[0];
const config={name:'Portable national experiment',mode:'national',states:testStates,year:testYear,chamber:'congressional',districts:null,structure:'standard-bisect',weights:'county',search:'multi',seed:42,seeds:8,steps:20,percentile:0,alpha_county:2,balance_tolerance:5,area_swing:1.1,iterations:10,timeout_seconds:30};
const lab=new WasmCatalog(manifest,fetcher,()=>new Worker());
const run=lab.submit(config);while(lab.active)await new Promise(resolve=>setTimeout(resolve,5));
const actual=await lab.api('/api/runs/'+run.id);assert.equal(actual.status,'completed');
const project=await createLabProject(lab,run.id);
assert.deepEqual(parseLabProject(JSON.stringify(project)),project);
const before=executions;
const restoredLab=new WasmCatalog(manifest,fetcher,()=>{throw new Error('Opening a project must not start a Worker.');});
const restored=await restoredLab.restoreProject(project);
assert.equal(executions,before);assert.equal(restoredLab.worker,null);assert.equal(restored.restored_from_project,true);
assert.notEqual(restored.id,run.id);assert.deepEqual(restored.config,config);
for(const state of restored.states){
  // JSON project files normalize -0 to 0; compare the serialized contract.
  assert.deepEqual(state.metrics,JSON.parse(JSON.stringify(actual.states.find(s=>s.code===state.code).metrics)));
  const map=await restoredLab.api(`/api/runs/${restored.id}/${state.code}/map`);
  assert.equal(map.features.length,state.metrics.units);
  assert.ok(map.features.every(feature=>feature.properties.district===project.assignments[state.code][feature.properties.geoid]));
  assert.ok(!restoredLab.assets.has(manifest.graphs[`${state.code}:${testYear}`].graph_ref));
}
const reopened=await createLabProject(restoredLab,restored.id);
assert.deepEqual(reopened.assignments,project.assignments);
assert.deepEqual(reopened.experiment.states,project.experiment.states);
const copy=()=>structuredClone(project), count=restoredLab.jobs.size, outputCount=restoredLab.outputs.size;
for(const tamper of [
  p=>p.experiment.states[0].metrics.population++,
  p=>p.experiment.states[0].metrics.graph_boundary_m++,
  p=>p.experiment.states[0].metrics.districts[0].components++,
  p=>{const id=Object.keys(p.assignments[primary])[0];p.assignments[primary][id]=p.assignments[primary][id]===1?2:1;},
  p=>p.inputs[primary].geometry_sha256='0'.repeat(64),
]){
  const bad=copy();tamper(bad);await assert.rejects(restoredLab.restoreProject(bad),/disagree|different|Inconsistent/);
  assert.equal(restoredLab.jobs.size,count);assert.equal(restoredLab.outputs.size,outputCount);
}
const mismatched=copy();mismatched.experiment.states[0].metrics.recorded_options.seed++;
assert.throws(()=>parseLabProject(JSON.stringify(mismatched)),/options/);
for(const change of [{seed:NaN},{steps:100001},{states:[primary,primary]},...(testStates.length>1?[{mode:'state'}]:[]),{districts:2}])assert.throws(()=>validateLabConfig({...config,...change}));
for(const code of [[primary],{toString:()=>primary},null,42])assert.throws(()=>validateLabConfig({...config,mode:'state',states:[code]}),/state selection/);
assert.throws(()=>parseLabProject(JSON.stringify({...project,execute_url:'https://attacker.invalid/'})),/format/);
const unsafe=copy();unsafe.experiment.config=JSON.parse(JSON.stringify(config).replace('"name":','"__proto__":{},"name":'));
assert.throws(()=>parseLabProject(JSON.stringify(unsafe)),/Unsafe/);

// Preserve historical provenance without treating a user-controlled hash as proof.
const provenance=copy();provenance.experiment.states[0].metrics.engine_provenance.wasm_sha256='1'.repeat(64);
const historical=await restoredLab.restoreProject(provenance);
assert.equal(historical.restored_from_project,true);
assert.equal(historical.states[0].metrics.engine_provenance.wasm_sha256,'1'.repeat(64));
assert.match(historical.logs.at(-1),/unverified imported claim/);

// Atomic restore: cancellation or editing during input reads must not commit.
let release, reading=false;
const heldFetcher=async name=>{reading=true;await new Promise(resolve=>release=resolve);return fetcher(name);};
const pendingLab=new WasmCatalog(manifest,heldFetcher,()=>{throw new Error('No Worker expected');});
const controller=new AbortController(), pending=pendingLab.restoreProject(project,{signal:controller.signal});
while(!reading)await new Promise(resolve=>setTimeout(resolve,1));controller.abort();release();
await assert.rejects(pending,/cancelled/);assert.equal(pendingLab.jobs.size,manifest.runs.length);assert.equal(pendingLab.outputs.size,0);
let current=true;reading=false;
const changed=pendingLab.restoreProject(project,{isCurrent:()=>current});
while(!reading)await new Promise(resolve=>setTimeout(resolve,1));current=false;release();
await assert.rejects(changed,/changed/);assert.equal(pendingLab.outputs.size,0);

// A snapshot of an unfinished national run retains completed states and makes
// unfinished states explicit; loading does not silently resume a computation.
const unfinished=structuredClone(actual);unfinished.id='unfinished-test';unfinished.status='running';const pendingIndex=unfinished.states.length-1;unfinished.states[pendingIndex]={code:unfinished.states[pendingIndex].code,status:'queued',metrics:null,error:null,elapsed_seconds:0,command:[]};
lab.jobs.set(unfinished.id,unfinished);if(pendingIndex>0)lab.outputs.set(`${unfinished.id}:${primary}`,project.assignments[primary]);
const interrupted=await createLabProject(lab,unfinished.id);
assert.equal(interrupted.experiment.status,'interrupted');if(pendingIndex>0)assert.equal(interrupted.experiment.states[0].status,'completed');assert.equal(interrupted.experiment.states[pendingIndex].status,'interrupted');
assert.deepEqual(Object.keys(interrupted.assignments),pendingIndex>0?[primary]:[]);
const partial=await restoredLab.restoreProject(interrupted);assert.equal(partial.states[pendingIndex].metrics,null);assert.equal(executions,before);
if(manifest.catalog.structures.includes('ratio-optimal-area')){
  const area=lab.submit({...config,structure:'ratio-optimal-area',weights:'geographic',alpha_county:0,seeds:32,iterations:20});
  while(lab.active)await new Promise(resolve=>setTimeout(resolve,5));
  const areaProject=await createLabProject(lab,area.id);assert.equal(areaProject.experiment.status,'completed');
  await restoredLab.restoreProject(areaProject);
  const tampered=structuredClone(areaProject);tampered.experiment.states[0].metrics.root_split.area_left_m2++;
  await assert.rejects(restoredLab.restoreProject(tampered),/root-split metrics/);
  const missing=structuredClone(areaProject);delete missing.experiment.states[0].metrics.root_split;
  await assert.rejects(restoredLab.restoreProject(missing),/omits root/);
}
await fs.writeFile('target/wasm-laboratory-project-fixture.bisect',JSON.stringify(project));
console.log('Laboratory projects: actual WASM county/Multi roundtrip, maps, hashes, independent metrics, tamper rejection, atomic cancellation/edit guards, historical provenance and unfinished snapshots passed; opening executes no engine.');
