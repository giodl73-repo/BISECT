import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
import {createLabProject,parseLabProject} from '../../web/lab/laboratory-project.js';
import {parseProject,MAX_PROJECT_BYTES} from '../../web/lab/project.js';
import {parseWorkbenchArchiveBytes} from '../../web/lab/workbench-archive.js';
import {exactContextText,exactSourceText,exactPlanText} from '../../web/lab/package-files.js';
const root=process.argv[2]||'dist/wasm-lab-build';
const states=(process.argv[3]||'CA,TX').split(','),year=process.argv[4]||'2020';
const evidenceRoot=process.env.BISECT_LAB_BUILD_EVIDENCE_ROOT||`target/lab-build-wasm/${states.join('-')}-${year}`;
const manifest=JSON.parse(await fs.readFile(path.join(root,'catalog.json'),'utf8'));
const engine=await instantiateEngine(await fs.readFile(path.join(root,'bisect_wasm.wasm')));
const fetcher=async name=>{const bytes=await fs.readFile(path.join(root,name));return {ok:true,arrayBuffer:async()=>bytes.buffer.slice(bytes.byteOffset,bytes.byteOffset+bytes.length)};};
let executions=0;
class Worker {
 postMessage(message){setImmediate(()=>{if(this.terminated)return;try{if(message.type==='initialize')this.onmessage({data:{type:'ready'}});else{executions++;this.onmessage({data:{type:'result',id:message.id,result:engine.execute(message.request),memoryBytes:engine.memoryBytes()}});}}catch(error){this.onmessage({data:{type:'error',id:message.id,error:error.message}});}});}
 terminate(){this.terminated=true;}
}
const config={name:'Native population build',mode:states.length===1?'state':'national',states,year,chamber:'congressional',districts:null,structure:'standard-bisect',weights:'geographic',search:'single',seed:42,seeds:1,steps:200,percentile:0,alpha_county:0,balance_tolerance:.5,area_swing:1.1,iterations:100,timeout_seconds:300,native_build:true};
const lab=new WasmCatalog(manifest,fetcher,()=>new Worker());
const job=process.argv.includes('--saved')?await lab.restoreProject(parseLabProject(await fs.readFile(evidenceRoot+'/project.bisect','utf8'))):lab.submit(config);
while(lab.active)await new Promise(resolve=>setTimeout(resolve,10));
const result=await lab.api('/api/runs/'+job.id);
assert.equal(result.status,'completed',JSON.stringify(result.states.map(s=>s.error)));
for(const state of result.states){
 const m=state.metrics,r=m.balance_build;
 assert.equal(m.balance_passed,true);assert.equal(m.contiguous,true);
 assert.equal(m.requested_options.seed,42);assert.equal(String(m.recorded_options.seed),r.attempts.at(-1).seed);
 assert.equal(String(m.effective_config.seed),r.attempts.at(-1).seed);
 assert.equal(r.attempts.length,r.accepted_attempt+1);
 const map=await lab.api(`/api/runs/${job.id}/${state.code}/map`);
 assert.equal(map.features.length,m.units);
 console.log(JSON.stringify({state:state.code,accepted_attempt:r.accepted_attempt,seed:r.attempts.at(-1).seed,deviation:m.max_deviation_percent}));
}
if(states.includes('CA')&&states.includes('TX')&&year==='2020')assert.ok(result.states.filter(s=>s.code==='CA'||s.code==='TX').every(s=>s.metrics.balance_build.accepted_attempt>0),'Large-state fixtures must exercise actual retries.');
const project=await createLabProject(lab,job.id);parseLabProject(JSON.stringify(project));
await fs.mkdir(evidenceRoot,{recursive:true});
await fs.writeFile(evidenceRoot+'/project.bisect',JSON.stringify(project));
const before=executions;
const restoredLab=new WasmCatalog(manifest,fetcher,()=>{throw new Error('Opening must not generate');});
const restored=await restoredLab.restoreProject(project);assert.equal(executions,before);
assert.equal(restored.config.seed,42);assert.equal(restoredLab.worker,null);
const count=restoredLab.jobs.size;
for(const tamper of [
 p=>p.experiment.states[0].metrics.balance_build.attempts.at(-1).seed='99',
 p=>p.experiment.states[0].metrics.balance_build.accepted_attempt++,
 p=>p.experiment.states[0].metrics.balance_build.attempts.at(-1).balanced=false,
 p=>p.experiment.states[0].metrics.balance_build.attempts.at(-1).max_deviation_percent++,
 p=>delete p.experiment.states[0].metrics.balance_build,
 p=>p.experiment.config.native_build=false,
 p=>p.experiment.states[0].metrics.recorded_options.seed=99,
]){const bad=structuredClone(project);tamper(bad);await assert.rejects(restoredLab.restoreProject(bad));assert.equal(restoredLab.jobs.size,count);}
let heldWorker;
const stalled=new WasmCatalog(manifest,fetcher,()=>{
 heldWorker=new Worker();const post=heldWorker.postMessage.bind(heldWorker);
 heldWorker.postMessage=message=>{if(message.type==='initialize')post(message);};return heldWorker;
});
const cancelled=stalled.submit(config);
while(!stalled.pending)await new Promise(resolve=>setTimeout(resolve,1));
await stalled.api(`/api/runs/${cancelled.id}/cancel`,{method:'POST'});
while(stalled.active)await new Promise(resolve=>setTimeout(resolve,1));
assert.equal(heldWorker.terminated,true);assert.equal(stalled.outputs.size,0);
assert.equal((await stalled.api('/api/runs/'+cancelled.id)).status,'cancelled');
for(const state of result.states){
 const portable=await lab.exportPractitionerProject(job.id,state.code);
 assert.equal(String(portable.exported.document.provenance.producer.engine_options.seed),state.metrics.balance_build.attempts.at(-1).seed);
 if(portable.archive===undefined){
  assert.ok(Buffer.byteLength(JSON.stringify(portable.project))<=MAX_PROJECT_BYTES);parseProject(JSON.stringify(portable.project));
  await fs.writeFile(`${evidenceRoot}/${state.code}-practitioner.bisect`,JSON.stringify(portable.project));
 }else{
  const bytes=typeof portable.archive==='string'?new TextEncoder().encode(portable.archive):portable.archive;
  assert.deepEqual((await parseWorkbenchArchiveBytes(bytes)).files,portable.project.files);
  assert.throws(()=>parseProject(JSON.stringify(portable.project)),/25 MB|too complex|too many values/);
  await fs.writeFile(`${evidenceRoot}/${state.code}-practitioner.bisect-workbench`,bytes);
 }
 assert.equal(exactContextText(portable.project.files.auditContextText),portable.exported.context_json);
 if(Buffer.byteLength(portable.exported.context_json)>3*1024*1024){assert.equal(portable.project.files.context,undefined);assert.throws(()=>exactSourceText(portable.project.files.auditContextText),/3 MiB/);}
 engine.execute({operation:'validate-rplan-text',document_json:exactPlanText(portable.project.files.auditPlanText)});
 const fixture=JSON.parse(await fs.readFile('crates/rplan-audit/fixtures/grid3x3-valid-certificate.json','utf8'));
 const profile={...fixture.legal_profile,contiguity_required:true,nesting_rule:{type:'not-evaluated'}};
 const audit=engine.execute({operation:'audit-rplan-text',document_json:portable.exported.document_json,context_json:exactContextText(portable.project.files.auditContextText),profile_json:JSON.stringify(profile),constraints:['plan-shape','population','contiguity'],generated_at_utc:'2026-10-07T00:00:00Z',lineage_json:null});
 assert.equal(audit.result,'pass');
 assert.equal(engine.execute({operation:'verify-audit-text',certificate_json:audit.certificate_json,document_json:portable.exported.document_json,context_json:portable.exported.context_json}).verified,true);
}
const oversized={name:'large.rctx',base64:Buffer.from(' '.repeat(8*1024*1024+1)).toString('base64')};
assert.throws(()=>exactContextText(oversized),/limit|encoding/);
const cert=await fs.readFile('crates/rplan-audit/fixtures/grid3x3-valid-certificate.json','utf8');
assert.throws(()=>engine.execute({operation:'verify-audit-text',certificate_json:cert,document_json:null,context_json:' '.repeat(8*1024*1024+1)}),/8 MiB/);
await fs.writeFile(evidenceRoot+'/evidence.json',JSON.stringify({states:result.states.map(s=>({state:s.code,report:s.metrics.balance_build,deviation:s.metrics.max_deviation_percent})),import_without_generation:true,tamper_rejected:true,accepted_seed_export:true,cancellation_terminates_worker:true,native_context_audit_and_certificate_verification:true},null,2));
console.log('Laboratory native retries, national maps, seed evidence, inert project reopening and tamper rejection passed.');
