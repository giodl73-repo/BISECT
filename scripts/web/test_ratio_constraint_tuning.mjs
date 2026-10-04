import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {validateLabConfig,verifyLabAssignments,createLabProject} from '../../web/lab/laboratory-project.js';
import {engineOptions} from '../../web/lab/static.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const n=144,w=12,adjacency=Array.from({length:n},(_,i)=>[i%w?i-1:-1,i%w<w-1?i+1:-1,i>=w?i-w:-1,i<n-w?i+w:-1].filter(v=>v>=0));
const graph={schema_version:1,state:'RI',year:'2020',geoids:Array.from({length:n},(_,i)=>`44001${String(i).padStart(6,'0')}`),adjacency,population:Array(n).fill(100),edges:adjacency.flatMap((ns,u)=>ns.filter(v=>v>u).map(v=>[u,v,1+(u%7)/10])),areas:Array(n).fill(100000),exterior_perimeters:Array(n).fill(10),centroids:Array.from({length:n},(_,i)=>[-71+i%w*.001,41+Math.floor(i/w)*.001])};
const demographic=g=>({schema_version:'bisect-demographic-fractions-v1',state:g.state,year:g.year,basis:'total-population',source_label:'SYNTHETIC tuning regression fractions; not Census demographics',minority_fractions:Object.fromEntries(g.geoids.map((id,i)=>[id,i<g.geoids.length/2?.8:.2]))});
const base={structure:'ratio-optimal-area',weights:'geographic',search:'single',districts:4,seed:42,seeds:4,steps:20,percentile:.5,alpha_county:0,balance_tolerance:10,area_swing:1.1,iterations:10};
const configBase={name:'Ratio constraints and refinement',mode:'state',states:['RI'],year:'2020',chamber:'congressional',timeout_seconds:60};
const fields=['objective','internal_trials','iterations','scope','internal_trial_selection','candidate_selection','seeds_per_ratio','seed_schedule','post_refinement','partial_assignment_fallback'];
let cases=0,failures=0,tamper=0,misses=0,disconnected=0,projects=0,exports=0,realFailures=0;const projectModes={};
const referenceFile='target/ratio-constraint-tuning-request.json';
async function native(request){await fs.writeFile(referenceFile,JSON.stringify(request).replace(/"seed":"([0-9]+)"/,'"seed":$1'));return spawnSync('target/release/examples/execute_request.exe',[referenceFile],{encoding:'utf8',maxBuffer:32*1024*1024});}
for(const structure of ['ratio-optimal-area','ratio-optimal-vra'])for(const districts of [2,4,6])for(const search of ['single','multi'])for(const metis_objective of ['cut','volume'])for(const metis_trials of [1,3])for(const seed of [42,'18446744073709551615'])for(const profile of [0,1,2]){
 const extra=structure==='ratio-optimal-vra'?{w_vra:[0,.4,1][profile]}:profile===2?{}:{area_init:profile===0?'ratio-optimal':'moving-knife'};
 const options={...base,...extra,structure,districts,search,metis_objective,metis_trials,seed};
 const request={graph,options,...(structure==='ratio-optimal-vra'?{demographics:demographic(graph)}:{})};
 const config={...configBase,...options,...(request.demographics?{demographics:{RI:request.demographics}}:{})};validateLabConfig(config);assert.equal(engineOptions(config).metis_objective,metis_objective);
 const ref=await native(request);
 if(ref.status!==0){const msg=JSON.parse(ref.stderr.trim().replace(/^Error: /,''));assert.throws(()=>engine.execute(request),e=>e.message===msg);failures++;continue;}
 const result=engine.execute(request);assert.deepEqual(result,JSON.parse(ref.stdout));
 const state={metrics:{...result.metrics,district_count:districts,districts:result.metrics.district_metrics,balance_passed:result.metrics.within_requested_tolerance}};
 verifyLabAssignments(graph,state,result.assignments,config);
 for(const key of fields){const bad=structuredClone(state);bad.metrics.structure_evidence.metis_refinement[key]='changed';assert.throws(()=>verifyLabAssignments(graph,bad,result.assignments,config),/ratio refinement evidence/);tamper++;}
 const missing=structuredClone(state);delete missing.metrics.structure_evidence.metis_refinement;assert.throws(()=>verifyLabAssignments(graph,missing,result.assignments,config),/ratio refinement evidence/);tamper++;
 misses+=!result.metrics.within_requested_tolerance;disconnected+=!result.metrics.contiguous;cases++;
}
for(const structure of ['ratio-optimal-area','ratio-optimal-vra']){
 const extra=structure==='ratio-optimal-vra'?{w_vra:.4}:{area_init:'ratio-optimal'},options={...base,...extra,structure};
 const request={graph,options,...(structure==='ratio-optimal-vra'?{demographics:demographic(graph)}:{})};
 assert.deepEqual(engine.execute(request).assignments,engine.execute({...request,options:{...options,metis_objective:'cut',metis_trials:1}}).assignments);
 const one={...options,districts:1,metis_objective:'volume',metis_trials:3};assert.equal(engine.execute({...request,options:one}).metrics.structure_evidence,null);
 for(const bad of [{metis_objective:'bad',metis_trials:1},{metis_objective:'cut',metis_trials:0},{metis_objective:'volume',metis_trials:101}]){const invalid={...request,options:{...options,...bad}};assert.throws(()=>engine.execute(invalid));assert.notEqual((await native(invalid)).status,0);}
}
if(process.argv[2]){
 const root=process.argv[2],manifest=JSON.parse(await fs.readFile(path.join(root,'catalog.json'),'utf8'));
 const fetcher=async ref=>{const b=await fs.readFile(path.join(root,ref));return{ok:true,arrayBuffer:async()=>b.buffer.slice(b.byteOffset,b.byteOffset+b.length)};};
 class Worker{postMessage(m){setImmediate(()=>{if(this.terminated)return;try{this.onmessage({data:m.type==='initialize'?{type:'ready'}:{type:'result',id:m.id,result:engine.execute(m.request)}});}catch(e){this.onmessage({data:{type:'error',id:m.id,error:e.message}});}});}terminate(){this.terminated=true;}}
 const lab=new WasmCatalog(manifest,fetcher,()=>new Worker());
 for(const [key,entry]of Object.entries(manifest.graphs))for(const structure of ['ratio-optimal-area','ratio-optimal-vra']){
  const [code,year]=key.split(':'),g=JSON.parse(await fs.readFile(path.join(root,entry.graph_ref),'utf8'));
  const extra=structure==='ratio-optimal-vra'?{w_vra:.4,demographics:{[code]:demographic(g)}}:{area_init:'ratio-optimal'};
  const config={...configBase,...base,...extra,states:[code],year,structure,search:'multi',seeds:8,districts:null,metis_objective:'volume',metis_trials:3};
  const job=lab.submit(config);while(lab.active)await new Promise(r=>setTimeout(r,5));const complete=lab.jobs.get(job.id);
  const options={...base,...(structure==='ratio-optimal-vra'?{w_vra:.4}:{area_init:'ratio-optimal'}),structure,search:'multi',seeds:8,districts:lab.districtCount(code,config),metis_objective:'volume',metis_trials:3};
  const request={graph:g,options,...(structure==='ratio-optimal-vra'?{demographics:extra.demographics[code]}:{})},ref=await native(request);
  if(complete.status!=='completed'){assert.notEqual(ref.status,0);realFailures++;continue;}
  assert.equal(ref.status,0,ref.stderr);assert.deepEqual(engine.execute(request),JSON.parse(ref.stdout));
  const saved=await createLabProject(lab,job.id),restore=new WasmCatalog(manifest,fetcher,()=>{throw new Error('Open must not generate.');});const opened=await restore.restoreProject(structuredClone(saved));assert.equal(opened.config.metis_objective,'volume');assert.deepEqual(await restore.api(`/api/runs/${opened.id}/${code}/assignments`),saved.assignments[code]);
  const exported=await lab.exportPractitionerProject(job.id,code);assert.equal(exported.project.files.plan.provenance.producer.engine_options.metis_trials,3);exports++;
  await fs.writeFile(`target/ratio-tuning-${code}-${structure}.bisect`,JSON.stringify(saved));projects++;projectModes[structure]=(projectModes[structure]||0)+1;
 }
 for(const mode of ['ratio-optimal-area','ratio-optimal-vra'])assert.ok(projectModes[mode]>0,`No real project fixture for ${mode}`);
}
await fs.writeFile('target/ratio-constraint-tuning-verification.json',JSON.stringify({exact_cases:cases,matching_failures:failures,tamper_rejections:tamper,balance_misses:misses,disconnected,real_state_projects:projects,practitioner_exports:exports,matching_real_state_failures:realFailures,project_modes:projectModes,legacy_assignments_preserved:true},null,2));
console.log(`Area/VRA tuning: ${cases} exact cases, ${failures} matching failures, ${tamper} evidence rejections; ${projects} real-state projects and ${exports} practitioner exports, ${realFailures} matching real-state failures; ${misses} balance misses, ${disconnected} disconnected plans.`);
