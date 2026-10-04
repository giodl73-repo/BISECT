import fs from 'node:fs/promises';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {validateLabConfig,verifyLabAssignments} from '../../web/lab/laboratory-project.js';
import {engineOptions} from '../../web/lab/static.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
import {createLabProject} from '../../web/lab/laboratory-project.js';
import pathModule from 'node:path';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const n=100,width=10,adjacency=Array.from({length:n},(_,i)=>[...(i%width?[i-1]:[]),...(i%width<width-1?[i+1]:[]),...(i>=width?[i-width]:[]),...(i<n-width?[i+width]:[])]);
const graph={schema_version:1,state:'RI',year:'2020',geoids:Array.from({length:n},(_,i)=>`44001${String(i).padStart(6,'0')}`),adjacency,population:Array.from({length:n},(_,i)=>100+i%7),edges:adjacency.flatMap((ns,u)=>ns.filter(v=>v>u).map(v=>[u,v,1+(u%9)/10])),areas:Array(n).fill(100),exterior_perimeters:Array(n).fill(0),centroids:Array.from({length:n},(_,i)=>[-71+i%width*.001,41+Math.floor(i/width)*.001])};
const base={structure:'nway',weights:'geographic',search:'single',districts:4,seed:42,seeds:1,steps:20,percentile:0,alpha_county:0,balance_tolerance:5,area_swing:1.1,iterations:10};
const config={...base,name:'N-way refinement controls',mode:'state',states:['RI'],year:'2020',chamber:'congressional',timeout_seconds:30,metis_objective:'volume',metis_trials:3};
validateLabConfig(config);assert.deepEqual(engineOptions(config),{metis_objective:'volume',metis_trials:3});
for(const change of [{metis_objective:'other'},{metis_trials:0},{metis_trials:101},{structure:'standard-bisect',search:'flip'}])assert.throws(()=>validateLabConfig({...config,...change}));
const legacy={...config};delete legacy.metis_objective;delete legacy.metis_trials;validateLabConfig(legacy);assert.deepEqual(engineOptions(legacy),{});
const path='target/nway-tuning-request.json';let cases=0,rejections=0,misses=0,disconnected=0;const signatures=new Set();
for(const metis_objective of ['cut','volume'])for(const metis_trials of [1,3])for(const iterations of [1,10])for(const seed of [42,4294967297,Number.MAX_SAFE_INTEGER]){
  const options={...base,metis_objective,metis_trials,iterations,seed},request={graph,options};await fs.writeFile(path,JSON.stringify(request));
  const native=spawnSync('target/release/examples/execute_request.exe',[path],{encoding:'utf8',maxBuffer:16*1024*1024});assert.equal(native.status,0,native.stderr);
  const result=engine.execute(request);assert.deepEqual(result,JSON.parse(native.stdout));
  assert.deepEqual(engine.execute(request),result);signatures.add(JSON.stringify(result.assignments));
  const state={metrics:{...result.metrics,district_count:4,districts:result.metrics.district_metrics,balance_passed:result.metrics.within_requested_tolerance}};
  verifyLabAssignments(graph,state,result.assignments,options);
  for(const key of ['refinement_objective','internal_trials','refinement_iterations','trial_selection']){const bad=structuredClone(state);bad.metrics.structure_evidence[key]='changed';assert.throws(()=>verifyLabAssignments(graph,bad,result.assignments,options),/n-way METIS evidence/);}
  misses+=!result.metrics.within_requested_tolerance;disconnected+=!result.metrics.contiguous;cases++;
}
const old=engine.execute({graph,options:base}),explicit=engine.execute({graph,options:{...base,metis_objective:'cut',metis_trials:1}});assert.deepEqual(old.assignments,explicit.assignments);assert.equal(old.metrics.structure_evidence,null);
for(const changed of [{metis_objective:'bad'},{metis_trials:0},{metis_trials:101},{metis_trials:-1},{structure:'standard-bisect',search:'flip',metis_objective:'volume'}]){const request={graph,options:{...base,...changed}};assert.throws(()=>engine.execute(request));await fs.writeFile(path,JSON.stringify(request));assert.notEqual(spawnSync('target/release/examples/execute_request.exe',[path],{stdio:'pipe'}).status,0);rejections++;}
let projects=0;
if(process.argv[2]){
  const root=process.argv[2],manifest=JSON.parse(await fs.readFile(pathModule.join(root,'catalog.json'),'utf8'));
  const fetcher=async file=>{const b=await fs.readFile(pathModule.join(root,file));return {ok:true,arrayBuffer:async()=>b.buffer.slice(b.byteOffset,b.byteOffset+b.length)};};
  class Worker{postMessage(m){setImmediate(()=>{if(this.terminated)return;try{this.onmessage({data:m.type==='initialize'?{type:'ready'}:{type:'result',id:m.id,result:engine.execute(m.request)}});}catch(e){this.onmessage({data:{type:'error',id:m.id,error:e.message}});}});}terminate(){this.terminated=true;}}
  const lab=new WasmCatalog(manifest,fetcher,()=>new Worker());
  for(const code of Object.keys(manifest.graphs).filter(k=>k.endsWith(':2020')).map(k=>k.split(':')[0]).slice(0,3)){
    const real={...config,states:[code],districts:null},job=lab.submit(real);while(lab.active)await new Promise(r=>setTimeout(r,5));
    assert.equal(lab.jobs.get(job.id).status,'completed');
    const saved=await createLabProject(lab,job.id);assert.equal(saved.experiment.config.metis_objective,'volume');assert.equal(saved.experiment.config.metis_trials,3);
    const restoredLab=new WasmCatalog(manifest,fetcher,()=>{throw new Error('Project Open must not execute.');});
    const restored=await restoredLab.restoreProject(JSON.parse(JSON.stringify(saved)));assert.equal(restoredLab.worker,null);
    assert.deepEqual(await restoredLab.api(`/api/runs/${restored.id}/${code}/assignments`),saved.assignments[code]);
    const forged=structuredClone(saved);forged.experiment.states[0].metrics.structure_evidence.internal_trials++;await assert.rejects(restoredLab.restoreProject(forged),/n-way METIS evidence/);
    await fs.writeFile(`target/nway-volume-${code}.bisect`,JSON.stringify(saved));projects++;
  }
}
await fs.writeFile('target/nway-tuning-verification.json',JSON.stringify({cases,rejections,real_state_projects:projects,distinct_assignments:signatures.size,balance_misses:misses,disconnected,legacy_default_unchanged:true,trial_selection:'population-excess-then-edge-cut'},null,2));
console.log(`N-way METIS: ${cases} exact native/WASM cases, ${rejections} invalid-request rejections, default compatibility and evidence tamper rejection passed; ${signatures.size} distinct plans, ${misses} balance misses and ${disconnected} disconnected plans.`);
