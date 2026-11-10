let strictComparator;
function compareNative(request,actual,expected){strictComparator??=createNativeCommandComparator(root,'target/wasm32-unknown-unknown/release/bisect_wasm.wasm');strictComparator.compare(request,actual,expected);}
import {createNativeCommandComparator} from './native_command_comparison.mjs';
// Review the 37 previously unclassified State fields against actual execution
// evidence. Explicit refusals are recorded as remaining work, never as parity.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync,spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {preparedEngineFromStateCommand,statePreparedFlagRefusals} from '../../web/lab/state-command-files.js';
import {createProject} from '../../web/lab/project.js';
const root='target/native-state-option-contract';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const rows=JSON.parse(await fs.readFile('target/native-state-run/records.json','utf8')),prior=JSON.parse(await fs.readFile('target/native-state-run/evidence.json','utf8'));
assert.deepEqual(prior.wasm,engine.artifactIdentity,'Regenerate the State Run matrix for this WASM artifact first.');assert.equal(prior.compiled,rows.length);assert.equal(prior.full_native_host_matches,prior.compiled+prior.executed_with_native_retries_and_exports);assert(prior.independent_native_state_assignment_matches>=24);
const contracts={},record=(field,cases,classification='compiled-selected-source-path')=>{assert(cases.length,field);contracts[field]={classification,cases};};
const compiled=row=>JSON.parse(row.result.prepared_request_json);
const option=row=>{const r=compiled(row);return (r.request??r.input?.request).options;};
const named=name=>{const row=rows.find(r=>r.name===name);assert(row,name);return row;};
const value=(row,field)=>{const f=row.result.native_command.fields.find(f=>f.id===field);assert.equal(f.source,'command-line',field);return f.values[0];};
for(const [field,caseName,path] of [
 ['seeds','search-multi','seeds'],['percentile','search-percentile','percentile'],['ensemble_steps','search-bisection-ensemble','steps'],['burst_length','search-short-burst','burst_length'],['n_bursts','search-short-burst','n_bursts'],['flip_steps','search-flip','steps'],['forest_steps','search-forest-recom','steps'],['merge_split_steps','search-merge-split','steps']
]){const row=named(caseName);assert.equal(option(row)[path],Number(value(row,field)));record(field,[caseName]);}
for(const prefix of ['structure','weights','search']){
 const selected=rows.filter(r=>r.name.startsWith(prefix+'-'));assert.equal(selected.length,{structure:14,weights:8,search:16}[prefix]);
 for(const row of selected){const name=row.name.slice(prefix.length+1);if(prefix==='structure')assert.equal(option(row).structure,name);else if(prefix==='search'&&!name.startsWith('multiscale'))assert.equal(option(row).search,name);else if(prefix==='weights')assert.equal(option(row).weights,({county:'geographic',proportional:'partisan'})[name]??name);}
 record(prefix==='weights'?'weights_override':prefix,selected.map(r=>r.name));
}
for(const [field,path,name] of [['multiscale_steps','total_steps','search-multiscale'],['multiscale_alpha','alpha','search-multiscale'],['ms_target_accept','target_accept','search-multiscale-adaptive'],['ms_adapt_interval','adapt_interval','search-multiscale-adaptive']]){const row=named(name),input=compiled(row).input;assert.equal((field.startsWith('ms_')?input.adaptive:input)[path],Number(value(row,field)));record(field,[name]);}
for(const [field,path] of [['multiscale_fine','fine_level'],['multiscale_coarse','coarse_level']]){const row=named('search-multiscale');assert.equal(compiled(row).input[path],value(row,field));record(field,[row.name]);}
let supplemented=0,refusals=0;
const base=structuredClone(named('mode-unweighted').request),rewrite=(args,flag,v)=>{const i=args.indexOf(flag);if(i>=0)args.splice(i,2,flag,v);else args.push(flag,v);};
async function supplement(name,flag,v,change=()=>{}){
 const request=structuredClone(base);rewrite(request.input.arguments,flag,v);change(request.input.prepared);
 const result=engine.execute(request),file=root+'/'+name+'.json';await fs.writeFile(file,JSON.stringify(request));const native=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[file],{encoding:'utf8',windowsHide:true,maxBuffer:32*1024*1024}));compareNative(request,result,native);supplemented++;return {name,result};
}
const state=await supplement('state-lowercase','--state','ri');assert.equal(state.result.prepared_review.state,'RI');record('state',[state.name]);
for(const year of ['2010','2000']){const row=await supplement('year-'+year,'--year',year,p=>{const value=JSON.parse(p.request_json);value.graph.year=year;p.request_json=JSON.stringify(value);});assert.equal(row.result.prepared_review.year,year);}
record('year',['year-2010','year-2000']);
record('resolution',['resolution-block_group','resolution-block']);for(const name of contracts.resolution.cases){assert(named(name).execution);}
const chamber=await supplement('chamber-house','--chamber','house',p=>p.chamber='house');const chamberRequest=JSON.parse(await fs.readFile(root+'/chamber-house.json','utf8'));const {request_json,...settings}=chamberRequest.input.prepared;const selected=await preparedEngineFromStateCommand({arguments_json:JSON.stringify(chamberRequest.input.arguments)},{source:{name:'scope.json',text:request_json},settings},chamber.result);assert.equal(selected.settings.chamber,'house');const mismatch=structuredClone(chamberRequest);mismatch.input.prepared.chamber='congressional';assert.throws(()=>engine.execute(mismatch),/chamber/);record('chamber',[chamber.name]);
const districts=await supplement('districts-two','--districts','2');assert.equal(JSON.parse(districts.result.prepared_request_json).request.options.districts,2);record('districts',[districts.name]);
const tolerance=await supplement('tolerance-three','--balance-tolerance','3');assert.equal(JSON.parse(tolerance.result.prepared_request_json).request.options.balance_tolerance,3);record('balance_tolerance',[tolerance.name]);
const profile=await supplement('profile-court','--profile','court');assert.equal(profile.result.native_command.profile_filter.allowed,true);record('profile',[profile.name]);
const unsupported={version:['--version','vContract'],output_dir:['--output-dir','unopened path'],dpi:['--dpi','72'],position:['--position','1'],print_only:['--print-only'],debug:['--debug'],reset:['--reset'],manifest:['--manifest'],force:['--force'],adjacency:['--adjacency','unopened.adj.bin'],state_name:['--state-name','unopened-state'],label:['--label','Unopened label'],time_partition:['--time-partition'],population_source:['--population-source','vap']};
const compiler=await fs.readFile('crates/bisect-wasm/src/native_state_run.rs','utf8');const guarded=JSON.parse('['+/let unsupported=\[([^\]]+)\]/.exec(compiler)[1]+']');assert.deepEqual(guarded.sort(),Object.keys(unsupported).sort());
assert.deepEqual(statePreparedFlagRefusals(named('mode-unweighted').result.native_command),[]);
for(const [field,args] of Object.entries(unsupported)){
 const request=structuredClone(base);request.input.arguments.push(...args);assert.deepEqual(statePreparedFlagRefusals(engine.execute({operation:'inspect-native-state-command',arguments:request.input.arguments})),[args[0]]);assert.throws(()=>engine.execute(request),e=>e.message.includes('orchestration flags')&&e.message.includes(args[0]));const file=root+'/refused-'+field+'.json';await fs.writeFile(file,JSON.stringify(request));const native=spawnSync('target/release/examples/execute_tool_request.exe',[file],{encoding:'utf8',windowsHide:true});assert.notEqual(native.status,0);assert((native.stdout+native.stderr).includes(args[0]));refusals++;if(field!=='population_source')record(field,['refused-'+field],'explicit-prepared-path-refusal');
}
assert.equal(Object.keys(contracts).length,37);const report={wasm:engine.artifactIdentity,compiler_sha256:createHash('sha256').update(compiler).digest('hex'),fields:contracts,supplemental_whole_native_host_compilations:supplemented,whole_native_host_refusals:refusals,baseline_matrix:prior,scope:'Selected prepared/native graph behavior only. Refused orchestration flags are incomplete work, not implemented native effects. No claim of every source/method combination or full State orchestration.',published:false};
await fs.writeFile(root+'/evidence.json',JSON.stringify(report,null,2));console.log(`PASS State option contract: 24 active/scope fields, 13 explicitly refused orchestration fields, ${supplemented} supplemental native compilations and ${refusals} host/WASM refusals; current 87-case execution matrix required.`);
await fs.writeFile(root+'/browser.bisect',JSON.stringify(createProject({name:'Native State preparation boundaries',files:{stateCommandInputs:{arguments_json:JSON.stringify(['--state','RI','--metis-engine','metis-core','--reset','--output-dir','unopened path'])}},operation:'inspect-native-state-command',constraints:[],result:null,lastOperation:null})));
