// Actual CLI State pipeline versus WASM, on retained original RI tract inputs.
// All outputs are isolated under target; no provider requests or source edits.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {validateLabConfig,verifyLabAssignments} from '../../web/lab/laboratory-project.js';
import {engineOptions} from '../../web/lab/static.js';

const root=path.resolve('target/native-compact-geometry'),work=path.join(root,'runs',String(Date.now()));
await fs.mkdir(work,{recursive:true});
const preparation=JSON.parse(await fs.readFile('target/source-projection-parity/after-wasm.json','utf8'));
const graph=JSON.parse(preparation.graph_json),sha=b=>createHash('sha256').update(b).digest('hex');
assert.equal(graph.geoids.length,250);assert.equal(graph.population.reduce((a,b)=>a+b,0),1097379);
// Independent RADJ v3 encoder; the actual native loader consumes these bytes.
const chunks=[],u32=n=>{const b=Buffer.alloc(4);b.writeUInt32LE(n);return b;},f64=n=>{const b=Buffer.alloc(8);b.writeDoubleLE(n);return b;};
chunks.push(Buffer.from('RADJ'),u32(3),u32(graph.geoids.length),u32(graph.edges.length));
for(const value of graph.population){const b=Buffer.alloc(8);b.writeBigInt64LE(BigInt(value));chunks.push(b);}
for(const neighbors of graph.adjacency)chunks.push(u32(neighbors.length),...neighbors.map(u32));
chunks.push(u32(graph.edges.length));for(const [u,v,w]of graph.edges)chunks.push(u32(u),u32(v),f64(w));
chunks.push(u32(1),...graph.exterior_perimeters.map(f64),...graph.areas.map(f64));
const binary=Buffer.concat(chunks),adjacency=path.join(work,'ri_2020.adj.bin');await fs.writeFile(adjacency,binary);
await fs.writeFile(path.join(work,'ri_2020_geoids.json'),JSON.stringify(Object.fromEntries(graph.geoids.map((id,i)=>[i,id]))));
await fs.writeFile(path.join(work,'ri_2020_centroids.json'),JSON.stringify({centroids:graph.centroids}));
const tiger='data/2020/tiger/tracts/tl_2020_44_tract/tl_2020_44_tract',sources={};
for(const extension of ['shp','dbf','shx','prj']){const name=tiger+'.'+extension,bytes=await fs.readFile(name),destination=path.join(work,name);await fs.mkdir(path.dirname(destination),{recursive:true});await fs.writeFile(destination,bytes);sources[extension]=sha(bytes);}
const before=path.join(root,'before/bisect.exe'),after=path.resolve('target/release/bisect.exe'),host=path.resolve('target/release/examples/execute_tool_request.exe');
const wasm=await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'),engine=await instantiateEngine(wasm);
const cases=[];for(const weights of ['unweighted','geographic'])for(const districts of [2,4])for(const seeds of [1,3,12])cases.push({weights,districts,seeds});
for(const districts of [2,4])cases.push({weights:'geographic',districts,seeds:3,coi:true});
const oldWasm=await fs.readFile(path.join(root,'before/engine.wasm')),oldEngine=await instantiateEngine(oldWasm);
const records=[];let defaultDifferences=0;
for(const [index,c]of cases.entries()){
 const folder=path.join(work,String(index));await fs.mkdir(folder,{recursive:true});
 const options={structure:'compact-polsby',weights:c.weights,search:'multi',districts:c.districts,seed:null,seeds:c.seeds,steps:20,percentile:0,alpha_county:0,balance_tolerance:25,area_swing:1.1,iterations:100,compact_epsilon:0.05,compact_geometry:'native-state-circular'};
 const request={graph,options};let coiArgs=[];
 if(c.coi){const weights_json=JSON.stringify(Object.fromEntries(graph.geoids.filter((_,i)=>i%3===0).map((id,i)=>[id,i%2?0.25:0])));const file=path.join(folder,'coi.json');await fs.writeFile(file,weights_json);request.coi={source_name:'coi.json',weights_json};coiArgs=['--coi-weights',file];}
 const generated=[];
 for(const [name,exe]of [['before',before],['after',after]]){
  const out=path.join(folder,name),args=['state','--state','RI','--state-name','rhode_island','--year','2020','--version','parity','--output-dir',out,'--label','compact_test','--adjacency',adjacency,'--districts',String(c.districts),'--partition-mode','compact-bisect','--weights-override',c.weights,'--compact-seeds',String(c.seeds),'--balance-tolerance','25','--niter','100',...coiArgs];
  const process=spawnSync(exe,args,{cwd:work,encoding:'utf8',windowsHide:true,timeout:180000,maxBuffer:16*1024*1024});await fs.writeFile(path.join(folder,name+'.log'),process.stdout+'\n'+process.stderr);assert.equal(process.status,0,process.stderr);
  const text=await fs.readFile(path.join(out,'2020/plans/compact_test/data/final_assignments.json'),'utf8'),raw=JSON.parse(text);assert.equal(Object.keys(raw).length,graph.geoids.length);generated.push({text,assignments:Object.fromEntries(graph.geoids.map((id,i)=>[id,raw[String(i)]]))});
 }
 assert.equal(generated[0].text,generated[1].text,'Extracting native geometry arithmetic must preserve the old CLI output byte for byte.');
 const tool={operation:'run-engine-build',request},file=path.join(folder,'request.json');await fs.writeFile(file,JSON.stringify(tool));const actual=engine.execute(tool),reference=spawnSync(host,[file],{encoding:'utf8',windowsHide:true,timeout:180000,maxBuffer:32*1024*1024});assert.equal(reference.status,0,reference.stderr);assert.deepEqual(actual,JSON.parse(reference.stdout));assert.deepEqual(actual.assignments,generated[1].assignments);
 assert.equal(actual.metrics.structure_evidence.geometry_inputs,'native-state-circular');
 const original=structuredClone(request);delete original.options.compact_geometry;const defaultResult=engine.execute({operation:'run-engine-build',request:original}),changed=JSON.stringify(defaultResult.assignments)!==JSON.stringify(actual.assignments);defaultDifferences+=changed;assert.deepEqual(defaultResult,oldEngine.execute({operation:'run-engine-build',request:original}),'Omitted profile must preserve the previous whole WASM result.');if(index===0){const explicit=structuredClone(original);explicit.options.compact_geometry='prepared';const tool={operation:'run-engine-build',request:explicit},result=engine.execute(tool),file=path.join(folder,'explicit-prepared.json');await fs.writeFile(file,JSON.stringify(tool));const native=spawnSync(host,[file],{encoding:'utf8',maxBuffer:32*1024*1024});assert.equal(native.status,0,native.stderr);assert.deepEqual(result,JSON.parse(native.stdout));assert.deepEqual(result.assignments,defaultResult.assignments);assert.equal(result.metrics.structure_evidence.geometry_inputs,'prepared');}
 if(!c.coi){const config={...options,name:'Native State geometry',mode:'state',states:['RI'],year:'2020',chamber:'congressional',timeout_seconds:60};validateLabConfig(config);assert.equal(engineOptions(config).compact_geometry,'native-state-circular');const row={metrics:{...actual.metrics,district_count:c.districts,districts:actual.metrics.district_metrics,balance_passed:actual.metrics.within_requested_tolerance}};verifyLabAssignments(graph,row,actual.assignments,options);const bad=structuredClone(row);bad.metrics.structure_evidence.geometry_inputs='prepared';assert.throws(()=>verifyLabAssignments(graph,bad,actual.assignments,options),/CompactBisect evidence/);}
 await fs.writeFile(path.join(folder,'result.json'),JSON.stringify(actual));records.push({...c,actual_cli_assignment_sha256:sha(generated[1].text),native_wasm_build_match:true,accepted_attempt:actual.build_report.accepted_attempt,prepared_geometry_selects_different_plan:changed});
}
for(const compact_geometry of ['prepared','native-state-circular'])assert.throws(()=>validateLabConfig({...cases[0],name:'bad',mode:'state',states:['RI'],year:'2020',chamber:'congressional',structure:'standard-bisect',compact_geometry}));
for(const patch of [{compact_geometry:'unknown'},{structure:'standard-bisect',compact_geometry:'native-state-circular',compact_epsilon:undefined}]){const request={graph,options:{structure:'compact-polsby',weights:'geographic',search:'single',districts:2,seed:null,seeds:1,steps:1,percentile:0,alpha_county:0,balance_tolerance:25,area_swing:1.1,iterations:10,...patch}};assert.throws(()=>engine.execute(request));const file=path.join(work,'invalid-'+patch.structure+'-'+patch.compact_geometry+'.json');await fs.writeFile(file,JSON.stringify({operation:'run-engine-build',request}));assert.notEqual(spawnSync(host,[file],{encoding:'utf8'}).status,0);}
await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({wasm_sha256:sha(wasm),native_cli_sha256:sha(await fs.readFile(after)),before_native_cli_sha256:sha(await fs.readFile(before)),before_wasm_sha256:sha(oldWasm),legacy_default_whole_wasm_results_unchanged:true,explicit_prepared_profile_alias_native_match:true,matching_native_wasm_profile_refusals:2,native_host_sha256:sha(await fs.readFile(host)),source_graph_sha256:sha(preparation.graph_json),radj_sha256:sha(binary),tiger_sources:sources,units:250,population:1097379,actual_cli_before_after_and_wasm_matches:records.length,prepared_geometry_different_plans:defaultDifferences,workspace:work,records,scope:'Actual native State CLI parsing, RADJ/companion/TIGER loading, resolved weights/COI, compact dispatch, population retry/check and atomic assignment writing. Same retained original RI2020 graph on all paths. Four districts is an explicit research override; native congressional allocation is two. Prepared graph geometry stays unchanged. No external acquisition, all-state capacity, certification or optimality claim.'},null,2)+'\n');
console.log(`Native State compact geometry: ${records.length} old/current actual CLI assignment-byte and WASM matches, including COI; ${defaultDifferences} plans differ from prepared-perimeter selection.`);
