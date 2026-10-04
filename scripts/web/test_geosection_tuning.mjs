import fs from 'node:fs/promises';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {verifyLabAssignments,validateLabConfig,createLabProject} from '../../web/lab/laboratory-project.js';
import {engineOptions} from '../../web/lab/static.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
import pathModule from 'node:path';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const n=144,w=12,adjacency=Array.from({length:n},(_,i)=>[...(i%w?[i-1]:[]),...(i%w<w-1?[i+1]:[]),...(i>=w?[i-w]:[]),...(i<n-w?[i+w]:[])]);
const graph={schema_version:1,state:'RI',year:'2020',geoids:Array.from({length:n},(_,i)=>`44001${String(i).padStart(6,'0')}`),adjacency,population:Array(n).fill(100),edges:adjacency.flatMap((ns,u)=>ns.filter(v=>v>u).map(v=>[u,v,1+(u%7)/10])),areas:Array(n).fill(100),exterior_perimeters:Array(n).fill(0),centroids:Array.from({length:n},(_,i)=>[-71+i%w*.001,41+Math.floor(i/w)*.001])};
const base={structure:'ratio-optimal',weights:'geographic',search:'single',districts:4,seed:42,seeds:4,steps:20,percentile:0,alpha_county:0,balance_tolerance:10,area_swing:1.1,iterations:10};
const config={...base,name:'GeoSection METIS controls',mode:'state',states:['RI'],year:'2020',chamber:'congressional',timeout_seconds:60,metis_objective:'volume',metis_trials:3};
validateLabConfig(config);assert.deepEqual(engineOptions(config),{metis_objective:'volume',metis_trials:3});
for(const search of ['merge-split','flip','forest-recom'])assert.throws(()=>validateLabConfig({...config,search}));
let cases=0,failed=0,misses=0,disconnected=0,tamper=0;
const file='target/geosection-tuning-request.json';
for(const districts of [2,3,4,6])for(const search of ['single','multi'])for(const metis_objective of ['cut','volume'])for(const metis_trials of [1,3])for(const iterations of [1,10])for(const seed of [42,4294967297,'18446744073709551615']){
  const options={...base,districts,search,metis_objective,metis_trials,iterations,seed},request={graph,options};await fs.writeFile(file,JSON.stringify(request).replace(/"seed":"([0-9]+)"/, '"seed":$1'));
  const native=spawnSync('target/release/examples/execute_request.exe',[file],{encoding:'utf8',maxBuffer:16*1024*1024});
  if(native.status!==0){assert.throws(()=>engine.execute(request));failed++;continue;}
  const result=engine.execute(request);assert.deepEqual(result,JSON.parse(native.stdout));assert.deepEqual(engine.execute(request),result);
  const state={metrics:{...result.metrics,district_count:districts,districts:result.metrics.district_metrics,balance_passed:result.metrics.within_requested_tolerance}};
  verifyLabAssignments(graph,state,result.assignments,options);
  for(const key of ['method','refinement_objective','internal_trials_per_candidate','refinement_iterations','scope','internal_trial_selection','candidate_selection','ratio_selection','seeds_per_ratio','seed_schedule','post_refinement','partial_assignment_fallback']){const bad=structuredClone(state);bad.metrics.structure_evidence[key]='changed';assert.throws(()=>verifyLabAssignments(graph,bad,result.assignments,options),/GeoSection refinement evidence/);tamper++;}
  const missing=structuredClone(state);delete missing.metrics.structure_evidence;assert.throws(()=>verifyLabAssignments(graph,missing,result.assignments,options),/GeoSection refinement evidence/);tamper++;
  misses+=!result.metrics.within_requested_tolerance;disconnected+=!result.metrics.contiguous;cases++;
}
assert.ok(cases>0);
for(const search of ['single','multi']){const old=engine.execute({graph,options:{...base,search}}),tuned=engine.execute({graph,options:{...base,search,metis_objective:'cut',metis_trials:1}});assert.deepEqual(old.assignments,tuned.assignments);assert.equal(old.metrics.structure_evidence,null);}
const oneOptions={...base,districts:1,metis_objective:'volume',metis_trials:3},one=engine.execute({graph,options:oneOptions});assert.equal(one.metrics.structure_evidence,null);verifyLabAssignments(graph,{metrics:{...one.metrics,district_count:1,districts:one.metrics.district_metrics,balance_passed:true}},one.assignments,oneOptions);
let rejected=0;for(const change of [{metis_trials:0},{metis_trials:101},{metis_objective:'bad'},{search:'flip'},{structure:'bfs-growth'}]){const request={graph,options:{...base,metis_objective:'volume',metis_trials:3,...change}};assert.throws(()=>engine.execute(request));await fs.writeFile(file,JSON.stringify(request).replace(/"seed":"([0-9]+)"/, '"seed":$1'));assert.notEqual(spawnSync('target/release/examples/execute_request.exe',[file],{stdio:'pipe'}).status,0);rejected++;}
let projects=0;
if(process.argv[2]){
 const root=process.argv[2],manifest=JSON.parse(await fs.readFile(pathModule.join(root,'catalog.json'),'utf8'));
 const fetcher=async name=>{const b=await fs.readFile(pathModule.join(root,name));return{ok:true,arrayBuffer:async()=>b.buffer.slice(b.byteOffset,b.byteOffset+b.length)};};
 class Worker{postMessage(m){setImmediate(()=>{if(this.terminated)return;try{this.onmessage({data:m.type==='initialize'?{type:'ready'}:{type:'result',id:m.id,result:engine.execute(m.request)}});}catch(e){this.onmessage({data:{type:'error',id:m.id,error:e.message}});}});}terminate(){this.terminated=true;}}
 const lab=new WasmCatalog(manifest,fetcher,()=>new Worker());
 for(const code of Object.keys(manifest.graphs).filter(k=>k.endsWith(':2020')).map(k=>k.split(':')[0]).slice(0,3))for(const search of ['single','multi']){
   const job=lab.submit({...config,states:[code],districts:null,search});while(lab.active)await new Promise(r=>setTimeout(r,5));assert.equal(lab.jobs.get(job.id).status,'completed');
   const saved=await createLabProject(lab,job.id),restore=new WasmCatalog(manifest,fetcher,()=>{throw new Error('Opening must not generate.');});const reopened=await restore.restoreProject(JSON.parse(JSON.stringify(saved)));assert.equal(restore.worker,null);
   assert.deepEqual(await restore.api(`/api/runs/${reopened.id}/${code}/assignments`),saved.assignments[code]);
   const bad=structuredClone(saved);bad.experiment.states[0].metrics.structure_evidence.seeds_per_ratio++;await assert.rejects(restore.restoreProject(bad),/GeoSection refinement evidence/);
   const exported=await lab.exportPractitionerProject(job.id,code);assert.equal(exported.project.files.plan.provenance.producer.engine_options.metis_objective,'volume');assert.equal(exported.project.files.plan.provenance.producer.engine_options.metis_trials,3);
   await fs.writeFile(`target/geosection-volume-${code}-${search}.bisect`,JSON.stringify(saved));projects++;
 }
}
await fs.writeFile('target/geosection-tuning-verification.json',JSON.stringify({exact_cases:cases,matching_generation_failures:failed,rejected,tamper,real_state_projects:projects,balance_misses:misses,disconnected,legacy_default_unchanged:true},null,2));
console.log(`GeoSection METIS: ${cases} exact native/WASM cases, ${failed} matching generation failures, ${rejected} invalid requests, ${tamper} evidence tamper rejections, ${projects} real-state project roundtrips; ${misses} balance misses, ${disconnected} disconnected plans.`);
