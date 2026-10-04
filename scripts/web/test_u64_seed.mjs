import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {parseSeed,isSeed} from '../../web/lab/static.js';
import {validateLabConfig,verifyLabAssignments,createLabProject} from '../../web/lab/laboratory-project.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const n=100,w=10,adjacency=Array.from({length:n},(_,i)=>[i%w?i-1:-1,i%w<w-1?i+1:-1,i>=w?i-w:-1,i<n-w?i+w:-1].filter(v=>v>=0));
const graph={schema_version:1,state:'RI',year:'2020',geoids:Array.from({length:n},(_,i)=>`44001${String(i).padStart(6,'0')}`),adjacency,population:Array(n).fill(100),edges:adjacency.flatMap((ns,u)=>ns.filter(v=>v>u).map(v=>[u,v,1+u%3])),areas:Array(n).fill(100),exterior_perimeters:Array(n).fill(0),centroids:Array.from({length:n},(_,i)=>[-71+i%w*.001,41+Math.floor(i/w)*.001])};
const base={structure:'standard-bisect',weights:'geographic',search:'single',districts:4,seed:42,seeds:3,steps:5,percentile:.5,alpha_county:0,balance_tolerance:10,area_swing:1.1,iterations:10};
const config={...base,name:'Exact seed',mode:'state',states:['RI'],year:'2020',chamber:'congressional',timeout_seconds:60};
const seeds=[Number.MAX_SAFE_INTEGER,'9007199254740992','9007199254740993','9223372036854775808','18446744073709551615'];
const profiles=[{},...['multi','percentile','convergence','bisection-ensemble'].map(search=>({search,metis_objective:'volume',metis_trials:3})),...['nway','ratio-optimal','ratio-optimal-area','prime-factor','bfs-growth','centroidal-voronoi','moving-knife'].map(structure=>({structure}))];
const file='target/u64-seed-request.json';let cases=0,failures=0,projects=0;
for(const profile of profiles)for(const seed of seeds){
 const options={...base,...profile,seed};validateLabConfig({...config,...options});assert.ok(isSeed(seed));assert.equal(parseSeed(String(seed)),seed);
 const request={graph,options};
 // Native reference receives an exact JSON integer, independently of string transport.
 const raw=JSON.stringify(request).replace(/"seed":"([0-9]+)"/, '"seed":$1');await fs.writeFile(file,raw);
 const native=spawnSync('target/release/examples/execute_request.exe',[file],{encoding:'utf8',maxBuffer:16*1024*1024});
 if(native.status!==0){const message=JSON.parse(native.stderr.trim().replace(/^Error: /,''));assert.throws(()=>engine.execute(request),e=>e.message===message);failures++;continue;}
 const actual=engine.execute(request),reference=JSON.parse(native.stdout);
 assert.deepEqual(actual,reference);cases++;
 assert.equal(actual.options.seed,seed);
 verifyLabAssignments(graph,{metrics:{...actual.metrics,district_count:options.districts,districts:actual.metrics.district_metrics,balance_passed:actual.metrics.within_requested_tolerance}},actual.assignments,options);
}
let invalid=0;
for(const seed of ['', '00','01','-1','-0','+1',' 1','1 ','1e3','1.0','18446744073709551616','999999999999999999999',true,null,{},-1,1.5,9007199254740992,-0]){
 assert.throws(()=>engine.execute({graph,options:{...base,seed}}));assert.equal(isSeed(seed),false);assert.throws(()=>validateLabConfig({...config,seed}));invalid++;
}
assert.equal(parseSeed('0'),0);assert.equal(parseSeed('42'),42);assert.equal(parseSeed('18446744073709551615'),'18446744073709551615');
for(const seed of ['00','-1','18446744073709551616',true,null]){
 await fs.writeFile(file,JSON.stringify({graph,options:{...base,seed}}));assert.notEqual(spawnSync('target/release/examples/execute_request.exe',[file],{stdio:'pipe'}).status,0);
}
// Small seed strings are accepted by the transport, normalized to legacy numeric output.
assert.deepEqual(engine.execute({graph,options:{...base,seed:'42'}}),engine.execute({graph,options:base}));
if(process.argv[2]){
 const root=process.argv[2],manifest=JSON.parse(await fs.readFile(path.join(root,'catalog.json'),'utf8'));
 const fetcher=async name=>{const b=await fs.readFile(path.join(root,name));return{ok:true,arrayBuffer:async()=>b.buffer.slice(b.byteOffset,b.byteOffset+b.length)};};
 class Worker{postMessage(m){setImmediate(()=>{if(this.terminated)return;try{this.onmessage({data:m.type==='initialize'?{type:'ready'}:{type:'result',id:m.id,result:engine.execute(m.request)}});}catch(e){this.onmessage({data:{type:'error',id:m.id,error:e.message}});}});}terminate(){this.terminated=true;}}
 const lab=new WasmCatalog(manifest,fetcher,()=>new Worker());
 const key=Object.keys(manifest.graphs).find(k=>k.startsWith('RI:'))||Object.keys(manifest.graphs)[0], [code,year]=key.split(':');
 for(const search of ['single','percentile','convergence']){
  const job=lab.submit({...config,states:[code],year,districts:null,seed:'18446744073709551615',search,seeds:3,steps:10,metis_objective:'cut',metis_trials:1});while(lab.active)await new Promise(r=>setTimeout(r,5));assert.equal(lab.jobs.get(job.id).status,'completed');
  const saved=await createLabProject(lab,job.id),restore=new WasmCatalog(manifest,fetcher,()=>{throw new Error('Opening must not generate.');});const reopened=await restore.restoreProject(JSON.parse(JSON.stringify(saved)));assert.equal(reopened.config.seed,'18446744073709551615');
  assert.deepEqual(await restore.api(`/api/runs/${reopened.id}/${code}/assignments`),saved.assignments[code]);
  const bad=structuredClone(saved);bad.experiment.config.seed='18446744073709551614';await assert.rejects(restore.restoreProject(bad),/options/);
  const {project}=await lab.exportPractitionerProject(job.id,code);assert.equal(project.files.plan.provenance.producer.engine_options.seed,'18446744073709551615');
  await fs.writeFile(`target/u64-${code}-${search}.bisect`,JSON.stringify(saved));projects++;
 }
}
assert.ok(cases>0);await fs.writeFile('target/u64-seed-verification.json',JSON.stringify({exact_cases:cases,known_parity_gaps:[],matching_failures:failures,invalid_browser_values:invalid,real_state_projects:projects,raw_native_integer_reference:true},null,2));
console.log(`Exact u64 seeds: ${cases} native/WASM cases, 0 BFS parity gaps, ${failures} matching failures, ${invalid} malformed/unsafe browser values rejected, ${projects} project/export roundtrips.`);
