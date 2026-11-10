import fs from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {loadMetisSidecar} from '../../web/lab/metis-c-sidecar.js';
import {nativeMapTigerHandoff,readNativeMapTigerDirectory,nativeMapTigerRequest} from '../../web/lab/native-map-folder.js';
import {addNativeMapToNationalBatch,nationalMapBatchInput,executeNationalMapBatch} from '../../web/lab/national-map-batch.js';
import {nativeMapRequest} from '../../web/lab/map-export-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';

import {loadFineUnitMapCorpus}from'./fixtures/fine-unit-map.mjs';
const args=process.argv.slice(2),option=n=>{const i=args.indexOf(n);return i<0?null:args[i+1];};
const wasm=path.resolve(option('--wasm')),host=path.resolve(option('--host-sequence')),cbase=option('--c-sidecar')&&path.resolve(option('--c-sidecar')),mode=cbase?'c':'core';
const parent=path.resolve(option('--output')||'target/fine-unit-map-regression');await fs.mkdir(parent,{recursive:true});const root=await fs.mkdtemp(path.join(parent,mode+'-'));
const sha=b=>createHash('sha256').update(b).digest('hex'),corpus=await loadFineUnitMapCorpus();let metisSidecar=null;
if(cbase){const manifest=JSON.parse(await fs.readFile(path.join(cbase,'metis-sidecar.json'),'utf8'));metisSidecar=await loadMetisSidecar({manifest,javascript:await fs.readFile(path.join(cbase,manifest.javascript.file)),wasm:await fs.readFile(path.join(cbase,manifest.wasm.file))});}
const engine=await instantiateEngine(await fs.readFile(wasm),{metisSidecar});
const requests=[],replies=[],evidence={status:'running',mode,engine:engine.runtimeArtifacts,cases:[],failures:[],scope:'Synthetic selected TIGER map inputs over retained native assignment references, actual corrected CLI PNG comparison, exact native-host/WASM replies, source-preserving project handoff and native national rendering. No new partition, original producer geometry, Census authority, all-state capacity or full fine-export/workflow acceptance.'};
const save=()=>fs.writeFile(path.join(root,'evidence.json'),JSON.stringify(evidence,null,2));await save();
function invoke(request){
 requests.push(structuredClone(request));let result;
 try{result={ok:true,result:engine.execute(request)};}catch(error){result={ok:false,error:error.message};}
 replies.push(structuredClone(result));return result;
}
const recorded={execute(request){const reply=invoke(request);if(!reply.ok)throw new Error(reply.error);return reply.result;}};
const fixtures=corpus.cases;
function settings(request){const r=request.input;return {plan:null,geometry:null,geometry_source:'tiger',tiger:structuredClone(r.tiger),source_mode:'native-assignments',raw_state:r.state,raw_year:r.tiger.year,assignments:{name:'final_assignments.json',base64:r.assignments_base64},assignment_key_basis:r.key_basis,assignment_index:r.unit_index_base64?{name:'geoids.json',base64:r.unit_index_base64}:null,analysis:r.analysis_base64?{name:r.map_type+'.json',base64:r.analysis_base64}:null,map_type:r.map_type,dpi:r.dpi,dissolve_method:'sequential'};}
try{
 // Directory-handle controls exercise selection only; no OS chooser or engine mock.
 for(const kind of ['tracts','block-groups']){
  const selected=fixtures.find(f=>f.kind===kind).request.input.tiger,reads=[];
  const directory={kind:'directory',async getFileHandle(name){reads.push(name);const part=Object.values(selected).find(v=>v?.name===name);assert(part,'Unexpected lookup: '+name);const bytes=Buffer.from(part.base64,'base64');return {kind:'file',async getFile(){return {name,size:bytes.length,lastModified:1,async arrayBuffer(){return bytes;}};}};}};
  assert.deepEqual(await readNativeMapTigerDirectory(directory,{state:'AL',year:2020,kind}),selected);assert.equal(reads.length,3);
  const noShx={...directory,async getFileHandle(name){if(name.endsWith('.shx')){const e=Error('Missing optional SHX');e.name='NotFoundError';throw e;}return directory.getFileHandle(name);}};
  assert.equal((await readNativeMapTigerDirectory(noShx,{state:'AL',year:2020,kind})).shx,null);
  assert.deepEqual(nativeMapTigerRequest(selected,{state:'AL',geometry_profile:'source-exact'}),{operation:'prepare-native-map-tiger',input:{state:'AL',tiger:selected,geometry_profile:'source-exact'}});
  const before=reads.length;await assert.rejects(readNativeMapTigerDirectory(directory,{state:'AL',year:2020,kind,isCurrent:()=>false}),/changed/);assert.equal(reads.length,before);
  evidence.cases.push({name:kind+'-explicit-folder-selection',sources_exact:true,optional_shx:true,stale_read_refused:true});
 }
 await assert.rejects(readNativeMapTigerDirectory({kind:'directory',getFileHandle(){throw Error('Should not read');}},{state:'AL',year:2020,kind:'counties'}),/tract or block-group/);evidence.cases.push({name:'unsupported-folder-kind',refused:true});
 for(const fixture of fixtures){
  const request=structuredClone(fixture.request),reply=invoke(request);
  await fs.writeFile(path.join(root,fixture.name+'-reply.json'),JSON.stringify(reply));assert(reply.ok,reply.error);
  const r=reply.result;assert.equal(r.execution_performed,false);assert.equal(r.plan_hash,null);assert.equal(r.png_sha256,fixture.native_png_sha256);assert.equal(sha(Buffer.from(r.png_base64,'base64')),fixture.native_png_sha256);assert.equal(sha(Buffer.from(r.svg_text)),r.svg_sha256);
  const expectedUnits=fixture.kind==='block-groups'?32:16;assert.equal(r.units,expectedUnits);if(fixture.kind==='block-groups')assert.equal(r.unit_kind,'block-group');
  if(fixture.kind==='tracts')assert.deepEqual(r,corpus.tract_replies[fixture.name]);
  const project=createProject({name:fixture.name,operation:'render-native-district-map',files:{nativeMapInputs:settings(request)},constraints:[],result:r,lastOperation:'render-native-district-map'});assert.deepEqual(parseProject(JSON.stringify(project)),project);
  evidence.cases.push({name:fixture.name,native_png_exact:true,units:r.units,matched:r.matched_assignment_units,tract_previous_reply_exact:fixture.kind==='tracts'});
 }
 const base=structuredClone(fixtures.find(f=>f.name==='bg-full-districts').request);
 const prepared=[];
 for(const kind of ['tracts','block-groups'])for(const geometry_profile of ['source-exact','source-exact-large']){
  const input=structuredClone(fixtures.find(f=>f.kind===kind&&f.map_type==='districts').request.input);
  const request={operation:'prepare-native-map-tiger',input:{state:'AL',tiger:input.tiger,geometry_profile}},reply=invoke(request);assert(reply.ok,reply.error);const result=reply.result,geometry=JSON.parse(result.geometry_json);assert.equal(result.execution_performed,false);assert.equal(result.units,kind==='block-groups'?32:16);assert.equal(geometry.features.length,result.units);
  if(kind==='tracts')assert.deepEqual(result,corpus.tract_replies[geometry_profile]);else{
   assert.equal(result.unit_kind,'block-group');const next=nativeMapTigerHandoff(result,settings(base),base.input.tiger);assert.deepEqual(next.tiger,base.input.tiger);assert.equal(next.geometry_source,'tiger');assert.equal(next.geometry,null);assert.deepEqual(nativeMapRequest(next),base);
   const map={operation:'render-native-district-map',input:{document_json:corpus.valid.document_json,unit_geometry_json:result.geometry_json,dpi:72,map_type:'districts',dissolve_method:'sequential'}},mapped=invoke(map);assert.equal(mapped.ok,false,'Original overlapping source must retain strict native refusal');assert.match(mapped.error,/overlap/);
   prepared.push({request,result,map,rplan_refusal:mapped.error});
  }
  evidence.cases.push({name:kind+'-'+geometry_profile,exact_coordinates:true,source_units:result.units});
 }
 const valid=corpus.valid;
 const prepareValid={operation:'prepare-native-map-tiger',input:{state:'AL',tiger:valid.tiger,geometry_profile:'source-exact'}},validReply=invoke(prepareValid);assert(validReply.ok,validReply.error);assert.equal(validReply.result.units,32);const exact=JSON.parse(validReply.result.geometry_json);assert(exact.features.some(f=>f.geometry.coordinates.length===2));assert(exact.features.some(f=>f.geometry.coordinates.some(poly=>poly.length===2)));
 const validMap={operation:'render-native-district-map',input:{document_json:valid.document_json,unit_geometry_json:validReply.result.geometry_json,dpi:72,map_type:'districts',dissolve_method:'sequential'}},validMapped=invoke(validMap);assert(validMapped.ok,validMapped.error);assert.equal(validMapped.result.png_sha256,valid.native_png_sha256);
 const validRaw=structuredClone(base);validRaw.input.tiger=valid.tiger;const raw=invoke(validRaw);assert(raw.ok,raw.error);assert.equal(raw.result.png_sha256,valid.native_png_sha256);
 prepared.push({request:prepareValid,result:validReply.result,map:validMap,result_map:validMapped.result,valid_topology:true});evidence.cases.push({name:'separate-valid-topology-rplan-and-raw',units:32,holes_and_islands:true,unchanged_native_strict_and_cli_png_exact:true,frozen_tract_reference_unchanged:true});
 const mutations=[
  ['fine-kind-coarse-parts',r=>{throw new Error('Coarse source mutation must be initialized before execution.');},null],
  ['wrong-state',r=>{r.input.state='RI';},null],
  ['unsupported-year',r=>{r.input.tiger.year=2021;},null],
  ['unsupported-kind',r=>{r.input.tiger.kind='counties';},null],
  ['missing-index',r=>{r.input.key_basis='native-index-map';delete r.input.unit_index_base64;},null],
  ['implicit-index',r=>{r.input.key_basis='infer';},null],
  ['bad-shp-header',r=>{const b=Buffer.from(r.input.tiger.shp.base64,'base64');b[24]^=1;r.input.tiger.shp.base64=b.toString('base64');},null],
  ['bad-shx-offset',r=>{const b=Buffer.from(r.input.tiger.shx.base64,'base64');b[103]^=1;r.input.tiger.shx.base64=b.toString('base64');},null],
 ];
 const coarse=structuredClone(fixtures.find(f=>f.name==='tract-full-districts').request);
 mutations[0][1]=r=>{r.input.tiger=structuredClone(coarse.input.tiger);r.input.tiger.kind='block-groups';};
 mutations.push(['coarse-kind-fine-parts',r=>{r.input.tiger.kind='tracts';},null]);
 for(const[name,mutate]of mutations){const request=structuredClone(base);mutate(request);const reply=invoke(request);assert.equal(reply.ok,false,name);await fs.writeFile(path.join(root,name+'-reply.json'),JSON.stringify(reply));evidence.cases.push({name,refused:true,error:reply.error});}
 const recovery=invoke(base);assert(recovery.ok,recovery.error);assert.equal(recovery.result.png_sha256,fixtures.find(f=>f.name==='bg-full-districts').native_png_sha256);evidence.cases.push({name:'success-after-refusals',native_png_exact:true});
 for(const fixture of corpus.national){
  const request=structuredClone(fixtures.find(f=>f.name==='bg-full-'+fixture.map_type).request);
  const selected=await addNativeMapToNationalBatch(null,settings(request));const batch=nationalMapBatchInput(selected.source),result=await executeNationalMapBatch(recorded,batch);assert.equal(result.png_sha256,fixture.native_png_sha256);assert.equal(sha(Buffer.from(result.png_base64,'base64')),fixture.native_png_sha256);await fs.writeFile(path.join(root,'national-'+fixture.map_type+'-reply.json'),JSON.stringify(result));evidence.cases.push({name:'national-'+fixture.map_type,actual_native_png_exact:true});
 }
 await fs.writeFile(path.join(root,'prepared-bg.json'),JSON.stringify(prepared));
 const sequence=path.join(root,'requests.jsonl');await fs.writeFile(sequence,requests.map(r=>JSON.stringify(r)).join('\n')+'\n');const native=spawnSync(host,[sequence],{encoding:'utf8',windowsHide:true,maxBuffer:128*1024*1024});await fs.writeFile(path.join(root,'host.stdout'),native.stdout);await fs.writeFile(path.join(root,'host.stderr'),native.stderr);assert.equal(native.status,0,native.stderr);const expected=native.stdout.trim().split('\n').map(JSON.parse);assert.deepEqual(replies,expected);evidence.whole_native_host_replies=expected.length;evidence.status='passed-maintained-fine-map-runtime';
}catch(error){evidence.status='failed-maintained-fine-map-runtime';evidence.failures.push({message:error.message,stack:error.stack});process.exitCode=1;}
await save();console.log(JSON.stringify({root,status:evidence.status,cases:evidence.cases.length,whole_host_replies:evidence.whole_native_host_replies,failures:evidence.failures}));
