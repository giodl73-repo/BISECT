import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync,spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
import {createLabProject,validateLabConfig} from '../../web/lab/laboratory-project.js';
import {verifyLaboratoryIlpReports} from '../../web/lab/laboratory-ilp-reports.js';
const root='target/laboratory-ilp-reports';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const graph=JSON.parse(await fs.readFile('target/ilp-balance-contract/branch-and-cut.json','utf8')).graph;
const fixture=JSON.parse(await fs.readFile('target/county-state-oracle/browser.bisect','utf8'));
const base=fixture.experiment.config;let cases=0,projects=0,refusals=0;
const normalize=packet=>{const p=structuredClone(packet);for(const n of p.nodes)if(n.report_json){const r=JSON.parse(n.report_json);r.result.solve_time_secs=0;n.report_json=JSON.stringify(r);}delete p.files;delete p.file_sha256;return p;};
for(const method of ['formulation-only','branch-and-cut','iterative-separation'])for(const max of [0,16])for(const k of [1,2,3,4])for(const build of [false,true]){
 const options={structure:'ilp',weights:'unweighted',search:'single',districts:k,seed:null,seeds:1,steps:1,percentile:0,alpha_county:0,balance_tolerance:50,area_swing:1.1,iterations:10,ilp_method:method,ilp_fallback:'metis',ilp_max_tracts:max,ilp_gap:0,capture_ilp_reports:true};
 const request={graph,options},operation=build?{operation:'run-engine-build',request}:request,actual=engine.execute(operation),file=root+'/request.json';await fs.writeFile(file,JSON.stringify(operation));
 const native=JSON.parse(execFileSync(build?'target/release/examples/execute_tool_request.exe':'target/release/examples/execute_request.exe',[file],{encoding:'utf8',maxBuffer:32*1024*1024}));
 assert.deepEqual(actual.assignments,native.assignments);assert.deepEqual(actual.metrics,native.metrics);assert.deepEqual(normalize(actual.ilp_report_capture),normalize(native.ilp_report_capture));
 const plain={graph,options:{...options}};delete plain.options.capture_ilp_reports;const original=engine.execute(build?{operation:'run-engine-build',request:plain}:plain);assert.deepEqual(actual.assignments,original.assignments);assert.deepEqual(actual.build_report,original.build_report);assert.equal(original.ilp_report_capture,undefined);
 const state={metrics:{...actual.metrics,recorded_options:actual.options,ilp_report_capture:actual.ilp_report_capture,...(build?{balance_build:actual.build_report}:{})}},config={structure:'ilp',capture_ilp_reports:true,native_build:build};
 await verifyLaboratoryIlpReports(graph,state,actual.assignments,config);assert.equal(actual.ilp_report_capture.accepted_attempt,build?0:null);assert.equal(JSON.parse(actual.ilp_report_capture.source_json).options.seed,actual.options.seed);
 if(method==='branch-and-cut'&&max===16&&k===4&&!build){
  for(const mutate of [p=>p.source_json+=' ',p=>p.source_sha256='0'.repeat(64),p=>p.returned_assignments_json+=' ',p=>p.accepted_attempt=1,p=>p.nodes.pop(),p=>p.nodes[1].unit_indices.reverse(),p=>p.nodes[0].lp_text+=' ',p=>p.file_sha256[Object.keys(p.files)[0]]='0'.repeat(64),p=>p.files['../node.lp']='x',p=>p.nodes[0].ilp_population_tolerance=1.25]){
   const forged=structuredClone(state);mutate(forged.metrics.ilp_report_capture);await assert.rejects(()=>verifyLaboratoryIlpReports(graph,forged,actual.assignments,config));refusals++;
  }
 }
 cases++;
}
const exhausted={graph,options:{structure:'ilp',weights:'unweighted',search:'single',districts:3,seed:null,seeds:1,steps:1,percentile:0,alpha_county:0,balance_tolerance:10,area_swing:1.1,iterations:10,ilp_method:'branch-and-cut',ilp_fallback:'error',ilp_max_tracts:16,capture_ilp_reports:true}};
assert.throws(()=>engine.execute({operation:'run-engine-build',request:exhausted}),/retry policy exhausted after 51 attempts/);
assert.throws(()=>engine.execute({...exhausted,options:{...exhausted.options,structure:'standard-bisect'}}),/requires native ILP structure/);refusals+=2;
// A dense synthetic graph stays within the request bound but exceeds the
// exported-model variable bound. Ordinary native fallback remains available;
// requesting capture must fail explicitly instead of returning partial files.
const dense={...graph,geoids:Array.from({length:300},(_,i)=>'01001'+String(i).padStart(6,'0')),adjacency:Array.from({length:300},(_,i)=>Array.from({length:300},(_,j)=>j).filter(j=>j!==i)),population:Array(300).fill(100),areas:Array(300).fill(1),exterior_perimeters:Array(300).fill(1),centroids:Array.from({length:300},()=>[0,0]),edges:[]};
for(let i=0;i<300;i++)for(let j=i+1;j<300;j++)dense.edges.push([i,j,1]);
const limited={graph:dense,options:{...exhausted.options,districts:2,ilp_fallback:'metis',ilp_max_tracts:300}};
assert.throws(()=>engine.execute(limited),/ILP observation model exceeds 200,000 variables/);refusals++;
const withoutCapture=structuredClone(limited);delete withoutCapture.options.capture_ilp_reports;assert.equal(Object.keys(engine.execute(withoutCapture).assignments).length,300);
await fs.writeFile(root+'/model-bound.json',JSON.stringify(limited));const refused=spawnSync('target/release/examples/execute_request.exe',[root+'/model-bound.json'],{encoding:'utf8'});assert.equal(refused.status,1);assert.match(refused.stderr,/ILP observation model exceeds 200,000 variables/);
const source='dist/wasm-lab-build',manifest=JSON.parse(await fs.readFile(source+'/catalog.json','utf8'));
class Worker{postMessage(m){queueMicrotask(()=>{try{this.onmessage({data:m.type==='initialize'?{type:'ready'}:{type:'result',id:m.id,result:engine.execute(m.request)}});}catch(e){this.onmessage({data:{type:'error',id:m.id,error:e.message}});}});}terminate(){}}
const fetcher=async ref=>{const b=await fs.readFile(source+'/'+ref);return{ok:true,arrayBuffer:async()=>b.buffer.slice(b.byteOffset,b.byteOffset+b.length)};};
for(const code of ['RI','IA'])for(const method of ['formulation-only','branch-and-cut','iterative-separation']){
 const config={...base,states:[code],structure:'ilp',weights:'geographic',alpha_county:0,search:'single',ilp_method:method,ilp_fallback:'metis',ilp_time_limit:300,ilp_gap:.01,ilp_max_tracts:0,balance_tolerance:50,capture_ilp_reports:true,native_build:true};
 const lab=new WasmCatalog(manifest,fetcher,()=>new Worker()),job=lab.submit(config);while(lab.active)await new Promise(r=>setTimeout(r,5));assert.equal(lab.jobs.get(job.id).status,'completed',lab.jobs.get(job.id).logs.join('\n'));
 const saved=await createLabProject(lab,job.id),restore=new WasmCatalog(manifest,fetcher,()=>{throw new Error('Open must not execute');}),reopened=await restore.restoreProject(saved);
 assert.deepEqual(reopened.states[0].metrics,saved.experiment.states[0].metrics);assert.equal(reopened.states[0].metrics.ilp_report_capture.accepted_attempt,0);
 const forged=structuredClone(saved);forged.experiment.states[0].metrics.ilp_report_capture.source_json+=' ';await assert.rejects(()=>restore.restoreProject(forged));refusals++;
 if(code==='RI'&&method==='formulation-only')await fs.writeFile(root+'/browser.bisect',JSON.stringify(saved));projects++;
}
for(const value of [null,1,'yes'])assert.throws(()=>validateLabConfig({...base,structure:'ilp',capture_ilp_reports:value}));refusals+=3;
await fs.writeFile(root+'/evidence.json',JSON.stringify({native_host_wasm_cases:cases,real_state_inert_projects:projects,refusals,accepted_attempt_and_final_assignments_bound:true,retry_exhaustion_returns_no_accepted_packet:true,wasm_sha256:engine.artifactIdentity.sha256},null,2));
console.log(`Laboratory ILP reports: ${cases} native/WASM cases, ${projects} RI/IA projects, ${refusals} invalid inputs/packets refused; same-dispatch assignments and accepted attempt bound.`);
