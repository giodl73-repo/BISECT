import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {readNativeAdjacencyBundle} from '../../web/lab/native-adjacency-bundle.js';
import {nativeEngineInputRequest} from '../../web/lab/native-adjacency-files.js';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {nativeWorkspaceCase} from './fixtures/native-workspace-case.mjs';
const root=path.resolve(process.env.BISECT_BUNDLE_TEST_OUTPUT||'target/native-adjacency-bundle/unit-'+Date.now());await fs.mkdir(root,{recursive:true});
const fixture=nativeWorkspaceCase(),reads=[];
function file(name,data){const bytes=Buffer.from(data);return {name,size:bytes.length,async arrayBuffer(){reads.push(name);return bytes.buffer.slice(bytes.byteOffset,bytes.byteOffset+bytes.byteLength);}};}
const selected=[...fixture.files].filter(([n])=>n.startsWith(fixture.stem)).map(([n,b])=>file(path.basename(n),b));
const imported=await readNativeAdjacencyBundle(selected,{state:'RI',year:'2020'});
assert.equal(imported.receipt.files.length,3);assert.equal(imported.receipt.ignored.length,3);assert(!reads.some(n=>n.endsWith('.pkl')||n.endsWith('.request.json')||n.endsWith('_display.geojson')));
for(const row of imported.receipt.files){const expected=fixture.files.get(path.dirname(fixture.stem)+'/'+row.name);assert.equal(row.sha256,createHash('sha256').update(expected).digest('hex'));assert.deepEqual(Buffer.from(imported.sources[row.kind].base64,'base64'),expected);}
const withCustomCounty=await readNativeAdjacencyBundle([...selected,{name:'ri_adjacency_2020_county_ids.json',size:2,async arrayBuffer(){throw new Error('Custom Census county companion must remain unread');}}],{state:'RI',year:'2020'});assert.equal(withCustomCounty.sources.county_ids,undefined);assert.equal(withCustomCounty.receipt.ignored.length,4);
const project=createProject({name:'Grouped release inputs',files:{nativeAdjacencyInputs:imported.sources,nativeAdjacencySettings:imported.settings},operation:'prepare-native-engine-input',constraints:[],result:null,lastOperation:null});assert.deepEqual(parseProject(JSON.stringify(project)),project);await fs.writeFile(root+'/source.bisect',JSON.stringify(project));
let rejected=0;const binary=selected.find(f=>f.name.endsWith('.adj.bin'));
for(const [files,settings]of [
 [[...selected,file('ca_adjacency_2020.adj.bin','x')],{state:'RI',year:'2020'}],
 [[...selected,binary],{state:'RI',year:'2020'}],
 [[file('../ri_adjacency_2020.adj.bin','x')],{state:'RI',year:'2020'}],
 [[file('ri_adjacency_2010.adj.bin','x')],{state:'RI',year:'2020'}],
 [[{...binary,size:64*1024*1024+1}],{state:'RI',year:'2020'}],
 [[{...binary,size:64*1024*1024},file('ri_adjacency_2020_geoids.json','{}')],{state:'RI',year:'2020'}],
 [[file('ri_adjacency_2020.pkl','x')],{state:'RI',year:'2020'}],
 [[binary],{state:'RI',year:'2020',unit_kind:'block'}],
 [[binary],{state:'ZZ',year:'2020'}],
]){const count=reads.length;await assert.rejects(readNativeAdjacencyBundle(files,settings));assert.equal(reads.length,count);rejected++;}
await assert.rejects(readNativeAdjacencyBundle([{...binary,size:binary.size+1}],{state:'RI',year:'2020'}),/changed while reading/);
let stale=false;await assert.rejects(readNativeAdjacencyBundle([{...binary,async arrayBuffer(){stale=true;return binary.arrayBuffer();}}],{state:'RI',year:'2020'},{assertCurrent(){if(stale)throw new Error('source selection retired');}}),/retired/);
const abort=new AbortController();abort.abort(new Error('cancel import'));const count=reads.length;await assert.rejects(readNativeAdjacencyBundle(selected,{state:'RI',year:'2020'},{signal:abort.signal}),/cancel import/);assert.equal(reads.length,count);
const wasmPath=path.resolve(process.env.BISECT_TEST_WASM||'target/wasm32-unknown-unknown/release/bisect_wasm.wasm'),engine=await instantiateEngine(await fs.readFile(wasmPath)),request=nativeEngineInputRequest(imported.sources,imported.settings);await fs.writeFile(root+'/request.json',JSON.stringify(request));
const wasm=engine.execute(request),nativeExe=path.resolve(process.env.BISECT_TEST_NATIVE_TOOL||'target/release/examples/execute_tool_request.exe'),native=JSON.parse(execFileSync(nativeExe,[root+'/request.json'],{encoding:'utf8',windowsHide:true,maxBuffer:64*1024*1024}));assert.deepEqual(wasm,native);assert.equal(wasm.execution_performed,false);
await fs.writeFile(root+'/native.json',JSON.stringify(native));await fs.writeFile(root+'/wasm.json',JSON.stringify(wasm));await fs.writeFile(root+'/evidence.json',JSON.stringify({status:'passed',wasm:engine.artifactIdentity,synthetic_fixture:true,exact_native_host_preparation:true,project_roundtrip_exact:true,pickle_and_request_not_read:true,pre_read_refusals:rejected,changed_size_stale_cancel_refused:true,partition_execution_performed:false,import_receipt:imported.receipt},null,2)+'\n');
console.log('PASS grouped native sources: exact native-host/WASM preparation, original bytes, inert project and bounded mixed/stale/unsafe refusals. '+root);
