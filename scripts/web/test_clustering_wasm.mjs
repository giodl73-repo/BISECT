import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFile} from 'node:child_process';
import {promisify} from 'node:util';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {verifyLabAssignments,createLabProject,parseLabProject} from '../../web/lab/laboratory-project.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
const root='dist/wasm-national', manifest=JSON.parse(await fs.readFile(path.join(root,'catalog.json'),'utf8'));
const bytes=await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'),engine=await instantiateEngine(bytes);
manifest.wasm_sha256=createHash('sha256').update(bytes).digest('hex');
for(const structure of ['capacity-clustering','regionalization']){if(!manifest.catalog.structures.includes(structure))manifest.catalog.structures.push(structure);manifest.catalog.search_compatibility[structure]=['single'];}
const native=path.resolve('target/release/examples/execute_request'+(process.platform==='win32'?'.exe':'')),run=promisify(execFile);
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
async function checkStructure(structure){
const cases=[];
const options={structure,weights:'unweighted',search:'single',districts:2,seed:42,seeds:8,steps:20,percentile:0,alpha_county:0,balance_tolerance:5,area_swing:1.1,iterations:10};
const n=6,line={schema_version:1,state:'RI',year:'2020',geoids:Array.from({length:n},(_,i)=>`44001${String(i).padStart(6,'0')}`),adjacency:Array.from({length:n},(_,i)=>[i-1,i+1].filter(v=>v>=0&&v<n)),population:Array(n).fill(100),edges:Array.from({length:n-1},(_,i)=>[i,i+1,100+i]),areas:Array(n).fill(100),exterior_perimeters:Array(n).fill(0),centroids:Array(n).fill([-71,41])};
const inputs=[{name:'line-k2',graph:line,k:2},{name:'line-zero',graph:{...line,population:[100,0,100,100,0,100]},k:2},{name:'line-k3',graph:line,k:3},{name:'infeasible',graph:{...line,population:[100000,...line.population.slice(1)]},k:2}];
for(const code of ['RI','IA'])inputs.push({name:code,graph:JSON.parse(await fs.readFile(path.join(root,manifest.graphs[`${code}:2020`].graph_ref),'utf8')),k:manifest.catalog.states.find(s=>s.code===code).years.find(y=>y.year==='2020').districts.congressional});
let successes=0,failures=0;
for(const {name,graph,k}of inputs)for(const weights of ['unweighted','geographic','county']){
 const request={graph,options:{...options,districts:k,weights,alpha_county:weights==='county'?2:0}},file=path.resolve(`target/${structure}-${name}-${weights}.json`);
 await fs.writeFile(file,JSON.stringify(request));
 let actual,error;try{actual=engine.execute(request);}catch(e){error=e.message;}
 if(error){
  await assert.rejects(run(native,[file],{timeout:120000,maxBuffer:16*1024*1024}),e=>e.stderr.includes(error));failures++;cases.push({name,weights,status:'rejected',error});continue;
 }
 const reference=JSON.parse((await run(native,[file],{timeout:120000,maxBuffer:16*1024*1024})).stdout);
 assert.deepEqual(actual.assignments,reference.assignments);assert.deepEqual(actual.metrics.district_metrics,reference.metrics.district_metrics);assert.deepEqual(actual.metrics.structure_evidence,reference.metrics.structure_evidence);
 assert.equal(actual.metrics.contiguous,true);assert.equal(actual.metrics.within_requested_tolerance,true);
 const changed=engine.execute({graph,options:{...request.options,seed:999,iterations:1,steps:1,seeds:1}});assert.deepEqual(changed.assignments,actual.assignments);assert.deepEqual(changed.metrics.structure_evidence,actual.metrics.structure_evidence);
 const state={metrics:{...actual.metrics,district_count:k,districts:actual.metrics.district_metrics,balance_passed:true}},config=request.options;
 verifyLabAssignments(graph,state,actual.assignments,config);
 for(const tamper of [e=>e.edge_cut++,e=>e.population_deviation++,e=>e.capacity_status='needs-repair',e=>e.seed_method='wrong']){
  const bad=structuredClone(state);tamper(structure==='regionalization'?bad.metrics.structure_evidence.summary:bad.metrics.structure_evidence);assert.throws(()=>verifyLabAssignments(graph,bad,actual.assignments,config),/clustering evidence|regionalization/);
 }
 successes++;cases.push({name,weights,status:'completed',districts:k,summary:actual.metrics.structure_evidence});
}
assert.ok(successes>=6);assert.ok(failures>=3);
assert.throws(()=>engine.execute({graph:line,options:{...options,search:'multi'}}),/does not execute/);
// Portable-project integration on a feasible fixture, including actual hashed
// inputs and a second catalog whose engine factory is forbidden during Open.
const catalog=structuredClone(manifest),assets=new Map();
function asset(value){const data=Buffer.from(JSON.stringify(value)),hash=sha(data),ref=`assets/${hash}.json`;assets.set(ref,data);catalog.assets[ref]=hash;return ref;}
const graphRef=asset(line),geometryRef=asset({type:'FeatureCollection',features:line.geoids.map((id,i)=>({type:'Feature',properties:{geoid:id,state:'RI',county:id.slice(0,5)},geometry:{type:'Point',coordinates:[-71+i*.01,41]}}))});
catalog.graphs['RI:2020']={...catalog.graphs['RI:2020'],graph_ref:graphRef,native_graph_sha256:'3'.repeat(64)};catalog.geometries['RI:2020']=geometryRef;
if(!catalog.catalog.structures.includes(structure))catalog.catalog.structures.push(structure);catalog.catalog.search_compatibility[structure]=['single'];
const fetcher=async ref=>{const data=assets.get(ref);assert.ok(data);return {ok:true,arrayBuffer:async()=>data.buffer.slice(data.byteOffset,data.byteOffset+data.length)};};
class Worker {postMessage(message){setImmediate(()=>{if(this.terminated)return;try{this.onmessage({data:message.type==='initialize'?{type:'ready'}:{type:'result',id:message.id,result:engine.execute(message.request)}});}catch(error){this.onmessage({data:{type:'error',error:error.message}});}});}terminate(){this.terminated=true;}}
const lab=new WasmCatalog(catalog,fetcher,()=>new Worker());
const config={name:'Flow fixture',mode:'state',states:['RI'],year:'2020',chamber:'congressional',...options,timeout_seconds:30};
const job=lab.submit(config);while(lab.active)await new Promise(resolve=>setTimeout(resolve,5));assert.equal(lab.jobs.get(job.id).status,'completed');
const saved=await createLabProject(lab,job.id),opened=new WasmCatalog(catalog,fetcher,()=>{throw new Error('Open must not execute');});
const restored=await opened.restoreProject(parseLabProject(JSON.stringify(saved)));assert.deepEqual(restored.states,saved.experiment.states);
assert.equal((await opened.api(`/api/runs/${restored.id}/RI/map`)).features.length,n);
const bad=structuredClone(saved);(structure==='regionalization'?bad.experiment.states[0].metrics.structure_evidence.summary:bad.experiment.states[0].metrics.structure_evidence).edge_cut++;
const count=opened.jobs.size;await assert.rejects(opened.restoreProject(bad),/clustering evidence|regionalization/);assert.equal(opened.jobs.size,count);
if(structure==='regionalization'){
 for(const tamper of [e=>e.merge_log[0].merged_population++,e=>e.merge_log[0].cut_edges_between++,e=>e.summary.hierarchy_depth++]){
  const bad=structuredClone(saved);tamper(bad.experiment.states[0].metrics.structure_evidence);await assert.rejects(opened.restoreProject(bad),/regionalization|Regionalization/);
 }
}
console.log(`${structure}: ${successes} exact native/WASM successes, ${failures} matching failures; deterministic options, portable project/map roundtrip and tamper checks passed.`);
return {structure,successes,failures,cases};
}
const results=[];
for(const structure of ['capacity-clustering','regionalization'])results.push(await checkStructure(structure));
// Execute the real merge-history renderer against a minimal DOM, checking
// display limits and text-only cells independently from schema validation.
const {runInNewContext}=await import('node:vm'),ui=await fs.readFile('web/lab/lab.js','utf8'),nodes=new Map();
function element(tag){return {tag,children:[],textContent:'',append(child){this.children.push(child);},replaceChildren(){this.children=[];},after(child){nodes.set(child.id,child);}};}
nodes.set('root-constraint-detail',element('p'));
const sandbox={document:{createElement:element},$:id=>nodes.get(id),number:value=>String(value)};
const start=ui.indexOf('function renderMergeHistory('),end=ui.indexOf('function renderCohortMetrics()',start);
runInNewContext(ui.slice(start,end),sandbox);
const evidence=results[1].cases.find(c=>c.status==='completed').summary;
sandbox.renderMergeHistory(evidence);assert.equal(nodes.get('merge-history').hidden,false);
assert.equal(nodes.get('merge-history').children[2].children.length,evidence.merge_log.length+1);
const large={method:'regionalization',merge_log:Array.from({length:1000},()=>({step:'<img onerror=alert(1)>',left_region:1,right_region:2,merged_population:100,cut_edges_between:1}))};
sandbox.renderMergeHistory(large);const table=nodes.get('merge-history').children[2];assert.equal(table.children.length,51);assert.equal(table.children[1].children[0].textContent,'—');
sandbox.renderMergeHistory(null);assert.equal(nodes.get('merge-history').hidden,true);
await fs.writeFile('target/native-wasm-clustering-parity.json',JSON.stringify({wasm_sha256:sha(bytes),native_sha256:sha(await fs.readFile(native)),results},null,2));
