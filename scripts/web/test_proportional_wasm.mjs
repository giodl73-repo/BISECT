import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
import {createLabProject} from '../../web/lab/laboratory-project.js';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {validateElectionInput,electionIdentity,verifyElectionResult} from '../../web/lab/election-input.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const n=256,w=16,adjacency=Array.from({length:n},(_,i)=>[i%w?i-1:-1,i%w<w-1?i+1:-1,i>=w?i-w:-1,i<n-w?i+w:-1].filter(v=>v>=0));
const graph={schema_version:1,state:'RI',year:'2020',geoids:Array.from({length:n},(_,i)=>`4400${i%2+1}${String(i).padStart(6,'0')}`),adjacency,population:Array(n).fill(1000),edges:adjacency.flatMap((ns,u)=>ns.filter(v=>v>u).map(v=>[u,v,100+(u%7)/10])),areas:Array(n).fill(100),exterior_perimeters:Array(n).fill(0),centroids:Array.from({length:n},(_,i)=>[-71+i%w*.001,41+Math.floor(i/w)*.001])};
const input=(profile='uniform')=>({schema_version:'bisect-election-counts-v1',state:'RI',year:'2020',election_year:'2016',source_label:'SYNTHETIC election counts for regression; not election results',counts:Object.fromEntries(graph.geoids.map((id,i)=>[id,{democratic:profile==='uniform'?500:profile==='asymmetric'?750:profile==='stripe'?(i<n/2?750:250):profile==='fractional'?500.125:0,two_party:1000}]))});
const base={structure:'proportional-bisect',weights:'geographic',search:'single',districts:4,seed:42,seeds:3,steps:20,percentile:.5,alpha_county:0,balance_tolerance:10,area_swing:1.1,iterations:30};
const raw=value=>JSON.stringify(value,(_key,v)=>Object.is(v,-0)?'__negative_zero__':v).replace(/"__negative_zero__"/g,'-0.0').replace(/"seed":"([0-9]+)"/,'"seed":$1');
async function native(request,tool=false){const path='target/proportional-request.json';await fs.writeFile(path,raw(request));return spawnSync(`target/release/examples/${tool?'execute_tool_request':'execute_request'}.exe`,[path],{encoding:'utf8',maxBuffer:32*1024*1024});}
let cases=0,failures=0,tamper=0,rejected=0,exports=0,misses=0,disconnected=0;
for(const profile of ['uniform','asymmetric','stripe','fractional','zero'])for(const districts of [1,2,3,4,6])for(const [structure,search,eta]of [['proportional-bisect','single',null],['proportional-section','single',1.1],['proportional-section','multi',1.5]])for(const weights of ['geographic','unweighted','county'])for(const seed of [42,'18446744073709551615']){
 const elections=input(profile),options={...base,structure,search,weights,districts,seed,alpha_county:weights==='county'?5:0,...(eta?{proportional_eta:eta}:{})},request={graph,options,elections};
 const ref=await native(request);
 if(ref.status!==0){const match=ref.stderr.match(/Error: (.+)\s*$/);assert.ok(match,ref.stderr);const message=JSON.parse(match[1]);assert.throws(()=>engine.execute(request),e=>e.message===message);failures++;continue;}
 const result=engine.execute(request);assert.deepEqual(result,JSON.parse(ref.stdout));await verifyElectionResult(graph,options,elections,result);cases++;
 misses+=!result.metrics.within_requested_tolerance;disconnected+=!result.metrics.contiguous;
 const evidence=result.metrics.election_input_evidence;
 for(const key of Object.keys(evidence)){const bad=structuredClone(result);bad.metrics.election_input_evidence[key]='altered';await assert.rejects(verifyElectionResult(graph,options,elections,bad));tamper++;}
 if(districts>1){
  for(const key of Object.keys(result.metrics.structure_evidence)){const bad=structuredClone(result);bad.metrics.structure_evidence[key]='altered';await assert.rejects(verifyElectionResult(graph,options,elections,bad));tamper++;}
  if(structure==='proportional-bisect')for(const key of Object.keys(result.metrics.structure_evidence.splits[0])){const bad=structuredClone(result);bad.metrics.structure_evidence.splits[0][key]='altered';await assert.rejects(verifyElectionResult(graph,options,elections,bad));tamper++;}
 }
 if(profile==='uniform'&&districts===4&&weights==='geographic'&&seed===42){
  const tool={operation:'export-engine-plan',request,assignments:result.assignments,label:'Synthetic proportional fixture',chamber:'test',created_at:'2026-10-04T00:00:00Z'},nativeExport=await native(tool,true);assert.equal(nativeExport.status,0,nativeExport.stderr);
  const exported=engine.execute(tool);assert.deepEqual(exported,JSON.parse(nativeExport.stdout));assert.deepEqual(exported.document.provenance.producer.election_input,elections);assert.equal(exported.context.source_hashes['bisect.election-counts'],await electionIdentity(elections));exports++;
 }
}
const good=input();
for(const change of [{schema_version:'bad'},{state:'IA'},{year:'2010'},{election_year:2016},{source_label:''},{counts:{}},{counts:{...good.counts,[graph.geoids[0]]:{democratic:-0,two_party:1000}}},{counts:{...good.counts,[graph.geoids[0]]:{democratic:1001,two_party:1000}}}]){
 const elections={...good,...change},request={graph,options:base,elections};assert.throws(()=>validateElectionInput(elections,graph));assert.throws(()=>engine.execute(request));assert.notEqual((await native(request)).status,0);rejected++;
}
for(const change of [{structure:'standard-bisect'},{search:'multi'},{proportional_eta:1.1},{weights:'partisan'},{metis_objective:'volume',metis_trials:3},{structure:'proportional-section',proportional_eta:.9},{structure:'proportional-section',proportional_eta:2.1}]){
 const request={graph,options:{...base,...change},elections:good};assert.throws(()=>engine.execute(request));assert.notEqual((await native(request)).status,0);rejected++;
}
assert.throws(()=>engine.execute({graph,options:base}));
assert.ok(cases>0);assert.equal(exports,3);
let projects=0,realFailures=0;const realDiagnostics=[];
if(process.argv[2]){
 const root=process.argv[2],manifest=JSON.parse(await fs.readFile(path.join(root,'catalog.json'),'utf8'));manifest.catalog.search_compatibility['proportional-bisect']=['single'];manifest.catalog.search_compatibility['proportional-section']=['single','multi'];
 const fetcher=async ref=>{const bytes=await fs.readFile(path.join(root,ref));return{ok:true,arrayBuffer:async()=>bytes.buffer.slice(bytes.byteOffset,bytes.byteOffset+bytes.length)};};
 class Worker{postMessage(m){setImmediate(()=>{if(this.terminated)return;try{this.onmessage({data:m.type==='initialize'?{type:'ready'}:{type:'result',id:m.id,result:engine.execute(m.request)}});}catch(error){this.onmessage({data:{type:'error',id:m.id,error:error.message}});}});}terminate(){this.terminated=true;}}
 const lab=new WasmCatalog(manifest,fetcher,()=>new Worker());
 for(const[key,entry]of Object.entries(manifest.graphs))for(const[structure,search]of [['proportional-bisect','single'],['proportional-section','single'],['proportional-section','multi']]){
  const [code,year]=key.split(':'),g=JSON.parse(await fs.readFile(path.join(root,entry.graph_ref),'utf8'));
  const elections={schema_version:'bisect-election-counts-v1',state:code,year,election_year:'2016',source_label:'SYNTHETIC census-scaled counts for portability; not election results',counts:Object.fromEntries(g.geoids.map((id,i)=>[id,{democratic:g.population[i]/2,two_party:g.population[i]}]))};
  const config={...base,name:'Proportional portability',mode:'state',states:[code],year,chamber:'congressional',districts:null,timeout_seconds:60,structure,search,elections:{[code]:elections},...(structure==='proportional-section'?{proportional_eta:1.1}:{})};
  // Project checks below exercise the catalog's own chamber allocation.
  const nativeOptions={...base,structure,search,districts:lab.districtCount(code,config),...(structure==='proportional-section'?{proportional_eta:1.1}:{})};
  const nativeState=await native({graph:g,options:nativeOptions,elections});
  const job=lab.submit(config);while(lab.active)await new Promise(r=>setTimeout(r,5));
  if(lab.jobs.get(job.id).status!=='completed'){assert.notEqual(nativeState.status,0);realFailures++;continue;}
  assert.equal(nativeState.status,0,nativeState.stderr);const nativeResult=JSON.parse(nativeState.stdout),actualState=lab.jobs.get(job.id).states[0];assert.deepEqual(await lab.api(`/api/runs/${job.id}/${code}/assignments`),nativeResult.assignments);for(const[key,value]of Object.entries(nativeResult.metrics))assert.deepEqual(actualState.metrics[key==='districts'?'district_count':key],value);
  realDiagnostics.push({state:code,structure,search,deviation:actualState.metrics.max_deviation_percent,contiguous:actualState.metrics.contiguous,root_population_passed:actualState.metrics.structure_evidence?.root_population_within_requested_multiplier??null,root_votes_passed:actualState.metrics.structure_evidence?.root_democratic_within_requested_multiplier??null});
  const saved=await createLabProject(lab,job.id),restore=new WasmCatalog(manifest,fetcher,()=>{throw new Error('Opening must not generate.');}),opened=await restore.restoreProject(structuredClone(saved));assert.deepEqual(opened.config.elections[code],elections);assert.deepEqual(await restore.api(`/api/runs/${opened.id}/${code}/assignments`),saved.assignments[code]);
  const bad=structuredClone(saved),id=g.geoids.find((id,i)=>g.population[i]>0);bad.experiment.config.elections[code].counts[id].democratic+=.1;await assert.rejects(restore.restoreProject(bad));
  const exported=await lab.exportPractitionerProject(job.id,code);assert.deepEqual(exported.project.files.plan.provenance.producer.election_input,elections);assert.equal(exported.project.files.context.source_hashes['bisect.election-counts'],await electionIdentity(elections));
  await fs.writeFile(`target/proportional-${code}-${structure}-${search}.bisect`,JSON.stringify(saved));await fs.writeFile(`target/proportional-${code}-counts.json`,JSON.stringify(elections));projects++;
 }
}
await fs.writeFile('target/proportional-wasm-verification.json',JSON.stringify({exact_native_wasm_results:cases,matching_failures:failures,independent_election_evidence_checks:cases,tamper_rejections:tamper,invalid_inputs_options:rejected,practitioner_exports:exports,real_state_projects:projects,real_state_failures:realFailures,real_state_diagnostics:realDiagnostics,balance_misses:misses,disconnected},null,2));
console.log(`Proportional API: ${cases} exact native/WASM results, ${failures} matching failures, ${tamper} tamper rejections, ${rejected} invalid inputs/options, ${exports} exact practitioner exports, ${projects} real-state projects, ${realFailures} real-state failures; ${misses} balance misses, ${disconnected} disconnected.`);
