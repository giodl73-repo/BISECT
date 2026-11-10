import {createNativeCommandComparator} from './native_command_comparison.mjs';
let strictComparator;
function compareNative(request,actual,expected){strictComparator??=createNativeCommandComparator(root,'target/wasm32-unknown-unknown/release/bisect_wasm.wasm');strictComparator.compare(request,actual,expected);}
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {writeFileSync} from 'node:fs';
import {spawnSync,execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const root='target/native-adjacency-state';await fs.mkdir(root,{recursive:true});
const wasmOnly=process.argv.includes('--wasm-only');
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm')),previous=process.argv.includes('--skip-previous')?null:await instantiateEngine(await fs.readFile(root+'/before.wasm'));
const hash=s=>createHash('sha256').update(s).digest('hex');
let nativeMatches=0,nativeErrors=0,kernels=0,runs=0,unchanged=0;const outputs=[];
function check(request){writeFileSync(root+'/request.json',JSON.stringify(request));let actual,error;try{actual=engine.execute(request);}catch(e){error=e;}
 if(wasmOnly){if(error)throw error;return actual;}
 const host=spawnSync('target/release/examples/execute_tool_request.exe',[root+'/request.json'],{encoding:'utf8',windowsHide:true,maxBuffer:64*1024*1024});
 if(error){assert.notEqual(host.status,0,host.stdout);const raw=host.stderr.trim().replace(/^Error: /,'');assert.equal(JSON.parse(raw),error.message);nativeErrors++;throw error;}
 assert.equal(host.status,0,host.stderr);compareNative(request,actual,JSON.parse(host.stdout));nativeMatches++;return actual;
}
const original=JSON.parse(await fs.readFile('scripts/web/fixtures/native-state-source.json','utf8'));
const source=(name,value)=>({name,base64:Buffer.from(JSON.stringify(value)).toString('base64')});
function custom(state,year,unit_kind){const ids=['ward-A','ward-B','ward-C','ward-D'];return {sources:{...structuredClone(original.nativeAdjacencyInputs),geoids:source('custom-ids.json',Object.fromEntries(ids.map((id,i)=>[i,id]))),county_ids:source('custom-counties.json',{schema_version:1,state,year,unit_ids:ids,county_ids:['North','North','South','South']})},state,year,unit_kind};}
const specs=[
 {name:'custom-imported',source:custom('ZZ','2024','imported'),chamber:'congressional',districts:2,resolution:'tract'},
 {name:'custom-precinct',source:custom('ZZ','2024','precinct'),chamber:'senate',districts:1,resolution:'block_group'},
 {name:'custom-year-house',source:custom('RI','2024','imported'),chamber:'house',districts:2,resolution:'block'},
 {name:'native-resolution-bypass',source:{sources:structuredClone(original.nativeAdjacencyInputs),state:'RI',year:'2020',unit_kind:'tract'},chamber:'congressional',districts:null,resolution:'block_group'}
];
for(const spec of specs){
 const arguments_=['--state',spec.source.state,'--year',spec.source.year,'--resolution',spec.resolution,'--chamber',spec.chamber,'--metis-engine','metis-core','--structure','nway','--search','single','--balance-tolerance','5','--seed','18446744073709551615','--adjacency','unopened/v3.adj.bin',...(spec.districts===null?[]:['--districts',String(spec.districts)])],request={operation:'prepare-native-adjacency-state-run',input:{arguments:arguments_,source:spec.source,label:spec.name,chamber:spec.chamber,created_at:'2026-10-21T00:00:00Z'}};
 const before=runs,compiled=check(request);assert.equal(runs,before);assert.equal(compiled.execution_performed,false);assert.equal(compiled.prepared_review.execution_performed,false);assert.equal(compiled.adjacency_override.path_accessed,false);assert.equal(compiled.adjacency_override.resolution_lookup_applied,false);assert.equal(compiled.adjacency_override.census_year_policy_applied,false);assert.deepEqual(compiled.native_command.arguments,arguments_);assert.equal(compiled.native_adjacency_source.state,spec.source.state);assert.equal(compiled.native_adjacency_source.year,spec.source.year);assert.equal(compiled.native_adjacency_source.graph_json,undefined);
 for(const [key,value] of Object.entries(spec.source.sources))if(value){const identity=compiled.native_adjacency_source.sources[key];assert(identity,key);assert.equal(identity.name,value.name);assert.equal(identity.sha256,hash(Buffer.from(value.base64,'base64')));}
 const parsed=JSON.parse(compiled.prepared_request_json),count=spec.districts??2;assert.equal(parsed.request.options.districts,count);assert.equal(parsed.request.options.seed,'18446744073709551615');assert.equal(parsed.request.graph.unit_kind,spec.source.unit_kind);assert.equal(parsed.request.graph.year,spec.source.year);assert.equal(parsed.request.graph.state,spec.source.state);
 const execute={operation:'run-prepared-engine',input:{request_json:compiled.prepared_request_json,inspect_only:false,label:spec.name,chamber:spec.chamber,created_at:'2026-10-21T00:00:00Z'}},accepted=check(execute);runs++;assert.equal(accepted.export.document.metadata.chamber,spec.chamber);assert.equal(accepted.export.document.plan.units.unit_kind,spec.source.unit_kind);assert.equal(accepted.export.document.plan.k,count);
 const nativeArgs=[...arguments_];if(spec.districts===null)nativeArgs.push('--districts','2');await fs.writeFile(root+'/kernel.json',JSON.stringify({state_args:nativeArgs,graph:parsed.request.graph}));const kernel=JSON.parse(execFileSync('target/debug/examples/geometric_state_reference.exe',[root+'/kernel.json'],{encoding:'utf8',windowsHide:true}));assert.deepEqual(accepted.result.assignments,kernel.assignments);kernels++;
 await fs.writeFile(root+'/'+spec.name+'.compiled.json',JSON.stringify(compiled));await fs.writeFile(root+'/'+spec.name+'.accepted.json',JSON.stringify(accepted));await fs.writeFile(root+'/'+spec.name+'.rplan',accepted.export.document_json);await fs.writeFile(root+'/'+spec.name+'.rctx',accepted.export.context_json);outputs.push({request,compiled,accepted});
}
const base=outputs[0].request;
for(const change of [r=>r.input.arguments.push('--reset'),r=>r.input.arguments.push('--manifest'),r=>r.input.arguments.push('--state-name','unused'),r=>r.input.arguments.splice(r.input.arguments.indexOf('--adjacency'),2),r=>r.input.arguments[r.input.arguments.indexOf('--adjacency')+1]='unopened/wrong.adj.bin',r=>r.input.source.state='RI',r=>r.input.source.year='2020',r=>r.input.chamber='house',r=>r.input.source.sources.county_ids=null,r=>r.input.arguments.splice(r.input.arguments.indexOf('--districts'),2)]){const bad=structuredClone(base);change(bad);assert.throws(()=>check(bad));}
const beforeRead=structuredClone(base);beforeRead.input.arguments.push('--force');beforeRead.input.source.sources.adjacency.base64='not-valid';assert.throws(()=>check(beforeRead),/--force/);
// Ordinary prepared graphs retain their refusal and scope checks.
const plainSource={operation:'prepare-native-engine-input',input:{sources:original.nativeAdjacencyInputs,...original.nativeAdjacencySettings}},graph=JSON.parse(check(plainSource).graph_json),plain={operation:'prepare-native-state-graph-run',input:{arguments:['--state','RI','--metis-engine','metis-core','--structure','nway','--search','single','--districts','2','--seed','18446744073709551615'],graph_json:JSON.stringify(graph),graph_source_name:'unchanged.json',label:'unchanged',chamber:'congressional',created_at:'2026-10-21T00:00:00Z'}};
if(previous)for(const r of [plainSource,plain]){assert.deepEqual(engine.execute(r),previous.execute(r));unchanged++;}
for(const extra of [['--adjacency','unopened/v3.adj.bin'],['--resolution','block_group'],['--year','2024']]){const bad=structuredClone(plain);bad.input.arguments.push(...extra);assert.throws(()=>check(bad));}
const ordinaryYear=structuredClone(plain),ordinaryGraph=JSON.parse(ordinaryYear.input.graph_json);ordinaryGraph.year='2024';ordinaryYear.input.graph_json=JSON.stringify(ordinaryGraph);ordinaryYear.input.arguments.push('--year','2024');assert.throws(()=>check(ordinaryYear),/not available/);
await fs.writeFile(root+'/outputs.json',JSON.stringify(outputs));await fs.writeFile(root+'/evidence.json',JSON.stringify({wasm:engine.artifactIdentity,previous_wasm:previous?.artifactIdentity??null,previous_comparison_performed:!!previous,native_comparison_performed:!wasmOnly,whole_native_host_replies:nativeMatches,whole_native_host_errors:nativeErrors,independent_native_nway_matches:kernels,explicit_prepared_runs:runs,unchanged_legacy_results:unchanged,cases:specs.map(s=>s.name),all_selected_source_bytes_hash_bound:true,custom_ids_counties_chambers_and_years_preserved:true,ordinary_graph_policy_unchanged:true,paths_accessed:false,ui_connected:false,published:false},null,2));console.log('PASS native adjacency State boundary: '+nativeMatches+' native replies, '+nativeErrors+' exact native errors, '+kernels+' independent native kernels; custom source identities/years/chambers and native adjacency bypass, inert preparation, ordinary policy unchanged. UI connection remains pending.');
