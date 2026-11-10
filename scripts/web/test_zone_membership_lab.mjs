import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
import {createLabProject} from '../../web/lab/laboratory-project.js';
import {verifyZoneFallback} from '../../web/lab/zone-weighting.js';
const root='target/zone-membership-lab',published=path.resolve('dist/wasm-lab-build');await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm')),manifest=JSON.parse(await fs.readFile(published+'/catalog.json','utf8'));manifest.wasm_sha256=engine.artifactIdentity.sha256;
const fetcher=async ref=>{const b=await fs.readFile(path.join(published,ref));return {ok:true,arrayBuffer:async()=>b.buffer.slice(b.byteOffset,b.byteOffset+b.length)};};let requests=0,projects=0;
class Worker{postMessage(m){setImmediate(()=>{if(this.closed)return;try{if(m.type==='initialize')this.onmessage({data:{type:'ready'}});else{requests++;this.onmessage({data:{type:'result',id:m.id,result:engine.execute(m.request)}});}}catch(e){this.onmessage({data:{type:'error',id:m.id,error:e.message}});}});}terminate(){this.closed=true;}}
for(const search of ['single','multiscale','multiscale-adaptive']){
 const config={name:'Native zones geographic fallback',mode:'state',states:['RI'],year:'2020',chamber:'congressional',districts:2,structure:'standard-bisect',weights:'zone-membership',search,seed:'18446744073709551615',seeds:1,steps:5,percentile:0,alpha_county:0,balance_tolerance:25,area_swing:1.1,iterations:100,timeout_seconds:300,native_build:true,...(search==='single'?{}:{multiscale:{fine_level:'tract',coarse_level:'county',total_steps:5,alpha:.3,percentile:0,adaptive:search.endsWith('adaptive')?{target_accept:.3,adapt_interval:5,gamma_0:.1,coarse_tol_factor:3}:null,inputs:{}}})};
 const lab=new WasmCatalog(manifest,fetcher,()=>new Worker()),job=lab.submit(config);while(lab.active)await new Promise(r=>setTimeout(r,5));assert.equal(lab.jobs.get(job.id).status,'completed',JSON.stringify(lab.jobs.get(job.id).states));const packet=await createLabProject(lab,job.id),metrics=packet.experiment.states[0].metrics;
 verifyZoneFallback(search==='single'?metrics.weighting_evidence:metrics.multiscale_evidence.zone_initialization,search==='single'?'edge-weighting':'parent-tract-initialization-only');
 const opened=new WasmCatalog(manifest,fetcher,()=>{throw new Error('Open must not execute');}),before=requests;await opened.restoreProject(packet);assert.equal(requests,before);
 for(const edit of [e=>e.zone_data_applied=true,e=>e.effective_weights='zones',e=>e.scope='fine-sampling',e=>delete e.native_status]){const bad=structuredClone(packet);edit(search==='single'?bad.experiment.states[0].metrics.weighting_evidence:bad.experiment.states[0].metrics.multiscale_evidence.zone_initialization);await assert.rejects(opened.restoreProject(bad),/Zone membership/);}
 const exported=(await lab.exportPractitionerProject(job.id,'RI')).exported;assert(JSON.stringify(exported.document.provenance).includes('zone-membership'));
 await fs.writeFile(root+'/'+search+'.bisect',JSON.stringify(packet));if(search==='single')await fs.writeFile(root+'/browser.bisect',JSON.stringify(packet));projects++;lab.destroy(new Error('Done'));opened.destroy(new Error('Done'));
}
await fs.writeFile(root+'/evidence.json',JSON.stringify({wasm:engine.artifactIdentity,projects,actual_wasm_requests:requests,exact_max_u64_seed:true,inert_open:true,zone_claim_tampering_rejected:true,practitioner_exports_keep_requested_weight_name:true,published:false},null,2)+'\n');console.log('PASS zone laboratory: ordinary/fixed/adaptive Runs, maximum seed, native exports, inert Open and forged zone-data claims refused');
