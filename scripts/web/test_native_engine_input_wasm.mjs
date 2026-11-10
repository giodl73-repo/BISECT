import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {nativeEngineInputRequest,nativeEngineGraphDownload,preparedNativeRunSources} from '../../web/lab/native-adjacency-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {createWorkbenchArchive,parseWorkbenchArchive} from '../../web/lab/workbench-archive.js';
import {CONFIG_TEMPLATE_DEFAULTS,configTemplateRequest} from '../../web/lab/config-template.js';
import {configRunRequest} from '../../web/lab/config-run.js';
const root='target/native-engine-input-wasm';await fs.mkdir(root,{recursive:true});
// Independent native byte fixture; population permits an exact two-way split.
execFileSync('python',['-c',String.raw`
import pathlib,struct,json
r=pathlib.Path('target/native-engine-input-wasm')
for version in [2,3]:
 body=b'RADJ'+struct.pack('<III',version,4,3)+struct.pack('<qqqq',10,10,10,10)
 for neighbors in [[1],[0,2],[1,3],[2]]:body+=struct.pack('<I',len(neighbors))+struct.pack('<'+'I'*len(neighbors),*neighbors)
 body+=struct.pack('<I',3)
 for u,v in [(0,1),(1,2),(2,3)]:body+=struct.pack('<IId',u,v,100.)
 if version==3:body+=struct.pack('<I',1)+struct.pack('<dddddddd',300.,200.,200.,300.,10000.,10000.,10000.,10000.)
 (r/f'v{version}.adj.bin').write_bytes(body)
 (r/f'v{version}_geoids.json').write_text(json.dumps({str(i):f'44001000{i+1}00' for i in range(4)}),encoding='utf-8')
 (r/f'v{version}_centroids.json').write_text(json.dumps({'centroids':[[-71.5+i*.01,41.5] for i in range(4)]}),encoding='utf-8')
`]);
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const source=async name=>({name:path.basename(name),base64:(await fs.readFile(name)).toString('base64')});
const sources={geometry:null,adjacency:await source(root+'/v3.adj.bin'),geoids:await source(root+'/v3_geoids.json'),centroids:await source(root+'/v3_centroids.json')},settings={state:'RI',year:'2020'};
const request=nativeEngineInputRequest(sources,settings),result=engine.execute(request),native=execFileSync('target/debug/examples/native_adjacency_reference.exe',[root+'/v3.adj.bin','RI','2020'],{encoding:'utf8'});
assert.equal(result.graph_json,native);assert.equal(result.execution_performed,false);assert.equal(result.population_text,'40');assert.equal(result.units,4);assert.equal(Buffer.from((await nativeEngineGraphDownload(result)).bytes).toString(),native);
const next=await preparedNativeRunSources(result);assert.equal(Buffer.from(next.graph.base64,'base64').toString(),native);
const project=createProject({name:'Native engine input',files:{nativeAdjacencyInputs:sources,nativeAdjacencySettings:settings},operation:request.operation,constraints:[],result,lastOperation:request.operation});assert.deepEqual(parseProject(JSON.stringify(project)),project);assert.deepEqual(parseWorkbenchArchive(createWorkbenchArchive(project)).files,project.files);await fs.writeFile(root+'/prepared.bisect',JSON.stringify(project));
const yaml=engine.execute(configTemplateRequest({...CONFIG_TEMPLATE_DEFAULTS,name:'native_binary_baseline',structure:'standard-bisect',seed:'42'})).yaml_text;
const runRequest=configRunRequest({name:'baseline.yml',base64:Buffer.from(yaml).toString('base64')},{profile:'open'},{graph:next.graph},next.settings),run=engine.execute(runRequest);await fs.writeFile(root+'/run-request.json',JSON.stringify(runRequest));
const nativeRun=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[root+'/run-request.json'],{encoding:'utf8',maxBuffer:32*1024*1024}));assert.deepEqual(run.result.assignments,nativeRun.result.assignments);assert.equal(Object.keys(run.result.assignments).length,4);await fs.writeFile(root+'/baseline.yml',yaml);await fs.writeFile(root+'/run-result.json',JSON.stringify(run));
await assert.rejects(()=>nativeEngineGraphDownload({...result,graph_json:native+' '}),/bytes changed/);
const jsonSource=value=>({name:'selected.json',base64:Buffer.from(JSON.stringify(value)).toString('base64')});
for(const bad of [{...sources,geoids:null},{...sources,centroids:null},{...sources,geoids:jsonSource({'0':'44001000100','1':'44001000200','2':'44001000300','9':'44001000400'})},{...sources,geoids:jsonSource({'0':'44001000100','1':'44001000100','2':'44001000300','3':'44001000400'})},{...sources,geoids:jsonSource({'0':'44001000100','1':'44001000200','2':'44001000300','3':'440010004001234'})},{...sources,centroids:jsonSource({centroids:[[181,0],[0,0],[0,0],[0,0]]})},{...sources,adjacency:await source(root+'/v2.adj.bin')}])assert.throws(()=>engine.execute(nativeEngineInputRequest(bad,settings)));
assert.throws(()=>engine.execute(nativeEngineInputRequest(sources,{state:'NC',year:'2020'})),/selected state/);assert.throws(()=>engine.execute({...request,input:{...request.input,year:'2030'}}));assert.throws(()=>engine.execute({...request,input:{...request.input,state:'ZZ'}}));
for(const [offset,kind]of [[16,'population'],[100,'boundary'],[144,'geometry']]){const bytes=Buffer.from(sources.adjacency.base64,'base64');if(kind==='population')bytes.writeBigInt64LE(9223372036854775807n,offset);else bytes.writeDoubleLE(-1,offset);assert.throws(()=>engine.execute(nativeEngineInputRequest({...sources,adjacency:{name:'bad.bin',base64:bytes.toString('base64')}},settings)));}
console.log('Native engine input: independent filesystem graph bytes, native/WASM seeded run assignments, source project/archive retention, source/output hashes, and identity/geometry/centroid/population/boundary/v2 guards passed.');
if(process.argv.includes('--local-ri')){
 // Transcribe previously native-verified RI attributes to the documented RADJ
 // layout independently of both serializers, then use real native file loading.
 execFileSync('python',['-c',String.raw`
import pathlib,struct,json
r=pathlib.Path('target/native-engine-input-wasm');g=json.loads(json.loads(pathlib.Path('target/tract-preparation-wasm/ri2020-result.json').read_text(encoding='utf-8'))['graph_json'])
n=len(g['geoids']);body=b'RADJ'+struct.pack('<III',3,n,len(g['edges']))+struct.pack('<'+'q'*n,*g['population'])
for neighbors in g['adjacency']:body+=struct.pack('<I',len(neighbors))+struct.pack('<'+'I'*len(neighbors),*neighbors)
body+=struct.pack('<I',len(g['edges']))
for u,v,w in g['edges']:body+=struct.pack('<IId',u,v,w)
body+=struct.pack('<I',1)+struct.pack('<'+'d'*n,*g['exterior_perimeters'])+struct.pack('<'+'d'*n,*g['areas'])
(r/'ri.adj.bin').write_bytes(body)
(r/'ri_geoids.json').write_text(json.dumps(dict(enumerate(g['geoids']))),encoding='utf-8')
(r/'ri_centroids.json').write_text(json.dumps({'centroids':g['centroids']}),encoding='utf-8')
`]);
 const riSources={geometry:null,adjacency:await source(root+'/ri.adj.bin'),geoids:await source(root+'/ri_geoids.json'),centroids:await source(root+'/ri_centroids.json')},ri=engine.execute(nativeEngineInputRequest(riSources,settings));
 assert.equal(ri.units,250);assert.equal(ri.population_text,'1097379');assert.equal(ri.graph_json,execFileSync('target/debug/examples/native_adjacency_reference.exe',[root+'/ri.adj.bin','RI','2020'],{encoding:'utf8',maxBuffer:32*1024*1024}));
 const nextRI=await preparedNativeRunSources(ri),requestRI=configRunRequest({name:'baseline.yml',base64:Buffer.from(yaml).toString('base64')},{profile:'open'},{graph:nextRI.graph},nextRI.settings);await fs.writeFile(root+'/ri-run-request.json',JSON.stringify(requestRI));
 const runRI=engine.execute(requestRI),nativeRI=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[root+'/ri-run-request.json'],{encoding:'utf8',maxBuffer:32*1024*1024}));assert.deepEqual(runRI.result.assignments,nativeRI.result.assignments);assert.equal(Object.keys(runRI.result.assignments).length,250);
 await fs.writeFile(root+'/ri-prepared.bisect',JSON.stringify(createProject({name:'Real RI native binary',files:{nativeAdjacencyInputs:riSources,nativeAdjacencySettings:settings},operation:'prepare-native-engine-input',constraints:[],result:ri,lastOperation:'prepare-native-engine-input'})));await fs.writeFile(root+'/ri-run-result.json',JSON.stringify(runRI));console.log('Real RI binary path: 250 tracts, population 1,097,379, exact native-file prepared graph and seed-42 native/WASM assignments passed. Year/geometry are retained source claims.');
}
