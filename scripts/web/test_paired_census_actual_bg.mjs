import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {readStateCensusArchives} from '../../web/lab/state-source-files.js';
import {extractCensusZip} from '../../web/lab/census-archive.js';
import {censusPreparationRequest,preparedTractRunSources} from '../../web/lab/tract-input-files.js';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createWorkbenchArchive,parseWorkbenchArchive} from '../../web/lab/workbench-archive.js';
const root='target/paired-census-resolution/actual-ri-bg';await fs.mkdir(root,{recursive:true});
const wasm=await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'),engine=await instantiateEngine(wasm),hash=x=>createHash('sha256').update(x).digest('hex');
const selected=await readStateCensusArchives(new File([await fs.readFile(root+'/tiger.zip')],'ri-bg.zip'),new File([await fs.readFile('target/census-zip-acquisition/ri-pl94.zip')],'ri-pl94.zip'),{state:'RI',year:2020,kind:'block-groups',archiveReader:async(f,s,{signal})=>{signal.throwIfAborted();return extractCensusZip(new Uint8Array(await f.arrayBuffer()),s);}});
const request=censusPreparationRequest(selected.files),prepared=engine.execute(request),graph=JSON.parse(prepared.graph_json);
assert.equal(prepared.population,'1097379');assert.equal(graph.unit_kind,'block-group');
const prefix='target/authentic-bg/RI/tiger/tl_2020_44_bg',geo='data/2020/redistricting/rhode_island/rigeo2020.pl',pop='data/2020/redistricting/rhode_island/ri000012020.pl';
const original=JSON.parse(await fs.readFile('target/real-census-preparation/RI/block-group/prepared.json','utf8'));
for(const part of ['shp','dbf','shx','prj']){const bytes=await fs.readFile(prefix+'.'+part),source=part==='prj'?original.sources.prj:original.sources[part+'_decoded'];assert.equal(hash(bytes),source[part==='prj'?'sha256':'decoded_sha256']);}
const native=JSON.parse(execFileSync('target/debug/examples/census_prepare_reference.exe',['RI','2020','block-group',prefix+'.shp',geo,pop],{encoding:'utf8',maxBuffer:128*1024*1024}));
let maxDelta=0;function numeric(a,b){if(Array.isArray(a)){assert.equal(a.length,b.length);a.forEach((v,i)=>numeric(v,b[i]));}else{const d=Math.abs(a-b);assert(Number.isFinite(d)&&d<=1e-5);maxDelta=Math.max(maxDelta,d);}}
for(const key of ['schema_version','unit_kind','state','year','geoids','population','adjacency','areas'])assert.deepEqual(graph[key],native.graph[key]);
assert.deepEqual(graph.edges.map(e=>e.slice(0,2)),native.graph.edges.map(e=>e.slice(0,2)));numeric(graph.edges.map(e=>e[2]),native.graph.edges.map(e=>e[2]));numeric(graph.centroids,native.graph.centroids);numeric(graph.exterior_perimeters,native.graph.exterior_perimeters);assert.deepEqual(JSON.parse(prepared.geometry_json),native.geometry);
const handoff=preparedTractRunSources(prepared),archive=createWorkbenchArchive({name:'Original RI paired block-group ZIPs',files:selected.files,operation:selected.operation,constraints:[],result:prepared,lastOperation:selected.operation}),opened=parseWorkbenchArchive(archive);assert.deepEqual(opened.files,selected.files);assert.deepEqual(opened.result,prepared);assert.equal(Buffer.from(handoff.graph.base64,'base64').toString(),prepared.graph_json);
await fs.writeFile(root+'/paired.bisect-workbench',archive);await fs.writeFile(root+'/evidence.json',JSON.stringify({units:prepared.units,population:prepared.population,original_tiger_source_hashes_rechecked:true,native_filesystem_discrete_values_equal:true,maximum_projected_float_delta:maxDelta,source_preserving_archive_roundtrip:true,configuration_handoff:true,wasm_sha256:hash(wasm),scope:'Authentic local Rhode Island 2020 block-group TIGER and PL94 ZIP extraction/preparation; native filesystem comparison with projected float tolerance 1e-5. This gate prepares sources; no provider acquisition, full plan run, national fine-data capacity or certification claim.'},null,2));
console.log(`Authentic RI paired BG ZIPs: ${prepared.units} units / ${prepared.population} people, original source hashes, Native filesystem preparation (float delta ${maxDelta}), source archive and graph/map handoff passed.`);
