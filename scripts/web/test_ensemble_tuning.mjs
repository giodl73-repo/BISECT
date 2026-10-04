import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {verifyLabAssignments,validateLabConfig} from '../../web/lab/laboratory-project.js';
import pathModule from 'node:path';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
import {createLabProject} from '../../web/lab/laboratory-project.js';
import {engineOptions} from '../../web/lab/static.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const n=30,width=6,height=5,adjacency=Array.from({length:n},(_,i)=>[i%width>0?i-1:-1,i%width<width-1?i+1:-1,i>=width?i-width:-1,i<width*(height-1)?i+width:-1].filter(v=>v>=0)),graph={schema_version:1,state:'RI',year:'2020',geoids:Array.from({length:n},(_,i)=>`44001${String(i).padStart(6,'0')}`),adjacency,population:Array(n).fill(100),edges:adjacency.flatMap((list,i)=>list.filter(j=>i<j).map(j=>[i,j,1+(i%3)])),areas:Array(n).fill(100),exterior_perimeters:Array(n).fill(10),centroids:Array.from({length:n},(_,i)=>[-71+(i%width)*.001,41+Math.floor(i/width)*.001])};
const base={structure:'standard-bisect',weights:'geographic',search:'bisection-ensemble',districts:2,seed:42,seeds:8,steps:20,percentile:0,alpha_county:0,balance_tolerance:5,area_swing:1.1,iterations:10};
const config={...base,name:'Standalone chain options',mode:'state',states:['RI'],year:'2020',chamber:'congressional',timeout_seconds:60,steps:0,metis_objective:'volume',metis_trials:3};validateLabConfig(config);assert.deepEqual(engineOptions(config),{metis_objective:'volume',metis_trials:3});
for(const bad of [{steps:-1},{steps:100001},{metis_objective:'bad'},{metis_trials:0},{metis_trials:101},{search:'flip'}])assert.throws(()=>validateLabConfig({...config,...bad}));
let count=0,misses=0,disconnected=0;const cases=[];
for(const metis_objective of ['cut','volume'])for(const metis_trials of [1,3])for(const search of ['bisection-ensemble'])for(const steps of [0,20])for(const districts of [2,3])for(const seed of [42,4294967297])for(const percentile of [0,.5,1])for(const weights of ['geographic','unweighted'])for(const iterations of [10]) {
 const request={graph,options:{...base,metis_objective,metis_trials,search,steps,districts,seed,percentile,weights,iterations}},file=`target/ensemble-tuning-request-${count}.json`;await fs.writeFile(file,JSON.stringify(request));
 const actual=engine.execute(request),native=spawnSync('target/release/examples/execute_request.exe',[file],{encoding:'utf8',maxBuffer:16*1024*1024});assert.equal(native.status,0,native.stderr);const reference=JSON.parse(native.stdout);assert.deepEqual(actual.assignments,reference.assignments);assert.deepEqual(actual.metrics,reference.metrics);assert.deepEqual(actual.options,request.options);
 const state={metrics:{...actual.metrics,district_count:districts,districts:actual.metrics.district_metrics,balance_passed:actual.metrics.within_requested_tolerance}};verifyLabAssignments(graph,state,actual.assignments,request.options);
 for(const field of ['method','rng','steps_per_split','percentile','selection','rank_rule','target_policy','small_region_fallback','scope','tree_sampler']){const bad=structuredClone(state);bad.metrics.structure_evidence[field]='tampered';assert.throws(()=>verifyLabAssignments(graph,bad,actual.assignments,request.options),/local-bisection evidence/);}
 // Historical projects without a policy ledger still receive independent metric checks.
 const legacy=structuredClone(state);legacy.metrics.structure_evidence=null;assert.throws(()=>verifyLabAssignments(graph,legacy,actual.assignments,request.options),/local-bisection evidence/);
 for(const field of ['objective','internal_trials','iterations','scope','internal_trial_selection','post_refinement']){const bad=structuredClone(state);bad.metrics.structure_evidence.metis_initialization[field]='tampered';assert.throws(()=>verifyLabAssignments(graph,bad,actual.assignments,request.options),/local-bisection evidence/);}
 if(metis_objective==='cut'&&metis_trials===1){const legacyOptions={...request.options};delete legacyOptions.metis_objective;delete legacyOptions.metis_trials;assert.deepEqual(actual.assignments,engine.execute({graph,options:legacyOptions}).assignments);}
 if(steps===0)assert.deepEqual(actual.assignments,engine.execute({graph,options:{...request.options,search:'single',steps:1}}).assignments);
 misses+=!actual.metrics.within_requested_tolerance;disconnected+=!actual.metrics.contiguous;count++;cases.push({metis_objective,metis_trials,search,steps,districts,seed,percentile,weights,iterations});
}
// The native shortcut performs a single METIS cut for regions of at most four
// tracts, regardless of local proposal budget or percentile.
for(const metis_objective of ['cut','volume'])for(const metis_trials of [1,3])for(const size of [3,4])for(const steps of [0,20])for(const seed of [42,4294967297])for(const percentile of [0,1]){
 const adjacency=Array.from({length:size},(_,i)=>[i>0?i-1:-1,i+1<size?i+1:-1].filter(v=>v>=0));
 const small={...graph,geoids:graph.geoids.slice(0,size),adjacency,population:size===3?[100,100,200]:Array(size).fill(100),edges:adjacency.flatMap((list,i)=>list.filter(j=>i<j).map(j=>[i,j,1])),areas:graph.areas.slice(0,size),exterior_perimeters:graph.exterior_perimeters.slice(0,size),centroids:graph.centroids.slice(0,size)};
 const request={graph:small,options:{...base,metis_objective,metis_trials,steps,seed,percentile}},file=`target/ensemble-tuning-fallback-${count}.json`;await fs.writeFile(file,JSON.stringify(request));
 const actual=engine.execute(request),native=spawnSync('target/release/examples/execute_request.exe',[file],{encoding:'utf8'});assert.equal(native.status,0,native.stderr);const reference=JSON.parse(native.stdout);
 assert.deepEqual(actual.assignments,reference.assignments);assert.deepEqual(actual.metrics,reference.metrics);
 assert.deepEqual(actual.assignments,engine.execute({graph:small,options:{...request.options,search:'single',steps:1}}).assignments);
 verifyLabAssignments(small,{metrics:{...actual.metrics,district_count:2,districts:actual.metrics.district_metrics,balance_passed:actual.metrics.within_requested_tolerance}},actual.assignments,request.options);
 misses+=!actual.metrics.within_requested_tolerance;disconnected+=!actual.metrics.contiguous;
 count++;cases.push({metis_objective,metis_trials,size,steps,seed,percentile,small_region_fallback:true});
}
let failures=0;for(const bad of [{steps:-1},{steps:100001},{metis_trials:0},{metis_trials:101},{metis_objective:'bad'},{search:'flip',metis_objective:'volume'}]){const request={graph,options:{...base,...bad}};assert.throws(()=>engine.execute(request));const file=`target/ensemble-tuning-invalid-${failures++}.json`;await fs.writeFile(file,JSON.stringify(request));assert.notEqual(spawnSync('target/release/examples/execute_request.exe',[file],{encoding:'utf8'}).status,0);}
let projects=0;
if(process.argv[2]){
 const root=process.argv[2],manifest=JSON.parse(await fs.readFile(pathModule.join(root,'catalog.json'),'utf8'));
 const fetcher=async name=>{const b=await fs.readFile(pathModule.join(root,name));return{ok:true,arrayBuffer:async()=>b.buffer.slice(b.byteOffset,b.byteOffset+b.length)};};
 class Worker{postMessage(m){setImmediate(()=>{if(this.terminated)return;try{this.onmessage({data:m.type==='initialize'?{type:'ready'}:{type:'result',id:m.id,result:engine.execute(m.request)}});}catch(e){this.onmessage({data:{type:'error',id:m.id,error:e.message}});}});}terminate(){this.terminated=true;}}
 const lab=new WasmCatalog(manifest,fetcher,()=>new Worker());
 for(const key of Object.keys(manifest.graphs).slice(0,3)){
   const [code,year]=key.split(':');
   const job=lab.submit({...config,states:[code],year,districts:null,steps:20,percentile:0,balance_tolerance:10});while(lab.active)await new Promise(r=>setTimeout(r,5));assert.equal(lab.jobs.get(job.id).status,'completed');
   const saved=await createLabProject(lab,job.id),restore=new WasmCatalog(manifest,fetcher,()=>{throw new Error('Opening must not generate.');});const reopened=await restore.restoreProject(JSON.parse(JSON.stringify(saved)));assert.equal(restore.worker,null);
   assert.deepEqual(await restore.api(`/api/runs/${reopened.id}/${code}/assignments`),saved.assignments[code]);
   const bad=structuredClone(saved);bad.experiment.states[0].metrics.structure_evidence.metis_initialization.internal_trials++;await assert.rejects(restore.restoreProject(bad),/local-bisection evidence/);
   await fs.writeFile(`target/ensemble-volume-${code}.bisect`,JSON.stringify(saved));projects++;
 }
}
await fs.writeFile('target/ensemble-tuning-native-wasm-parity.json',JSON.stringify({cases:count,exact_matches:count,real_state_projects:projects,matching_rejections:failures,balance_misses:misses,disconnected_plans:disconnected,rng:'chacha12-u64-v1',results:cases},null,2));
console.log(`Tuned local bisection ensemble: ${count} exact native/WASM cases, ${failures} matching rejections, ${projects} real-state projects, evidence tamper and zero-step checks; ${misses} balance misses, ${disconnected} disconnected plans.`);
