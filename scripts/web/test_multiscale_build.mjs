import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
import {createLabProject,parseLabProject} from '../../web/lab/laboratory-project.js';
import {validateMultiscaleBundle} from '../../web/lab/multiscale-input.js';
const root=process.argv[2]||'dist/wasm-lab-build',out='target/multiscale-build-wasm';
await fs.mkdir(out,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const base=JSON.parse(await fs.readFile('target/multiscale-fixtures/request-0-0.json','utf8')).input;
base.request.graph.population=base.request.graph.population.map((_,i)=>100+(i*37)%67);
base.request.options={...base.request.options,districts:4,balance_tolerance:.5};
const bgFor=g=>({schema_version:'bisect-block-group-graph-v1',state:g.state,year:g.year,geoids:g.geoids.flatMap(id=>[id+'1',id+'2']),population:g.population.flatMap(p=>[Math.floor(p/2),p-Math.floor(p/2)]),adjacency:Array.from({length:g.geoids.length*2},(_,i)=>[i^1,...g.adjacency[Math.floor(i/2)].map(j=>j*2+i%2)].sort((a,b)=>a-b))});
let exact=0,attemptOracles=0,generationErrors=0;
async function native(request){const file=out+'/request.json';await fs.writeFile(file,JSON.stringify(request));return JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[file],{encoding:'utf8',maxBuffer:32*1024*1024,stdio:['ignore','pipe','pipe']}));}
async function parity(request){const result=engine.execute(request);assert.deepEqual(result,await native(request));exact++;return result;}
const cases=[];
for(const [fine,coarse] of [['tract','county'],['bg','tract'],['bg','county']])for(const adaptive of [false,true])for(const seed of [null,'42','18446744073709551615'])for(const steps of [0,10]){
 const input=structuredClone(base);Object.assign(input,{fine_level:fine,coarse_level:coarse,total_steps:steps,alpha:.3,percentile:0,block_groups:fine==='bg'?bgFor(input.request.graph):null,adaptive:adaptive?{target_accept:.3,adapt_interval:5,gamma_0:.1,coarse_tol_factor:3}:null});input.request.options.seed=seed;
 const request={operation:'run-multiscale-build',input};let result;
 try{result=await parity(request);}catch(error){if(error instanceof assert.AssertionError)throw error;assert.throws(()=>engine.execute(request));await assert.rejects(native(request));generationErrors++;cases.push({fine,coarse,adaptive,seed,steps,error:error.message});continue;}
 const r=result.build_report;assert.equal(r.attempts.length,r.accepted_attempt+1);
 const fineGraph=input.block_groups||input.request.graph;
 for(const trial of r.attempts){
  const expected=trial.attempt===0?seed:((BigInt(seed??0)+BigInt(trial.attempt))%(1n<<64n)).toString();assert.equal(trial.seed,expected);
  const oracle=await native({operation:'run-multiscale',input:{...input,request:{...input.request,options:{...input.request.options,seed:expected}}}});attemptOracles++;
  const pop=Array(4).fill(0);fineGraph.geoids.forEach((id,i)=>pop[oracle.assignments[id]-1]+=fineGraph.population[i]);
  const ideal=fineGraph.population.reduce((a,b)=>a+b,0)/4,max=Math.max(...pop.map(p=>Math.abs((p-ideal)/ideal)*100));
  assert.equal(trial.max_deviation_percent,max);assert.equal(trial.balanced,max<=input.request.options.balance_tolerance);
  assert.equal(trial.balanced,trial.attempt===r.accepted_attempt);
  if(trial.balanced)assert.deepEqual(result.assignments,oracle.assignments);
 }
 cases.push({fine,coarse,adaptive,seed,steps,accepted_attempt:r.accepted_attempt,effective_seed:r.attempts.at(-1).seed});
}
await fs.writeFile(out+'/cases.json',JSON.stringify(cases,null,2));
assert.ok(cases.some(c=>c.accepted_attempt>0),'Exercise an actual retry');
assert.ok(cases.some(c=>c.seed==='18446744073709551615'&&c.accepted_attempt>0),'Exercise u64 wraparound');
const impossible=structuredClone(base);impossible.request.graph.population=impossible.request.graph.population.map((_,i)=>i===0?100000:1);impossible.request.options.districts=2;impossible.total_steps=0;impossible.block_groups=null;impossible.adaptive=null;
assert.throws(()=>engine.execute({operation:'run-multiscale-build',input:impossible}),/exhausted after 51 attempts/);
await assert.rejects(native({operation:'run-multiscale-build',input:impossible}));
const unsafe=structuredClone(base);unsafe.request.options.seed=9007199254740992;assert.throws(()=>engine.execute({operation:'run-multiscale-build',input:unsafe}),/exact unsigned/);

