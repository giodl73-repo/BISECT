import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
import {createLabProject,verifyLabAssignments} from '../../web/lab/laboratory-project.js';
const root=process.argv[2];assert.ok(root,'Usage: test_bfs_wasm.mjs <catalog directory>');
const manifest=JSON.parse(await fs.readFile(path.join(root,'catalog.json'),'utf8'));
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const fetcher=async ref=>{const b=await fs.readFile(path.join(root,ref));return{ok:true,arrayBuffer:async()=>b.buffer.slice(b.byteOffset,b.byteOffset+b.length)};};
class Worker{postMessage(m){setImmediate(()=>{if(this.terminated)return;try{this.onmessage({data:m.type==='initialize'?{type:'ready'}:{type:'result',id:m.id,result:engine.execute(m.request)}});}catch(e){this.onmessage({data:{type:'error',id:m.id,error:e.message}});}});}terminate(){this.terminated=true;}}
const lab=new WasmCatalog(manifest,fetcher,()=>new Worker());
let cases=0,projects=0,misses=0,disconnected=0;
for(const [key,entry] of Object.entries(manifest.graphs)){
 const [code,year]=key.split(':');
 const graph=JSON.parse(await fs.readFile(path.join(root,entry.graph_ref),'utf8'));
 const config={name:'Portable BFS',mode:'state',states:[code],year,chamber:'congressional',districts:null,structure:'bfs-growth',weights:'geographic',search:'single',seed:42,seeds:3,steps:20,percentile:.5,alpha_county:0,balance_tolerance:.5,area_swing:1.1,iterations:10,timeout_seconds:60};
 for(const seed of [0,42,'18446744073709551615'])for(const weights of ['geographic','unweighted']){
  const options={...config,seed,weights,districts:lab.districtCount(code,config)};
  for(const name of ['name','mode','states','year','chamber','timeout_seconds'])delete options[name];
  const request={graph,options},actual=engine.execute(request);
  await fs.writeFile('target/bfs-reference-request.json',JSON.stringify(request).replace(/"seed":"([0-9]+)"/, '"seed":$1'));
  const native=spawnSync('target/release/examples/execute_request.exe',['target/bfs-reference-request.json'],{encoding:'utf8',maxBuffer:32*1024*1024});assert.equal(native.status,0,native.stderr);
  assert.deepEqual(actual,JSON.parse(native.stdout));
  verifyLabAssignments(graph,{metrics:{...actual.metrics,district_count:options.districts,districts:actual.metrics.district_metrics,balance_passed:actual.metrics.within_requested_tolerance}},actual.assignments,options);
  cases++;misses+=!actual.metrics.within_requested_tolerance;disconnected+=!actual.metrics.contiguous;
 }
 const job=lab.submit({...config,seed:'18446744073709551615'});while(lab.active)await new Promise(r=>setTimeout(r,5));assert.equal(lab.jobs.get(job.id).status,'completed');
 const saved=await createLabProject(lab,job.id),restorer=new WasmCatalog(manifest,fetcher,()=>{throw new Error('Open must not execute the engine.');});
 const opened=await restorer.restoreProject(structuredClone(saved));assert.equal(opened.config.seed,'18446744073709551615');assert.deepEqual(await restorer.api(`/api/runs/${opened.id}/${code}/assignments`),saved.assignments[code]);projects++;
}
assert.ok(cases>0);
await fs.writeFile('target/bfs-native-wasm-parity.json',JSON.stringify({exact_real_state_cases:cases,project_roundtrips:projects,balance_misses:misses,disconnected_plans:disconnected,rng:'xoshiro256plusplus-rand08-default-u64-expansion',engine_executed_on_open:false},null,2));
console.log(`BFS: ${cases} exact real-state native/WASM results, ${projects} project roundtrips; ${misses} balance misses, ${disconnected} disconnected plans reported without implying feasibility from parity.`);
