import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {execFile} from 'node:child_process';
import {promisify} from 'node:util';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
import {createLabProject,parseLabProject} from '../../web/lab/laboratory-project.js';
const root=process.argv[2]||'dist/wasm-national';
const wasm=await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'),engine=await instantiateEngine(wasm);
const native=path.resolve('target/release/examples/execute_request'+(process.platform==='win32'?'.exe':''));
const manifest=JSON.parse(await fs.readFile(path.join(root,'catalog.json'),'utf8'));
const sha=value=>createHash('sha256').update(value).digest('hex'),cases=[];
const n=1020;
const line={schema_version:1,state:'IA',year:'2020',geoids:Array.from({length:n},(_,i)=>`19001${String(i).padStart(6,'0')}`),
 adjacency:Array.from({length:n},(_,i)=>[...(i?[i-1]:[]),...(i<n-1?[i+1]:[])]),population:Array(n).fill(100),
 edges:Array.from({length:n-1},(_,i)=>[i,i+1,100]),areas:Array(n).fill(10000),exterior_perimeters:Array(n).fill(0),centroids:Array.from({length:n},(_,i)=>[-93+i/10000,42])};
const options={structure:'prime-factor',weights:'geographic',search:'single',districts:2,seed:42,seeds:8,steps:20,percentile:0,alpha_county:0,balance_tolerance:5,area_swing:1.1,iterations:10};
const nativeRun=promisify(execFile);
async function compare(label,graph,districts,weights='geographic'){
 const request={graph,options:{...options,districts,weights,alpha_county:weights==='county'?2:0}};
 const input=path.resolve(`target/prime-${label}.json`);await fs.writeFile(input,JSON.stringify(request));
 let reference,error;
 try{reference=JSON.parse((await nativeRun(native,[input],{maxBuffer:16*1024*1024,timeout:120000})).stdout);}catch(e){error=e.stderr;}
 if(error){const nativeError=JSON.parse(error.trim().replace(/^Error: /,''));assert.throws(()=>engine.execute(request),e=>e.message===nativeError);assert.match(error,/apportion-regions/);cases.push({label,status:'matching-rejection',error:error.trim()});}
 else{
  const actual=engine.execute(request);assert.deepEqual(actual.assignments,reference.assignments,label);
  assert.deepEqual(actual.metrics.district_metrics,reference.metrics.district_metrics,label);
  assert.deepEqual(actual.metrics.structure_evidence,reference.metrics.structure_evidence,label);
  assert.equal(Object.keys(actual.assignments).length,graph.geoids.length);
  assert.ok(actual.metrics.max_deviation_percent<=3);
  cases.push({label,status:'completed',districts,units:graph.geoids.length,contiguous:actual.metrics.contiguous,deviation_percent:actual.metrics.max_deviation_percent,evidence:actual.metrics.structure_evidence,assignment_sha256:sha(JSON.stringify(graph.geoids.map(id=>actual.assignments[id])))});
 }
 console.log(JSON.stringify(cases.at(-1)));
}
for(const k of [1,2,3,5,6,10,12,15,17,34,51])await compare(`line-${k}`,line,k);
for(const [width,counts]of [[12,[3,6,12]],[20,[5]]]){
 const n=width*width, adjacency=Array.from({length:n},(_,i)=>[...(i%width?[i-1]:[]),...(i%width<width-1?[i+1]:[]),...(i>=width?[i-width]:[]),...(i<n-width?[i+width]:[])]);
 const grid={...line,geoids:line.geoids.slice(0,n),adjacency,population:Array(n).fill(100),areas:Array(n).fill(10000),exterior_perimeters:Array(n).fill(0),centroids:line.centroids.slice(0,n),edges:adjacency.flatMap((neighbors,u)=>neighbors.filter(v=>v>u).map(v=>[u,v,100]))};
 for(const k of counts)await compare(`grid-${width}-${k}`,grid,k);
}
for(const code of ['RI','IA'])for(const weights of ['geographic','unweighted','county']){
 const graph=JSON.parse(await fs.readFile(path.join(root,manifest.graphs[`${code}:2020`].graph_ref),'utf8'));
 const k=manifest.catalog.states.find(s=>s.code===code).years.find(y=>y.year==='2020').districts.congressional;
 await compare(`${code}-${weights}`,graph,k,weights);
}
assert.ok(cases.filter(c=>c.status==='completed').length>=10);
assert.ok(cases.some(c=>c.label==='grid-20-5'&&c.status==='completed'));
assert.throws(()=>engine.execute({graph:line,options:{...options,search:'multi'}}),/does not execute/);
// An indivisible heavy tract must produce the native research-limit failure.
const heavy={...line,population:[10000000,...Array(n-1).fill(1)]};
assert.throws(()=>engine.execute({graph:heavy,options}),/3% research limit/);
// Save/Open checks assignments independently and must not start another engine.
const fetcher=async name=>{const bytes=await fs.readFile(path.join(root,name));return{ok:true,arrayBuffer:async()=>bytes.buffer.slice(bytes.byteOffset,bytes.byteOffset+bytes.length)};};
class Worker {postMessage(message){setImmediate(()=>{if(this.terminated)return;try{this.onmessage({data:message.type==='initialize'?{type:'ready'}:{type:'result',id:message.id,result:engine.execute(message.request)}});}catch(error){this.onmessage({data:{type:'error',error:error.message}});}});}terminate(){this.terminated=true;}}
manifest.catalog.structures.push('prime-factor');manifest.catalog.search_compatibility['prime-factor']=['single'];
const lab=new WasmCatalog(manifest,fetcher,()=>new Worker());
const config={name:'ApportionRegions RI',mode:'state',states:['RI'],year:'2020',chamber:'congressional',districts:null,...options,timeout_seconds:30};delete config.districts;config.districts=null;
const run=lab.submit(config);while(lab.active)await new Promise(resolve=>setTimeout(resolve,5));
assert.equal(lab.jobs.get(run.id).status,'completed');
const saved=await createLabProject(lab,run.id),reopened=new WasmCatalog(manifest,fetcher,()=>{throw new Error('Open must not execute');});
await reopened.restoreProject(parseLabProject(JSON.stringify(saved)));
const tampered=structuredClone(saved);tampered.experiment.states[0].metrics.structure_evidence.total_edge_cut_integer++;
await assert.rejects(reopened.restoreProject(tampered),/ApportionRegions evidence/);
await fs.writeFile('target/native-wasm-prime-parity.json',JSON.stringify({scope:'Prime/composite and large-prime fallback fixtures; RI/IA 2020 congressional, three weight modes; seed 42, iterations 10, tolerance 5%; existing ApportionRegions 3% research gate; portable project and evidence tampering checks.',wasm_sha256:sha(wasm),native_sha256:sha(await fs.readFile(native)),cases},null,2));
console.log('ApportionRegions: native/WASM parity, unsupported search, research-limit rejection and secure Save/Open passed.');
