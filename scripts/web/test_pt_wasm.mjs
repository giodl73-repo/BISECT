import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {verifyLabAssignments,validateLabConfig} from '../../web/lab/laboratory-project.js';
import {engineOptions} from '../../web/lab/static.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const n=16,adjacency=Array.from({length:n},(_,i)=>[i%4>0?i-1:-1,i%4<3?i+1:-1,i>=4?i-4:-1,i<12?i+4:-1].filter(v=>v>=0)),graph={schema_version:1,state:'RI',year:'2020',geoids:Array.from({length:n},(_,i)=>`44001${String(i).padStart(6,'0')}`),adjacency,population:Array(n).fill(100),edges:adjacency.flatMap((list,i)=>list.filter(j=>i<j).map(j=>[i,j,1+(i%3)])),areas:Array(n).fill(100),exterior_perimeters:Array(n).fill(10),centroids:Array.from({length:n},(_,i)=>[-71+(i%4)*.001,41+Math.floor(i/4)*.001])};
const base={structure:'standard-bisect',weights:'geographic',search:'parallel-tempering',districts:2,seed:42,seeds:20,steps:20,percentile:0,alpha_county:0,balance_tolerance:5,area_swing:1.1,iterations:10};
const config={...base,name:'Tempering settings',mode:'state',states:['RI'],year:'2020',chamber:'congressional',timeout_seconds:30,pt_replicas:4,pt_swap_interval:7,pt_cold_tol:.05,pt_hot_tol:.2};validateLabConfig(config);assert.deepEqual(engineOptions(config),{pt_replicas:4,pt_swap_interval:7,pt_cold_tol:.05,pt_hot_tol:.2});
for(const bad of [{pt_replicas:0},{pt_replicas:33},{pt_swap_interval:0},{pt_hot_tol:.01},{pt_cold_tol:.01},{pt_hot_tol:1.1},{pt_replicas:32,seeds:10000},{search:'multi'},{structure:'nway'}])assert.throws(()=>validateLabConfig({...config,...bad}));
let count=0,misses=0,disconnected=0;const cases=[];
for(const seeds of [0,20])for(const pt_replicas of [1,2,4])for(const pt_swap_interval of [1,7])for(const pt_hot_tol of [.05,.1,.2])for(const districts of [2,4])for(const seed of [42,4294967297])for(const percentile of [0,1]) {
 const request={graph,options:{...base,seeds,pt_replicas,pt_swap_interval,pt_cold_tol:.05,pt_hot_tol,districts,seed,percentile}},file=`target/pt-request-${count}.json`;await fs.writeFile(file,JSON.stringify(request));
 const actual=engine.execute(request),native=spawnSync('target/release/examples/execute_request.exe',[file],{encoding:'utf8',maxBuffer:16*1024*1024});assert.equal(native.status,0,native.stderr);const reference=JSON.parse(native.stdout);assert.deepEqual(actual.assignments,reference.assignments);assert.deepEqual(actual.metrics,reference.metrics);assert.deepEqual(actual.options,request.options);
 const state={metrics:{...actual.metrics,district_count:districts,districts:actual.metrics.district_metrics,balance_passed:actual.metrics.within_requested_tolerance}};verifyLabAssignments(graph,state,actual.assignments,request.options);
 for(const field of ['replicas','swap_interval','cold_tolerance','hot_tolerance','steps','selection','rng','swap_support']){const bad=structuredClone(state);bad.metrics.structure_evidence[field]='tampered';assert.throws(()=>verifyLabAssignments(graph,bad,actual.assignments,request.options),/parallel-tempering evidence/);}
 if(seeds===0)assert.deepEqual(actual.assignments,engine.execute({graph,options:{...base,search:'single',districts,seed}}).assignments);
 misses+=!actual.metrics.within_requested_tolerance;disconnected+=!actual.metrics.contiguous;count++;cases.push({seeds,pt_replicas,pt_swap_interval,pt_hot_tol,districts,seed,percentile});
}
let failures=0;const explicit={...base,pt_replicas:4,pt_swap_interval:7,pt_cold_tol:.05,pt_hot_tol:.2};for(const bad of [{pt_replicas:0},{pt_replicas:33},{pt_swap_interval:0},{pt_hot_tol:.01},{pt_cold_tol:.01},{pt_hot_tol:1.1},{pt_replicas:32,seeds:10000},{search:'multi'},{pt_hot_tol:undefined}]){const request={graph,options:{...explicit,...bad}};assert.throws(()=>engine.execute(request));const file=`target/pt-invalid-${failures++}.json`;await fs.writeFile(file,JSON.stringify(request));assert.notEqual(spawnSync('target/release/examples/execute_request.exe',[file],{encoding:'utf8'}).status,0);}
assert.deepEqual(engine.execute({graph,options:base}).assignments,engine.execute({graph,options:{...explicit,pt_swap_interval:10}}).assignments);
await fs.writeFile('target/pt-native-wasm-parity.json',JSON.stringify({cases:count,exact_matches:count,matching_rejections:failures,balance_misses:misses,disconnected_plans:disconnected,rng:'chacha12-u64-v1',results:cases},null,2));
console.log(`Parallel tempering: ${count} exact native/WASM cases, ${failures} matching rejections, evidence tamper checks; ${misses} balance misses, ${disconnected} disconnected plans.`);
