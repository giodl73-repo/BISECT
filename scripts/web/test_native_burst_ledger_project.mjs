import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
import {createLabProject} from '../../web/lab/laboratory-project.js';
const source='dist/wasm-lab-build',manifest=JSON.parse(await fs.readFile(source+'/catalog.json','utf8')),engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));manifest.wasm_sha256=engine.artifactIdentity.sha256;
class Worker{postMessage(m){queueMicrotask(()=>{try{this.onmessage({data:m.type==='initialize'?{type:'ready'}:{type:'result',id:m.id,result:engine.execute(m.request)}});}catch(e){this.onmessage({data:{type:'error',id:m.id,error:e.message}});}});}terminate(){}}
const fetcher=async ref=>{const b=await fs.readFile(source+'/'+ref);return{ok:true,arrayBuffer:async()=>b.buffer.slice(b.byteOffset,b.byteOffset+b.length)};};
const fixture=JSON.parse(await fs.readFile('target/partisan-bursts/browser.bisect','utf8'));let projects=0,refusals=0;
for(const legacy of [false,true]){
 const config={...fixture.experiment.config,search:'short-burst',steps:21};if(legacy){delete config.burst_length;delete config.n_bursts;}else{config.burst_length=0;config.n_bursts=3;config.percentile=1;}
 const lab=new WasmCatalog(manifest,fetcher,()=>new Worker()),job=lab.submit(config);while(lab.active)await new Promise(r=>setTimeout(r,5));assert.equal(lab.jobs.get(job.id).status,'completed',lab.jobs.get(job.id).logs.join('\n'));
 const p=await createLabProject(lab,job.id),e=p.experiment.states[0].metrics.structure_evidence,l=e.native_burst_ledger;assert.equal(l.burst_seeds.length,legacy?2:3);assert.equal(e.burst_length,legacy?20:0);
 const restore=new WasmCatalog(manifest,fetcher,()=>{throw new Error('Open must not execute');});await restore.restoreProject(p);
 const mutations=[p=>p.experiment.states[0].metrics.structure_evidence.native_burst_ledger.burst_seeds[0]='0',p=>p.experiment.config.seed=0,p=>p.experiment.states[0].metrics.structure_evidence.native_burst_ledger.selected_burst_index=999];
 if(!legacy)mutations.push(p=>p.experiment.states[0].metrics.structure_evidence.native_burst_ledger.selected_burst_index=0);
 for(const mutate of mutations){const v=structuredClone(p);mutate(v);await assert.rejects(()=>restore.restoreProject(v));refusals++;}
 const historical=structuredClone(p);delete historical.experiment.states[0].metrics.structure_evidence.native_burst_ledger;if(legacy)historical.experiment.states[0].metrics.structure_evidence=null;await restore.restoreProject(historical);projects++;
}
await fs.writeFile('target/native-burst-ledger/project-evidence.json',JSON.stringify({legacy_and_zero_length_projects:projects,changed_seed_ledger_or_index_refusals:refusals,inert_open:true,historical_omission_accepted_as_claim:true,wasm_sha256:engine.artifactIdentity.sha256},null,2));console.log(`Burst ledger projects: ${projects} legacy/zero-length runs, ${refusals} seed/ledger/index refusals, inert Open and historical omission checks passed.`);
