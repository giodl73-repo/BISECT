import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {nativeAdjacencyRequest,nativeAdjacencyDownload,validateNativeAdjacencyInputs} from '../../web/lab/native-adjacency-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {createWorkbenchArchive,parseWorkbenchArchive} from '../../web/lab/workbench-archive.js';
const root='target/native-adjacency-wasm';await fs.mkdir(root,{recursive:true});
execFileSync('python',['-c',String.raw`
import pathlib,struct,json
r=pathlib.Path('target/native-adjacency-wasm')
for version in [2,3]:
 for companions in [False,True]:
  stem=f'v{version}-'+('companions' if companions else 'bare')
  body=b'RADJ'+struct.pack('<III',version,3,2)+struct.pack('<qqq',11,7,9223372036854775807)
  for neighbors in [[1],[0,2],[1]]:body+=struct.pack('<I',len(neighbors))+struct.pack('<'+'I'*len(neighbors),*neighbors)
  body+=struct.pack('<I',2)+struct.pack('<IIdIId',0,1,2.5,1,2,3.5)
  if version==3:body+=struct.pack('<I',1)+struct.pack('<dddddd',4.,5.,6.,7.,8.,9.)
  (r/(stem+'.adj.bin')).write_bytes(body)
  if companions:
   (r/(stem+'_geoids.json')).write_text(json.dumps({'0':'44001000100','1':'44001000200','2':'44001000300','legacy':'ignored'}),encoding='utf-8')
   (r/(stem+'_centroids.json')).write_text(json.dumps({'centroids':[[-71.5,41.5],[-71.4,41.6],[-71.3,41.7]]}),encoding='utf-8')
`]);
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const source=async name=>({name:path.basename(name),base64:(await fs.readFile(name)).toString('base64')});
for(const version of [2,3])for(const companions of [false,true]){
 const name=`v${version}-${companions?'companions':'bare'}`,binary=root+'/'+name+'.adj.bin',input={geometry:null,adjacency:await source(binary),geoids:companions?await source(root+'/'+name+'_geoids.json'):null,centroids:companions?await source(root+'/'+name+'_centroids.json'):null};
 const result=engine.execute(nativeAdjacencyRequest(input)),native=execFileSync('target/debug/examples/native_adjacency_reference.exe',[binary],{encoding:'utf8'});assert.equal(result.loaded_graph_json,native);assert(result.loaded_graph_json.includes('9223372036854775807'));assert.equal(result.format_version,version);assert.equal(result.n_vertices,3);assert.equal(result.n_edges,2);
 const hash=text=>createHash('sha256').update(text).digest('hex');assert.equal(result.loaded_graph_sha256,hash(native));assert.equal(result.sources.adjacency.sha256,hash(Buffer.from(input.adjacency.base64,'base64')));assert.equal(Buffer.from((await nativeAdjacencyDownload(result)).bytes).toString(),native);
 const project=createProject({name:'Native binary '+name,files:{nativeAdjacencyInputs:input},operation:'inspect-native-adjacency',constraints:[],result,lastOperation:'inspect-native-adjacency'});assert.deepEqual(parseProject(JSON.stringify(project)),project);assert.deepEqual(parseWorkbenchArchive(createWorkbenchArchive(project)).files,project.files);await fs.writeFile(root+'/'+name+'.bisect',JSON.stringify(project));
 await assert.rejects(()=>nativeAdjacencyDownload({...result,loaded_graph_json:native+' '}),/bytes changed/);
}
const valid=JSON.parse(await fs.readFile(root+'/v3-companions.bisect','utf8')).files.nativeAdjacencyInputs;
for(const offset of [8,40]){const body=Buffer.from(valid.adjacency.base64,'base64');body.writeUInt32LE(4294967295,offset);assert.throws(()=>engine.execute(nativeAdjacencyRequest({...valid,adjacency:{name:'bad.bin',base64:body.toString('base64')}})));}
assert.throws(()=>engine.execute(nativeAdjacencyRequest({...valid,adjacency:{name:'truncated.bin',base64:Buffer.from('RADJ').toString('base64')}})));
assert.throws(()=>validateNativeAdjacencyInputs({...valid,geoids:{name:'duplicate.json',base64:Buffer.from('{"0":"first","0":"second"}').toString('base64')}}));
assert.throws(()=>engine.execute({operation:'inspect-native-adjacency',input:{...valid,centroids:{name:'bad.json',base64:Buffer.from('{"centroids":[[0,0]]}').toString('base64')}}}));
console.log('Native adjacency: four v2/v3 actual native filesystem/WASM exact-byte comparisons, companion presence/filtering/geometry, exact i64/source/output hashes, project/archive retention and malformed count/truncation/duplicate/centroid/export guards passed.');
