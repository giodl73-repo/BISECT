import fs from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {loadMetisSidecar} from '../../web/lab/metis-c-sidecar.js';
import {compareCommandReplies} from './native_command_comparison.mjs';
import {nativeBinaryWidth} from './fixtures/native-recorded-replay.mjs';
import {loadNativeUnitIdentityCorpus} from './fixtures/native-unit-identity.mjs';
const args=process.argv.slice(2),arg=n=>{const i=args.indexOf(n);return i<0?null:args[i+1];};
const wasm=path.resolve(arg('--wasm')),host=path.resolve(arg('--host')),sequence=path.resolve(arg('--host-sequence'));
const cbase=arg('--c-sidecar')&&path.resolve(arg('--c-sidecar')),mode=cbase?'c':'core';
const parent=path.resolve(arg('--output')||'target/native-unit-identity');await fs.mkdir(parent,{recursive:true});
const root=await fs.mkdtemp(path.join(parent,mode+'-')),sha=b=>createHash('sha256').update(b).digest('hex');
const bytes=await fs.readFile(wasm),hostBytes=await fs.readFile(host),width=nativeBinaryWidth(hostBytes);
let metisSidecar=null;
if(cbase){const manifest=JSON.parse(await fs.readFile(path.join(cbase,'metis-sidecar.json'),'utf8'));metisSidecar=await loadMetisSidecar({manifest,javascript:await fs.readFile(path.join(cbase,manifest.javascript.file)),wasm:await fs.readFile(path.join(cbase,manifest.wasm.file))});}
const engine=await instantiateEngine(bytes,{metisSidecar}),fixtures=await loadNativeUnitIdentityCorpus();
const evidence={status:'running',mode,wasm_sha256:sha(bytes),host_sha256:sha(hostBytes),cases:[],refusals:[],scope:'Twelve complete frozen actual desktop source/context/plan references, including original three Imported cases, native block/BG output fallback, absent/sparse original index, COI non-aliasing and foreign-prefix metadata claims. Both profiles execute metis-core; no C execution, source authentication, real-state/capacity or full release claim.'};
const save=()=>fs.writeFile(path.join(root,'evidence.json'),JSON.stringify(evidence,null,2));await save();
const replies=new Map();
try{
 for(const fixture of fixtures){
  const file=path.join(root,fixture.name+'.request.json');await fs.writeFile(file,JSON.stringify(fixture.request));
  const native=spawnSync(host,[file],{encoding:'utf8',windowsHide:true,maxBuffer:32*1024*1024});
  await fs.writeFile(path.join(root,fixture.name+'.host.stdout'),native.stdout);await fs.writeFile(path.join(root,fixture.name+'.host.stderr'),native.stderr);assert.equal(native.status,0,native.stderr);
  const reference=JSON.parse(native.stdout),actual=engine.execute(fixture.request);await fs.writeFile(path.join(root,fixture.name+'.wasm.json'),JSON.stringify(actual));
  assert.equal(actual.status,'match',actual.error);assert.equal(actual.replay.export_error,null);
  compareCommandReplies({operation:'prepare-native-state-graph-run'},actual.prepared,reference.prepared,width);
  const expected=structuredClone(reference);expected.prepared.native_command.target_pointer_width=32;assert.deepEqual(actual,expected);
  const exported=actual.replay.export,plan=JSON.parse(fixture.document_json).plan;
  assert.equal(exported.context_json,fixture.context_json);assert.deepEqual(exported.document.plan,plan);assert.deepEqual(exported.context,JSON.parse(fixture.context_json));
  assert.equal(exported.original_context_source_sha256,sha(Buffer.from(fixture.context_json)));
  assert.equal(actual.recorded_paths_opened,false);assert.equal(actual.replay.executed_request.graph.native_source_identity,true);
  const graph=actual.replay.executed_request.graph;assert.deepEqual(graph.geoids,plan.units.unit_ids);
  const sparse=fixture.source_index===null||Object.keys(fixture.source_index).length!==4;
  if(sparse)assert.deepEqual(graph.native_geoid_index,fixture.source_index||{});else assert.equal(graph.native_geoid_index,undefined);
  if(fixture.name==='absent-index-coi')assert.equal(actual.replay.result.metrics.native_coi_source_evidence.changed_edges,0);
  if(fixture.name==='sparse-index-coi')assert.equal(actual.replay.result.metrics.native_coi_source_evidence.changed_edges,1);
  replies.set(fixture.name,actual);
  evidence.cases.push({name:fixture.name,status:'passed',whole_host_reply_with_measured_width_only:true,literal_ids_original_context_and_plan_body:true,source_index_preserved:sparse?'explicit-present-map':'complete-implicit-map'});await save();
 }
 const base=replies.get('absent-index').replay.executed_request,controls=[];
 for(const name of ['wide-index','noncanonical-index','changed-index-id','missing-policy','county-override','invalid-kind']){
  const request=structuredClone(base);
  if(name==='wide-index')request.graph.native_geoid_index={'18446744073709551615':'outside'};
  if(name==='noncanonical-index')request.graph.native_geoid_index={'01':'outside'};
  if(name==='changed-index-id')request.graph.native_geoid_index={'0':'changed-id'};
  if(name==='missing-policy')request.graph.native_source_identity=false;
  if(name==='county-override')request.graph.county_ids=['invented','invented','invented','invented'];
  if(name==='invalid-kind')request.graph.unit_kind='tract';
  controls.push({name,input:{operation:'run-engine-build',request}});
 }
 const legacy=structuredClone(replies.get('foreign-prefix-tract').replay.executed_request);legacy.graph.native_source_identity=false;
 controls.push({name:'legacy-census-stays-strict',input:{operation:'run-engine-build',request:legacy}});
 const input=path.join(root,'refusals.jsonl');await fs.writeFile(input,controls.map(c=>JSON.stringify(c.input)).join('\n')+'\n');
 const native=spawnSync(sequence,[input],{encoding:'utf8',windowsHide:true,maxBuffer:16*1024*1024});assert.equal(native.status,0,native.stderr);
 await fs.writeFile(path.join(root,'refusals.host.stdout'),native.stdout);await fs.writeFile(path.join(root,'refusals.host.stderr'),native.stderr);
 const expected=native.stdout.trim().split(/\r?\n/).map(line=>JSON.parse(line));assert.equal(expected.length,7);
 const actual=controls.map(c=>{try{return{ok:true,result:engine.execute(c.input)}}catch(error){return{ok:false,error:error.message}}});
 await fs.writeFile(path.join(root,'refusals.wasm.json'),JSON.stringify(actual));assert.deepEqual(actual,expected);assert(actual.every(reply=>reply.ok===false));
 evidence.refusals=controls.map((c,i)=>({name:c.name,status:'refused-exact-host-match',error:actual[i].error}));
 assert.equal(evidence.cases.length,12);evidence.status='passed-maintained-native-unit-identity';await save();console.log(JSON.stringify({root,status:evidence.status,cases:12,refusals:7}));
}catch(error){evidence.status='failed-maintained-native-unit-identity';evidence.error={message:error.message,stack:error.stack};await save();throw error;}
