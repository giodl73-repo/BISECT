import fs from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
const root=process.argv[2]||'dist/wasm-laboratory';
const manifest=JSON.parse(await fs.readFile(path.join(root,'catalog.json'),'utf8'));
const engine=await instantiateEngine(await fs.readFile(path.join(root,'bisect_wasm.wasm')));
const fetcher=async name=>{const bytes=await fs.readFile(path.join(root,name));return{ok:true,arrayBuffer:async()=>bytes.buffer.slice(bytes.byteOffset,bytes.byteOffset+bytes.length)};};
class Worker{
  postMessage(message){setImmediate(()=>{if(this.terminated)return;try{this.onmessage({data:message.type==='initialize'?{type:'ready'}:{type:'result',id:message.id,result:engine.execute(message.request),memoryBytes:engine.memoryBytes()}});}catch(error){this.onmessage({data:{type:'error',id:message.id,error:error.message}});}});}
  terminate(){this.terminated=true;}
}
const lab=new WasmCatalog(manifest,fetcher,()=>new Worker());await lab.initialize();
const testYear=process.argv[4]||'2020', testStates=(process.argv[3]||'RI,IA,NC').split(',');
const config={name:'WASM national test',mode:'national',states:testStates,year:testYear,chamber:'congressional',districts:null,structure:'standard-bisect',weights:'geographic',search:'single',seed:42,seeds:2,steps:20,percentile:0,alpha_county:0,balance_tolerance:5,area_swing:1.1,iterations:10,timeout_seconds:30};
const job=await lab.api('/api/runs',{method:'POST',body:JSON.stringify(config)});
while(lab.active)await new Promise(resolve=>setTimeout(resolve,10));
const result=await lab.api('/api/runs/'+job.id);
assert.equal(result.status,'completed',JSON.stringify(result.states.map(s=>s.error)));
for(const state of result.states){
 assert.ok(state.metrics.wasm_memory_bytes<=128*1024*1024,'pilot WASM memory bound');
 assert.equal(state.metrics.contiguous,true,state.code);
 assert.equal(state.metrics.balance_passed,true,state.code);
 const map=await lab.api(`/api/runs/${job.id}/${state.code}/map`), assignments=await lab.api(`/api/runs/${job.id}/${state.code}/assignments`);
 assert.equal(map.features.length,Object.keys(assignments).length);
 assert.ok(map.features.every(f=>f.properties.district===assignments[f.properties.geoid]));
 assert.ok(!lab.assets.has(manifest.graphs[`${state.code}:${testYear}`].graph_ref),'working graph must be released');
}
const multi=lab.submit({...config,states:testStates.slice(0,2),search:'multi',seeds:8});
while(lab.active)await new Promise(resolve=>setTimeout(resolve,10));
const multiResult=await lab.api('/api/runs/'+multi.id);
assert.equal(multiResult.status,'completed');
for(const state of multiResult.states){
 assert.equal(state.metrics.contiguous,true);
 assert.equal(state.metrics.balance_passed,true);
 assert.equal(state.metrics.recorded_options.search,'multi');
 assert.equal(state.metrics.recorded_options.seeds,8);
}
const nway=lab.submit({...config,structure:'nway'});
while(lab.active)await new Promise(resolve=>setTimeout(resolve,10));
const nwayResult=await lab.api('/api/runs/'+nway.id);assert.equal(nwayResult.status,'completed');
for(const state of nwayResult.states){
 assert.equal(state.metrics.contiguous,true,state.code+' nway');
 const graph=await lab.asset(manifest.graphs[`${state.code}:${testYear}`].graph_ref), assignments=await lab.api(`/api/runs/${nway.id}/${state.code}/assignments`);
 const populations=Array(state.metrics.district_count).fill(0);for(let i=0;i<graph.geoids.length;i++)populations[assignments[graph.geoids[i]]-1]+=graph.population[i];
 const ideal=graph.population.reduce((a,b)=>a+b,0)/populations.length;
 const deviation=Math.max(...populations.map(pop=>Math.abs(pop-ideal)/ideal*100));
 assert.ok(Math.abs(state.metrics.max_deviation_percent-deviation)<1e-9);
 assert.equal(state.metrics.balance_passed,deviation<=config.balance_tolerance);
 lab.assets.delete(manifest.graphs[`${state.code}:${testYear}`].graph_ref);
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
console.log(JSON.stringify({national_status:result.status,states:result.states.map(s=>({state:s.code,units:s.metrics.units,deviation:s.metrics.max_deviation_percent,contiguous:s.metrics.contiguous})),nway_diagnostics:nwayResult.states.map(s=>({state:s.code,deviation:s.metrics.max_deviation_percent,balance_passed:s.metrics.balance_passed})),saved_maps:true,failed_hash:true,cancellation:true}));
