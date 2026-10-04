import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {validatePartisanInput,partisanIdentity} from '../../web/lab/partisan-input.js';
import {validateLabConfig,verifyLabAssignments,verifyDemographicEvidence,createLabProject} from '../../web/lab/laboratory-project.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const n=144,w=12,adjacency=Array.from({length:n},(_,i)=>[i%w?i-1:-1,i%w<w-1?i+1:-1,i>=w?i-w:-1,i<n-w?i+w:-1].filter(v=>v>=0));
const graph={schema_version:1,state:'RI',year:'2020',geoids:Array.from({length:n},(_,i)=>`44001${String(i).padStart(6,'0')}`),adjacency,population:Array(n).fill(100),edges:adjacency.flatMap((ns,u)=>ns.filter(v=>v>u).map(v=>[u,v,100+(u%7)/10])),areas:Array(n).fill(100),exterior_perimeters:Array(n).fill(0),centroids:Array.from({length:n},(_,i)=>[-71+i%w*.001,41+Math.floor(i/w)*.001])};
const shares=(g,profile='stripe')=>({schema_version:'bisect-partisan-shares-v1',state:g.state,year:g.year,source_label:'SYNTHETIC partisan shares for regression; not election results',dem_shares:Object.fromEntries(g.geoids.map((id,i)=>[id,profile==='mixed'?(i%3===0?.8:.5):profile==='uniform'?.5:profile==='zero'?0:profile==='border'?(i%2?.55:.45):i<g.geoids.length/2?.8:.2]))});
const base={structure:'standard-bisect',weights:'partisan',search:'single',districts:4,seed:42,seeds:4,steps:20,percentile:.5,alpha_county:0,balance_tolerance:10,area_swing:1.1,iterations:10,dem_threshold:.55,rep_threshold:.45};
const configBase={name:'Partisan weighting',mode:'state',states:['RI'],year:'2020',chamber:'congressional',timeout_seconds:60};
let cases=0,tamper=0,failures=0,rejected=0,projects=0,misses=0,disconnected=0;
async function native(request){await fs.writeFile('target/partisan-request.json',JSON.stringify(request,(_key,value)=>Object.is(value,-0)?'__negative_zero__':value).replace(/"__negative_zero__"/g,'-0.0').replace(/"seed":"([0-9]+)"/,'"seed":$1'));return spawnSync('target/release/examples/execute_request.exe',['target/partisan-request.json'],{encoding:'utf8',maxBuffer:32*1024*1024});}
for(const profile of ['uniform','stripe','border','zero','mixed'])for(const [dem_threshold,rep_threshold]of [[.55,.45],[.5,.5],[1,0]])for(const districts of [2,3,4])for(const search of ['single','multi'])for(const [metis_objective,metis_trials]of [['cut',1],['volume',3]])for(const seed of [42,'18446744073709551615']){
 const partisan=shares(graph,profile),options={...base,dem_threshold,rep_threshold,districts,search,metis_objective,metis_trials,seed},request={graph,options,partisan},config={...configBase,...options,partisans:{RI:partisan}};
 validateLabConfig(config);const ref=await native(request);if(ref.status!==0){assert.throws(()=>engine.execute(request));failures++;continue;}
 const result=engine.execute(request);assert.deepEqual(result,JSON.parse(ref.stdout));assert.equal(result.metrics.weighting_evidence.shares_sha256,await partisanIdentity(partisan));
 const state={code:'RI',metrics:{...result.metrics,district_count:districts,districts:result.metrics.district_metrics,balance_passed:result.metrics.within_requested_tolerance}};verifyLabAssignments(graph,state,result.assignments,config);await verifyDemographicEvidence(graph,state,config);
 // Independent weighted-graph oracle: apply the documented formula and run the ordinary geographic path.
 const values=graph.geoids.map(id=>partisan.dem_shares[id]),strong=values.reduce((sum,v)=>sum+(v>=dem_threshold||v<=rep_threshold?1:0),0),alpha=Math.max(3,10*(1-.7*(strong/n)));
 const oracleGraph={...graph,edges:graph.edges.map(([u,v])=>[u,v,((values[u]>=dem_threshold&&values[v]>=dem_threshold)||(values[u]<=rep_threshold&&values[v]<=rep_threshold))?alpha:1])};
 const oracleOptions={...options,weights:'geographic'};delete oracleOptions.dem_threshold;delete oracleOptions.rep_threshold;
 const oracle=engine.execute({graph:oracleGraph,options:oracleOptions});assert.deepEqual(result.assignments,oracle.assignments);assert.equal(result.metrics.weighted_boundary,oracle.metrics.weighted_boundary);
 for(const key of ['method','dem_threshold','rep_threshold','strong_tracts','tracts','alpha','baseline','boost','scope']){const bad=structuredClone(state);bad.metrics.weighting_evidence[key]='changed';assert.throws(()=>verifyLabAssignments(graph,bad,result.assignments,config),/partisan weighting evidence/);tamper++;}
 const missing=structuredClone(state);delete missing.metrics.weighting_evidence;assert.throws(()=>verifyLabAssignments(graph,missing,result.assignments,config));tamper++;
 const badHash=structuredClone(state);badHash.metrics.weighting_evidence.shares_sha256='0'.repeat(64);await assert.rejects(verifyDemographicEvidence(graph,badHash,config),/partisan input identity/);tamper++;
 misses+=!result.metrics.within_requested_tolerance;disconnected+=!result.metrics.contiguous;cases++;
}
const good=shares(graph);for(const change of [{state:'IA'},{year:'2010'},{schema_version:'bad'},{source_label:''},{source_label:'x'.repeat(201)},{dem_shares:{}},{dem_shares:{...good.dem_shares,[graph.geoids[0]]:-0}},{dem_shares:{...good.dem_shares,[graph.geoids[0]]:1.1}}]){const partisan={...good,...change};assert.throws(()=>validatePartisanInput(partisan,graph));const request={graph,options:base,partisan};assert.throws(()=>engine.execute(request));assert.notEqual((await native(request)).status,0);rejected++;}
for(const change of [{dem_threshold:.4,rep_threshold:.6},{dem_threshold:-1},{rep_threshold:2},{structure:'ratio-optimal'},{search:'flip'},{alpha_county:2},{weights:'geographic'}]){const request={graph,options:{...base,...change},partisan:good};assert.throws(()=>engine.execute(request));assert.notEqual((await native(request)).status,0);rejected++;}
assert.throws(()=>engine.execute({graph,options:base}));
if(process.argv[2]){
 const root=process.argv[2],manifest=JSON.parse(await fs.readFile(path.join(root,'catalog.json'),'utf8'));manifest.catalog.weights=[...new Set([...manifest.catalog.weights,'partisan'])];
 const fetcher=async ref=>{const b=await fs.readFile(path.join(root,ref));return{ok:true,arrayBuffer:async()=>b.buffer.slice(b.byteOffset,b.byteOffset+b.length)};};
 class Worker{postMessage(m){setImmediate(()=>{if(this.terminated)return;try{this.onmessage({data:m.type==='initialize'?{type:'ready'}:{type:'result',id:m.id,result:engine.execute(m.request)}});}catch(e){this.onmessage({data:{type:'error',id:m.id,error:e.message}});}});}terminate(){this.terminated=true;}}
 const lab=new WasmCatalog(manifest,fetcher,()=>new Worker());
 for(const [key,entry]of Object.entries(manifest.graphs))for(const search of ['single','multi']){
  const [code,year]=key.split(':'),g=JSON.parse(await fs.readFile(path.join(root,entry.graph_ref),'utf8')),partisan=shares(g),config={...configBase,...base,states:[code],year,search,districts:null,partisans:{[code]:partisan},metis_objective:'volume',metis_trials:3};
  const job=lab.submit(config);while(lab.active)await new Promise(r=>setTimeout(r,5));assert.equal(lab.jobs.get(job.id).status,'completed');
  const saved=await createLabProject(lab,job.id),restore=new WasmCatalog(manifest,fetcher,()=>{throw new Error('Open must not generate.');});const opened=await restore.restoreProject(structuredClone(saved));assert.deepEqual(opened.config.partisans[code],partisan);assert.deepEqual(await restore.api(`/api/runs/${opened.id}/${code}/assignments`),saved.assignments[code]);
  const altered=structuredClone(saved);altered.experiment.config.partisans[code].dem_shares[g.geoids[0]]=.7;await assert.rejects(restore.restoreProject(altered));
  const exported=await lab.exportPractitionerProject(job.id,code);assert.deepEqual(exported.project.files.plan.provenance.producer.partisan_input,partisan);assert.equal(exported.project.files.context.source_hashes['bisect.partisan-shares'],await partisanIdentity(partisan));
  await fs.writeFile(`target/partisan-${code}-${search}.bisect`,JSON.stringify(saved));await fs.writeFile(`target/partisan-${code}-shares.json`,JSON.stringify(partisan));projects++;
 }
}
assert.ok(cases>0);await fs.writeFile('target/partisan-wasm-verification.json',JSON.stringify({exact_cases:cases,independent_weighted_graph_replays:cases,evidence_tamper_rejections:tamper,matching_generation_failures:failures,invalid_input_option_rejections:rejected,real_state_project_export_roundtrips:projects,balance_misses:misses,disconnected},null,2));
console.log(`Partisan weights: ${cases} exact native/WASM cases and weighted-graph replays; ${tamper} evidence rejections, ${rejected} invalid inputs/options, ${projects} project/export roundtrips, ${failures} matching failures; ${misses} balance misses, ${disconnected} disconnected plans.`);
