import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {verifyLabAssignments} from '../../web/lab/laboratory-project.js';
const root='target/county-all-states';await fs.mkdir(root,{recursive:true});
const source=process.argv[2]||'dist/wasm-lab-build',manifest=JSON.parse(await fs.readFile(source+'/catalog.json','utf8')),engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const states=manifest.catalog.states.filter(s=>manifest.graphs[s.code+':2020']).sort((a,b)=>a.code.localeCompare(b.code));assert.equal(states.length,50);
const reference_sha256=createHash('sha256').update(await fs.readFile('target/release/examples/county_state_reference.exe')).digest('hex');
let records=[];if(process.argv.includes('--resume')){const checkpoint=JSON.parse(await fs.readFile(root+'/checkpoint.json','utf8'));assert.equal(checkpoint.wasm_sha256,engine.artifactIdentity.sha256);assert.equal(checkpoint.reference_sha256,reference_sha256);records=checkpoint.records;}
const recordCase=async record=>{records.push(record);await fs.writeFile(root+'/checkpoint.json',JSON.stringify({completed_comparisons:records.length,records,wasm_sha256:engine.artifactIdentity.sha256,reference_sha256},null,2));console.log(`${records.length}/200 ${record.state}/${record.structure}/${record.search}: ${record.matching_failure?'matching failure':'exact assignments'}`);};
let totalUnits=0,totalPopulation=0,totalDistricts=0;
for(const state of states){
 const entry=manifest.graphs[state.code+':2020'],bytes=await fs.readFile(source+'/'+entry.graph_ref);assert.equal(createHash('sha256').update(bytes).digest('hex'),manifest.assets[entry.graph_ref]);const graph=JSON.parse(bytes),k=state.years.find(y=>y.year==='2020').districts.congressional;totalUnits+=graph.geoids.length;totalPopulation+=graph.population.reduce((a,b)=>a+b,0);totalDistricts+=k;
 for(const structure of ['standard-bisect','ratio-optimal'])for(const search of ['single','multi']){
  const prior=records.find(r=>r.state===state.code&&r.structure===structure&&r.search===search);if(prior){assert.equal(prior.prepared_graph_sha256,manifest.assets[entry.graph_ref]);continue;}
  const seed=search==='single'?null:'18446744073709551615',alpha=search==='single'?.125:100;
  const options={structure,weights:'county',search,districts:k,seed,seeds:3,steps:4,percentile:0,alpha_county:alpha,balance_tolerance:5,area_swing:1.1,iterations:10};
  const state_args=['--state',state.code,'--year','2020','--partition-mode','edge-weighted','--structure',structure,'--alpha-county',String(alpha),'--metis-engine','metis-core','--search',search,'--seeds','3','--niter','10','--balance-tolerance','5',...(seed===null?[]:['--seed',seed])],file=root+'/request.json';await fs.writeFile(file,JSON.stringify({graph,state_args,districts:k}));
  const native=spawnSync('target/release/examples/county_state_reference.exe',[file],{encoding:'utf8',maxBuffer:64*1024*1024,timeout:600000});
  if(native.status!==0){assert.equal(native.error,undefined,'Native timeout/process error is not a matching algorithm failure.');let error;try{engine.execute({graph,options});}catch(e){error=e.message;}assert(error,'WASM must also reject a native failure.');assert(native.stderr.includes(error),native.stderr+' vs '+error);await recordCase({state:state.code,structure,search,seed,alpha,k,units:graph.geoids.length,prepared_graph_sha256:manifest.assets[entry.graph_ref],matching_failure:error});continue;}
  const ref=JSON.parse(native.stdout),actual=engine.execute({graph,options});assert.deepEqual(actual.assignments,ref.assignments);assert.equal(actual.metrics.within_requested_tolerance,ref.balanced);assert.equal(ref.alpha,alpha);assert(Math.abs(actual.metrics.weighted_boundary-ref.weighted_cut)<=1e-8*Math.max(1,ref.weighted_cut));
  if(k>1){assert.equal(actual.metrics.weighting_evidence.alpha,alpha);assert.equal(actual.metrics.weighting_evidence.within_county_multiplier,1+alpha);}
  const weights=new Map(ref.weighted_edges.map(([u,v,w])=>[u+':'+v,w]));for(const [u,v,length]of graph.edges)assert.equal(weights.get(u+':'+v),length*(graph.geoids[u].slice(0,5)===graph.geoids[v].slice(0,5)?1+alpha:1));
  const config={...options,name:'All-state county oracle',mode:'state',states:[state.code],year:'2020',chamber:'congressional',timeout_seconds:120};verifyLabAssignments(graph,{code:state.code,metrics:{...actual.metrics,district_count:k,districts:actual.metrics.district_metrics,balance_passed:ref.balanced}},actual.assignments,config);
  await recordCase({state:state.code,structure,search,seed,alpha,k,units:graph.geoids.length,prepared_graph_sha256:manifest.assets[entry.graph_ref],balanced:ref.balanced,contiguous:actual.metrics.contiguous});
 }
  console.log(state.code+': four independent native State comparisons');
}
const failures=records.filter(r=>r.matching_failure).length,misses=records.filter(r=>r.balanced===false).length,disconnected=records.filter(r=>r.contiguous===false).length;
await fs.writeFile(root+'/evidence.json',JSON.stringify({states:states.length,comparisons:records.length,exact_assignment_matches:records.length-failures,matching_failures:failures,total_units:totalUnits,total_population:totalPopulation,total_districts:totalDistricts,balance_misses:misses,disconnected,one_district_comparisons:records.filter(r=>r.k===1).length,records,wasm_sha256:engine.artifactIdentity.sha256},null,2));console.log(`All-state county: ${records.length-failures} exact native State assignment matches, ${failures} matching failures, ${misses} final balance misses, ${disconnected} disconnected.`);
