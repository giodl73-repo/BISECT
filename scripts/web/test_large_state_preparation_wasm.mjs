// Explicit local Census inputs; this test never downloads or activates data.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {tractPreparationRequest,preparedTractRunSources} from '../../web/lab/tract-input-files.js';
import {parseWorkbenchArchive,createWorkbenchArchive} from '../../web/lab/workbench-archive.js';
const root='target/large-state-preparation-wasm';await fs.mkdir(root,{recursive:true});
const tiger=parseWorkbenchArchive(await fs.readFile('target/tiger-gzip-wasm/ca-inputs.bisect-workbench','utf8')).files.tigerInputs;
const pl94=parseWorkbenchArchive(await fs.readFile('target/pl94-large-wasm/california.bisect-workbench','utf8')).files.pl94Inputs;
const prefix='data/2020/tiger/tracts/tl_2020_06_tract/tl_2020_06_tract';
const files={tigerInputs:tiger,pl94Inputs:pl94,tractPreparationSettings:{state:'CA',projection:{name:'tl_2020_06_tract.prj',base64:(await fs.readFile(prefix+'.prj')).toString('base64')}}};
const request=tractPreparationRequest(files),hash=text=>createHash('sha256').update(text).digest('hex');
await fs.writeFile(root+'/ca-request.json',JSON.stringify(request));
await fs.writeFile(root+'/ca-inputs.bisect-workbench',createWorkbenchArchive({name:'California original Census sources',files,operation:request.operation}));
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
console.log('Starting California WASM preparation from original retained sources.');
// Explicit continuation of this local test may reuse its already-written WASM
// artifact; native comparisons below still bind graph and source identities.
const result=process.argv.includes('--reuse-wasm-result')?JSON.parse(await fs.readFile(root+'/ca-result.json','utf8')):engine.execute(request);await fs.writeFile(root+'/ca-result.json',JSON.stringify(result));
await fs.writeFile(root+'/ca-graph.json',result.graph_json);await fs.writeFile(root+'/ca-geometry.json',result.geometry_json);
assert.equal(result.units,9129);assert.equal(result.population,'39538223');
console.log('WASM prepared '+result.units+' units; starting native comparisons.');
const options={encoding:'utf8',maxBuffer:128*1024*1024};
const host=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[root+'/ca-request.json'],options));
await fs.writeFile(root+'/ca-host-result.json',JSON.stringify(host));
const native=JSON.parse(execFileSync('target/debug/examples/tract_prepare_reference.exe',['CA','2020',prefix+'.shp','data/2020/redistricting/california/cageo2020.pl','data/2020/redistricting/california/ca000012020.pl'],options));
await fs.writeFile(root+'/ca-native-result.json',JSON.stringify(native));
const graph=JSON.parse(result.graph_json),hostGraph=JSON.parse(host.graph_json);assert.deepEqual(hostGraph,native.graph);
let maximumDelta=0;
function numeric(a,b){if(Array.isArray(a)){assert.equal(a.length,b.length);a.forEach((v,i)=>numeric(v,b[i]));}else{assert.ok(Number.isFinite(a)&&Number.isFinite(b));const delta=Math.abs(a-b);maximumDelta=Math.max(maximumDelta,delta);assert.ok(delta<=1e-5,'Projected graph exceeds 10-micrometre cross-platform bound.');}}
for(const key of ['schema_version','state','year','geoids','population','adjacency','areas'])assert.deepEqual(graph[key],hostGraph[key]);
assert.deepEqual(graph.edges.map(e=>e.slice(0,2)),hostGraph.edges.map(e=>e.slice(0,2)));
for(const key of ['centroids','exterior_perimeters'])numeric(graph[key],hostGraph[key]);numeric(graph.edges.map(e=>e[2]),hostGraph.edges.map(e=>e[2]));
assert.deepEqual(JSON.parse(result.geometry_json),native.geometry);assert.equal(host.geometry_json,result.geometry_json);assert.deepEqual(result.sources,host.sources);
assert.deepEqual(result.bridges.map(b=>[b.from,b.to]),native.bridges.map(b=>[b.from,b.to]));numeric(result.bridges.map(b=>b.cost),native.bridges.map(b=>b.cost));
assert.equal(result.graph_sha256,hash(result.graph_json));assert.equal(result.geometry_sha256,hash(result.geometry_json));assert.equal(result.execution_performed,false);
const next=preparedTractRunSources(result);assert.equal(Buffer.from(next.graph.base64,'base64').toString(),result.graph_json);assert.equal(Buffer.from(next.geometry.base64,'base64').toString(),result.geometry_json);
const archive=createWorkbenchArchive({name:'Prepared California Census sources',files,operation:request.operation,result,lastOperation:request.operation});assert.deepEqual(parseWorkbenchArchive(archive).files,files);await fs.writeFile(root+'/ca-prepared.bisect-workbench',archive);
await fs.writeFile(root+'/ca-receipt.json',JSON.stringify({units:result.units,population:result.population,edges:result.edges,bridges:result.bridges.length,graph_sha256:result.graph_sha256,geometry_sha256:result.geometry_sha256,maximum_projected_delta_m:maximumDelta,archive_bytes:Buffer.byteLength(archive),scope:'Full selected CA 2020 tract preparation profile parity; no engine execution, source acquisition/cache installation or certified NRS claim.'}));
console.log('California: full population retained, native/WASM graph topology/population/display parity, maximum projected delta '+maximumDelta+'m, source/result archive '+Buffer.byteLength(archive)+' bytes and run handoff passed.');
