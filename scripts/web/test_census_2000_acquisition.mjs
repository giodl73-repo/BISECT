import {createNativeCommandComparator} from './native_command_comparison.mjs';
let strictComparator,strictComparisonPairs=0;
function compareNative(request,actual,expected){strictComparator??=createNativeCommandComparator(root,'target/wasm32-unknown-unknown/release/bisect_wasm.wasm');strictComparator.compare(request,actual,expected);strictComparisonPairs++;}
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {writeFileSync} from 'node:fs';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {CENSUS_ARCHIVE_ENDPOINTS} from '../../web/lab/census-endpoints.js';
import {publicFetchUrl} from '../../web/lab/fetch-acquisition.js';
import {sourceRelayUrl} from '../../web/lab/source-relay-policy.js';
import {extractCensusZip} from '../../web/lab/census-archive.js';
import {acquireStateSourceJob,stateSourceAcquisitionChoices} from '../../web/lab/state-source-acquisition.js';
import {executeNativeStateJob} from '../../web/lab/native-state-job.js';
const root='target/census-2000-acquisition';await fs.mkdir(root,{recursive:true});
const original=process.argv.includes('--original'),engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const manifest=JSON.parse(await fs.readFile('data/manifest.json','utf8'));
const selection={manifest_json:null,states:Object.keys(manifest.states),year:'2000',data_types:['tiger','redistricting'],selected_paths:[]};
const all=engine.execute({operation:'plan-fetch-sources',input:selection});assert.equal(all.items.length,100);assert.equal(stateSourceAcquisitionChoices(all).length,50);
for(const item of all.items){assert.equal(publicFetchUrl(item),item.url);assert.equal(CENSUS_ARCHIVE_ENDPOINTS[item.state_code+':2000:'+item.kind],item.url);assert.equal(new URL(sourceRelayUrl(item.url,'https://bisect.test')).searchParams.get('url'),item.url);for(const url of [item.url+'?redirect=bad',item.url.replace('www2.census.gov','attacker.invalid')]){assert.throws(()=>publicFetchUrl({...item,url}));assert.throws(()=>sourceRelayUrl(url,'https://bisect.test'));}}
const plan=engine.execute({operation:'plan-fetch-sources',input:{...selection,states:['VT']}}),archives={};
for(const kind of ['tiger','pl94171'])archives[kind]=await fs.readFile(original?'target/census-2000-sources/VT/'+kind+'.zip':'target/stream-native-run-years/fixtures/2000-'+(kind==='tiger'?'tiger':'pl94')+'.zip');
await assert.rejects(()=>extractCensusZip(archives.pl94171,{kind:'pl94',year:2010}),/geography/);
await assert.rejects(async()=>extractCensusZip(await fs.readFile('target/stream-native-run-years/fixtures/ambiguous-2000.zip'),{kind:'pl94',year:2000}),/Ambiguous/);
const files={nativeStateSourceSettings:{enabled:true,command_kind:'run'},runCommandInputs:{arguments_json:JSON.stringify(['--year','2000','--states','VT'])},stateGraphInputs:{graph:null,auxiliary:null,settings:{inspect_only:true,label:'Census 2000 original VT',chamber:'congressional',created_at:'2026-10-20T00:00:00Z'}}};
let calls=0;const job=await acquireStateSourceJob(plan,{state:'VT',year:'2000'},files,null,null,null,{fetcher:async(url,options)=>{calls++;assert.equal(options.credentials,'omit');assert.equal(options.redirect,'error');return new Response(archives[plan.items.find(i=>i.url===url).kind]);},archiveReader:async(file,options)=>extractCensusZip(new Uint8Array(await file.arrayBuffer()),options)});assert.equal(calls,2);job.scope.districts=1;
let matches=0;
const result=await executeNativeStateJob({execute(r,o){const reply=engine.execute(r,o),file=root+'/native.json';writeFileSync(file,JSON.stringify(r));const native=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[file],{encoding:'utf8',windowsHide:true,maxBuffer:128*1024*1024}));compareNative(r,reply,native);matches++;return reply;}},job);
assert.equal(result.source_preparation.year,'2000');assert.equal(result.result.scope.districts,1);assert.equal(result.result.native_run.recipe.command.arguments_json,files.runCommandInputs.arguments_json);assert.equal(result.result.prepared.executed_request.options.balance_tolerance,0.5);
if(original){assert.equal(result.source_preparation.population,'608827');assert(result.source_preparation.units>100);const verification=JSON.parse(await fs.readFile('target/census-2000-sources/verified.json','utf8'));for(const receipt of job.acquisition.receipts){const expected=verification.original_archives.find(r=>r.kind===receipt.kind);assert.equal(receipt.source_sha256,expected.sha256);assert.equal(createHash('sha256').update(archives[receipt.kind]).digest('hex'),expected.sha256);}}
await fs.writeFile(root+(original?'/original-result.json':'/synthetic-result.json'),JSON.stringify(result));await fs.writeFile(root+(original?'/original-evidence.json':'/synthetic-evidence.json'),JSON.stringify({strict_comparison_pairs:strictComparisonPairs,wasm:engine.artifactIdentity,original_sources:original,plan_states:50,pinned_endpoints:100,whole_native_wasm_replies:matches,population:result.source_preparation.population,tracts:result.source_preparation.units,zip_hashes_bound:true,strict_native_balance:0.5,published:false},null,2));console.log(`PASS Census 2000 acquisition: 50 state pairs, 100 pinned endpoints, ${matches} whole native/WASM replies, ${original?'original':'synthetic'} VT population ${result.source_preparation.population}`);
