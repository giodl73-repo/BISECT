import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {verifyLabAssignments,validateLabConfig} from '../../web/lab/laboratory-project.js';
import {engineOptions,effectiveConfig} from '../../web/lab/static.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const n=24,graph={schema_version:1,state:'RI',year:'2020',geoids:Array.from({length:n},(_,i)=>`44001${String(i).padStart(6,'0')}`),adjacency:Array.from({length:n},(_,i)=>[i-1,i+1].filter(v=>v>=0&&v<n)),population:Array(n).fill(100),edges:Array.from({length:n-1},(_,i)=>[i,i+1,100+i]),areas:Array(n).fill(100),exterior_perimeters:Array(n).fill(0),centroids:Array.from({length:n},(_,i)=>[-71+(i%16)*.001,41+Math.floor(i/16)*.001])};
const base={structure:'moving-knife',weights:'unweighted',search:'single',districts:2,seed:42,seeds:1,steps:20,percentile:0,alpha_county:0,balance_tolerance:5,area_swing:1.1,iterations:10};
const config={...base,name:'Moving knife controls',mode:'state',states:['RI'],year:'2020',chamber:'congressional',timeout_seconds:30,mka_orientations:20,mka_metric:'reock'};
validateLabConfig(config);assert.deepEqual(engineOptions(config),{mka_orientations:20,mka_metric:'reock'});assert.equal(effectiveConfig(config,2).budget,20);assert.equal(effectiveConfig(config,2).iterations,0);
for(const bad of [{mka_orientations:10001},{mka_metric:'invalid'},{mka_orientations:undefined},{structure:'standard-bisect'}])assert.throws(()=>validateLabConfig({...config,...bad}));
const legacyConfig={...config};delete legacyConfig.mka_orientations;delete legacyConfig.mka_metric;validateLabConfig(legacyConfig);assert.deepEqual(engineOptions(legacyConfig),{});
let count=0,misses=0,disconnected=0;
for(const mka_metric of ['reock','polsby'])for(const mka_orientations of [0,1,36,180])for(const seed of [42,4294967297,Number.MAX_SAFE_INTEGER]){
  const request={graph,options:{...base,mka_metric,mka_orientations,seed}},file=`target/mka-request-${count}.json`;await fs.writeFile(file,JSON.stringify(request));
  const actual=engine.execute(request),native=spawnSync('target/release/examples/execute_request.exe',[file],{encoding:'utf8',maxBuffer:16*1024*1024});assert.equal(native.status,0,native.stderr);
  const reference=JSON.parse(native.stdout);assert.deepEqual(actual.assignments,reference.assignments);assert.deepEqual(actual.metrics,reference.metrics);assert.deepEqual(actual.options,request.options);
  assert.equal(actual.metrics.structure_evidence.requested_metric,mka_metric);assert.equal(actual.metrics.structure_evidence.effective_metric,'reock');assert.deepEqual(actual.assignments,engine.execute({graph,options:{...request.options,mka_metric:'reock'}}).assignments);assert.equal(actual.metrics.structure_evidence.orientations_per_split,Math.max(1,mka_orientations));
  verifyLabAssignments(graph,{metrics:{...actual.metrics,district_count:2,districts:actual.metrics.district_metrics,balance_passed:actual.metrics.within_requested_tolerance}},actual.assignments,request.options);
  for(const field of ['requested_metric','orientations_per_split','effective_metric','scoring_boundary']) {const bad=structuredClone(actual.metrics);bad.structure_evidence[field]='tampered';assert.throws(()=>verifyLabAssignments(graph,{metrics:{...bad,district_count:2,districts:bad.district_metrics,balance_passed:bad.within_requested_tolerance}},actual.assignments,request.options),/moving-knife evidence/);}
  misses+=!actual.metrics.within_requested_tolerance;disconnected+=!actual.metrics.contiguous;count++;
}
let failures=0;
for(const options of [{...base,mka_orientations:10001},{...base,mka_orientations:-1},{...base,mka_metric:'invalid'},{...base,structure:'standard-bisect',mka_orientations:20},{...base,structure:'standard-bisect',mka_metric:'geographic'}]) {
  const request={graph,options};assert.throws(()=>engine.execute(request));const file=`target/mka-invalid-${failures++}.json`;await fs.writeFile(file,JSON.stringify(request));assert.notEqual(spawnSync('target/release/examples/execute_request.exe',[file],{encoding:'utf8'}).status,0);
}
const legacy=engine.execute({graph,options:base});assert.deepEqual(legacy.assignments,engine.execute({graph,options:{...base,mka_orientations:36,mka_metric:'reock'}}).assignments);
console.log(`Moving knife: ${count} exact native/WASM assignments and metrics; both requested scoring metrics, zero/one/36/180 orientations and high seeds; five invalid settings rejected. Native outcomes: ${misses} balance misses, ${disconnected} disconnected plans; no success claim inferred from parity.`);
await fs.writeFile('target/mka-native-wasm-parity.json',JSON.stringify({cases:count,exact_assignment_and_metric_matches:count,matching_rejections:failures,balance_misses:misses,disconnected_plans:disconnected,effective_metric:'reock',polsby_fallback:true},null,2));
