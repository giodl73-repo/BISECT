import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {readStateCensusArchives} from '../../web/lab/state-source-files.js';
import {extractCensusZip} from '../../web/lab/census-archive.js';
import {censusPreparationRequest,tractPreparationRequest,preparedTractRunSources} from '../../web/lab/tract-input-files.js';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {createWorkbenchArchive,parseWorkbenchArchive} from '../../web/lab/workbench-archive.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm')),root='target/paired-census-resolution';
const archiveReader=async(file,settings,{signal})=>{signal.throwIfAborted();return extractCensusZip(new Uint8Array(await file.arrayBuffer()),settings);};
let cases=0,maxDelta=0;const rows=[];
function numeric(a,b){if(Array.isArray(a)){assert.equal(a.length,b.length);a.forEach((v,i)=>numeric(v,b[i]));}else{const delta=Math.abs(a-b);assert(Number.isFinite(delta)&&delta<=1e-5);maxDelta=Math.max(maxDelta,delta);}}
for(const year of [2000,2010,2020])for(const [kind,unit] of [['counties','county'],['tracts','tract'],['block-groups','block-group'],['blocks','block']]){
 const dir=root+'/'+year+'-'+unit,tiger=new File([await fs.readFile(dir+'/tiger.zip')],'tiger.zip'),pl94=new File([await fs.readFile(dir+'/pl94.zip')],'pl94.zip');
 const selected=await readStateCensusArchives(tiger,pl94,{state:'RI',year,kind,archiveReader});
 const request=(kind==='tracts'?tractPreparationRequest:censusPreparationRequest)(selected.files);assert.equal(selected.operation,request.operation);assert.equal(selected.files.tigerInputs.kind,kind);
 const result=engine.execute(request),expected=JSON.parse(await fs.readFile(dir+'/expected.json','utf8'));
 const expectedGraph=JSON.parse(expected.graph_json);if(kind==='tracts')delete expectedGraph.unit_kind;assert.deepEqual(JSON.parse(result.graph_json),expectedGraph);assert.equal(result.geometry_json,expected.geometry_json);assert.equal(result.population,'18');
 await fs.writeFile(dir+'/request.json',JSON.stringify(request));const native=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[dir+'/request.json'],{encoding:'utf8',maxBuffer:32*1024*1024}));
 const a=JSON.parse(result.graph_json),b=JSON.parse(native.graph_json);for(const key of ['edges','centroids','exterior_perimeters']){numeric(a[key],b[key]);delete a[key];delete b[key];}assert.deepEqual(a,b);
 const strip=x=>{const {graph_json,graph_sha256,bridges,...rest}=x;return rest;};assert.deepEqual(strip(result),strip(native));assert.deepEqual(result.bridges.map(({cost,...rest})=>rest),native.bridges.map(({cost,...rest})=>rest));numeric(result.bridges.map(x=>x.cost),native.bridges.map(x=>x.cost));
 const sources=preparedTractRunSources(result),graph=JSON.parse(Buffer.from(sources.graph.base64,'base64'));assert.equal(graph.unit_kind||'tract',unit);
 const project=createProject({name:'Paired '+unit+' '+year,files:selected.files,operation:selected.operation,constraints:[],result,lastOperation:selected.operation});
 assert.deepEqual(parseProject(JSON.stringify(project)),project);const opened=parseWorkbenchArchive(createWorkbenchArchive(project));assert.deepEqual(opened.files,project.files);assert.deepEqual(opened.result,project.result);assert.equal(opened.operation,project.operation);
 rows.push({year,kind,unit_kind:graph.unit_kind||'tract',units:graph.geoids.length,population:result.population});cases++;
}
let reads=0;for(const kind of ['block-centroids','bg','',null])await assert.rejects(readStateCensusArchives({}, {},{state:'RI',year:2020,kind,archiveReader:async()=>{reads++;}}));assert.equal(reads,0);
await fs.writeFile(root+'/evidence.json',JSON.stringify({cases,invalid_resolution_refusals:4,native_host_discrete_values_equal:true,maximum_projected_float_delta:maxDelta,prior_wasm_native_filesystem_fixture_values_equal:true,project_archive_roundtrips:true,rows,wasm_sha256:engine.artifactIdentity.sha256,scope:'Synthetic ZIP fixtures at all four resolutions and three vintages; local extraction and existing native preparation behavior. Native projected floats within 1e-5; byte-identical graphs across platforms not claimed. No provider acquisition claim.'},null,2));
console.log(`Paired Census resolution: ${cases} ZIP/native-host preparation comparisons, projected float delta ${maxDelta}, exact retained WASM fixture graph/map, project/archive roundtrips and four pre-read resolution refusals passed.`);
