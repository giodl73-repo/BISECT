// Explicit local Census fixture gate; does not download or fabricate source data.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
import {createLabProject} from '../../web/lab/laboratory-project.js';
import {validateMultiscaleBundle} from '../../web/lab/multiscale-input.js';
const [root='dist/wasm-multiscale-export',bundlePath='target/authentic-bg/RI/browser-input-v2.json']=process.argv.slice(2);
const manifest=JSON.parse(await fs.readFile(path.join(root,'catalog.json'),'utf8')),bundle=JSON.parse(await fs.readFile(bundlePath,'utf8'));
const code=bundle.graph.state,year=bundle.graph.year,entry=manifest.graphs[`${code}:${year}`];
const graph=JSON.parse(await fs.readFile(path.join(root,entry.graph_ref),'utf8'));validateMultiscaleBundle(bundle,graph);
const report=JSON.parse(await fs.readFile(bundlePath.replace(/\.json$/,'.sources.json'),'utf8'));
assert.equal(report.bundle_sha256,createHash('sha256').update(await fs.readFile(bundlePath)).digest('hex'));assert.equal(Object.keys(report.sources).length,7);
assert.equal(report.units,bundle.graph.geoids.length);assert.equal(report.state,code);assert.equal(report.year,Number(year));
for(const b of report.bridges){const i=bundle.graph.geoids.indexOf(b.from),j=bundle.graph.geoids.indexOf(b.to);assert.ok(i>=0&&j>=0);assert.equal(bundle.boundary_edges.find(([u,v])=>u===Math.min(i,j)&&v===Math.max(i,j))[2],b.cost);}
const engine=await instantiateEngine(await fs.readFile(path.join(root,'bisect_wasm.wasm')));
let nativeCases=0,projects=0,maps=0,audits=0,misses=0,lastRequest,lastResult,browserProject;
class Worker{postMessage(m){setImmediate(()=>{if(m.type==='initialize')this.onmessage({data:{type:'ready'}});else{try{lastRequest=m.request;lastResult=engine.execute(m.request);this.onmessage({data:{type:'result',id:m.id,result:lastResult}});}catch(e){this.onmessage({data:{type:'error',id:m.id,error:e.message}});}}});}terminate(){}}
const fetcher=async ref=>{const b=await fs.readFile(path.join(root,ref));return{ok:true,arrayBuffer:async()=>b.buffer.slice(b.byteOffset,b.byteOffset+b.length)};};
async function parity(request,result){await fs.writeFile('target/authentic-bg/native-request.json',JSON.stringify(request));const native=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',['target/authentic-bg/native-request.json'],{encoding:'utf8',maxBuffer:32*1024*1024}));assert.deepEqual(result,native);nativeCases++;}
const baseProfile=JSON.parse(await fs.readFile('crates/rplan-audit/fixtures/grid3x3-valid-certificate.json','utf8')).legal_profile;
for(const coarse of ['tract','county'])for(const adaptive of [false,true])for(const steps of [0,10])for(const seed of [42,'18446744073709551615']){
 const c={name:`Census ${code} ${year} block groups`,mode:'state',states:[code],year,chamber:'congressional',districts:2,structure:'standard-bisect',search:adaptive?'multiscale-adaptive':'multiscale',weights:'geographic',seed,seeds:1,steps,percentile:0,alpha_county:0,balance_tolerance:.5,area_swing:1.1,iterations:10,timeout_seconds:60,multiscale:{fine_level:'bg',coarse_level:coarse,total_steps:steps,alpha:.3,percentile:0,adaptive:adaptive?{target_accept:.3,adapt_interval:5,gamma_0:.1,coarse_tol_factor:3}:null,inputs:{[code]:bundle}}};
 const lab=new WasmCatalog(manifest,fetcher,()=>new Worker()),job=lab.submit(c);while(lab.active)await new Promise(r=>setTimeout(r,5));const completed=lab.jobs.get(job.id);assert.equal(completed.status,'completed',JSON.stringify(completed.states));await parity(lastRequest,lastResult);
 const project=await createLabProject(lab,job.id),open=new WasmCatalog(manifest,fetcher,()=>new Worker());await open.restoreProject(project);projects++;
 const map=await lab.api(`/api/runs/${job.id}/${code}/map`);assert.equal(map.features.length,bundle.graph.geoids.length);maps++;
 const {exported}=await lab.exportPractitionerProject(job.id,code);await parity(lastRequest,lastResult);assert.equal(exported.context.units.unit_kind,'block-group');
 const profile={...baseProfile,population_tolerance:{type:'exact-ppm',max_deviation_ppm:5000},contiguity_required:true,nesting_rule:{type:'not-evaluated'}};
 const req={operation:'audit-plan',plan:exported.document.plan,context:exported.context,profile,constraints:['plan-shape','population','contiguity'],generated_at_utc:'2026-10-04T00:00:00Z',lineage:null};
 const cert=engine.execute(req);await parity(req,cert);audits++;const balanced=completed.states[0].metrics.balance_passed;assert.equal(cert.result,balanced?'pass':'fail');if(!balanced)misses++;
 const verify={operation:'verify-certificate',certificate:cert,plan:exported.document.plan,context:exported.context};await parity(verify,engine.execute(verify));
 if(coarse==='tract'&&adaptive&&steps===10&&typeof seed==='string')browserProject=project;
 lab.destroy(new Error('Test done.'));open.destroy(new Error('Test done.'));
}
await fs.writeFile('target/authentic-bg/browser-project.bisect',JSON.stringify(browserProject));
await fs.writeFile('target/authentic-bg/verification.json',JSON.stringify({state:code,year,units:report.units,source_hashes:Object.keys(report.sources).length,derived_bridges:report.bridges.length,native_wasm_exact:nativeCases,projects,maps,audits,balance_misses:misses,scope:'Local Census geometry and PL block counts; explicit native median-cost bridge profile, not authenticated provenance'},null,2));
console.log(`Authentic ${code}: ${report.units} block groups, ${report.bridges.length} declared bridges; ${nativeCases} exact native/WASM operations, ${projects} projects, ${maps} maps, ${audits} audits; ${misses} balance misses.`);
