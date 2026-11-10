import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
import {createLabProject,parseLabProject,validateLabConfig} from '../../web/lab/laboratory-project.js';
import {engineOptions} from '../../web/lab/static.js';
const root='dist/wasm-lab-build',out='target/convergence-unlimited';await fs.mkdir(out,{recursive:true});
const manifest=JSON.parse(await fs.readFile(root+'/catalog.json','utf8')),engine=await instantiateEngine(await fs.readFile(root+'/bisect_wasm.wasm'));
const options={structure:'standard-bisect',weights:'geographic',search:'convergence',districts:2,seed:42,seeds:2,steps:1,percentile:0,alpha_county:0,balance_tolerance:5,area_swing:1.1,iterations:10,convergence_unlimited:true};
let matches=0,overCap=0;const cases=[];
for(const code of ['RI','IA'])for(const weights of ['geographic','unweighted'])for(const seed of [42,'18446744073709551615']){
 const graph=JSON.parse(await fs.readFile(root+'/'+manifest.graphs[code+':2020'].graph_ref,'utf8')),districts=manifest.catalog.states.find(s=>s.code===code).years.find(y=>y.year==='2020').districts.congressional;
 const request={graph,options:{...options,districts,weights,seed}},file=path.resolve(out+'/request.json');await fs.writeFile(file,JSON.stringify(request));
 const native=JSON.parse(execFileSync('target/release/examples/execute_request.exe',[file],{encoding:'utf8',maxBuffer:16*1024*1024})),actual=engine.execute(request);assert.deepEqual(actual,native);matches++;
 const evidence=actual.metrics.structure_evidence;assert.equal(evidence.seed_limit,null);assert.equal(evidence.halt,'threshold');assert.equal(evidence.consecutive_non_improving,2);assert(evidence.attempted>request.options.steps);overCap++;
 // Replay every attempted candidate, including seed wrap, independently of sweep.
 let best,tail=0,rejected=0;for(let index=0;index<evidence.attempted;index++){
  const candidateOptions={...request.options,search:'single',seed:((BigInt(seed)+BigInt(index))%(1n<<64n)).toString()};delete candidateOptions.convergence_unlimited;
  let candidate;try{candidate=engine.execute({graph,options:candidateOptions});}catch{}
  if(!candidate||!candidate.metrics.contiguous||!candidate.metrics.within_requested_tolerance){rejected++;tail++;continue;}
  const cost=normalized(graph,candidate.assignments,districts,weights);
  if(!best||cost<best.cost){best={cost,index,candidate};tail=0;}else tail++;
 }
 assert(best);assert.deepEqual(actual.assignments,best.candidate.assignments);assert.equal(evidence.selected_index,best.index);assert.equal(evidence.rejected,rejected);assert.equal(evidence.best_normalized_cut,best.cost);assert.equal(tail,2);
 const omitted={...request.options};delete omitted.convergence_unlimited;const explicitFalse=engine.execute({graph,options:{...omitted,convergence_unlimited:false}}),legacy=engine.execute({graph,options:omitted});assert.deepEqual(explicitFalse.assignments,legacy.assignments);assert.deepEqual(explicitFalse.metrics.structure_evidence,legacy.metrics.structure_evidence);assert.equal(legacy.metrics.structure_evidence.seed_limit,1);
 cases.push({code,weights,seed,attempted:evidence.attempted});
}
function normalized(graph,assignments,k,weights){let sum=0;for(const [u,v,length]of [...graph.edges].sort((a,b)=>a[0]-b[0]||a[1]-b[1])){const du=assignments[graph.geoids[u]],dv=assignments[graph.geoids[v]];if(du===dv)continue;let first=1,count=k;while(true){const left=Math.floor(count/2),middle=first+left;if((du<middle)!==(dv<middle)){sum+=(weights==='unweighted'?1:length)/Math.sqrt(Math.min(left,count-left));break;}if(du<middle)count=left;else{first=middle;count-=left;}}}return sum;}
const config={name:'Native uncapped convergence',mode:'state',states:['RI'],year:'2020',chamber:'congressional',...options,metis_objective:'cut',metis_trials:1,districts:null,timeout_seconds:30};validateLabConfig(config);assert.equal(engineOptions(config).convergence_unlimited,true);
for(const bad of [{...config,search:'single'},{...config,structure:'nway'},{...config,convergence_unlimited:'true'},{...config,convergence_unlimited:1}])assert.throws(()=>validateLabConfig(bad));
let executions=0;class Worker{postMessage(m){setImmediate(()=>{if(this.dead)return;try{if(m.type==='initialize')this.onmessage({data:{type:'ready'}});else{executions++;this.onmessage({data:{type:'result',id:m.id,result:engine.execute(m.request)}});}}catch(e){this.onmessage({data:{type:'error',id:m.id,error:e.message}});}});}terminate(){this.dead=true;}}
const fetcher=async name=>{const b=await fs.readFile(root+'/'+name);return{ok:true,arrayBuffer:async()=>b.buffer.slice(b.byteOffset,b.byteOffset+b.byteLength)};};
for(const mode of ['state','national']){const lab=new WasmCatalog(manifest,fetcher,()=>new Worker()),job=lab.submit({...config,mode,states:mode==='state'?['RI']:['RI','IA']});while(lab.active)await new Promise(r=>setTimeout(r,5));assert.equal(lab.jobs.get(job.id).status,'completed');const saved=await createLabProject(lab,job.id);await fs.writeFile(out+'/'+mode+'.bisect',JSON.stringify(saved));const restored=new WasmCatalog(manifest,fetcher,()=>{throw new Error('Open must be inert');}),before=executions;await restored.restoreProject(parseLabProject(JSON.stringify(saved)));assert.equal(executions,before);
 for(const tamper of [p=>p.experiment.states[0].metrics.structure_evidence.seed_limit=1,p=>p.experiment.states[0].metrics.structure_evidence.halt='seed-limit',p=>p.experiment.config.convergence_unlimited=false,p=>p.experiment.states[0].metrics.structure_evidence.threshold++]){const bad=structuredClone(saved);tamper(bad);await assert.rejects(restored.restoreProject(bad),/convergence|Recorded options/);}
}
await fs.writeFile(out+'/evidence.json',JSON.stringify({whole_native_wasm_matches:matches,uncapped_runs_exceed_inactive_cap:overCap,independent_candidate_replay:true,max_u64_seed_wrap:true,legacy_and_false_unchanged:true,state_national_inert_roundtrip:true,tampered_stopping_evidence_refused:true,cases},null,2));console.log('Uncapped convergence: eight real-state native/WASM matches, independent sweep replay, seed wrap, cap compatibility and secure state/national Save/Open passed.');
