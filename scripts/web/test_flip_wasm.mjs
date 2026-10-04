import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {verifyLabAssignments,validateLabConfig} from '../../web/lab/laboratory-project.js';
import {engineOptions} from '../../web/lab/static.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const n=16,adjacency=Array.from({length:n},(_,i)=>[i%4>0?i-1:-1,i%4<3?i+1:-1,i>=4?i-4:-1,i<12?i+4:-1].filter(v=>v>=0)),graph={schema_version:1,state:'RI',year:'2020',geoids:Array.from({length:n},(_,i)=>`44001${String(i).padStart(6,'0')}`),adjacency,population:Array(n).fill(100),edges:adjacency.flatMap((list,i)=>list.filter(j=>i<j).map(j=>[i,j,1+(i%3)])),areas:Array(n).fill(100),exterior_perimeters:Array(n).fill(10),centroids:Array.from({length:n},(_,i)=>[-71+(i%4)*.001,41+Math.floor(i/4)*.001])};
const base={structure:'standard-bisect',weights:'geographic',search:'flip',districts:2,seed:42,seeds:8,steps:20,percentile:0,alpha_county:0,balance_tolerance:5,area_swing:1.1,iterations:10};
const config={...base,name:'Standalone chain options',mode:'state',states:['RI'],year:'2020',chamber:'congressional',timeout_seconds:30,steps:0};validateLabConfig(config);assert.deepEqual(engineOptions(config),{});
for(const bad of [{steps:-1},{steps:100001},{search:'multi'}])assert.throws(()=>validateLabConfig({...config,...bad}));
let count=0,misses=0,disconnected=0;const cases=[];
for(const search of ['flip'])for(const steps of [0,1,20,200])for(const districts of [2,4])for(const seed of [42,4294967297])for(const percentile of [0,.5,1])for(const weights of ['geographic','unweighted'])for(const balance_tolerance of [5,25])for(const iterations of [1,100]) {
 const request={graph,options:{...base,search,steps,districts,seed,percentile,weights,balance_tolerance,iterations}},file=`target/flip-request-${count}.json`;await fs.writeFile(file,JSON.stringify(request));
 const actual=engine.execute(request),native=spawnSync('target/release/examples/execute_request.exe',[file],{encoding:'utf8',maxBuffer:16*1024*1024});assert.equal(native.status,0,native.stderr);const reference=JSON.parse(native.stdout);assert.deepEqual(actual.assignments,reference.assignments);assert.deepEqual(actual.metrics,reference.metrics);assert.deepEqual(actual.options,request.options);
 const state={metrics:{...actual.metrics,district_count:districts,districts:actual.metrics.district_metrics,balance_passed:actual.metrics.within_requested_tolerance}};verifyLabAssignments(graph,state,actual.assignments,request.options);
 for(const field of ['method','rng','steps','percentile','selection','rank_rule','seed_domain','initial_refinement_iterations','population_allowance','nonempty_districts','record_count','selected_rank']){const bad=structuredClone(state);bad.metrics.structure_evidence[field]='tampered';assert.throws(()=>verifyLabAssignments(graph,bad,actual.assignments,request.options),/boundary-flip evidence/);}
 // Historical projects without a policy ledger still receive independent metric checks.
 const legacy=structuredClone(state);legacy.metrics.structure_evidence=null;assert.doesNotThrow(()=>verifyLabAssignments(graph,legacy,actual.assignments,request.options));
 if(steps===0)assert.deepEqual(actual.assignments,engine.execute({graph,options:{...request.options,search:'single',steps:1,iterations:100}}).assignments);
 assert.ok(actual.metrics.within_requested_tolerance);assert.ok(actual.metrics.contiguous);
 misses+=!actual.metrics.within_requested_tolerance;disconnected+=!actual.metrics.contiguous;count++;cases.push({search,steps,districts,seed,percentile,weights,balance_tolerance,iterations});
}
let failures=0;for(const bad of [{steps:-1},{steps:100001},{search:'multi',steps:0},{structure:'nway',search:'flip'}]){const request={graph,options:{...base,...bad}};assert.throws(()=>engine.execute(request));const file=`target/flip-invalid-${failures++}.json`;await fs.writeFile(file,JSON.stringify(request));assert.notEqual(spawnSync('target/release/examples/execute_request.exe',[file],{encoding:'utf8'}).status,0);}
await fs.writeFile('target/flip-native-wasm-parity.json',JSON.stringify({cases:count,exact_matches:count,matching_rejections:failures,balance_misses:misses,disconnected_plans:disconnected,rng:'chacha12-u64-v1',results:cases},null,2));
console.log(`Boundary flip: ${count} exact native/WASM cases, ${failures} matching rejections, evidence tamper and zero-step checks; ${misses} balance misses, ${disconnected} disconnected plans.`);
