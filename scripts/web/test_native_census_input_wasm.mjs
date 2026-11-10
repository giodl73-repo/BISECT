import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {nativeEngineInputRequest,preparedNativeRunSources,nativeEngineGraphDownload} from '../../web/lab/native-adjacency-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {configRunRequest} from '../../web/lab/config-run.js';
const root='target/native-engine-input-wasm';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const base=JSON.parse(await fs.readFile(root+'/prepared.bisect','utf8'));
const encode=(name,value)=>({name,base64:Buffer.from(JSON.stringify(value)).toString('base64')});
const reference=request=>{const file=path.resolve(root+'/typed-request.json');return fs.writeFile(file,JSON.stringify(request)).then(()=>JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[file],{encoding:'utf8',maxBuffer:32*1024*1024})));};
let exact=0,refusals=0;
for(const [kind,ids] of [['county',['44001','44003','44005','44007']],['tract',['44001000100','44001000200','44001000300','44001000400']],['block-group',['440010001001','440010002001','440010003001','440010004001']],['block',['440010001001001','440010002001001','440010003001001','440010004001001']]]){
 const sources={...base.files.nativeAdjacencyInputs,geoids:encode(kind+'_geoids.json',Object.fromEntries(ids.map((id,i)=>[i,id])))},settings={state:'RI',year:'2020',unit_kind:kind};
 await fs.writeFile(root+'/'+kind+'_geoids.json',Buffer.from(sources.geoids.base64,'base64'));
 const request=nativeEngineInputRequest(sources,settings),result=engine.execute(request);assert.deepEqual(result,await reference(request));exact++;
 const graph=JSON.parse(result.graph_json),legacy=JSON.parse(base.result.graph_json);assert.deepEqual({...graph,geoids:legacy.geoids,unit_kind:undefined},{...legacy,unit_kind:undefined});assert.deepEqual(graph.geoids,ids);assert.equal(result.unit_kind,kind);assert.equal(result.execution_performed,false);
 const download=await nativeEngineGraphDownload(result);assert.equal(Buffer.from(download.bytes).toString(),result.graph_json);
 const next=await preparedNativeRunSources(result);assert.equal(Buffer.from(next.graph.base64,'base64').toString(),result.graph_json);
 const project=createProject({name:kind+' native binary',files:{nativeAdjacencyInputs:sources,nativeAdjacencySettings:settings},operation:request.operation,constraints:[],result,lastOperation:request.operation});assert.deepEqual(parseProject(JSON.stringify(project)),project);await fs.writeFile(root+'/'+kind+'-prepared.bisect',JSON.stringify(project));
 const run=configRunRequest({name:'baseline.yml',base64:(await fs.readFile(root+'/baseline.yml')).toString('base64')},{profile:'open'},{graph:next.graph},{chamber:'congressional',districts:2});assert.deepEqual(engine.execute(run),await reference(run));exact++;
 for(const changed of [{...request,input:{...request.input,unit_kind:kind==='county'?'tract':'county'}},{...request,input:{...request.input,state:'NC'}},{...request,input:{...request.input,sources:{...sources,geoids:encode('duplicate.json',Object.fromEntries(ids.map((id,i)=>[i,ids[0]])))}}}]){
  assert.throws(()=>engine.execute(changed));await assert.rejects(()=>reference(changed));refusals++;
 }
}
for(const unit_kind of ['precinct','imported','unsupported']){assert.throws(()=>nativeEngineInputRequest(base.files.nativeAdjacencyInputs,{state:'RI',year:'2020',unit_kind}));const request={operation:'prepare-native-engine-input',input:{sources:base.files.nativeAdjacencyInputs,state:'RI',year:'2020',unit_kind}};assert.throws(()=>engine.execute(request));await assert.rejects(()=>reference(request));refusals++;}
await fs.writeFile(root+'/typed-evidence.json',JSON.stringify({scope:'Synthetic independently encoded four-unit RADJ v3; county/tract/block-group/block native-host/WASM complete preparation and configured Run equality. Does not establish all-state capacity or source authenticity.',whole_native_matches:exact,matching_refusals:refusals,wasm_sha256:createHash('sha256').update(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm')).digest('hex')},null,2));
console.log(`Native Census binary inputs: ${exact} whole native/WASM preparation/run matches, ${refusals} matching refusals, exact arrays/IDs/typed downloads and inert project retention passed.`);
