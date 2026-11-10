import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {verifyElectionResult} from '../../web/lab/election-input.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
import {createLabProject} from '../../web/lab/laboratory-project.js';

const root='target/partisan-proportional-bisect',source='dist/wasm-lab-build';await fs.mkdir(root,{recursive:true});
const manifest=JSON.parse(await fs.readFile(source+'/catalog.json','utf8'));
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));manifest.wasm_sha256=engine.artifactIdentity.sha256;
const reference='target/release/examples/partisan_proportional_api_reference.exe';
const graphs=[JSON.parse(await fs.readFile('target/ilp-balance-contract/branch-and-cut.json','utf8')).graph];
for(const code of ['RI','IA'])graphs.push(JSON.parse(await fs.readFile(source+'/'+manifest.graphs[code+':2020'].graph_ref,'utf8')));
const sharesFor=(g,profile)=>({schema_version:'bisect-partisan-shares-v1',state:g.state,year:g.year,source_label:'SYNTHETIC proportional API boundary shares',dem_shares:Object.fromEntries(g.geoids.map((id,i)=>[id,profile==='uniform'?.5:i%4<2?.8:.2]))});
const votesFor=(g,profile)=>({schema_version:'bisect-election-counts-v1',state:g.state,year:g.year,election_year:'2016',source_label:'SYNTHETIC API votes, not observations',counts:Object.fromEntries(g.geoids.map((id,i)=>[id,{democratic:g.population[i]*(profile==='uniform'?.5:i%4<2?.8:.2),two_party:g.population[i]}]))});
const base={structure:'proportional-bisect',weights:'partisan',search:'single',districts:3,seed:null,seeds:3,steps:20,percentile:0,alpha_county:0,balance_tolerance:25,area_swing:1.1,iterations:10,dem_threshold:.55,rep_threshold:.45};
let cases=0,failures=0,coiCases=0;const records=[];
for(const graph of graphs)for(const profile of ['uniform','stripe'])for(const voteProfile of ['uniform','stripe'])for(const seed of [null,'18446744073709551615'])for(const districts of [1,2,3])for(const withCoi of [false,true]){
 const partisan=sharesFor(graph,profile),elections=votesFor(graph,voteProfile),options={...base,seed,districts},request={graph,options,partisan,elections},tsv=root+'/shares-'+cases+'.tsv',csv=root+'/votes-'+cases+'.csv';
 await fs.writeFile(tsv,'geoid\tdem_share\n'+graph.geoids.map(id=>id+'\t'+partisan.dem_shares[id]).join('\n')+'\n');
 await fs.writeFile(csv,'geoid,dem,rep\n'+graph.geoids.map(id=>id+','+elections.counts[id].democratic+','+(elections.counts[id].two_party-elections.counts[id].democratic)).join('\n')+'\n');
 const fixture={graph,options,shares_path:path.resolve(tsv),election_path:path.resolve(csv)};
 if(withCoi){const weights_json=JSON.stringify({[graph.geoids[0]]:9,[graph.geoids[1]]:0,'unknown-parity-unit':4})+'\n',file=root+'/coi-'+cases+'.json';await fs.writeFile(file,weights_json);fixture.coi_path=path.resolve(file);request.coi={source_name:'SYNTHETIC COI.json',weights_json};}
 const file=root+'/native-'+cases+'.json';await fs.writeFile(file,JSON.stringify(fixture));const n=spawnSync(reference,[file],{encoding:'utf8',maxBuffer:16*1024*1024});assert.equal(n.error,undefined);
 let actual,error;try{actual=engine.execute(request);}catch(e){error=e;}
 assert.equal(Boolean(actual),n.status===0,`${graph.state}/${profile}/${voteProfile}/${districts}/${withCoi}: ${n.stderr}; ${error?.message}`);
 if(actual){const expected=JSON.parse(n.stdout);assert.deepEqual(actual.assignments,expected.assignments);assert(Math.abs(actual.metrics.weighted_boundary-expected.weighted_boundary)<1e-8);if(districts>1){assert.deepEqual(actual.metrics.structure_evidence.splits,expected.splits);assert.equal(actual.metrics.structure_evidence.requested_seed,seed);}
  await verifyElectionResult(graph,{...options,partisans:{[graph.state]:partisan},...(request.coi?{coi:request.coi}:{})},elections,actual);if(withCoi)coiCases++;
 }else{assert(n.stderr.includes(error.message));failures++;}
 records.push({state:graph.state,profile,voteProfile,seed,districts,withCoi,status:actual?'matched':'matching-native-failure'});cases++;
}
class Worker{postMessage(m){queueMicrotask(()=>{try{this.onmessage({data:m.type==='initialize'?{type:'ready'}:{type:'result',id:m.id,result:engine.execute(m.request)}});}catch(e){this.onmessage({data:{type:'error',id:m.id,error:e.message}});}});}terminate(){}}
const fetcher=async ref=>{const b=await fs.readFile(source+'/'+ref);return{ok:true,arrayBuffer:async()=>b.buffer.slice(b.byteOffset,b.byteOffset+b.length)};};
const configBase=JSON.parse(await fs.readFile('target/county-state-oracle/browser.bisect','utf8')).experiment.config;let projects=0,refusals=0;
for(const code of ['RI','IA']){
 const graph=graphs.find(g=>g.state===code),partisan=sharesFor(graph,'stripe'),elections=votesFor(graph,'stripe');
 const config={...configBase,...base,states:[code],districts:3,seed:'18446744073709551615',partisans:{[code]:partisan},elections:{[code]:elections}},lab=new WasmCatalog(manifest,fetcher,()=>new Worker()),job=lab.submit(config);while(lab.active)await new Promise(r=>setTimeout(r,5));assert.equal(lab.jobs.get(job.id).status,'completed',lab.jobs.get(job.id).logs.join('\n'));
 const p=await createLabProject(lab,job.id),restore=new WasmCatalog(manifest,fetcher,()=>{throw new Error('Open must not execute');});await restore.restoreProject(p);
 for(const mutate of [p=>{p.experiment.states[0].metrics.structure_evidence.splits[0].left_seats++;},p=>{p.experiment.config.partisans[code].dem_shares[graph.geoids[0]]=.6;},p=>{p.experiment.config.elections[code].counts[graph.geoids[0]].democratic++;}]){const changed=structuredClone(p);mutate(changed);await assert.rejects(()=>restore.restoreProject(changed));refusals++;}
 const exported=await lab.exportPractitionerProject(job.id,code);assert.deepEqual(exported.exported.document.provenance.producer.partisan_input,partisan);assert.deepEqual(exported.exported.document.provenance.producer.election_input,elections);
 if(projects===0)await fs.writeFile(root+'/browser.bisect',JSON.stringify(p));projects++;
}
assert.equal(cases,144);
await fs.writeFile(root+'/evidence.json',JSON.stringify({native_kernel_tsv_and_vote_csv_cases:cases,matching_failures:failures,coi_completed_cases:coiCases,real_state_projects_and_dual_source_exports:projects,source_or_schedule_refusals:refusals,inert_open:true,native_kernel_api_not_state_preset:true,denominator_census_population_preserved:true,records,wasm_sha256:engine.artifactIdentity.sha256},null,2));
console.log(`Partisan Proportional Bisect API: ${cases} native kernel/TSV/vote CSV cases, ${failures} matching failures, ${coiCases} COI cases; ${projects} dual-source projects/exports and ${refusals} tamper refusals.`);
