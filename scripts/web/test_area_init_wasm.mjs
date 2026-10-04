import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {verifyLabAssignments,validateLabConfig} from '../../web/lab/laboratory-project.js';
import {engineOptions} from '../../web/lab/static.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const n=16,adjacency=Array.from({length:n},(_,i)=>[i%4>0?i-1:-1,i%4<3?i+1:-1,i>=4?i-4:-1,i<12?i+4:-1].filter(v=>v>=0)),graph={schema_version:1,state:'RI',year:'2020',geoids:Array.from({length:n},(_,i)=>`44001${String(i).padStart(6,'0')}`),adjacency,population:Array(n).fill(100),edges:adjacency.flatMap((list,i)=>list.filter(j=>i<j).map(j=>[i,j,1+(i%3)])),areas:Array(n).fill(100),exterior_perimeters:Array(n).fill(10),centroids:Array.from({length:n},(_,i)=>[-71+(i%4)*.001,41+Math.floor(i/4)*.001])};
const base={structure:'ratio-optimal-area',weights:'geographic',search:'multi',districts:2,seed:42,seeds:8,steps:20,percentile:0,alpha_county:0,balance_tolerance:5,area_swing:1.1,iterations:10};
const config={...base,name:'Area initializer',mode:'state',states:['RI'],year:'2020',chamber:'congressional',timeout_seconds:30,area_init:'moving-knife'};validateLabConfig(config);assert.deepEqual(engineOptions(config),{area_init:'moving-knife'});
for(const bad of [{area_init:'unknown'},{structure:'standard-bisect'}])assert.throws(()=>validateLabConfig({...config,...bad}));
let count=0,misses=0,disconnected=0;const cases=[];
for(const area_init of ['ratio-optimal','moving-knife'])for(const search of ['single','multi'])for(const districts of [2,4])for(const weights of ['unweighted','geographic']) {
 const request={graph,options:{...base,area_init,search,districts,weights}},file=`target/area-init-request-${count}.json`;await fs.writeFile(file,JSON.stringify(request));
 const actual=engine.execute(request),native=spawnSync('target/release/examples/execute_request.exe',[file],{encoding:'utf8',maxBuffer:16*1024*1024});assert.equal(native.status,0,native.stderr);const reference=JSON.parse(native.stdout);assert.deepEqual(actual.assignments,reference.assignments);assert.deepEqual(actual.metrics,reference.metrics);assert.deepEqual(actual.options,request.options);
 const state={metrics:{...actual.metrics,district_count:districts,districts:actual.metrics.district_metrics,balance_passed:actual.metrics.within_requested_tolerance}};verifyLabAssignments(graph,state,actual.assignments,request.options);
 for(const field of ['initialization','orientations','directional_lambda','scope','direction_radians']){const bad=structuredClone(state);bad.metrics.structure_evidence[field]='tampered';assert.throws(()=>verifyLabAssignments(graph,bad,actual.assignments,request.options),/AreaSection initialization evidence/);}
 misses+=!actual.metrics.within_requested_tolerance;disconnected+=!actual.metrics.contiguous;count++;cases.push({area_init,search,districts,weights,direction:actual.metrics.structure_evidence.direction_radians});
}
let failures=0;for(const options of [{...base,area_init:'unknown'},{...base,structure:'standard-bisect',area_init:'moving-knife'}]){const request={graph,options};assert.throws(()=>engine.execute(request));const file=`target/area-init-invalid-${failures++}.json`;await fs.writeFile(file,JSON.stringify(request));assert.notEqual(spawnSync('target/release/examples/execute_request.exe',[file],{encoding:'utf8'}).status,0);}
assert.deepEqual(engine.execute({graph,options:base}).assignments,engine.execute({graph,options:{...base,area_init:'ratio-optimal'}}).assignments);
await fs.writeFile('target/area-init-native-wasm-parity.json',JSON.stringify({cases:count,exact_matches:count,matching_rejections:failures,balance_misses:misses,disconnected_plans:disconnected,scope:'root-only',results:cases},null,2));
console.log(`AreaSection initialization: ${count} exact native/WASM cases, 2 matching rejections, evidence tamper checks; ${misses} balance misses, ${disconnected} disconnected plans.`);
