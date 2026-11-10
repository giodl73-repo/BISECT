import {multiscaleCharacterFixture} from './multiscale-character-fixture.mjs';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
import {createLabProject} from '../../web/lab/laboratory-project.js';
const source='dist/wasm-lab-build',manifest=JSON.parse(await fs.readFile(source+'/catalog.json','utf8')),engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const base=JSON.parse(await fs.readFile('target/lab-multiscale-character/browser.bisect','utf8')).experiment.config;
class Worker{postMessage(m){queueMicrotask(()=>{try{this.onmessage({data:m.type==='initialize'?{type:'ready'}:{type:'result',id:m.id,result:engine.execute(m.request)}});}catch(e){this.onmessage({data:{type:'error',id:m.id,error:e.message}});}});}terminate(){}}
const fetcher=async ref=>{const b=await fs.readFile(source+'/'+ref);return{ok:true,arrayBuffer:async()=>b.buffer.slice(b.byteOffset,b.byteOffset+b.length)};};
let projects=0;
for(const code of ['RI','IA'])for(const adaptive of [false,true])for(const kind of ['economic','housing']){
 const graph=JSON.parse(await fs.readFile(source+'/'+manifest.graphs[code+':2020'].graph_ref,'utf8')),partisan={schema_version:'bisect-partisan-shares-v1',state:code,year:'2020',source_label:'SYNTHETIC shares on actual prepared tract graph',dem_shares:Object.fromEntries(graph.geoids.map((id,i)=>[id,i%4<2?.8:.2]))};
 const c={...base,states:[code],weights:kind+'-character',character_alpha:.5,characters:{[code]:multiscaleCharacterFixture(engine,graph,kind)},search:adaptive?'multiscale-adaptive':'multiscale',districts:2,steps:2,balance_tolerance:25,multiscale:{fine_level:'tract',coarse_level:'county',total_steps:2,alpha:.3,percentile:0,adaptive:adaptive?{target_accept:.3,adapt_interval:1,gamma_0:.1,coarse_tol_factor:3}:null,inputs:{}}};
 delete c.partisans;delete c.dem_threshold;delete c.rep_threshold;const lab=new WasmCatalog(manifest,fetcher,()=>new Worker()),job=lab.submit(c);while(lab.active)await new Promise(r=>setTimeout(r,5));assert.equal(lab.jobs.get(job.id).status,'completed',lab.jobs.get(job.id).logs.join('\n'));const p=await createLabProject(lab,job.id),open=new WasmCatalog(manifest,fetcher,()=>{throw new Error('Open must not execute');});await open.restoreProject(p);assert.equal(p.experiment.states[0].metrics.weighted_boundary,p.experiment.states[0].metrics.graph_boundary_m);const exported=await lab.exportPractitionerProject(job.id,code);assert.deepEqual(exported.exported.document.provenance.producer.multiscale_input.request.character,c.characters[code]);projects++;
}
await fs.writeFile('target/lab-multiscale-character/actual-tract-projects.json',JSON.stringify({actual_prepared_tract_projects:projects,inert_open:true,character_sources_retained:true,fine_physical_cost_scope:true,wasm_sha256:engine.artifactIdentity.sha256},null,2));console.log(`Character multiscale actual tract laboratory: ${projects} RI/IA fixed/adaptive projects, exports and inert Open passed.`);
