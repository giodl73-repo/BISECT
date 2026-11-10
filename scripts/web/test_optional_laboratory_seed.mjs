import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
import {createLabProject,validateLabConfig} from '../../web/lab/laboratory-project.js';
import {acceptedBuildConfig} from '../../web/lab/balance-build.js';
const root='target/optional-laboratory-seed';await fs.mkdir(root,{recursive:true});
const source='dist/wasm-lab-build',manifest=JSON.parse(await fs.readFile(source+'/catalog.json','utf8')),engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
class Worker{postMessage(m){queueMicrotask(()=>{try{this.onmessage({data:m.type==='initialize'?{type:'ready'}:{type:'result',id:m.id,result:engine.execute(m.request)}});}catch(e){this.onmessage({data:{type:'error',id:m.id,error:e.message}});}});}terminate(){}}
const fetcher=async ref=>{const b=await fs.readFile(source+'/'+ref);return{ok:true,arrayBuffer:async()=>b.buffer.slice(b.byteOffset,b.byteOffset+b.length)};};
const fixture=JSON.parse(await fs.readFile('target/county-state-oracle/browser.bisect','utf8')),base={...fixture.experiment.config,weights:'geographic',alpha_county:0,seed:null};
let cases=0;
for(const state of ['RI','IA'])for(const search of ['single','multi'])for(const native_build of [false,true]){
 const config={...base,states:[state],search,native_build,metis_objective:'cut',metis_trials:1},lab=new WasmCatalog(manifest,fetcher,()=>new Worker());const job=lab.submit(config);while(lab.active)await new Promise(r=>setTimeout(r,5));assert.equal(lab.jobs.get(job.id).status,'completed');
 const project=await createLabProject(lab,job.id),row=project.experiment.states[0],graph=JSON.parse(await fs.readFile(source+'/'+manifest.graphs[state+':2020'].graph_ref,'utf8'));assert.equal(project.experiment.config.seed,null);assert.equal(row.metrics.requested_options.seed,null);
 const request={graph,options:row.metrics.requested_options},result=engine.execute(native_build?{operation:'run-engine-build',request}:request),file=root+'/'+cases+'.json';await fs.writeFile(file,JSON.stringify(native_build?{operation:'run-engine-build',request}:request));const native=JSON.parse(execFileSync('target/release/examples/'+(native_build?'execute_tool_request.exe':'execute_request.exe'),[file],{encoding:'utf8',maxBuffer:32*1024*1024}));assert.deepEqual(native,result);assert.deepEqual(project.assignments[state],result.assignments);
 const accepted=acceptedBuildConfig(config,row.metrics),yaml=`name: omitted\nalgorithm:\n  structure: standard-bisect\n  weights: geographic\n  search: ${search}\n  seeds: 3\n  engine: metis-core\n  balance_tolerance: 25\n${accepted.seed===null?'':`  seed: ${accepted.seed}\n`}`;const oracleFile=root+'/'+cases+'-yaml.json';await fs.writeFile(oracleFile,JSON.stringify({graph,options:{...request.options,seed:accepted.seed},yaml_text:yaml}));const oracle=JSON.parse(execFileSync('target/release/examples/config_seed_reference.exe',[oracleFile],{encoding:'utf8',maxBuffer:32*1024*1024}));assert.deepEqual(oracle.assignments,result.assignments);assert.equal(oracle.declared_seed,accepted.seed===null?null:String(accepted.seed));
 const restored=new WasmCatalog(manifest,fetcher,()=>{throw new Error('Open must not run');});const reopened=await restored.restoreProject(project);assert.equal(reopened.config.seed,null);assert.deepEqual(reopened.states[0].metrics,row.metrics);
 if(state==='RI'&&search==='single'&&!native_build)await fs.writeFile(root+'/browser.bisect',JSON.stringify(project));cases++;
}
// Exercise native None -> Some(1) retry evidence, including attempts not selected.
const metrics={balance_passed:true,max_deviation_percent:2,balance_build:{schema_version:'bisect-balance-build-v1',accepted_attempt:1,max_balance_retries:50,attempts:[{attempt:0,seed:null,balanced:false,max_deviation_percent:30},{attempt:1,seed:'1',balanced:true,max_deviation_percent:2}],scope:'Synthetic native retry-order evidence fixture'}};assert.equal(acceptedBuildConfig({...base,native_build:true},metrics).seed,1);const forged=structuredClone(metrics);forged.balance_build.attempts[0].seed='0';assert.throws(()=>acceptedBuildConfig({...base,native_build:true},forged));
for(const seed of ['',undefined,'00',-1,Number.MAX_SAFE_INTEGER+1])assert.throws(()=>validateLabConfig({...base,seed}));
for(const structure of ['ratio-optimal','nway','bfs-growth'])validateLabConfig({...base,structure,seed:null});
await fs.writeFile(root+'/evidence.json',JSON.stringify({native_host_and_independent_yaml_assignment_cases:cases,inert_projects:cases,none_first_attempt_then_one_retry_checked:true,invalid_values_refused_and_additional_null_paths_valid:true,wasm_sha256:engine.artifactIdentity.sha256},null,2));console.log(`Optional laboratory seed: ${cases} native host/YAML assignment comparisons and inert project roundtrips; None retry evidence and invalid/unexposed paths checked.`);
