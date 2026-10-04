import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {verifyLabAssignments,validateLabConfig} from '../../web/lab/laboratory-project.js';
import {engineOptions} from '../../web/lab/static.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const n=30,width=6,height=5,adjacency=Array.from({length:n},(_,i)=>[i%width>0?i-1:-1,i%width<width-1?i+1:-1,i>=width?i-width:-1,i<width*(height-1)?i+width:-1].filter(v=>v>=0)),graph={schema_version:1,state:'RI',year:'2020',geoids:Array.from({length:n},(_,i)=>`44001${String(i).padStart(6,'0')}`),adjacency,population:Array(n).fill(100),edges:adjacency.flatMap((list,i)=>list.filter(j=>i<j).map(j=>[i,j,1+(i%3)])),areas:Array(n).fill(100),exterior_perimeters:Array(n).fill(10),centroids:Array.from({length:n},(_,i)=>[-71+(i%width)*.001,41+Math.floor(i/width)*.001])};
const base={structure:'standard-bisect',weights:'geographic',search:'bisection-ensemble',districts:2,seed:42,seeds:8,steps:20,percentile:0,alpha_county:0,balance_tolerance:5,area_swing:1.1,iterations:10};
const config={...base,name:'Standalone chain options',mode:'state',states:['RI'],year:'2020',chamber:'congressional',timeout_seconds:30,steps:0};validateLabConfig(config);assert.deepEqual(engineOptions(config),{});
for(const bad of [{steps:-1},{steps:100001},{search:'multi'}])assert.throws(()=>validateLabConfig({...config,...bad}));
let count=0,misses=0,disconnected=0;const cases=[];
for(const search of ['bisection-ensemble'])for(const steps of [0,1,20,60])for(const districts of [2,3,5])for(const seed of [42,4294967297])for(const percentile of [0,.5,1])for(const weights of ['geographic','unweighted'])for(const iterations of [10,100]) {
 const request={graph,options:{...base,search,steps,districts,seed,percentile,weights,iterations}},file=`target/local-ensemble-request-${count}.json`;await fs.writeFile(file,JSON.stringify(request));
 const actual=engine.execute(request),native=spawnSync('target/release/examples/execute_request.exe',[file],{encoding:'utf8',maxBuffer:16*1024*1024});assert.equal(native.status,0,native.stderr);const reference=JSON.parse(native.stdout);assert.deepEqual(actual.assignments,reference.assignments);assert.deepEqual(actual.metrics,reference.metrics);assert.deepEqual(actual.options,request.options);
 const state={metrics:{...actual.metrics,district_count:districts,districts:actual.metrics.district_metrics,balance_passed:actual.metrics.within_requested_tolerance}};verifyLabAssignments(graph,state,actual.assignments,request.options);
 for(const field of ['method','rng','steps_per_split','percentile','selection','rank_rule','target_policy','small_region_fallback','scope','tree_sampler']){const bad=structuredClone(state);bad.metrics.structure_evidence[field]='tampered';assert.throws(()=>verifyLabAssignments(graph,bad,actual.assignments,request.options),/local-bisection evidence/);}
 // Historical projects without a policy ledger still receive independent metric checks.
 const legacy=structuredClone(state);legacy.metrics.structure_evidence=null;assert.doesNotThrow(()=>verifyLabAssignments(graph,legacy,actual.assignments,request.options));
 if(steps===0)assert.deepEqual(actual.assignments,engine.execute({graph,options:{...request.options,search:'single',steps:1}}).assignments);
 misses+=!actual.metrics.within_requested_tolerance;disconnected+=!actual.metrics.contiguous;count++;cases.push({search,steps,districts,seed,percentile,weights,iterations});
}
// The native shortcut performs a single METIS cut for regions of at most four
// tracts, regardless of local proposal budget or percentile.
for(const size of [3,4])for(const steps of [0,20])for(const seed of [42,4294967297])for(const percentile of [0,1]){
 const adjacency=Array.from({length:size},(_,i)=>[i>0?i-1:-1,i+1<size?i+1:-1].filter(v=>v>=0));
 const small={...graph,geoids:graph.geoids.slice(0,size),adjacency,population:size===3?[100,100,200]:Array(size).fill(100),edges:adjacency.flatMap((list,i)=>list.filter(j=>i<j).map(j=>[i,j,1])),areas:graph.areas.slice(0,size),exterior_perimeters:graph.exterior_perimeters.slice(0,size),centroids:graph.centroids.slice(0,size)};
 const request={graph:small,options:{...base,steps,seed,percentile}},file=`target/local-ensemble-fallback-${count}.json`;await fs.writeFile(file,JSON.stringify(request));
 const actual=engine.execute(request),native=spawnSync('target/release/examples/execute_request.exe',[file],{encoding:'utf8'});assert.equal(native.status,0,native.stderr);const reference=JSON.parse(native.stdout);
 assert.deepEqual(actual.assignments,reference.assignments);assert.deepEqual(actual.metrics,reference.metrics);
 assert.deepEqual(actual.assignments,engine.execute({graph:small,options:{...request.options,search:'single',steps:1}}).assignments);
 verifyLabAssignments(small,{metrics:{...actual.metrics,district_count:2,districts:actual.metrics.district_metrics,balance_passed:actual.metrics.within_requested_tolerance}},actual.assignments,request.options);
 misses+=!actual.metrics.within_requested_tolerance;disconnected+=!actual.metrics.contiguous;
 count++;cases.push({size,steps,seed,percentile,small_region_fallback:true});
}
let failures=0;for(const bad of [{steps:-1},{steps:100001},{search:'multi',steps:0},{structure:'nway',search:'bisection-ensemble'}]){const request={graph,options:{...base,...bad}};assert.throws(()=>engine.execute(request));const file=`target/local-ensemble-invalid-${failures++}.json`;await fs.writeFile(file,JSON.stringify(request));assert.notEqual(spawnSync('target/release/examples/execute_request.exe',[file],{encoding:'utf8'}).status,0);}
await fs.writeFile('target/local-ensemble-native-wasm-parity.json',JSON.stringify({cases:count,exact_matches:count,matching_rejections:failures,balance_misses:misses,disconnected_plans:disconnected,rng:'chacha12-u64-v1',results:cases},null,2));
console.log(`Local bisection ensemble: ${count} exact native/WASM cases, ${failures} matching rejections, evidence tamper and zero-step checks; ${misses} balance misses, ${disconnected} disconnected plans.`);
