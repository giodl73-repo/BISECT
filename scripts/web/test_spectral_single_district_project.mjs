import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
import {createLabProject} from '../../web/lab/laboratory-project.js';
const root='target/spectral-single-district-project';await fs.mkdir(root,{recursive:true});const source='dist/wasm-lab-build',manifest=JSON.parse(await fs.readFile(source+'/catalog.json','utf8')),engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
class Worker{postMessage(m){queueMicrotask(()=>{try{this.onmessage({data:m.type==='initialize'?{type:'ready'}:{type:'result',id:m.id,result:engine.execute(m.request)}});}catch(e){this.onmessage({data:{type:'error',id:m.id,error:e.message}});}});}terminate(){}}
const fetcher=async ref=>{const b=await fs.readFile(source+'/'+ref);return{ok:true,arrayBuffer:async()=>b.buffer.slice(b.byteOffset,b.byteOffset+b.length)};};const base=JSON.parse(await fs.readFile('target/county-state-oracle/browser.bisect','utf8')).experiment.config;let cases=0,refusals=0;
for(const code of ['RI','IA'])for(const build of [false,true]){
 const config={...base,states:[code],structure:'spectral',weights:'unweighted',search:'single',districts:1,steps:20,alpha_county:0,balance_tolerance:10,...(build?{native_build:true}:{})},lab=new WasmCatalog(manifest,fetcher,()=>new Worker()),job=lab.submit(config);while(lab.active)await new Promise(r=>setTimeout(r,5));assert.equal(lab.jobs.get(job.id).status,'completed');const saved=await createLabProject(lab,job.id),row=saved.experiment.states[0];assert.deepEqual(row.metrics.structure_evidence,{schema_version:'bisect-spectral-run-summary-v1',method:'spectral',max_iters:20,tolerance:.1,k:1,edge_cut:0,nodes:[]});
 const restore=new WasmCatalog(manifest,fetcher,()=>{throw new Error('Open must be inert');});await restore.restoreProject(saved);
 for(const mutate of [e=>e.max_iters++,e=>e.k=2,e=>e.edge_cut=1,e=>e.nodes.push({}),e=>e.tolerance=.2]){const forged=structuredClone(saved);mutate(forged.experiment.states[0].metrics.structure_evidence);await assert.rejects(()=>restore.restoreProject(forged));refusals++;}
 const legacy=structuredClone(saved);legacy.experiment.states[0].metrics.structure_evidence=null;await restore.restoreProject(legacy);
 if(code==='RI'&&!build){await fs.writeFile(root+'/browser.bisect',JSON.stringify(saved));await fs.writeFile(root+'/legacy.bisect',JSON.stringify(legacy));}cases++;
}
await fs.writeFile(root+'/evidence.json',JSON.stringify({real_state_projects:cases,changed_summary_refusals:refusals,inert_open:true,historical_missing_summary_loadable_as_claim:true,native_population_build_summary_retained:true,wasm_sha256:engine.artifactIdentity.sha256},null,2));console.log(`Single-district spectral projects: ${cases} RI/IA ordinary/build runs, ${refusals} altered summaries refused; inert current and historical project retention passed.`);
