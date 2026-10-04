import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {validateCharacterInput,characterWeights,characterIdentity} from '../../web/lab/character-input.js';
import {validateLabConfig,verifyLabAssignments,verifyDemographicEvidence,createLabProject} from '../../web/lab/laboratory-project.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const n=64,w=8,adjacency=Array.from({length:n},(_,i)=>[i%w?i-1:-1,i%w<w-1?i+1:-1,i>=w?i-w:-1,i<n-w?i+w:-1].filter(v=>v>=0));
const graph={schema_version:1,state:'RI',year:'2020',geoids:Array.from({length:n},(_,i)=>`44001${String(i).padStart(6,'0')}`),adjacency,population:Array(n).fill(100),edges:adjacency.flatMap((ns,u)=>ns.filter(v=>v>u).map(v=>[u,v,100+(u%7)/10])),areas:Array(n).fill(100),exterior_perimeters:Array(n).fill(0),centroids:Array.from({length:n},(_,i)=>[-71+i%w*.001,41+Math.floor(i/w)*.001])};
async function inputFor(g,kind){
 const header=kind==='economic'?'geoid,c000,cns01,cns02,cns05,cns07,cns08,cns09,cns10,cns11':'geoid,pct_single_family,pct_multifamily,pct_owner,housing_vintage';
 const rows=g.geoids.map((id,i)=>id+','+(kind==='economic'?(i%3===0?'0,0,0,0,0,0,0,0,0':'1000,10,20,30,'+(i%2?'400,40,100,50,50':'100,200,200,100,100')):(i%2?'.8,.1,.6,.2':'.1,.7,.2,.8')));
 const csv=header+'\n'+rows.join('\n')+'\n';
 const input=engine.execute({operation:'import-character-csv',state:g.state,year:g.year,data_year:'2022',kind,source_label:'SYNTHETIC character values; not observed LODES/ACS data',source_base64:Buffer.from(csv).toString('base64')});validateCharacterInput(input,g,kind);
 return {input,csv};
}
const base={structure:'standard-bisect',weights:'economic-character',search:'single',districts:4,seed:'18446744073709551615',seeds:3,steps:4,percentile:.5,alpha_county:0,balance_tolerance:10,area_swing:1.1,iterations:10,character_alpha:.5};
const scope={name:'Character weighting test',mode:'state',states:['RI'],year:'2020',chamber:'congressional',timeout_seconds:60};
let exact=0,failures=0,replays=0,tamper=0,rejected=0,misses=0,disconnected=0,projects=0;
const methodResults=Object.create(null),generationFailures=[];
async function native(req,tool=false){await fs.writeFile('target/character-engine-request.json',JSON.stringify(req,(_key,value)=>Object.is(value,-0)?'__negative_zero__':value).replaceAll('"__negative_zero__"','-0.0'));return spawnSync(`target/release/examples/${tool?'execute_tool_request':'execute_request'}.exe`,['target/character-engine-request.json'],{encoding:'utf8',maxBuffer:32*1024*1024});}
const methods=['standard-bisect','nway','ratio-optimal','ratio-optimal-area','ratio-optimal-vra','prime-factor','compact-polsby','bfs-growth','centroidal-voronoi','moving-knife','spectral','capacity-clustering','regionalization','flow-construction','simulated-annealing','proportional-bisect','proportional-section'];
for(const kind of ['economic','housing']){
 const {input:character}=await inputFor(graph,kind);
 for(const structure of methods)for(const search of structure==='standard-bisect'?['single','multi','percentile','convergence','bisection-ensemble','flip','forest-recom','merge-split','short-burst','short-burst-forest','short-burst-merge-split','parallel-tempering','smc-percentile','vra-recom']:['single',...(['ratio-optimal','ratio-optimal-area','ratio-optimal-vra','compact-polsby','proportional-section'].includes(structure)?['multi']:[])])for(const districts of [1,3,4])for(const character_alpha of [0,.5,1]){
  const options={...base,weights:kind+'-character',structure,search,districts,character_alpha};
  if(structure==='simulated-annealing')Object.assign(options,{sa_steps_per_tract:1,sa_t0_factor:1,sa_t_final:.001});
  if(search.startsWith('short-burst'))Object.assign(options,{burst_length:2,n_bursts:2});
  if(search==='parallel-tempering')Object.assign(options,{pt_replicas:2,pt_swap_interval:1,pt_cold_tol:.1,pt_hot_tol:.25});
  if(search==='smc-percentile')Object.assign(options,{smc_particles:3,smc_resample_threshold:.5,balance_tolerance:.5});
  if(search==='vra-recom')Object.assign(options,{vra_threshold:.5,balance_tolerance:.5});
  if(structure==='ratio-optimal-vra')options.w_vra=.4;
  if(structure==='proportional-section')options.proportional_eta=1.1;
  const demographics=search==='vra-recom'||structure==='ratio-optimal-vra'?{schema_version:'bisect-demographic-fractions-v1',state:graph.state,year:graph.year,basis:'total-population',source_label:'SYNTHETIC demographics',minority_fractions:Object.fromEntries(graph.geoids.map((id,i)=>[id,i%2?.7:.3]))}:null;
  const elections=structure.startsWith('proportional')?{schema_version:'bisect-election-counts-v1',state:graph.state,year:graph.year,election_year:'2022',source_label:'SYNTHETIC counts',counts:Object.fromEntries(graph.geoids.map(id=>[id,{democratic:20,two_party:40}]))}:null;
  const extra={...(demographics?{demographics}:{}),...(elections?{elections}:{})},request={graph,options,character,...extra},config={...scope,...options,characters:{RI:character},...(demographics?{demographics:{RI:demographics}}:{}),...(elections?{elections:{RI:elections},...(structure==='proportional-section'?{proportional_eta:1.1}:{})}:{})};
  const methodKey=`${kind}:${structure}:${search}`,stats=methodResults[methodKey]??={exact:0,failures:0,balance_misses:0,disconnected:0};
  validateLabConfig(config);const ref=await native(request);if(ref.status!==0){let error;try{engine.execute(request);}catch(e){error=e;}assert.ok(error);assert.ok(ref.stderr.includes(error.message)||ref.stderr.includes(JSON.stringify(error.message)),`Native/WASM failure reasons differ: ${ref.stderr} / ${error.message}`);generationFailures.push({kind,structure,search,districts,alpha:character_alpha,error:error.message});stats.failures++;failures++;continue;}
  const result=engine.execute(request);assert.deepEqual(result,JSON.parse(ref.stdout));
  const state={code:'RI',metrics:{...result.metrics,district_count:districts,districts:result.metrics.district_metrics,balance_passed:result.metrics.within_requested_tolerance}};
  verifyLabAssignments(graph,state,result.assignments,config);await verifyDemographicEvidence(graph,state,config,result.assignments);
  const weighted=characterWeights(graph,config),oracleOptions={...options,weights:'geographic'};delete oracleOptions.character_alpha;
  const oracle=engine.execute({graph:{...graph,edges:graph.edges.map(([u,v,length])=>[u,v,weighted.weight(u,v,length)])},options:oracleOptions,...extra});
  assert.deepEqual(result.assignments,oracle.assignments);assert.equal(result.metrics.weighted_boundary,oracle.metrics.weighted_boundary);replays++;
  for(const field of ['method','kind','alpha','formula','zero_policy','data_year','baseline','tracts','scope']){const bad=structuredClone(state);bad.metrics.weighting_evidence[field]='altered';assert.throws(()=>verifyLabAssignments(graph,bad,result.assignments,config),/character weighting evidence/);tamper++;}
  const missing=structuredClone(state);delete missing.metrics.weighting_evidence;assert.throws(()=>verifyLabAssignments(graph,missing,result.assignments,config));tamper++;
  const bad=structuredClone(state);bad.metrics.weighting_evidence.character_sha256='0'.repeat(64);await assert.rejects(verifyDemographicEvidence(graph,bad,config,result.assignments),/character input identity/);tamper++;
  exact++;stats.exact++;stats.balance_misses+=!result.metrics.within_requested_tolerance;stats.disconnected+=!result.metrics.contiguous;misses+=!result.metrics.within_requested_tolerance;disconnected+=!result.metrics.contiguous;
 }
 for(const change of [{character_alpha:-1},{character_alpha:1.1},{character_alpha:-0},{weights:'geographic'},{weights:(kind==='economic'?'housing':'economic')+'-character'},{alpha_county:2}]){
  const req={graph,options:{...base,weights:kind+'-character',...change},character};assert.throws(()=>engine.execute(req));assert.notEqual((await native(req)).status,0);rejected++;
 }
 for(const change of [{state:'IA'},{year:'2010'},{schema_version:'bad'},{data_year:'bad'},{source_label:''},{data:{...character.data,characters:{}}}]){
  const req={graph,options:{...base,weights:kind+'-character'},character:{...character,...change}};assert.throws(()=>engine.execute(req));assert.notEqual((await native(req)).status,0);rejected++;
 }
 assert.throws(()=>engine.execute({graph,options:{...base,weights:kind+'-character'}}));
}
if(process.argv[2]){
 const root=process.argv[2],manifest=JSON.parse(await fs.readFile(path.join(root,'catalog.json'),'utf8'));
 const fetcher=async ref=>{const b=await fs.readFile(path.join(root,ref));return{ok:true,arrayBuffer:async()=>b.buffer.slice(b.byteOffset,b.byteOffset+b.length)};};
 class Worker{postMessage(m){setImmediate(()=>{if(this.terminated)return;try{this.onmessage({data:m.type==='initialize'?{type:'ready'}:{type:'result',id:m.id,result:engine.execute(m.request)}});}catch(e){this.onmessage({data:{type:'error',id:m.id,error:e.message}});}});}terminate(){this.terminated=true;}}
 const lab=new WasmCatalog(manifest,fetcher,()=>new Worker());
 for(const [key,entry]of Object.entries(manifest.graphs))for(const kind of ['economic','housing'])for(const search of ['single','multi','percentile','convergence','bisection-ensemble']){
  const [code,year]=key.split(':'),g=JSON.parse(await fs.readFile(path.join(root,entry.graph_ref),'utf8')),{input:character,csv}=await inputFor(g,kind),config={...scope,...base,states:[code],year,search,districts:null,weights:kind+'-character',characters:{[code]:character},metis_objective:'volume',metis_trials:3};
  const job=lab.submit(config);while(lab.active)await new Promise(r=>setTimeout(r,5));assert.equal(lab.jobs.get(job.id).status,'completed');
  const saved=await createLabProject(lab,job.id),restore=new WasmCatalog(manifest,fetcher,()=>{throw new Error('Open must not generate.');}),opened=await restore.restoreProject(structuredClone(saved));assert.deepEqual(opened.config.characters[code],character);assert.deepEqual(await restore.api(`/api/runs/${opened.id}/${code}/assignments`),saved.assignments[code]);
  const req={graph:g,options:saved.experiment.states[0].metrics.requested_options,character},ref=await native(req);assert.equal(ref.status,0,ref.stderr);assert.deepEqual(engine.execute(req),JSON.parse(ref.stdout));assert.deepEqual(engine.execute(req).assignments,saved.assignments[code]);
  const changed=structuredClone(saved);changed.experiment.config.character_alpha=.7;await assert.rejects(restore.restoreProject(changed));
  const exported=await lab.exportPractitionerProject(job.id,code);assert.deepEqual(exported.project.files.plan.provenance.producer.character_input,character);assert.equal(exported.project.files.context.source_hashes['bisect.character-input'],await characterIdentity(character));
  const exportReq={operation:'export-engine-plan',request:req,assignments:saved.assignments[code],label:'Character export',chamber:'congressional',created_at:'2026-10-04T00:00:00Z'},exportRef=await native(exportReq,true);assert.equal(exportRef.status,0,exportRef.stderr);assert.deepEqual(engine.execute(exportReq),JSON.parse(exportRef.stdout));
  await fs.writeFile(`target/character-${code}-${kind}-${search}.bisect`,JSON.stringify(saved));await fs.writeFile(`target/character-${code}-${kind}.csv`,csv);projects++;
 }
}
await fs.writeFile('target/character-wasm-verification.json',JSON.stringify({exact_cases:exact,independent_weighted_replays:replays,evidence_tamper_rejections:tamper,matching_failures:failures,invalid_rejections:rejected,real_state_project_export_roundtrips:projects,balance_misses:misses,disconnected,method_results:methodResults,generation_failures:generationFailures},null,2)+'\n');
console.log(`Character engine: ${exact} exact native/WASM cases, ${replays} weighted replays, ${tamper} tamper rejections, ${failures} matching generation failures, ${rejected} invalid requests, ${projects} real-state project/export replays; ${misses} balance misses, ${disconnected} disconnected.`);
