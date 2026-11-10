// Synthetic jurisdiction labels on retained RI topology; no foreign source authenticity claim.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {nativeEngineInputRequest} from '../../web/lab/native-adjacency-files.js';
import {configRunRequest,configuredRunExportRequest} from '../../web/lab/config-run.js';
import {engineReplayFromConfigured,engineReplayRequest} from '../../web/lab/engine-replay-files.js';
import {contextEngineRequest} from '../../web/lab/context-engine-files.js';
import {configuredSourceBundle} from '../../web/lab/configured-source-bundle.js';
import {importConfiguredSourceBundle} from '../../web/lab/configured-source-import.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const root='target/custom-jurisdiction-engine';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const old=await instantiateEngine(await fs.readFile(root+'/before/engine.wasm'));
const source=(name,text)=>({name,base64:Buffer.from(text).toString('base64')});let matches=0,refusals=0,unchanged=0;
async function native(request,name){const result=engine.execute(request),file=root+'/'+name+'.request.json';await fs.writeFile(file,JSON.stringify(request));const proc=spawnSync(path.resolve('target/release/examples/execute_tool_request.exe'),[file],{encoding:'utf8',windowsHide:true,timeout:180000,maxBuffer:64*1024*1024});assert.equal(proc.status,0,proc.stderr);assert.deepEqual(result,JSON.parse(proc.stdout));await fs.writeFile(root+'/'+name+'.json',JSON.stringify(result));matches++;return result;}
async function refuse(request,name,pattern){assert.throws(()=>engine.execute(request),pattern);const file=root+'/'+name+'.request.json';await fs.writeFile(file,JSON.stringify(request));const proc=spawnSync(path.resolve('target/release/examples/execute_tool_request.exe'),[file],{encoding:'utf8',windowsHide:true,timeout:180000,maxBuffer:1024*1024});assert.notEqual(proc.status,0);if(pattern)assert.match(proc.stderr,pattern);refusals++;}
for(const kind of ['precinct','imported']){
 const original=JSON.parse(await fs.readFile('target/native-custom-units/'+kind+'-2024.bisect','utf8'));
 const before=nativeEngineInputRequest(original.files.nativeAdjacencyInputs,original.files.nativeAdjacencySettings);
 assert.deepEqual(engine.execute(before),old.execute(before));unchanged++;
 for(const state of ['GB','ZZ']){
  const files=structuredClone(original.files);files.nativeAdjacencySettings.state=state;
  const companion=JSON.parse(Buffer.from(files.nativeAdjacencyInputs.county_ids.base64,'base64'));companion.state=state;
  files.nativeAdjacencyInputs.county_ids=source(state+'-counties.json',JSON.stringify(companion));
  const request=nativeEngineInputRequest(files.nativeAdjacencyInputs,files.nativeAdjacencySettings);
  assert.throws(()=>old.execute(request),/known state/);
  const prepared=await native(request,state+'-'+kind+'-prepared'),graph=JSON.parse(prepared.graph_json);
  assert.equal(graph.state,state);assert.equal(prepared.suggested_districts,null);assert.deepEqual(graph.county_ids,companion.county_ids);
  files.configRunInputs={graph:source(state+'-graph.json',prepared.graph_json)};
  const configured=await native(configRunRequest(files.configSource,files.configSettings,files.configRunInputs,files.configRunSettings),state+'-'+kind+'-configured');
  const baseline=old.execute(configRunRequest(original.files.configSource,original.files.configSettings,original.files.configRunInputs,original.files.configRunSettings));assert.deepEqual(configured.result,{...baseline.result,state},'Only the recorded jurisdiction changes for the same explicit graph/options');
  configured.practitioner_export=await native(configuredRunExportRequest(configured,{label:'custom_jurisdiction',created_at:'2026-10-18T00:00:00Z'}),state+'-'+kind+'-export');
  assert.equal(configured.practitioner_export.context.units.state,state);assert.equal(configured.practitioner_export.context.units.unit_kind,kind);
  assert.deepEqual(configured.practitioner_export.context.subdivisions.county_ids,companion.county_ids);
  assert.equal((await native(engineReplayRequest(engineReplayFromConfigured(configured)),state+'-'+kind+'-replay')).status,'match');
  const bundle=await configuredSourceBundle(configured,files);await fs.writeFile(root+'/'+state+'-'+kind+'-sources.zip',bundle.bytes);const imported=await importConfiguredSourceBundle(bundle.bytes);assert.equal(JSON.parse(Buffer.from(imported.files.configRunInputs.graph.base64,'base64')).state,state);
  const contextInputs={source:source(state+'.rctx',configured.practitioner_export.context_json),physical_edge_weights_metres:true};
  const fromContext=await native(contextEngineRequest(contextInputs),state+'-'+kind+'-context');assert.equal(JSON.parse(fromContext.graph_json).state,state);assert.deepEqual(JSON.parse(fromContext.graph_json).county_ids,companion.county_ids);
  const packet=createProject({name:'SYNTHETIC '+state+' '+kind,operation:'prepare-native-engine-input',lastOperation:'prepare-native-engine-input',files,constraints:[],result:prepared});assert.deepEqual(parseProject(JSON.stringify(packet)),packet);await fs.writeFile(root+'/'+state+'-'+kind+'.bisect',JSON.stringify(packet));await fs.writeFile(root+'/'+state+'-'+kind+'-configured-result.json',JSON.stringify(configured));
  const mismatched=structuredClone(request);mismatched.input.state='CA';await refuse(mismatched,state+'-'+kind+'-foreign-companion');
  const census=structuredClone(request);census.input.unit_kind='tract';delete census.input.sources.county_ids;await refuse(census,state+'-'+kind+'-not-census');
  for(const bad of ['gb','GBR','G1','']){const r=configRunRequest(files.configSource,files.configSettings,files.configRunInputs,files.configRunSettings);r.input.graph.state=bad;await refuse(r,state+'-'+kind+'-invalid-'+(bad||'empty'),/jurisdiction code/);}
 }
}
await fs.writeFile(root+'/evidence.json',JSON.stringify({wasm:engine.artifactIdentity,before_wasm_sha256:old.artifactIdentity.sha256,native_reference_sha256:createHash('sha256').update(await fs.readFile('target/release/examples/execute_tool_request.exe')).digest('hex'),native_whole_replies:matches,previous_US_preparations_unchanged:unchanged,refusals,original_US_topology_with_synthetic_jurisdiction:true,source_bundle_roundtrips:4,context_repreparations:4,export_and_replay:true,published:false},null,2)+'\n');
console.log('PASS custom jurisdiction: '+matches+' complete native/WASM replies, '+unchanged+' unchanged US preparations, four source ZIP/context roundtrips, '+refusals+' refusals');
