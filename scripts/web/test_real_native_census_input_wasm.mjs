// Real local input gate: original sources are checked before native RADJ transcription.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {nativeEngineInputRequest,preparedNativeRunSources,nativeEngineGraphDownload} from '../../web/lab/native-adjacency-files.js';
import {tuiRunRequest} from '../../web/lab/tui-run-files.js';
import {parseWorkbenchArchive,createWorkbenchArchive} from '../../web/lab/workbench-archive.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const kinds=process.argv.slice(2);if(!kinds.length)kinds.push('county','block-group','block');
assert(kinds.every(k=>['county','block-group','block'].includes(k))&&new Set(kinds).size===kinds.length);
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const wasm=await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'),engine=await instantiateEngine(wasm),rows=[];
for(const kind of kinds){
 const root=path.resolve('target/real-native-census-input/RI/'+kind);await fs.mkdir(root,{recursive:true});
 execFileSync('python',['scripts/web/prepare_real_native_census_input.py',kind],{stdio:'inherit'});
 const preparation=JSON.parse(await fs.readFile('target/real-census-preparation/RI/'+kind+'/prepared.json','utf8')),original=JSON.parse(preparation.graph_json);
 const source=async file=>({name:path.basename(file),base64:(await fs.readFile(file)).toString('base64')});
 const sources={geometry:null,adjacency:await source(root+'/original.adj.bin'),geoids:await source(root+'/original_geoids.json'),centroids:await source(root+'/original_centroids.json')},settings={state:'RI',year:'2020',unit_kind:kind};
 const native=JSON.parse(execFileSync('target/debug/examples/native_adjacency_reference.exe',[root+'/original.adj.bin','RI','2020'],{encoding:'utf8',maxBuffer:32*1024*1024}));
 const request=nativeEngineInputRequest(sources,settings),prepared=engine.execute(request);await fs.writeFile(root+'/prepare-request.json',JSON.stringify(request));
 const host=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[root+'/prepare-request.json'],{encoding:'utf8',maxBuffer:32*1024*1024}));assert.deepEqual(prepared,host);
 const graph=JSON.parse(prepared.graph_json);assert.deepEqual(graph,{...native,unit_kind:kind});assert.deepEqual(graph,{...original,edges:[...original.edges].sort((a,b)=>a[0]-b[0]||a[1]-b[1])});
 assert.equal(prepared.population_text,'1097379');assert.equal(prepared.units,{county:5,'block-group':792,block:25649}[kind]);
 assert.equal(Buffer.from((await nativeEngineGraphDownload(prepared)).bytes).toString(),prepared.graph_json);
 const next=await preparedNativeRunSources(prepared),project=createProject({name:'Original RI '+kind+' native binary input',files:{nativeAdjacencyInputs:sources,nativeAdjacencySettings:settings},operation:request.operation,constraints:[],result:prepared,lastOperation:request.operation});assert.deepEqual(parseProject(JSON.stringify(project)),project);await fs.writeFile(root+'/prepared.bisect',JSON.stringify(project));
 const baseline=parseWorkbenchArchive(await fs.readFile('target/real-census-preparation/RI/'+kind+'/run.bisect-workbench','utf8')),files={...baseline.files,nativeAdjacencyInputs:sources,nativeAdjacencySettings:settings,configRunInputs:{...baseline.files.configRunInputs,graph:next.graph}},runRequest=tuiRunRequest(files.tuiRunSettings,files.configSource,files.configSettings,files.configRunInputs,files.configRunSettings);
 await fs.writeFile(root+'/baseline.yml',Buffer.from(files.configSource.base64,'base64'));
 await fs.writeFile(root+'/run-request.json',JSON.stringify(runRequest));console.log('Starting original RI '+kind+' native binary Run on '+prepared.units+' units.');
 const actual=engine.execute(runRequest),expected=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[root+'/run-request.json'],{encoding:'utf8',maxBuffer:128*1024*1024}));assert.deepEqual(actual,expected);assert.deepEqual(actual.configured_run.result.assignments,baseline.result.configured_run.result.assignments);
 const assignments=actual.configured_run.result.assignments,k=files.configRunSettings.districts,groups=Array.from({length:k},()=>[]);graph.geoids.forEach((id,i)=>{const d=assignments[id];assert(Number.isInteger(d)&&d>=1&&d<=k);groups[d-1].push(i);});assert.equal(Object.keys(assignments).length,graph.geoids.length);
 const total=graph.population.reduce((a,b)=>a+b,0);assert.equal(total,1097379);
 for(const group of groups){assert(group.length);const population=group.reduce((a,i)=>a+graph.population[i],0);assert(Math.abs(population/(total/k)-1)<=0.005);const members=new Set(group),seen=new Set([group[0]]),queue=[group[0]];for(let j=0;j<queue.length;j++)for(const v of graph.adjacency[queue[j]])if(members.has(v)&&!seen.has(v)){seen.add(v);queue.push(v);}assert.equal(seen.size,group.length);}
 const plan=JSON.parse(actual.practitioner_export.document_json).plan,context=JSON.parse(actual.practitioner_export.context_json);assert.equal(plan.units.unit_kind,kind);assert.equal(context.units.unit_kind,kind);assert.deepEqual(plan.units.unit_ids,graph.geoids);assert.deepEqual(context.populations,graph.population);
 const archive=createWorkbenchArchive({name:'Original RI '+kind+' binary Run',files,operation:runRequest.operation,result:actual,lastOperation:runRequest.operation});const reopened=parseWorkbenchArchive(archive);assert.deepEqual(reopened.files,files);assert.deepEqual(reopened.result,actual);await fs.writeFile(root+'/run.bisect-workbench',archive);await fs.writeFile(root+'/result.json',JSON.stringify(actual));
 rows.push({unit_kind:kind,units:prepared.units,population_text:prepared.population_text,districts:k,whole_native_preparation:true,native_filesystem_graph:true,original_graph_values_exact:true,whole_native_run:true,original_accepted_assignments:true,independent_graph_contiguity:true,independent_population_balance:true,graph_sha256:prepared.graph_sha256,archive_sha256:sha(archive),archive_bytes:archive.length,memory_bytes:engine.memoryBytes(),source_proof:JSON.parse(await fs.readFile(root+'/original-source-evidence.json','utf8'))});
 await fs.writeFile(root+'/evidence.json',JSON.stringify({wasm_sha256:sha(wasm),...rows.at(-1),scope:'Original RI local Census graph transcribed to RADJ, original source hashes rechecked, complete native file/preparation/Run parity, existing accepted assignments retained. County uses its recorded one-district fixture; no equal two-district county success or national capacity claim.'},null,2));console.log(kind+' original binary path verified.');
}
await fs.writeFile('target/real-native-census-input/evidence.json',JSON.stringify({wasm_sha256:sha(wasm),cases:rows},null,2));
