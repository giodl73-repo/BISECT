import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
import {createLabProject} from '../../web/lab/laboratory-project.js';
const root='target/optional-seed-methods';await fs.mkdir(root,{recursive:true});
const source='dist/wasm-lab-build',manifest=JSON.parse(await fs.readFile(source+'/catalog.json','utf8')),engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const fixture=JSON.parse(await fs.readFile('target/county-state-oracle/browser.bisect','utf8')),records=[];
for(const code of ['RI','IA']){
 const graph=JSON.parse(await fs.readFile(source+'/'+manifest.graphs[code+':2020'].graph_ref,'utf8'));
 const demographics={schema_version:'bisect-demographic-fractions-v1',state:code,year:'2020',basis:'total-population',source_label:'Synthetic uniform minority fixture',minority_fractions:Object.fromEntries(graph.geoids.map(id=>[id,.5]))};
 const elections={schema_version:'bisect-election-counts-v1',state:code,year:'2020',election_year:'2020',source_label:'Synthetic uniform election fixture',counts:Object.fromEntries(graph.geoids.map((id,i)=>[id,{democratic:graph.population[i]/2,two_party:graph.population[i]}]))};
 for(const [structure,searches]of Object.entries(manifest.catalog.search_compatibility))for(const search of searches){
  let request,raw;
  class Worker{postMessage(m){queueMicrotask(()=>{try{if(m.type==='initialize'){this.onmessage({data:{type:'ready'}});return;}request=m.request;raw=engine.execute(request);this.onmessage({data:{type:'result',id:m.id,result:raw}});}catch(e){this.onmessage({data:{type:'error',id:m.id,error:e.message}});}});}terminate(){}}
  const fetcher=async ref=>{const b=await fs.readFile(source+'/'+ref);return{ok:true,arrayBuffer:async()=>b.buffer.slice(b.byteOffset,b.byteOffset+b.length)};};
  const config={...fixture.experiment.config,states:[code],structure,search,weights:'geographic',alpha_county:0,seed:null,seeds:3,steps:4,percentile:0,balance_tolerance:50,iterations:10};
  if(['proportional-bisect','proportional-section'].includes(structure)){config.elections={[code]:elections};if(structure==='proportional-section')config.proportional_eta=1.1;}
  if(structure==='ratio-optimal-vra'){config.w_vra=.4;config.demographics={[code]:demographics};}
  if(structure==='ratio-optimal-area')config.area_init='ratio-optimal';
  if(structure==='compact-polsby')config.compact_epsilon=.05;
  if(structure==='moving-knife'){config.mka_orientations=8;config.mka_metric='reock';}
  if(structure==='centroidal-voronoi'){config.cvd_iters=4;config.cvd_metric='geographic';}
  if(structure==='simulated-annealing'){config.sa_steps_per_tract=1;config.sa_t0_factor=.01;config.sa_t_final=1e-4;}
  if(structure==='flow-construction')config.flow_repair='bfs';
  if(search==='vra-recom'){config.vra_threshold=.5;config.demographics={[code]:demographics};}
  if(search==='smc-percentile'){config.smc_particles=4;config.smc_resample_threshold=.5;}
  if(search==='parallel-tempering'){config.pt_replicas=2;config.pt_swap_interval=2;config.pt_cold_tol=.5;config.pt_hot_tol=1;}
  if(search.startsWith('short-burst')){config.burst_length=2;config.n_bursts=2;}
  if(['standard-bisect','ratio-optimal','ratio-optimal-area','ratio-optimal-vra','nway'].includes(structure)&&['single','multi','percentile','convergence','bisection-ensemble'].includes(search)){config.metis_objective='cut';config.metis_trials=1;}
  if(search.startsWith('multiscale')){config.seeds=1;config.multiscale={fine_level:'tract',coarse_level:'county',total_steps:4,alpha:.3,percentile:0,adaptive:search==='multiscale-adaptive'?{target_accept:.3,adapt_interval:2,gamma_0:.1,coarse_tol_factor:1}:null,inputs:{}};config.native_build=true;}
  const lab=new WasmCatalog(manifest,fetcher,()=>new Worker()),job=lab.submit(config);while(lab.active)await new Promise(r=>setTimeout(r,5));const state=lab.jobs.get(job.id).states[0];assert(request,'Must reach the actual engine boundary.');const file=root+'/'+records.length+'.json';await fs.writeFile(file,JSON.stringify(request));const native=spawnSync('target/release/examples/'+(request.operation?'execute_tool_request.exe':'execute_request.exe'),[file],{encoding:'utf8',maxBuffer:64*1024*1024,timeout:120000});
  if(state.status==='failed'){assert.notEqual(native.status,0);assert(native.stderr.includes(state.error),native.stderr+' vs '+state.error);records.push({code,structure,search,matching_failure:state.error});continue;}
  assert.equal(state.status,'completed',state.error);assert.equal(native.status,0,native.stderr);assert.deepEqual(JSON.parse(native.stdout),raw);
  const saved=await createLabProject(lab,job.id);assert.equal(saved.experiment.config.seed,null);const restored=new WasmCatalog(manifest,fetcher,()=>{throw new Error('Open must not execute');});const reopened=await restored.restoreProject(saved);assert.deepEqual(reopened.states[0].metrics,state.metrics);assert.deepEqual(await restored.api(`/api/runs/${reopened.id}/${code}/assignments`),saved.assignments[code]);
  for(const change of [p=>p.experiment.config.seed=0,p=>p.experiment.states[0].metrics.recorded_options.seed=p.experiment.states[0].metrics.recorded_options.seed===1?2:1]){const altered=structuredClone(saved);change(altered);await assert.rejects(()=>restored.restoreProject(altered));}
  if(code==='RI'&&structure==='ratio-optimal'&&search==='single')await fs.writeFile(root+'/ratio.bisect',JSON.stringify(saved));
  if(code==='RI'&&search==='multiscale')await fs.writeFile(root+'/multiscale.bisect',JSON.stringify(saved));
  records.push({code,structure,search,exact_native_result:true,inert_project:true,balance_passed:state.metrics.balance_passed});
 }
}
await fs.writeFile(root+'/evidence.json',JSON.stringify({records,attempts:records.length,exact_native_results:records.filter(r=>r.exact_native_result).length,matching_failures:records.filter(r=>r.matching_failure).length,wasm_sha256:engine.artifactIdentity.sha256},null,2));console.log(`Optional method seeds: ${records.length} pairs, ${records.filter(r=>r.exact_native_result).length} exact native results/inert projects, ${records.filter(r=>r.matching_failure).length} matching native failures.`);
