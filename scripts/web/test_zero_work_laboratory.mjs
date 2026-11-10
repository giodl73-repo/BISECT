import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
import {createLabProject} from '../../web/lab/laboratory-project.js';
const root='target/zero-work-laboratory';await fs.mkdir(root,{recursive:true});
const source='dist/wasm-lab-build',manifest=JSON.parse(await fs.readFile(source+'/catalog.json','utf8')),engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const base=JSON.parse(await fs.readFile('target/county-state-oracle/browser.bisect','utf8')).experiment.config,records=[];
for(const code of ['RI','IA'])for(const search of ['forest-recom','merge-split','flip','bisection-ensemble','vra-recom','parallel-tempering','short-burst','multiscale']){
 const graph=JSON.parse(await fs.readFile(source+'/'+manifest.graphs[code+':2020'].graph_ref,'utf8'));let request,raw;
 class Worker{postMessage(m){queueMicrotask(()=>{try{if(m.type==='initialize'){this.onmessage({data:{type:'ready'}});return;}request=m.request;raw=engine.execute(request);this.onmessage({data:{type:'result',id:m.id,result:raw}});}catch(e){this.onmessage({data:{type:'error',id:m.id,error:e.message}});}});}terminate(){}}
 const fetcher=async ref=>{const b=await fs.readFile(source+'/'+ref);return{ok:true,arrayBuffer:async()=>b.buffer.slice(b.byteOffset,b.byteOffset+b.length)};};
 const config={...base,states:[code],structure:'standard-bisect',search,weights:'geographic',alpha_county:0,seed:42,seeds:3,steps:0,percentile:0,balance_tolerance:50};
 if(search==='vra-recom'){config.vra_threshold=.5;config.demographics={[code]:{schema_version:'bisect-demographic-fractions-v1',state:code,year:'2020',basis:'total-population',source_label:'Synthetic uniform zero-work fixture',minority_fractions:Object.fromEntries(graph.geoids.map(id=>[id,.5]))}};}
 if(search==='parallel-tempering'){config.steps=1;config.seeds=0;config.pt_replicas=2;config.pt_swap_interval=2;config.pt_cold_tol=.5;config.pt_hot_tol=1;}
 if(search==='short-burst'){config.steps=1;config.burst_length=0;config.n_bursts=0;}
 if(search==='bisection-ensemble'){config.metis_objective='cut';config.metis_trials=1;}
 if(search==='multiscale'){config.seeds=1;config.multiscale={fine_level:'tract',coarse_level:'county',total_steps:0,alpha:.3,percentile:0,adaptive:null,inputs:{}};}
 const lab=new WasmCatalog(manifest,fetcher,()=>new Worker()),job=lab.submit(config);while(lab.active)await new Promise(r=>setTimeout(r,5));const row=lab.jobs.get(job.id).states[0];assert.equal(row.status,'completed',row.error);const file=root+'/'+records.length+'.json';await fs.writeFile(file,JSON.stringify(request));const native=spawnSync('target/release/examples/'+(request.operation?'execute_tool_request.exe':'execute_request.exe'),[file],{encoding:'utf8',maxBuffer:64*1024*1024});assert.equal(native.status,0,native.stderr);assert.deepEqual(JSON.parse(native.stdout),raw);
 const saved=await createLabProject(lab,job.id),restored=new WasmCatalog(manifest,fetcher,()=>{throw new Error('Open must not run');}),reopened=await restored.restoreProject(saved);assert.deepEqual(reopened.config,config);assert.deepEqual(reopened.states[0].metrics,row.metrics);if(code==='RI')await fs.writeFile(root+'/'+search+'.bisect',JSON.stringify(saved));records.push({code,search,exact_native_result:true,inert_project:true,balance_passed:row.metrics.balance_passed});
}
await fs.writeFile(root+'/evidence.json',JSON.stringify({cases:records.length,records,wasm_sha256:engine.artifactIdentity.sha256},null,2));console.log(`Zero-work laboratory: ${records.length} exact native results and inert project roundtrips across chains, ensemble, VRA, tempering, burst and multiscale.`);
