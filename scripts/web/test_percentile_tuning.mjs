import fs from 'node:fs/promises';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import path from 'node:path';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {validateLabConfig,verifyLabAssignments,createLabProject} from '../../web/lab/laboratory-project.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const n=100,w=10,adjacency=Array.from({length:n},(_,i)=>[...(i%w?[i-1]:[]),...(i%w<w-1?[i+1]:[]),...(i>=w?[i-w]:[]),...(i<n-w?[i+w]:[])]);
const graph={schema_version:1,state:'RI',year:'2020',geoids:Array.from({length:n},(_,i)=>`44001${String(i).padStart(6,'0')}`),adjacency,population:Array(n).fill(100),edges:adjacency.flatMap((ns,u)=>ns.filter(v=>v>u).map(v=>[u,v,1+u%7/10])),areas:Array(n).fill(100),exterior_perimeters:Array(n).fill(0),centroids:Array.from({length:n},(_,i)=>[-71+i%w*.001,41+Math.floor(i/w)*.001])};
const base={structure:'standard-bisect',weights:'geographic',search:'percentile',districts:4,seed:42,seeds:4,steps:20,percentile:.5,alpha_county:0,balance_tolerance:10,area_swing:1.1,iterations:10};
const config={...base,name:'Percentile METIS controls',mode:'state',states:['RI'],year:'2020',chamber:'congressional',timeout_seconds:60,metis_objective:'volume',metis_trials:3};validateLabConfig(config);
let cases=0,tamper=0,misses=0,disconnected=0,replays=0,projects=0;
const file='target/percentile-tuning-request.json';
function native(){const p=spawnSync('target/release/examples/execute_request.exe',[file],{encoding:'utf8',maxBuffer:16*1024*1024});assert.equal(p.status,0,p.stderr);return JSON.parse(p.stdout);}
for(const districts of [3,4])for(const metis_objective of ['cut','volume'])for(const metis_trials of [1,3])for(const percentile of [0,.5,1])for(const seed of [42,4294967297]){
 const options={...base,districts,metis_objective,metis_trials,percentile,seed},request={graph,options};await fs.writeFile(file,JSON.stringify(request));const result=engine.execute(request);assert.deepEqual(result,native());
 const e=result.metrics.structure_evidence,indices=e.cuts_by_seed.map((_,i)=>i).sort((a,b)=>e.cuts_by_seed[a]-e.cuts_by_seed[b]||a-b);assert.equal(e.rank,Math.min(Math.floor(percentile*options.seeds),options.seeds-1));assert.equal(e.selected_seed_index,indices[e.rank]);
 const state={metrics:{...result.metrics,district_count:districts,districts:result.metrics.district_metrics,balance_passed:result.metrics.within_requested_tolerance}};verifyLabAssignments(graph,state,result.assignments,options);
 for(const key of ['method','refinement_objective','internal_trials_per_candidate','refinement_iterations','seed_count','rank','selected_seed_index','selected_edge_cut','seed_walk','selection','scope','internal_trial_selection']){const bad=structuredClone(state);bad.metrics.structure_evidence[key]='changed';assert.throws(()=>verifyLabAssignments(graph,bad,result.assignments,options),/percentile/);tamper++;}
 const bad=structuredClone(state);bad.metrics.structure_evidence.cuts_by_seed[e.selected_seed_index]++;assert.throws(()=>verifyLabAssignments(graph,bad,result.assignments,options),/percentile/);tamper++;
 // Independently reconstruct the exact fixed-width native seed walk. Replay
 // native Single candidates with raw u64 decimal JSON to avoid JS rounding.
 if(seed===42&&percentile===.5){const plans=[];for(let i=0;i<options.seeds;i++){const index=Buffer.alloc(8),root=Buffer.alloc(8);index.writeBigUInt64LE(BigInt(i));root.writeBigUInt64LE(BigInt(seed));const digest=createHash('sha256').update('PERCENTILE_SWEEP_V1_').update(index).update('_').update(root).digest();const derived=digest.readBigUInt64LE();const single={graph,options:{...options,search:'single',seed:0}};const text=JSON.stringify(single).replace('"seed":0',`"seed":${derived}`);await fs.writeFile(file,text);const plan=native().assignments;const cut=graph.edges.filter(([u,v])=>plan[graph.geoids[u]]!==plan[graph.geoids[v]]).length;assert.equal(e.cuts_by_seed[i],cut);plans.push(plan);replays++;}assert.deepEqual(result.assignments,plans[e.selected_seed_index]);}
 misses+=!result.metrics.within_requested_tolerance;disconnected+=!result.metrics.contiguous;cases++;
}
for(const percentile of [0,.5,1]){const old=engine.execute({graph,options:{...base,percentile}}),explicit=engine.execute({graph,options:{...base,percentile,metis_objective:'cut',metis_trials:1}});assert.deepEqual(old.assignments,explicit.assignments);assert.equal(old.metrics.structure_evidence,null);}
if(process.argv[2]){
 const root=process.argv[2],manifest=JSON.parse(await fs.readFile(path.join(root,'catalog.json'),'utf8'));const fetcher=async name=>{const b=await fs.readFile(path.join(root,name));return{ok:true,arrayBuffer:async()=>b.buffer.slice(b.byteOffset,b.byteOffset+b.length)};};
 class Worker{postMessage(m){setImmediate(()=>{if(this.terminated)return;try{this.onmessage({data:m.type==='initialize'?{type:'ready'}:{type:'result',id:m.id,result:engine.execute(m.request)}});}catch(e){this.onmessage({data:{type:'error',id:m.id,error:e.message}});}});}terminate(){this.terminated=true;}}
 const lab=new WasmCatalog(manifest,fetcher,()=>new Worker());for(const code of Object.keys(manifest.graphs).filter(k=>k.endsWith(':2020')).map(k=>k.split(':')[0]).slice(0,3)){const job=lab.submit({...config,states:[code],districts:null});while(lab.active)await new Promise(r=>setTimeout(r,5));assert.equal(lab.jobs.get(job.id).status,'completed');const saved=await createLabProject(lab,job.id),restore=new WasmCatalog(manifest,fetcher,()=>{throw new Error('Open must not generate.');});const reopened=await restore.restoreProject(JSON.parse(JSON.stringify(saved)));assert.equal(restore.worker,null);assert.deepEqual(await restore.api(`/api/runs/${reopened.id}/${code}/assignments`),saved.assignments[code]);const bad=structuredClone(saved);bad.experiment.states[0].metrics.structure_evidence.selected_edge_cut++;await assert.rejects(restore.restoreProject(bad),/percentile/);await fs.writeFile(`target/percentile-volume-${code}.bisect`,JSON.stringify(saved));projects++;}
}
await fs.writeFile('target/percentile-tuning-verification.json',JSON.stringify({exact_cases:cases,independent_native_candidate_replays:replays,tamper,real_state_projects:projects,balance_misses:misses,disconnected,legacy_default_unchanged:true},null,2));
console.log(`Percentile METIS: ${cases} exact native/WASM cases, ${replays} independent seed-walk candidate replays, ${tamper} tamper rejections, ${projects} real-state project roundtrips; ${misses} balance misses, ${disconnected} disconnected plans.`);
