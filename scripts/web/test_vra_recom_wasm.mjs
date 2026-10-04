import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {demographicIdentity,tractMeanFractions} from '../../web/lab/demographic-input.js';
import {verifyLabAssignments,verifyDemographicEvidence} from '../../web/lab/laboratory-project.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const n=16,adjacency=Array.from({length:n},(_,i)=>[i%4>0?i-1:-1,i%4<3?i+1:-1,i>=4?i-4:-1,i<12?i+4:-1].filter(v=>v>=0));
const graph={schema_version:1,state:'RI',year:'2020',geoids:Array.from({length:n},(_,i)=>`44001${String(i).padStart(6,'0')}`),adjacency,population:Array(n).fill(100),edges:adjacency.flatMap((list,i)=>list.filter(j=>i<j).map(j=>[i,j,1+(i%3)])),areas:Array(n).fill(100),exterior_perimeters:Array(n).fill(10),centroids:Array.from({length:n},(_,i)=>[-71+(i%4)*.001,41+Math.floor(i/4)*.001])};
const base={structure:'standard-bisect',weights:'geographic',search:'vra-recom',districts:2,seed:42,seeds:8,steps:20,percentile:0,alpha_county:0,balance_tolerance:0.5,area_swing:1.1,iterations:100,vra_threshold:0.5};
let cases=0,balanceMisses=0,disconnected=0,rejectionCases=0;const records=[];
for(const profile of ['zero','one','stripe','decimal'])for(const districts of [2,4])for(const steps of [0,1,20])for(const seed of [42,4294967297])for(const percentile of [0,0.5,1])for(const vra_threshold of [0.25,0.5,0.75]){
 const demographics={schema_version:'bisect-demographic-fractions-v1',state:'RI',year:'2020',basis:'total-population',source_label:'Synthetic test data',minority_fractions:Object.fromEntries(graph.geoids.map((id,i)=>[id,profile==='zero'?0:profile==='one'?1:profile==='stripe'?(i<8?0.9:0.1):(i+1)/17]))};
 const request={graph,demographics,options:{...base,districts,steps,seed,percentile,vra_threshold}},file=`target/vra-request-${cases}.json`;await fs.writeFile(file,JSON.stringify(request));
 const actual=engine.execute(request),native=spawnSync('target/release/examples/execute_request.exe',[file],{encoding:'utf8',maxBuffer:16*1024*1024});assert.equal(native.status,0,native.stderr);const reference=JSON.parse(native.stdout);
 assert.deepEqual(actual.assignments,reference.assignments);assert.deepEqual(actual.metrics,reference.metrics);assert.deepEqual(actual.options,request.options);
 const e=actual.metrics.structure_evidence,r=e.run;
 const config={...request.options,demographics:{RI:demographics}},state={code:'RI',metrics:{...actual.metrics,district_count:districts,districts:actual.metrics.district_metrics,balance_passed:actual.metrics.within_requested_tolerance}};
 verifyLabAssignments(graph,state,actual.assignments,config);await verifyDemographicEvidence(graph,state,config);
 if(cases===0){
  for(const key of Object.keys(e).filter(key=>!['run','demographics_sha256'].includes(key))){const bad=structuredClone(state);bad.metrics.structure_evidence[key]='tampered';assert.throws(()=>verifyLabAssignments(graph,bad,actual.assignments,config),/VRA/);}
  for(const key of Object.keys(r)){const bad=structuredClone(state);bad.metrics.structure_evidence.run[key]='tampered';assert.throws(()=>verifyLabAssignments(graph,bad,actual.assignments,config),/VRA|demographic/);}
  const bad=structuredClone(state);bad.metrics.structure_evidence.demographics_sha256='0'.repeat(64);await assert.rejects(verifyDemographicEvidence(graph,bad,config),/identity mismatch/);
 }
 assert.equal(e.demographics_sha256,await demographicIdentity(demographics));assert.equal(e.minority_aggregation,'unweighted-tract-mean');assert.equal(e.threshold,vra_threshold);
 assert.equal(r.proposals,steps);assert.equal(r.accepted_moves+r.mh_rejections+r.minority_rejections,steps);assert.equal(r.retained_records,r.accepted_moves+1);assert.equal(r.selected_rank,Math.min(Math.floor(percentile*r.retained_records),r.retained_records-1));
 const initial=Object.fromEntries(graph.geoids.map((id,i)=>[id,r.initial_assignment[i]])),initialMeans=tractMeanFractions(graph,initial,demographics,districts),finalMeans=tractMeanFractions(graph,actual.assignments,demographics,districts);
 assert.deepEqual(r.protected_districts,initialMeans.flatMap((fraction,i)=>fraction>=vra_threshold?[i+1]:[]));
 for(const d of r.protected_districts)assert.ok(finalMeans[d-1]>=vra_threshold);
 if(!steps)assert.deepEqual(actual.assignments,engine.execute({graph,options:{...base,search:'single',vra_threshold:undefined,districts,seed,steps:1,percentile}}).assignments);
 balanceMisses+=!actual.metrics.within_requested_tolerance;disconnected+=!actual.metrics.contiguous;rejectionCases+=r.minority_rejections>0;
 records.push({profile,districts,steps,seed,percentile,threshold:vra_threshold,protected:r.protected_districts,minority_rejections:r.minority_rejections});cases++;
}
const valid={graph,options:base,demographics:{schema_version:'bisect-demographic-fractions-v1',state:'RI',year:'2020',basis:'total-population',source_label:'Synthetic',minority_fractions:Object.fromEntries(graph.geoids.map(id=>[id,0.5]))}};
let identityProbes=0;
for(const basis of ['total-population','voting-age-population','citizen-voting-age-population']){
 const request=structuredClone(valid);request.demographics.basis=basis;
 request.demographics.minority_fractions=Object.fromEntries(Object.entries(request.demographics.minority_fractions).reverse());
 const file=`target/vra-identity-${identityProbes}.json`;await fs.writeFile(file,JSON.stringify(request));
 const actual=engine.execute(request),native=spawnSync('target/release/examples/execute_request.exe',[file],{encoding:'utf8'});assert.equal(native.status,0,native.stderr);
 assert.deepEqual(actual,JSON.parse(native.stdout));assert.equal(actual.metrics.structure_evidence.demographics_sha256,await demographicIdentity(request.demographics));
 request.demographics.source_label='Different user label';assert.deepEqual(engine.execute(request),actual);identityProbes++;
}
const one=structuredClone(valid);one.options.districts=1;
const oneFile='target/vra-one-district.json';await fs.writeFile(oneFile,JSON.stringify(one));
const oneNative=spawnSync('target/release/examples/execute_request.exe',[oneFile],{encoding:'utf8'});assert.equal(oneNative.status,0,oneNative.stderr);
const oneActual=engine.execute(one);assert.deepEqual(oneActual,JSON.parse(oneNative.stdout));assert.ok(Object.values(oneActual.assignments).every(d=>d===1));assert.equal(oneActual.metrics.structure_evidence,null);
let rejected=0;
for(const mutate of [r=>delete r.demographics,r=>delete r.options.vra_threshold,r=>r.options.vra_threshold=1.01,r=>r.options.balance_tolerance=5,r=>r.options.search='forest-recom',r=>r.demographics.year='2010',r=>r.demographics.basis='unknown',r=>delete r.demographics.minority_fractions[graph.geoids[0]],r=>r.demographics.minority_fractions[graph.geoids[0]]=1.01,r=>r.demographics.minority_fractions['99999999999']=0,r=>r.demographics.extra=true,r=>r.options.structure='nway']){
 const request=structuredClone(valid);mutate(request);const file=`target/vra-invalid-${rejected}.json`;await fs.writeFile(file,JSON.stringify(request));assert.throws(()=>engine.execute(request));assert.notEqual(spawnSync('target/release/examples/execute_request.exe',[file],{encoding:'utf8'}).status,0);rejected++;
}
assert.ok(rejectionCases,'The matrix must exercise actual minority-rule rejections.');
await fs.writeFile('target/vra-native-wasm-parity.json',JSON.stringify({cases,exact_matches:cases,identity_probes:identityProbes,one_district_match:true,matching_rejections:rejected,balance_misses:balanceMisses,disconnected_plans:disconnected,minority_rejection_cases:rejectionCases,records},null,2));
console.log(`VRA ReCom boundary: ${cases} exact native/WASM cases, ${rejected} matching rejections, complete input identities and protected-district witnesses; ${rejectionCases} cases exercise minority rollback, ${balanceMisses} balance misses, ${disconnected} disconnected plans. Independent preservation-ledger and demographic-hash checks passed; UI/project roundtrips have a separate real-input gate.`);
