import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
import {createLabProject} from '../../web/lab/laboratory-project.js';

const root='target/partisan-area',source='dist/wasm-lab-build';await fs.mkdir(root,{recursive:true});
const manifest=JSON.parse(await fs.readFile(source+'/catalog.json','utf8'));
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));manifest.wasm_sha256=engine.artifactIdentity.sha256;
const reference='target/release/examples/partisan_structure_reference.exe';
const graphs=[JSON.parse(await fs.readFile('target/ilp-balance-contract/branch-and-cut.json','utf8')).graph];
for(const code of ['RI','IA'])graphs.push(JSON.parse(await fs.readFile(source+'/'+manifest.graphs[code+':2020'].graph_ref,'utf8')));
const sharesFor=(g,profile)=>({schema_version:'bisect-partisan-shares-v1',state:g.state,year:g.year,source_label:'SYNTHETIC AreaSection parity shares',dem_shares:Object.fromEntries(g.geoids.map((id,i)=>[id,profile==='uniform'?.5:profile==='stripe'?(i%4<2?.8:.2):i%3===0?.55:.45]))});
const base={structure:'ratio-optimal-area',weights:'partisan',search:'single',districts:2,seed:null,seeds:3,steps:20,percentile:0,alpha_county:0,balance_tolerance:25,area_swing:1.1,iterations:10,dem_threshold:.55,rep_threshold:.45};
let cases=0,failures=0,empty=0;const records=[];
for(const graph of graphs)for(const profile of ['uniform','stripe','threshold-boundary'])for(const seed of [null,'18446744073709551615'])for(const districts of [1,2,3])for(const search of ['single','multi'])for(const area_init of ['ratio-optimal','moving-knife']){
 const partisan=sharesFor(graph,profile),tsv=root+'/shares-'+cases+'.tsv';await fs.writeFile(tsv,'geoid\tdem_share\n'+graph.geoids.map(id=>id+'\t'+partisan.dem_shares[id]).join('\n')+'\n');
 const file=root+'/native-'+cases+'.json';await fs.writeFile(file,JSON.stringify({graph,state_args:['--state',graph.state,'--year',graph.year,'--partition-mode','partisan-weighted','--structure','ratio-optimal-area','--partisan-shares',path.resolve(tsv),'--districts',String(districts),'--search',search,'--seeds','3','--area-swing','1.1','--area-section-init',area_init,'--niter','10','--balance-tolerance','25','--alpha-county','0','--dem-threshold','.55','--rep-threshold','.45','--metis-engine','metis-core',...(seed===null?[]:['--seed',seed])]}));
 const n=spawnSync(reference,[file],{encoding:'utf8',maxBuffer:16*1024*1024});assert.equal(n.error,undefined);
 const options={...base,seed,districts,search,area_init};let actual,error;try{actual=engine.execute({graph,options,partisan});}catch(e){error=e;}
 assert.equal(Boolean(actual),n.status===0,`${graph.state}/${profile}/${districts}/${area_init}: ${n.stderr}; ${error?.message}`);
 if(actual){const expected=JSON.parse(n.stdout);assert.deepEqual(actual.assignments,expected.assignments);assert(Math.abs(actual.metrics.weighted_boundary-expected.weighted_boundary)<1e-8);
  if(districts>1){const r=actual.metrics.root_split;assert.equal(r.native_candidate_cut,expected.root.cut);assert.equal(r.left_districts,expected.root.left);assert.equal(r.right_districts,expected.root.right);assert.equal(r.area_constraint_active,true);assert.equal(r.population_multiplier,1.001);assert.equal(r.area_within_requested_swing,true);if(profile==='uniform'){assert(r.native_candidate_cut===0);assert(r.weighted_boundary>0);empty++;}}
 }else{assert(n.stderr.includes(error.message));failures++;}
 records.push({state:graph.state,profile,seed,districts,search,area_init,status:actual?'matched':'matching-native-failure'});cases++;
}
class Worker{postMessage(m){queueMicrotask(()=>{try{this.onmessage({data:m.type==='initialize'?{type:'ready'}:{type:'result',id:m.id,result:engine.execute(m.request)}});}catch(e){this.onmessage({data:{type:'error',id:m.id,error:e.message}});}});}terminate(){}}
const fetcher=async ref=>{const b=await fs.readFile(source+'/'+ref);return{ok:true,arrayBuffer:async()=>b.buffer.slice(b.byteOffset,b.byteOffset+b.length)};};
const configBase=JSON.parse(await fs.readFile('target/county-state-oracle/browser.bisect','utf8')).experiment.config;let projects=0,rejectedRuns=0,refusals=0;
for(const code of ['RI','IA'])for(const search of ['single','multi'])for(const area_init of ['ratio-optimal','moving-knife']){
 const graph=graphs.find(g=>g.state===code),partisan=sharesFor(graph,'uniform');
 const config={...configBase,...base,states:[code],seed:'18446744073709551615',search,area_init,partisans:{[code]:partisan}},lab=new WasmCatalog(manifest,fetcher,()=>new Worker()),job=lab.submit(config);while(lab.active)await new Promise(r=>setTimeout(r,5));
 if(lab.jobs.get(job.id).status==='failed'){assert(lab.jobs.get(job.id).logs.some(l=>/area|ratio/.test(l)));rejectedRuns++;continue;}
 const p=await createLabProject(lab,job.id),restore=new WasmCatalog(manifest,fetcher,()=>{throw new Error('Open must not execute');});await restore.restoreProject(p);
 for(const mutate of [p=>{p.experiment.states[0].metrics.root_split.native_candidate_cut++;},p=>{p.experiment.config.partisans[code].dem_shares[graph.geoids[0]]=.6;},p=>{p.experiment.states[0].metrics.root_split.population_multiplier=1.1;}]){const changed=structuredClone(p);mutate(changed);await assert.rejects(()=>restore.restoreProject(changed));refusals++;}
 const exported=await lab.exportPractitionerProject(job.id,code);assert.deepEqual(exported.exported.document.provenance.producer.partisan_input,partisan);
 if(projects===0)await fs.writeFile(root+'/browser.bisect',JSON.stringify(p));projects++;
}
assert.equal(cases,216);assert(projects>0);
await fs.writeFile(root+'/evidence.json',JSON.stringify({native_state_tsv_cases:cases,matching_failures:failures,empty_map_native_cost_zero_reported_cost_positive:empty,real_state_projects_and_exports:projects,native_rejected_project_runs:rejectedRuns,source_cost_or_constraint_refusals:refusals,inert_open:true,supplied_geometry_not_native_filesystem_acquisition:true,records,wasm_sha256:engine.artifactIdentity.sha256},null,2));
console.log(`Partisan AreaSection: ${cases} native State/TSV cases, ${failures} matching failures, ${empty} sparse-empty root cost distinctions; ${projects} projects/exports and ${refusals} altered-source/cut/constraint refusals.`);
