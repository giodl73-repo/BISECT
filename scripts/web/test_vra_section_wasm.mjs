import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {verifyLabAssignments,verifyDemographicEvidence,validateLabConfig} from '../../web/lab/laboratory-project.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const n=30,adjacency=Array.from({length:n},(_,i)=>[i%6?i-1:-1,i%6<5?i+1:-1,i>=6?i-6:-1,i<24?i+6:-1].filter(v=>v>=0));
const graph={schema_version:1,state:'RI',year:'2020',geoids:Array.from({length:n},(_,i)=>`${i<15?'44001':'44003'}${String(i).padStart(6,'0')}`),adjacency,population:Array(n).fill(100),edges:adjacency.flatMap((list,i)=>list.filter(j=>i<j).map(j=>[i,j,1+(i%3)])),areas:Array(n).fill(100),exterior_perimeters:Array(n).fill(10),centroids:Array.from({length:n},(_,i)=>[-71+i%6*.001,41+Math.floor(i/6)*.001])};
const base={structure:'ratio-optimal-vra',search:'single',districts:5,weights:'geographic',seed:42,seeds:4,steps:20,percentile:0,alpha_county:0,balance_tolerance:0.5,area_swing:1.1,iterations:100,w_vra:0.4};
let cases=0,misses=0,disconnected=0,tamper=0,negativeScoreCases=0,uniformAlignmentCases=0;const records=[];
for(const profile of ['zero','uniform','stripe','decimal'])for(const districts of [2,3,5])for(const search of ['single','multi'])for(const w_vra of [0,0.4,1])for(const seed of [42,4294967297])for(const weight of ['geographic','unweighted','county','tiny']){
 const g=structuredClone(graph);if(weight==='tiny')g.edges=g.edges.map(([u,v,w])=>[u,v,w*1e-5]);
 const demographics={schema_version:'bisect-demographic-fractions-v1',state:'RI',year:'2020',basis:'total-population',source_label:'SYNTHETIC VRASection regression inputs',minority_fractions:Object.fromEntries(g.geoids.map((id,i)=>[id,profile==='zero'?0:profile==='uniform'?0.5:profile==='stripe'?(i<15?0.9:0.1):(i+1)/31]))};
 const options={...base,districts,search,w_vra,seed,alpha_county:weight==='county'?3:0,weights:weight==='tiny'?'geographic':weight},request={graph:g,demographics,options},file=`target/vra-section-request-${cases}.json`;await fs.writeFile(file,JSON.stringify(request));
 const actual=engine.execute(request),native=spawnSync('target/release/examples/execute_request.exe',[file],{encoding:'utf8',maxBuffer:16*1024*1024});assert.equal(native.status,0,native.stderr);assert.deepEqual(actual,JSON.parse(native.stdout));
 const config={...options,demographics:{RI:demographics},name:'VRASection matrix',mode:'state',states:['RI'],year:'2020',chamber:'congressional',timeout_seconds:30};validateLabConfig(config);
 const state={code:'RI',metrics:{...actual.metrics,district_count:districts,districts:actual.metrics.district_metrics,balance_passed:actual.metrics.within_requested_tolerance}};
 verifyLabAssignments(g,state,actual.assignments,config);await verifyDemographicEvidence(g,state,config);
 const e=actual.metrics.structure_evidence;
 if(w_vra===0||profile==='zero'){
  const plain={...options,structure:'ratio-optimal'};delete plain.w_vra;assert.deepEqual(actual.assignments,engine.execute({graph:g,options:plain}).assignments);
 }
 if(cases===0){
  for(const key of Object.keys(e).filter(key=>key!=='demographics_sha256')){const bad=structuredClone(state);bad.metrics.structure_evidence[key]='tampered';assert.throws(()=>verifyLabAssignments(g,bad,actual.assignments,config),/VRASection/);tamper++;}
  const bad=structuredClone(state);bad.metrics.structure_evidence.demographics_sha256='0'.repeat(64);await assert.rejects(verifyDemographicEvidence(g,bad,config),/identity mismatch/);tamper++;
 }
 negativeScoreCases+=e.selection_score<0;uniformAlignmentCases+=profile==='uniform'&&e.alignment>0;misses+=!actual.metrics.within_requested_tolerance;disconnected+=!actual.metrics.contiguous;records.push({profile,districts,search,w_vra,seed,weight,root_ratio:actual.metrics.root_split.left_districts,alignment:e.alignment,score:e.selection_score});cases++;
}
const valid={graph,options:base,demographics:{schema_version:'bisect-demographic-fractions-v1',state:'RI',year:'2020',basis:'total-population',source_label:'Synthetic',minority_fractions:Object.fromEntries(graph.geoids.map(id=>[id,0.5]))}};let rejected=0;
for(const mutate of [r=>delete r.demographics,r=>delete r.options.w_vra,r=>r.options.w_vra=1.01,r=>r.options.w_vra=-0.1,r=>r.options.structure='ratio-optimal',r=>r.options.search='percentile',r=>r.demographics.basis='voting-age-population',r=>delete r.demographics.minority_fractions[graph.geoids[0]],r=>r.options.vra_threshold=0.5]){
 const r=structuredClone(valid);mutate(r);assert.throws(()=>engine.execute(r));const file=`target/vra-section-invalid-${rejected}.json`;await fs.writeFile(file,JSON.stringify(r));assert.notEqual(spawnSync('target/release/examples/execute_request.exe',[file],{encoding:'utf8'}).status,0);rejected++;
}
assert.ok(negativeScoreCases>0,'Exercise max(normalized,1) for tiny cuts.');assert.ok(uniformAlignmentCases>0,'Preserve native unequal-half mass alignment.');
await fs.writeFile('target/vra-section-native-wasm-parity.json',JSON.stringify({cases,exact_matches:cases,matching_rejections:rejected,ledger_tamper_rejections:tamper,balance_misses:misses,disconnected_plans:disconnected,negative_score_cases:negativeScoreCases,uniform_fraction_alignment_cases:uniformAlignmentCases,records},null,2));
console.log(`VRASection: ${cases} exact native/WASM cases; ${rejected} matching input/option rejections, ${tamper} ledger tamper rejections; ${negativeScoreCases} negative adjusted-score cases and ${uniformAlignmentCases} uniform-fraction alignment cases; ${misses} balance misses, ${disconnected} disconnected plans.`);
