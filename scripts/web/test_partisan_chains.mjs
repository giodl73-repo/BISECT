import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
import {createLabProject} from '../../web/lab/laboratory-project.js';
const root='target/partisan-chains',source='dist/wasm-lab-build';await fs.mkdir(root,{recursive:true});
const manifest=JSON.parse(await fs.readFile(source+'/catalog.json','utf8')),engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));manifest.wasm_sha256=engine.artifactIdentity.sha256;
const graphs=[JSON.parse(await fs.readFile('target/ilp-balance-contract/branch-and-cut.json','utf8')).graph];for(const code of ['RI','IA'])graphs.push(JSON.parse(await fs.readFile(source+'/'+manifest.graphs[code+':2020'].graph_ref,'utf8')));
const sharesFor=(g,profile)=>({schema_version:'bisect-partisan-shares-v1',state:g.state,year:g.year,source_label:'SYNTHETIC chain parity shares',dem_shares:Object.fromEntries(g.geoids.map((id,i)=>[id,profile==='uniform'?.5:i%4<2?.8:.2]))});
const base={structure:'standard-bisect',weights:'partisan',search:'flip',districts:3,seed:null,seeds:3,steps:4,percentile:0,alpha_county:0,balance_tolerance:25,area_swing:1.1,iterations:10,dem_threshold:.55,rep_threshold:.45};
let cases=0,failures=0,coiCases=0;const records=[];
for(const graph of graphs)for(const search of ['flip','forest-recom','merge-split'])for(const profile of ['uniform','stripe'])for(const seed of [null,'18446744073709551615'])for(const districts of [1,3])for(const steps of [0,4])for(const percentile of [0,1]){
 const partisan=sharesFor(graph,profile),options={...base,search,seed,districts,steps,percentile},request={graph,options,partisan};
 const tsv=root+'/shares.tsv';await fs.writeFile(tsv,'geoid\tdem_share\n'+graph.geoids.map(id=>id+'\t'+partisan.dem_shares[id]).join('\n')+'\n');
 const tokens=['--state',graph.state,'--year',graph.year,'--partition-mode','partisan-weighted','--structure','standard-bisect','--partisan-shares',path.resolve(tsv),'--dem-threshold','.55','--rep-threshold','.45','--alpha-county','0','--districts',String(districts),'--search',search,'--niter','10','--balance-tolerance','25','--percentile',String(percentile),search==='flip'?'--flip-steps':search==='forest-recom'?'--forest-steps':'--merge-split-steps',String(steps),'--metis-engine','metis-core',...(seed===null?[]:['--seed',seed])];
 const fixture={graph,state_args:tokens};if(profile==='stripe'){const weights_json=JSON.stringify({[graph.geoids[0]]:9,[graph.geoids[1]]:0,'unknown-parity-unit':4})+'\n',file=root+'/coi.json';await fs.writeFile(file,weights_json);fixture.coi_path=path.resolve(file);request.coi={source_name:'SYNTHETIC COI.json',weights_json};}
 const file=root+'/request.json';await fs.writeFile(file,JSON.stringify(fixture));const n=spawnSync('target/release/examples/partisan_structure_reference.exe',[file],{encoding:'utf8',maxBuffer:16*1024*1024});assert.equal(n.error,undefined);let actual,error;try{actual=engine.execute(request);}catch(e){error=e;}
 assert.equal(Boolean(actual),n.status===0,`${graph.state}/${search}/${profile}/${districts}: ${n.stderr}; ${error?.message}`);
 if(actual){const expected=JSON.parse(n.stdout);assert.deepEqual(actual.assignments,expected.assignments);assert(Math.abs(actual.metrics.weighted_boundary-expected.weighted_boundary)<1e-8);if(search==='flip'&&districts>1){assert.equal(actual.metrics.structure_evidence.record_count,expected.summary.records);assert.equal(actual.metrics.structure_evidence.selected_rank,expected.summary.rank);}if(request.coi)coiCases++;}else{assert(n.stderr.includes(error.message));failures++;}
 records.push({state:graph.state,search,profile,seed,districts,steps,percentile,status:actual?'matched':'matching-native-failure'});cases++;
}
class Worker{postMessage(m){queueMicrotask(()=>{try{this.onmessage({data:m.type==='initialize'?{type:'ready'}:{type:'result',id:m.id,result:engine.execute(m.request)}});}catch(e){this.onmessage({data:{type:'error',id:m.id,error:e.message}});}});}terminate(){}}
const fetcher=async ref=>{const b=await fs.readFile(source+'/'+ref);return{ok:true,arrayBuffer:async()=>b.buffer.slice(b.byteOffset,b.byteOffset+b.length)};};
const configBase=JSON.parse(await fs.readFile('target/county-state-oracle/browser.bisect','utf8')).experiment.config;let projects=0,refusals=0;
for(const code of ['RI','IA'])for(const search of ['flip','forest-recom','merge-split']){
 const graph=graphs.find(g=>g.state===code),partisan=sharesFor(graph,'stripe'),config={...configBase,...base,search,states:[code],seed:'18446744073709551615',partisans:{[code]:partisan}},lab=new WasmCatalog(manifest,fetcher,()=>new Worker()),job=lab.submit(config);while(lab.active)await new Promise(r=>setTimeout(r,5));assert.equal(lab.jobs.get(job.id).status,'completed',lab.jobs.get(job.id).logs.join('\n'));
 const p=await createLabProject(lab,job.id),restore=new WasmCatalog(manifest,fetcher,()=>{throw new Error('Open must not execute');});await restore.restoreProject(p);
 for(const mutate of [p=>{p.experiment.states[0].metrics.structure_evidence.percentile=.5;},p=>{p.experiment.config.partisans[code].dem_shares[graph.geoids[0]]=.6;}]){const changed=structuredClone(p);mutate(changed);await assert.rejects(()=>restore.restoreProject(changed));refusals++;}
 const exported=await lab.exportPractitionerProject(job.id,code);assert.deepEqual(exported.exported.document.provenance.producer.partisan_input,partisan);
 if(projects===0)await fs.writeFile(root+'/browser.bisect',JSON.stringify(p));projects++;
}
assert.equal(cases,288);await fs.writeFile(root+'/evidence.json',JSON.stringify({native_state_tsv_cases:cases,matching_failures:failures,coi_completed_cases:coiCases,real_state_projects:projects,source_or_chain_refusals:refusals,inert_open:true,records,wasm_sha256:engine.artifactIdentity.sha256},null,2));console.log(`Partisan chains: ${cases} native State/TSV cases, ${failures} matching failures, ${coiCases} COI cases; ${projects} source-preserving projects/exports and ${refusals} tamper refusals.`);
