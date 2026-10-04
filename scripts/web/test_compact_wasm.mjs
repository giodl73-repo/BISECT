import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {verifyLabAssignments,validateLabConfig} from '../../web/lab/laboratory-project.js';
import {engineOptions,effectiveConfig} from '../../web/lab/static.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const n=16,adjacency=Array.from({length:n},(_,i)=>[i%4>0?i-1:-1,i%4<3?i+1:-1,i>=4?i-4:-1,i<12?i+4:-1].filter(v=>v>=0)),graph={schema_version:1,state:'RI',year:'2020',geoids:Array.from({length:n},(_,i)=>`44001${String(i).padStart(6,'0')}`),adjacency,population:Array(n).fill(100),edges:adjacency.flatMap((list,i)=>list.filter(j=>i<j).map(j=>[i,j,1+(i%3)])),areas:Array.from({length:n},(_,i)=>100+i*5),exterior_perimeters:Array.from({length:n},(_,i)=>10+i%4),centroids:Array.from({length:n},(_,i)=>[-71+(i%4)*.001,41+Math.floor(i/4)*.001])};
const base={structure:'compact-polsby',weights:'unweighted',search:'single',districts:2,seed:42,seeds:8,steps:20,percentile:0,alpha_county:0,balance_tolerance:5,area_swing:1.1,iterations:10};
const config={...base,name:'Compact slack',mode:'state',states:['RI'],year:'2020',chamber:'congressional',timeout_seconds:30,compact_epsilon:0.2};validateLabConfig(config);assert.deepEqual(engineOptions(config),{compact_epsilon:0.2});assert.equal(effectiveConfig(config,2).seed,0);
for(const bad of [{compact_epsilon:-1},{compact_epsilon:1.1},{compact_epsilon:NaN},{structure:'standard-bisect'}])assert.throws(()=>validateLabConfig({...config,...bad}));
let count=0,misses=0,disconnected=0;
for(const compact_epsilon of [0,.05,1])for(const search of ['single','multi'])for(const districts of [2,4])for(const weights of ['unweighted','geographic']) {
 const request={graph,options:{...base,compact_epsilon,search,districts,weights}},file=`target/compact-request-${count}.json`;await fs.writeFile(file,JSON.stringify(request));
 const actual=engine.execute(request),native=spawnSync('target/release/examples/execute_request.exe',[file],{encoding:'utf8',maxBuffer:16*1024*1024});assert.equal(native.status,0,native.stderr);const reference=JSON.parse(native.stdout);assert.deepEqual(actual.assignments,reference.assignments);assert.deepEqual(actual.metrics,reference.metrics);assert.deepEqual(actual.options,request.options);
 assert.deepEqual(actual.assignments,engine.execute({graph,options:{...request.options,seed:9007199254740991}}).assignments);
 const state={metrics:{...actual.metrics,district_count:districts,districts:actual.metrics.district_metrics,balance_passed:actual.metrics.within_requested_tolerance}};verifyLabAssignments(graph,state,actual.assignments,request.options);
 for(const field of ['epsilon','seeds_per_split','seed_policy','selection']){const bad=structuredClone(state);bad.metrics.structure_evidence[field]='tampered';assert.throws(()=>verifyLabAssignments(graph,bad,actual.assignments,request.options),/CompactBisect evidence/);}
 misses+=!actual.metrics.within_requested_tolerance;disconnected+=!actual.metrics.contiguous;count++;
}
let failures=0;for(const options of [{...base,compact_epsilon:-1},{...base,compact_epsilon:1.1},{...base,structure:'standard-bisect',compact_epsilon:.05}]){const request={graph,options};assert.throws(()=>engine.execute(request));const file=`target/compact-invalid-${failures++}.json`;await fs.writeFile(file,JSON.stringify(request));assert.notEqual(spawnSync('target/release/examples/execute_request.exe',[file],{encoding:'utf8'}).status,0);}
assert.deepEqual(engine.execute({graph,options:base}).assignments,engine.execute({graph,options:{...base,compact_epsilon:.05}}).assignments);
await fs.writeFile('target/compact-native-wasm-parity.json',JSON.stringify({cases:count,exact_matches:count,matching_rejections:failures,balance_misses:misses,disconnected_plans:disconnected,seed_policy:'fixed-1-through-budget'},null,2));
console.log(`CompactBisect: ${count} exact native/WASM cases, single/multi, k=2/4, two weights, cut slack 0/5/100 percent, fixed seed policy, evidence tamper and three matching rejections passed; ${misses} balance misses, ${disconnected} disconnected plans.`);