// The laboratory uses explicit synthetic maps; no fabricated Census input claim.
const manifest=JSON.parse(await fs.readFile(path.join(root,'catalog.json'),'utf8')),assets=new Map(),bundles={};
const asset=value=>{const bytes=Buffer.from(JSON.stringify(value)),hash=createHash('sha256').update(bytes).digest('hex'),ref=`assets/${hash}.json`;assets.set(ref,bytes);manifest.assets[ref]=hash;return ref;};
const polygon=i=>({type:'Polygon',coordinates:[[[-86+i/1000,32],[-86+i/1000+.0005,32],[-86+i/1000+.0005,32.0005],[-86+i/1000,32.0005],[-86+i/1000,32]]]});
for(const [code,fips] of [['AL','01'],['RI','44']]){
 const g=structuredClone(base.request.graph);g.state=code;g.geoids=g.geoids.map(id=>fips+id.slice(2));
 const geometry={type:'FeatureCollection',features:g.geoids.map((id,i)=>({type:'Feature',properties:{geoid:id},geometry:polygon(i)}))};
 manifest.graphs[code+':2020']={graph_ref:asset(g),native_graph_sha256:createHash('sha256').update(JSON.stringify(g)).digest('hex')};manifest.geometries[code+':2020']=asset(geometry);
 const bg=bgFor(g);bundles[code]=validateMultiscaleBundle({schema_version:'bisect-multiscale-input-v1',source_label:'SYNTHETIC retry regression',graph:bg,boundary_edges:bg.adjacency.flatMap((ns,i)=>ns.filter(j=>j>i).map(j=>[i,j,100])),geometry:{type:'FeatureCollection',features:bg.geoids.map((id,i)=>({type:'Feature',properties:{geoid:id},geometry:polygon(i)}))}},g);
}
manifest.catalog.search_compatibility['standard-bisect']= [...new Set([...manifest.catalog.search_compatibility['standard-bisect'],'multiscale','multiscale-adaptive'])];
const fetcher=async ref=>{const bytes=assets.get(ref)||await fs.readFile(path.join(root,ref));return {ok:true,arrayBuffer:async()=>bytes.buffer.slice(bytes.byteOffset,bytes.byteOffset+bytes.length)};};
let executions=0,projects=0;
class Worker{postMessage(m){setImmediate(()=>{if(this.terminated)return;try{if(m.type==='initialize')this.onmessage({data:{type:'ready'}});else{executions++;this.onmessage({data:{type:'result',id:m.id,result:engine.execute(m.request)}});}}catch(error){this.onmessage({data:{type:'error',id:m.id,error:error.message}});}});}terminate(){this.terminated=true;}}
for(const fine of ['tract','bg'])for(const adaptive of [false,true])for(const states of [['AL'],['AL','RI']]){
 const config={name:'SYNTHETIC multiscale retry regression',mode:states.length===1?'state':'national',states,year:'2020',chamber:'congressional',districts:states.length===1?4:null,structure:'standard-bisect',weights:'geographic',search:adaptive?'multiscale-adaptive':'multiscale',seed:'18446744073709551615',seeds:1,steps:0,percentile:0,alpha_county:0,balance_tolerance:2,area_swing:1.1,iterations:10,timeout_seconds:60,native_build:true,multiscale:{fine_level:fine,coarse_level:fine==='bg'?'tract':'county',total_steps:0,alpha:.3,percentile:0,adaptive:adaptive?{target_accept:.3,adapt_interval:5,gamma_0:.1,coarse_tol_factor:3}:null,inputs:fine==='bg'?Object.fromEntries(states.map(code=>[code,bundles[code]])):{}}};
 // Override only synthetic allocations for national fixtures.
 for(const code of states)manifest.catalog.states.find(s=>s.code===code).years.find(y=>y.year==='2020').districts.congressional=4;
 const lab=new WasmCatalog(manifest,fetcher,()=>new Worker()),job=lab.submit(config);while(lab.active)await new Promise(resolve=>setTimeout(resolve,5));
 assert.equal(lab.jobs.get(job.id).status,'completed',JSON.stringify(lab.jobs.get(job.id).states));
 const project=await createLabProject(lab,job.id);parseLabProject(JSON.stringify(project));const before=executions;
 await fs.writeFile(`${out}/${fine}-${adaptive?'adaptive':'fixed'}-${config.mode}.bisect`,JSON.stringify(project));
 const open=new WasmCatalog(manifest,fetcher,()=>{throw new Error('Opening must not generate');});const restored=await open.restoreProject(project);assert.equal(executions,before);projects++;
 for(const state of restored.states){assert.equal(String(state.metrics.recorded_options.seed),state.metrics.balance_build.attempts.at(-1).seed);const map=await open.api(`/api/runs/${restored.id}/${state.code}/map`);assert.equal(map.features.length,state.metrics.units);
  const portable=await lab.exportPractitionerProject(job.id,state.code);assert.equal(String(portable.exported.document.provenance.producer.multiscale_input.request.options.seed),state.metrics.balance_build.attempts.at(-1).seed);
 }
 for(const mutate of [p=>p.experiment.states[0].metrics.multiscale_evidence.seed='99',p=>p.experiment.states[0].metrics.balance_build.attempts.at(-1).seed='99',p=>p.experiment.states[0].metrics.balance_build.accepted_attempt++,p=>p.experiment.states[0].metrics.recorded_options.seed='99']){const bad=structuredClone(project);mutate(bad);await assert.rejects(open.restoreProject(bad));}
}
const realManifest=JSON.parse(await fs.readFile(path.join(root,'catalog.json'),'utf8'));
let realProjects=0;
if(realManifest.graphs['RI:2020'])for(const adaptive of [false,true]){
 realManifest.catalog.search_compatibility['standard-bisect']=[...new Set([...realManifest.catalog.search_compatibility['standard-bisect'],'multiscale','multiscale-adaptive'])];
 const config={name:'Real RI multiscale population build',mode:'state',states:['RI'],year:'2020',chamber:'congressional',districts:2,structure:'standard-bisect',weights:'geographic',search:adaptive?'multiscale-adaptive':'multiscale',seed:42,seeds:1,steps:10,percentile:0,alpha_county:0,balance_tolerance:.5,area_swing:1.1,iterations:100,timeout_seconds:60,native_build:true,multiscale:{fine_level:'tract',coarse_level:'county',total_steps:10,alpha:.3,percentile:0,adaptive:adaptive?{target_accept:.3,adapt_interval:5,gamma_0:.1,coarse_tol_factor:3}:null,inputs:{}}};
 const realFetcher=async ref=>{const bytes=await fs.readFile(path.join(root,ref));return {ok:true,arrayBuffer:async()=>bytes.buffer.slice(bytes.byteOffset,bytes.byteOffset+bytes.length)};};
 const lab=new WasmCatalog(realManifest,realFetcher,()=>new Worker()),job=lab.submit(config);while(lab.active)await new Promise(resolve=>setTimeout(resolve,5));
 assert.equal(lab.jobs.get(job.id).status,'completed');
 const project=await createLabProject(lab,job.id),before=executions;
 const open=new WasmCatalog(realManifest,realFetcher,()=>{throw new Error('Open must not generate');});await open.restoreProject(project);assert.equal(executions,before);realProjects++;
 await fs.writeFile(`${out}/real-RI-${adaptive?'adaptive':'fixed'}.bisect`,JSON.stringify(project));
}
let heldWorker;
const stalled=new WasmCatalog(manifest,fetcher,()=>{heldWorker=new Worker();const post=heldWorker.postMessage.bind(heldWorker);heldWorker.postMessage=m=>{if(m.type==='initialize')post(m);};return heldWorker;});
const cancelConfig=JSON.parse(await fs.readFile(out+'/tract-fixed-state.bisect','utf8')).experiment.config;
const cancelled=stalled.submit(cancelConfig);while(!stalled.pending)await new Promise(resolve=>setTimeout(resolve,1));await stalled.api(`/api/runs/${cancelled.id}/cancel`,{method:'POST'});while(stalled.active)await new Promise(resolve=>setTimeout(resolve,1));assert.equal(heldWorker.terminated,true);assert.equal(stalled.outputs.size,0);
await fs.writeFile(out+'/evidence.json',JSON.stringify({cases,exact_native_wasm:exact,native_attempt_oracles:attemptOracles,matching_errors:generationErrors,laboratory_projects:projects,real_RI_projects:realProjects,fine_population_assertion:true,inert_open:true,accepted_seed_export:true,cancellation_terminates_worker:true},null,2));
console.log(`Multiscale retries: ${exact} native/WASM builds, ${attemptOracles} native per-attempt oracles, ${projects} state/national projects, fine population checks and accepted-seed exports passed.`);
