import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {gzipSync} from 'node:zlib';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {readNativeMapStateDirectory,readNativeMapTigerDirectory,nativeMapTigerRequest,nativeMapTigerHandoff} from '../../web/lab/native-map-folder.js';
import {nativeMapRequest,validateNativeMapInputs} from '../../web/lab/map-export-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const root=path.resolve('target/native-map-folder');await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm')),requests=[],outcomes=[];
function execute(request){requests.push(request);try{const result=engine.execute(request);outcomes.push({ok:true,result});return result;}catch(error){outcomes.push({ok:false,error:error.message});throw error;}}
const missing=()=>Object.assign(new Error('Missing selected entry'),{name:'NotFoundError'});
function directory(entries){return {kind:'directory',async getDirectoryHandle(name){const h=entries[name];if(!h)throw missing();return h;},async getFileHandle(name){const h=entries[name];if(!h)throw missing();return h;}};}
function file(name,bytes){return {kind:'file',async getFile(){return {name,size:bytes.length,lastModified:1,arrayBuffer:async()=>Uint8Array.from(bytes).buffer};}};}
const stem='tl_2020_44_tract',dir=path.resolve('data/2020/tiger/tracts',stem),entries={};
for(const part of ['shp','dbf','shx'])entries[stem+'.'+part]=file(stem+'.'+part,await fs.readFile(path.join(dir,stem+'.'+part)));
const tiger=await readNativeMapTigerDirectory(directory(entries),{state:'RI',year:2020}),request=nativeMapTigerRequest(tiger,{state:'RI',geometry_profile:'source-exact-large'}),prepared=execute(request);
assert.equal(prepared.units,250);assert.equal(prepared.execution_performed,false);assert.equal(prepared.graph_json,undefined);
assert.equal(prepared.shp.sha256,createHash('sha256').update(Buffer.from(tiger.shp.base64,'base64')).digest('hex'));
const prior=JSON.parse(await fs.readFile('target/sequential-dissolve-ri/raw-sequential.bisect','utf8')),geometry=JSON.parse(prepared.geometry_json),original=JSON.parse(Buffer.from(prior.files.nativeMapInputs.geometry.base64,'base64'));
const originals=new Map(original.features.map(f=>[f.properties.GEOID||f.properties.geoid,f.geometry]));
for(const f of geometry.features){const old=originals.get(f.properties.geoid);assert(old);const coordinates=g=>g.type==='Polygon'?[g.coordinates]:g.coordinates;assert.equal(JSON.stringify(coordinates(f.geometry)),JSON.stringify(coordinates(old)), 'Original coordinate order/values differ for '+f.properties.geoid);}
const raw=Buffer.from(prior.files.nativeMapInputs.assignments.base64,'base64'),stateRoot=directory({data:directory({'final_assignments.json':file('final_assignments.json',raw)})}),selected=await readNativeMapStateDirectory(stateRoot);
const inputs=nativeMapTigerHandoff(prepared,{...prior.files.nativeMapInputs,...selected}),rendered=execute(nativeMapRequest(validateNativeMapInputs(inputs)));
assert.equal(rendered.png_base64,prior.result.png_base64);assert.equal(rendered.svg_text,prior.result.svg_text);assert.equal(rendered.plan_hash,null);
const project=createProject({name:'Native folder RI',operation:'prepare-native-map-tiger',lastOperation:null,files:{tigerInputs:tiger,tractPreparationSettings:{state:'RI',geometry_profile:'source-exact-large',projection:null}},constraints:[],result:null});
assert.deepEqual(parseProject(JSON.stringify(project)),project);await fs.writeFile(path.join(root,'prepare.bisect'),JSON.stringify(project));await fs.writeFile(path.join(root,'expected.json'),JSON.stringify({prepared,rendered}));
for(const mutate of [r=>{r.input.state='WA';},r=>{r.input.tiger.year=2024;},r=>{r.input.tiger.kind='block-groups';},r=>{r.input.geometry_profile='display-simplified';},r=>{r.input.extra=true;},r=>{r.input.tiger.shp.base64=Buffer.from('bad header').toString('base64');},r=>{r.input.tiger.shx.base64=Buffer.from('bad index').toString('base64');}]){const bad=structuredClone(request);mutate(bad);assert.throws(()=>execute(bad));}
const compressed=structuredClone(request);for(const k of ['shp','dbf','shx'])compressed.input.tiger[k]={name:stem+'.'+k+'.gz',base64:gzipSync(Buffer.from(tiger[k].base64,'base64')).toString('base64')};assert.equal(execute(compressed).geometry_json,prepared.geometry_json);
const standard=structuredClone(request);standard.input.geometry_profile='source-exact';assert.deepEqual(JSON.parse(execute(standard).geometry_json).features,geometry.features);
const noIndex=structuredClone(request);noIndex.input.tiger.shx=null;assert.equal(execute(noIndex).geometry_json,prepared.geometry_json);
await assert.rejects(()=>readNativeMapStateDirectory(directory({})),/Missing/);
assert.equal((await readNativeMapStateDirectory(stateRoot,{map_type:'political'})).analysis,null);
const malformed=await readNativeMapStateDirectory(directory({data:directory({'final_assignments.json':file('final_assignments.json',Buffer.alloc(0))})}));assert.equal(malformed.assignments.base64,'');
await assert.rejects(()=>readNativeMapStateDirectory(stateRoot,{map_type:'rounds'}),/separate/);
await assert.rejects(()=>readNativeMapStateDirectory(stateRoot,{isCurrent:()=>false}),/changed/);
await assert.rejects(()=>readNativeMapStateDirectory(stateRoot,{signal:AbortSignal.abort()}));
let bodyRead=false;await assert.rejects(()=>readNativeMapStateDirectory(directory({data:directory({'final_assignments.json':{kind:'file',async getFile(){return {name:'big',size:3*1024*1024+1,arrayBuffer(){bodyRead=true;}};}}})})),/limit/);assert.equal(bodyRead,false);
let reads=0;await assert.rejects(()=>readNativeMapStateDirectory(directory({data:directory({'final_assignments.json':{kind:'file',async getFile(){return {name:'changing',size:1,lastModified:++reads,arrayBuffer:async()=>new Uint8Array([1]).buffer};}}})})),/changed/);
await assert.rejects(()=>readNativeMapTigerDirectory(directory(entries),{state:'WA',year:2020}),/Missing/);
assert.throws(()=>nativeMapTigerRequest(tiger,{state:'RI',geometry_profile:'display-simplified'}),/exact/);
const requestFile=path.join(root,'requests.jsonl');await fs.writeFile(requestFile,requests.map(r=>JSON.stringify(r)).join('\n')+'\n');
const native=spawnSync(path.resolve('target/debug/examples/execute_tool_sequence.exe'),[requestFile],{encoding:'utf8',maxBuffer:128*1024*1024});assert.equal(native.status,0,native.stderr);assert.deepEqual(native.stdout.trim().split('\n').map(JSON.parse),outcomes);
await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({wasm_sha256:engine.artifactIdentity.sha256,native_protocol_matches:outcomes.length,original_ri_tracts:250,exact_original_coordinates:true,exact_previous_native_cli_png:true,population_graph_and_rplan_not_created:true,gzip_and_optional_index_equivalence:true,read_only_selected_roots:true,missing_malformed_cancel_stale_size_and_mutation_guards:true,inert_source_project:true},null,2));
console.log('Native map folders: '+outcomes.length+' native/WASM replies; original RI coordinates and native PNG, gzip/index equivalence, inert projects and read-only source guards passed.');
