import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
import {createLabProject} from '../../web/lab/laboratory-project.js';

const root='target/partisan-vra',source='dist/wasm-lab-build';await fs.mkdir(root,{recursive:true});
const manifest=JSON.parse(await fs.readFile(source+'/catalog.json','utf8'));
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));manifest.wasm_sha256=engine.artifactIdentity.sha256;
const reference='target/release/examples/partisan_structure_reference.exe';
const graphs=[JSON.parse(await fs.readFile('target/ilp-balance-contract/branch-and-cut.json','utf8')).graph];
for(const code of ['RI','IA'])graphs.push(JSON.parse(await fs.readFile(source+'/'+manifest.graphs[code+':2020'].graph_ref,'utf8')));
const sharesFor=(g,profile)=>({schema_version:'bisect-partisan-shares-v1',state:g.state,year:g.year,source_label:'SYNTHETIC VRASection partisan shares',dem_shares:Object.fromEntries(g.geoids.map((id,i)=>[id,profile==='uniform'?.5:i%4<2?.8:.2]))});
const demoFor=(g,profile)=>({schema_version:'bisect-demographic-fractions-v1',state:g.state,year:g.year,basis:'total-population',source_label:'SYNTHETIC minority mass fractions, not VAP observations',minority_fractions:Object.fromEntries(g.geoids.map((id,i)=>[id,profile==='uniform'?.6:i<g.geoids.length/2?.9:.2]))});
const base={structure:'ratio-optimal-vra',weights:'partisan',search:'single',districts:3,seed:null,seeds:3,steps:20,percentile:0,alpha_county:0,balance_tolerance:25,area_swing:1.1,iterations:10,dem_threshold:.55,rep_threshold:.45,w_vra:.4};
let cases=0,failures=0,distinctions=0;const records=[];
for(const graph of graphs)for(const profile of ['uniform','stripe'])for(const demoProfile of ['uniform','stripe'])for(const seed of [null,'18446744073709551615'])for(const districts of [1,2,3])for(const search of ['single','multi'])for(const w_vra of [0,.4]){
 const partisan=sharesFor(graph,profile),demographics=demoFor(graph,demoProfile),tsv=root+'/shares-'+cases+'.tsv',csv=root+'/demographics-'+cases+'.csv';
 await fs.writeFile(tsv,'geoid\tdem_share\n'+graph.geoids.map(id=>id+'\t'+partisan.dem_shares[id]).join('\n')+'\n');
 await fs.writeFile(csv,'GEOID,total_pop,white_non_hispanic\n'+graph.geoids.map((id,i)=>id+',100,'+(demoProfile==='uniform'?40:i<graph.geoids.length/2?10:80)).join('\n')+'\n');
 const file=root+'/native-'+cases+'.json';await fs.writeFile(file,JSON.stringify({graph,demographics_path:path.resolve(csv),state_args:['--state',graph.state,'--year',graph.year,'--partition-mode','partisan-weighted','--structure','ratio-optimal-vra','--partisan-shares',path.resolve(tsv),'--districts',String(districts),'--search',search,'--seeds','3','--w-vra',String(w_vra),'--niter','10','--balance-tolerance','25','--alpha-county','0','--dem-threshold','.55','--rep-threshold','.45','--metis-engine','metis-core',...(seed===null?[]:['--seed',seed])]}));
 const n=spawnSync(reference,[file],{encoding:'utf8',maxBuffer:16*1024*1024});assert.equal(n.error,undefined);
 let actual,error;try{actual=engine.execute({graph,options:{...base,seed,districts,search,w_vra},partisan,demographics});}catch(e){error=e;}
 assert.equal(Boolean(actual),n.status===0,`${graph.state}/${profile}/${demoProfile}/${districts}/${search}: ${n.stderr}; ${error?.message}`);
 if(actual){const expected=JSON.parse(n.stdout);assert.deepEqual(actual.assignments,expected.assignments);assert(Math.abs(actual.metrics.weighted_boundary-expected.weighted_boundary)<1e-8);
  if(districts>1){const r=actual.metrics.root_split,e=actual.metrics.structure_evidence;assert.equal(r.native_candidate_cut,expected.root.cut);assert.equal(r.weighted_boundary,expected.root.cut);assert.equal(r.left_districts,expected.root.left);assert.equal(r.right_districts,expected.root.right);
   let reported=0,leftMass=0,totalMass=0;for(const [i,id]of graph.geoids.entries()){const mass=demographics.minority_fractions[id]*graph.population[i];totalMass+=mass;if(actual.assignments[id]<=r.left_districts)leftMass+=mass;}
   for(const [u,v]of graph.edges)if((actual.assignments[graph.geoids[u]]<=r.left_districts)!==(actual.assignments[graph.geoids[v]]<=r.left_districts)){const a=partisan.dem_shares[graph.geoids[u]],b=partisan.dem_shares[graph.geoids[v]];reported+=a>=.55&&b>=.55||a<=.45&&b<=.45?3:1;}
   assert(Math.abs(r.reported_weighted_boundary-reported)<1e-8);assert(Math.abs(e.minority_mass_total-totalMass)<1e-8);assert(Math.abs(e.minority_mass_left-leftMass)<1e-8);
   const alignment=totalMass?Math.abs(leftMass/totalMass-.5)*2:0,normalised=r.native_candidate_cut/Math.sqrt(Math.min(r.left_districts,r.right_districts));
   assert(Math.abs(e.normalised_cut-normalised)<1e-9);assert(Math.abs(e.selection_score-(normalised-w_vra*alignment*Math.max(normalised,1)))<1e-9);
   if(profile==='uniform'){assert(r.native_candidate_cut===0);assert(r.reported_weighted_boundary>0);distinctions++;}
  }
 }else{assert(n.stderr.includes(error.message));failures++;}
 records.push({state:graph.state,profile,demoProfile,seed,districts,search,w_vra,status:actual?'matched':'matching-native-failure'});cases++;
}
class Worker{postMessage(m){queueMicrotask(()=>{try{this.onmessage({data:m.type==='initialize'?{type:'ready'}:{type:'result',id:m.id,result:engine.execute(m.request)}});}catch(e){this.onmessage({data:{type:'error',id:m.id,error:e.message}});}});}terminate(){}}
const fetcher=async ref=>{const b=await fs.readFile(source+'/'+ref);return{ok:true,arrayBuffer:async()=>b.buffer.slice(b.byteOffset,b.byteOffset+b.length)};};
const configBase=JSON.parse(await fs.readFile('target/county-state-oracle/browser.bisect','utf8')).experiment.config;let projects=0,refusals=0;
for(const code of ['RI','IA'])for(const search of ['single','multi']){
 const graph=graphs.find(g=>g.state===code),partisan=sharesFor(graph,'uniform'),demographics=demoFor(graph,'stripe');
 const config={...configBase,...base,states:[code],seed:'18446744073709551615',search,partisans:{[code]:partisan},demographics:{[code]:demographics}},lab=new WasmCatalog(manifest,fetcher,()=>new Worker()),job=lab.submit(config);while(lab.active)await new Promise(r=>setTimeout(r,5));assert.equal(lab.jobs.get(job.id).status,'completed',lab.jobs.get(job.id).logs.join('\n'));
 const p=await createLabProject(lab,job.id),restore=new WasmCatalog(manifest,fetcher,()=>{throw new Error('Open must not execute');});await restore.restoreProject(p);
 for(const mutate of [p=>{p.experiment.states[0].metrics.root_split.native_candidate_cut++;},p=>{p.experiment.states[0].metrics.root_split.reported_weighted_boundary++;},p=>{p.experiment.config.partisans[code].dem_shares[graph.geoids[0]]=.6;},p=>{p.experiment.config.demographics[code].minority_fractions[graph.geoids[0]]=.5;},p=>{p.experiment.states[0].metrics.structure_evidence.selection_score++;}]){const changed=structuredClone(p);mutate(changed);await assert.rejects(()=>restore.restoreProject(changed));refusals++;}
 const exported=await lab.exportPractitionerProject(job.id,code);assert.deepEqual(exported.exported.document.provenance.producer.partisan_input,partisan);assert.deepEqual(p.experiment.config.demographics[code],demographics);assert.match(exported.exported.context.source_hashes['bisect.demographic-input'],/^sha256:[a-f0-9]{64}$/);
 if(projects===0)await fs.writeFile(root+'/browser.bisect',JSON.stringify(p));projects++;
}
assert.equal(cases,288);
await fs.writeFile(root+'/evidence.json',JSON.stringify({native_state_tsv_and_demographic_csv_cases:cases,matching_failures:failures,empty_map_native_vs_reported_root_distinctions:distinctions,real_state_projects_and_dual_source_exports:projects,source_cost_or_score_refusals:refusals,inert_open:true,minority_mass_proxy_not_vap_or_legal_certificate:true,records,wasm_sha256:engine.artifactIdentity.sha256},null,2));
console.log(`Partisan VRASection: ${cases} native State/TSV/demographic CSV cases, ${failures} matching failures, ${distinctions} sparse root-cost distinctions; ${projects} projects/dual-source exports and ${refusals} tamper refusals.`);
