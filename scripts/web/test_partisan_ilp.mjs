import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
import {createLabProject} from '../../web/lab/laboratory-project.js';

const root='target/partisan-ilp',source='dist/wasm-lab-build';await fs.mkdir(root,{recursive:true});
const manifest=JSON.parse(await fs.readFile(source+'/catalog.json','utf8'));
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));manifest.wasm_sha256=engine.artifactIdentity.sha256;
const reference='target/release/examples/partisan_structure_reference.exe';
const graphs=[JSON.parse(await fs.readFile('target/ilp-balance-contract/branch-and-cut.json','utf8')).graph];
for(const code of ['RI','IA'])graphs.push(JSON.parse(await fs.readFile(source+'/'+manifest.graphs[code+':2020'].graph_ref,'utf8')));
const sharesFor=(g,profile)=>({schema_version:'bisect-partisan-shares-v1',state:g.state,year:g.year,source_label:'SYNTHETIC ILP reporting shares',dem_shares:Object.fromEntries(g.geoids.map((id,i)=>[id,profile==='uniform'?.5:i%4<2?.8:.2]))});
const base={structure:'ilp',search:'single',weights:'partisan',districts:2,seed:null,seeds:1,steps:20,percentile:0,alpha_county:0,balance_tolerance:10,area_swing:1.1,iterations:10,dem_threshold:.55,rep_threshold:.45};
const scenarios=[{ilp_max_tracts:0,ilp_fallback:'metis'},{ilp_max_tracts:16,ilp_fallback:'metis'},{ilp_max_tracts:0,ilp_fallback:'error'}];
let cases=0,failures=0,invariant=0;const records=[];
for(const graph of graphs)for(const profile of ['uniform','stripe'])for(const ilp_method of ['formulation-only','branch-and-cut','iterative-separation'])for(const scenario of scenarios)for(const seed of [null,'18446744073709551615'])for(const districts of [1,2]){
 const partisan=sharesFor(graph,profile),tsv=root+'/shares-'+cases+'.tsv';await fs.writeFile(tsv,'geoid\tdem_share\n'+graph.geoids.map(id=>id+'\t'+partisan.dem_shares[id]).join('\n')+'\n');
 const options={...base,...scenario,ilp_method,seed,districts},file=root+'/native-'+cases+'.json';
 await fs.writeFile(file,JSON.stringify({graph,state_args:['--state',graph.state,'--year',graph.year,'--partition-mode','ilp','--weights-override','proportional','--partisan-shares',path.resolve(tsv),'--districts',String(districts),'--balance-tolerance','10','--ilp-method',ilp_method,'--ilp-fallback',scenario.ilp_fallback,'--ilp-max-tracts',String(scenario.ilp_max_tracts),'--metis-engine','metis-core',...(seed===null?[]:['--seed',seed])]}));
 const native=spawnSync(reference,[file],{encoding:'utf8',maxBuffer:16*1024*1024});assert.equal(native.error,undefined);let actual,error;
 try{actual=engine.execute({graph,options,partisan});}catch(e){error=e;}
 assert.equal(Boolean(actual),native.status===0,`${graph.state}/${ilp_method}/${districts}: ${native.stderr}; ${error?.message}`);
 if(actual){const n=JSON.parse(native.stdout);assert.deepEqual(actual.assignments,n.assignments);assert(Math.abs(actual.metrics.weighted_boundary-n.weighted_boundary)<1e-8);
  const unweighted={...options,weights:'unweighted'};delete unweighted.dem_threshold;delete unweighted.rep_threshold;const baseline=engine.execute({graph,options:unweighted});assert.deepEqual(actual.assignments,baseline.assignments);assert.deepEqual(actual.metrics.structure_evidence,baseline.metrics.structure_evidence);invariant++;
 }else{assert(native.stderr.includes(error.message));failures++;}
 records.push({state:graph.state,profile,ilp_method,...scenario,seed,districts,status:actual?'matched':'matching-native-failure'});cases++;
}
class Worker{postMessage(m){queueMicrotask(()=>{try{this.onmessage({data:m.type==='initialize'?{type:'ready'}:{type:'result',id:m.id,result:engine.execute(m.request)}});}catch(e){this.onmessage({data:{type:'error',id:m.id,error:e.message}});}});}terminate(){}}
const fetcher=async ref=>{const b=await fs.readFile(source+'/'+ref);return{ok:true,arrayBuffer:async()=>b.buffer.slice(b.byteOffset,b.byteOffset+b.length)};};
const configBase=JSON.parse(await fs.readFile('target/county-state-oracle/browser.bisect','utf8')).experiment.config;
let projects=0,refusals=0;
for(const code of ['RI','IA']){
 const graph=graphs.find(g=>g.state===code),partisan=sharesFor(graph,'stripe'),config={...configBase,...base,states:[code],seed:'18446744073709551615',balance_tolerance:50,ilp_method:'formulation-only',ilp_fallback:'metis',ilp_max_tracts:0,capture_ilp_reports:true,partisans:{[code]:partisan}};
 const lab=new WasmCatalog(manifest,fetcher,()=>new Worker()),job=lab.submit(config);while(lab.active)await new Promise(r=>setTimeout(r,5));assert.equal(lab.jobs.get(job.id).status,'completed',lab.jobs.get(job.id).logs.join('\n'));
 const project=await createLabProject(lab,job.id),restore=new WasmCatalog(manifest,fetcher,()=>{throw new Error('Open must not execute');});await restore.restoreProject(project);
 const packet=project.experiment.states[0].metrics.ilp_report_capture;assert.deepEqual(JSON.parse(packet.source_json).partisan,partisan);assert.deepEqual(packet.files,{});
 for(const mutate of [
  p=>{p.experiment.config.partisans[code].dem_shares[graph.geoids[0]]=.6;},
  p=>{const packet=p.experiment.states[0].metrics.ilp_report_capture,request=JSON.parse(packet.source_json);request.partisan.dem_shares[graph.geoids[0]]=.6;packet.source_json=JSON.stringify(request);packet.source_sha256=createHash('sha256').update(packet.source_json).digest('hex');},
  p=>{const packet=p.experiment.states[0].metrics.ilp_report_capture,request=JSON.parse(packet.source_json);request.demographics={injected:'source'};packet.source_json=JSON.stringify(request);packet.source_sha256=createHash('sha256').update(packet.source_json).digest('hex');},
 ]){const changed=structuredClone(project);mutate(changed);await assert.rejects(()=>restore.restoreProject(changed));refusals++;}
 const exported=await lab.exportPractitionerProject(job.id,code);assert.deepEqual(exported.exported.document.provenance.producer.partisan_input,partisan);
 if(code==='RI')await fs.writeFile(root+'/browser.bisect',JSON.stringify(project));projects++;
}
assert.equal(cases,216);
await fs.writeFile(root+'/evidence.json',JSON.stringify({native_state_tsv_cases:cases,matching_failures:failures,assignment_and_structure_invariance:invariant,real_state_captured_projects_and_exports:projects,rehashed_source_or_config_refusals:refusals,inert_open:true,construction_ignores_partisan_weights:true,records,wasm_sha256:engine.artifactIdentity.sha256},null,2));
console.log(`Partisan ILP: ${cases} native State/TSV cases, ${failures} matching failures and ${invariant} unweighted-construction checks; ${projects} captured projects/exports and ${refusals} source/config refusals.`);
