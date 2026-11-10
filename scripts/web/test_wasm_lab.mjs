import {planMapModel} from '../../web/lab/plan-map.js';
import fs from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
import {mkdirSync,writeFileSync} from 'node:fs';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
const root=process.argv[2]||'dist/wasm-laboratory';
const manifest=JSON.parse(await fs.readFile(path.join(root,'catalog.json'),'utf8'));
const engine=await instantiateEngine(await fs.readFile(path.join(root,'bisect_wasm.wasm')));
const evidenceRoot=process.env.BISECT_LAB_EVIDENCE_ROOT||'target/laboratory-integration';
mkdirSync(evidenceRoot,{recursive:true});
let nativeComparisons=0;const quality=[];
const fetcher=async name=>{const bytes=await fs.readFile(path.join(root,name));return{ok:true,arrayBuffer:async()=>bytes.buffer.slice(bytes.byteOffset,bytes.byteOffset+bytes.length)};};
class Worker{
  postMessage(message){setImmediate(()=>{if(this.terminated)return;try{if(message.type==='initialize'){this.onmessage({data:{type:'ready'}});return;}const result=engine.execute(message.request);if(message.request.graph&&message.request.options){const name=String(nativeComparisons).padStart(3,'0'),requestFile=path.resolve(evidenceRoot,name+'-request.json');writeFileSync(requestFile,JSON.stringify(message.request));const native=execFileSync(path.resolve(process.env.BISECT_LAB_NATIVE||'target/release/examples/execute_request.exe'),[requestFile],{maxBuffer:96*1024*1024});writeFileSync(path.join(evidenceRoot,name+'-native.json'),native);writeFileSync(path.join(evidenceRoot,name+'-wasm.json'),JSON.stringify(result));assert.deepEqual(result,JSON.parse(native));nativeComparisons++;}this.onmessage({data:{type:'result',id:message.id,result,memoryBytes:engine.memoryBytes()}});}catch(error){this.onmessage({data:{type:'error',id:message.id,error:error.message}});}});}
  terminate(){this.terminated=true;}
}
const lab=new WasmCatalog(manifest,fetcher,()=>new Worker());await lab.initialize();
async function verifyQuality(id,state,config){
 const ref=manifest.graphs[`${state.code}:${config.year}`].graph_ref,graph=await lab.asset(ref),assignments=await lab.api(`/api/runs/${id}/${state.code}/assignments`),k=state.metrics.district_count;
 assert.deepEqual(Object.keys(assignments).sort(),[...graph.geoids].sort());
 const labels=graph.geoids.map(g=>assignments[g]);assert.deepEqual([...new Set(labels)].sort((a,b)=>a-b),Array.from({length:k},(_,i)=>i+1));
 const populations=Array(k).fill(0),units=Array(k).fill(0),components=Array(k).fill(0),adjacency=graph.adjacency.map(ns=>new Set(ns));
 graph.adjacency.forEach((ns,u)=>ns.forEach(v=>{assert(v>=0&&v<labels.length);adjacency[v].add(u);}));
 labels.forEach((label,i)=>{populations[label-1]+=graph.population[i];units[label-1]++;});
 const seen=new Set();labels.forEach((label,u)=>{if(seen.has(u))return;components[label-1]++;seen.add(u);const stack=[u];while(stack.length)for(const v of adjacency[stack.pop()])if(!seen.has(v)&&labels[v]===label){seen.add(v);stack.push(v);}});
 const total=populations.reduce((a,b)=>a+b,0),deviation=Math.max(...populations.map(p=>Math.abs(p*k-total)*100/total));
 assert.equal(total,state.metrics.population);assert.equal(labels.length,state.metrics.units);
 assert(Math.abs(state.metrics.max_deviation_percent-deviation)<1e-9);
 assert.equal(state.metrics.balance_passed,deviation<=config.balance_tolerance);
 assert.equal(state.metrics.contiguous,components.every(n=>n===1));
 for(const district of state.metrics.districts){const i=district.district-1;assert.deepEqual([district.population,district.units,district.components],[populations[i],units[i],components[i]]);}
 quality.push({state:state.code,structure:config.structure,search:config.search,deviation,balance_passed:state.metrics.balance_passed,contiguous:state.metrics.contiguous});
 lab.assets.delete(ref);
}
const testYear=process.argv[4]||'2020', testStates=(process.argv[3]||'RI,IA,NC').split(',');
const config={name:'WASM national test',mode:'national',states:testStates,year:testYear,chamber:'congressional',districts:null,structure:'standard-bisect',weights:'geographic',search:'single',seed:42,seeds:2,steps:20,percentile:0,alpha_county:0,balance_tolerance:5,area_swing:1.1,iterations:10,timeout_seconds:30};
const job=await lab.api('/api/runs',{method:'POST',body:JSON.stringify(config)});
while(lab.active)await new Promise(resolve=>setTimeout(resolve,10));
const result=await lab.api('/api/runs/'+job.id);
assert.equal(result.status,'completed',JSON.stringify(result.states.map(s=>s.error)));
for(const state of result.states){
 assert.ok(state.metrics.wasm_memory_bytes<=128*1024*1024,'pilot WASM memory bound');
 assert.equal(state.metrics.contiguous,true,state.code);
 if(['RI','IA','NC'].includes(state.code))assert.equal(state.metrics.balance_passed,true,state.code+' pilot');
 const map=await lab.api(`/api/runs/${job.id}/${state.code}/map`), assignments=await lab.api(`/api/runs/${job.id}/${state.code}/assignments`);
 assert.equal(map.features.length,Object.keys(assignments).length);
 assert.ok(map.features.every(f=>f.properties.district===assignments[f.properties.geoid]));
 assert.ok(!lab.assets.has(manifest.graphs[`${state.code}:${testYear}`].graph_ref),'working graph must be released');
 await verifyQuality(job.id,state,config);
}
const practitioner=await lab.exportPractitionerProject(job.id,testStates[0]);
assert.equal(practitioner.project.operation,'validate-rplan-text');
const exportedGeometry=JSON.parse(Buffer.from(practitioner.project.files.planGeometry.base64,'base64').toString());
assert.equal(planMapModel(practitioner.exported.document,exportedGeometry).units,practitioner.exported.document.plan.assignment.length);
assert.equal(typeof practitioner.exported.document_json,'string');assert.equal(typeof practitioner.exported.context_json,'string');
assert.equal(Buffer.from(practitioner.project.files.auditPlanText.base64,'base64').toString(),practitioner.exported.document_json);
assert.equal(Buffer.from(practitioner.project.files.auditContextText.base64,'base64').toString(),practitioner.exported.context_json);
assert.equal(engine.execute({operation:'sample-smc-context',context_json:practitioner.exported.context_json,settings:{k:1,particles:1,population_tolerance:0,resample_threshold:0.5,base_seed:'42'}}).base_seed,'42');
for(const code of testStates.slice(1)){
 const portable=await lab.exportPractitionerProject(job.id,code);
 const geo=JSON.parse(Buffer.from(portable.project.files.planGeometry.base64,'base64').toString());
 assert.equal(planMapModel(portable.exported.document,geo).units,portable.exported.document.plan.assignment.length);
 assert.equal(portable.project.operation,'validate-rplan-text');
}
await fs.writeFile('target/laboratory-map-handoff.bisect',JSON.stringify(practitioner.project));
const multi=lab.submit({...config,states:testStates.slice(0,2),search:'multi',seeds:8});
while(lab.active)await new Promise(resolve=>setTimeout(resolve,10));
const multiResult=await lab.api('/api/runs/'+multi.id);
assert.equal(multiResult.status,'completed');
for(const state of multiResult.states){
 assert.equal(state.metrics.contiguous,true);
 if(['RI','IA','NC'].includes(state.code))assert.equal(state.metrics.balance_passed,true,state.code+' multi pilot');
 assert.equal(state.metrics.recorded_options.search,'multi');
 assert.equal(state.metrics.recorded_options.seeds,8);
 await verifyQuality(multi.id,state,{...config,search:'multi',seeds:8});
}
const nway=lab.submit({...config,structure:'nway'});
while(lab.active)await new Promise(resolve=>setTimeout(resolve,10));
const nwayResult=await lab.api('/api/runs/'+nway.id);assert.equal(nwayResult.status,'completed');
for(const state of nwayResult.states){
 assert.equal(state.metrics.contiguous,true,state.code+' nway');
 await verifyQuality(nway.id,state,{...config,structure:'nway'});
}
assert.throws(()=>lab.submit({...config,states:['XX']}),/unavailable/);
assert.throws(()=>lab.submit({...config,states:[testStates[0],testStates[0]]}),/selection/);
const saved=manifest.runs.find(run=>run.states.some(s=>s.metrics));
const savedState=saved.states.find(s=>s.metrics);assert.ok((await lab.api(`/api/runs/${saved.id}/${savedState.code}/map`)).features.length);
const negative=new WasmCatalog(manifest,async()=>({ok:true,arrayBuffer:async()=>new TextEncoder().encode('{}').buffer}),()=>new Worker());
const bad=await negative.api('/api/runs',{method:'POST',body:JSON.stringify({...config,states:[testStates[0]]})});
while(negative.active)await new Promise(resolve=>setTimeout(resolve,10));
assert.equal((await negative.api('/api/runs/'+bad.id)).status,'failed');
const cancelled=lab.submit(config);await lab.api(`/api/runs/${cancelled.id}/cancel`,{method:'POST'});
while(lab.active)await new Promise(resolve=>setTimeout(resolve,10));
assert.equal((await lab.api('/api/runs/'+cancelled.id)).status,'cancelled');
assert.equal(lab.worker,null);
let heldWorker;
const stalled=new WasmCatalog(manifest,fetcher,()=>{
 heldWorker=new Worker();const post=heldWorker.postMessage.bind(heldWorker);heldWorker.postMessage=message=>{if(message.type==='initialize')post(message);};return heldWorker;
});await stalled.initialize();
const active=stalled.submit(config);
while(!stalled.pending)await new Promise(resolve=>setTimeout(resolve,1));
await stalled.api(`/api/runs/${active.id}/cancel`,{method:'POST'});
while(stalled.active)await new Promise(resolve=>setTimeout(resolve,1));
assert.equal(heldWorker.terminated,true);
assert.ok((await stalled.api('/api/runs/'+active.id)).states.every(s=>s.status==='cancelled'));
const neverReady=new WasmCatalog(manifest,fetcher,()=>({postMessage(){},terminate(){}}));
const initializing=neverReady.submit({...config,states:[testStates[0]]});
while(!neverReady.initReject)await new Promise(resolve=>setTimeout(resolve,1));
await neverReady.api(`/api/runs/${initializing.id}/cancel`,{method:'POST'});
while(neverReady.active)await new Promise(resolve=>setTimeout(resolve,1));
assert.equal((await neverReady.api('/api/runs/'+initializing.id)).status,'cancelled');
await fs.writeFile(path.join(evidenceRoot,'evidence.json'),JSON.stringify({native_comparisons:nativeComparisons,quality,population_misses:quality.filter(row=>!row.balance_passed).length,disconnected:quality.filter(row=>!row.contiguous).length,single_attempt_feasibility_is_not_guaranteed:true,pilot_acceptance_preserved:true},null,2));
console.log(JSON.stringify({national_status:result.status,states:result.states.map(s=>({state:s.code,units:s.metrics.units,deviation:s.metrics.max_deviation_percent,contiguous:s.metrics.contiguous})),nway_diagnostics:nwayResult.states.map(s=>({state:s.code,deviation:s.metrics.max_deviation_percent,balance_passed:s.metrics.balance_passed})),saved_maps:true,failed_hash:true,cancellation:true}));
