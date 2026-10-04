import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
import {createLabProject,parseLabProject} from '../../web/lab/laboratory-project.js';

const root=process.argv[2]||'dist/wasm-standalone-chains',year=process.argv[3]||'2020';
const manifest=JSON.parse(await fs.readFile(path.join(root,'catalog.json'),'utf8'));
const available=Object.keys(manifest.graphs).filter(key=>key.endsWith(`:${year}`)).map(key=>key.split(':')[0]);
const preferred=['RI','IA','NC'].filter(code=>available.includes(code));
const states=[...preferred,...available.filter(code=>!preferred.includes(code))].slice(0,3);
assert.ok(states.length,'No prepared inputs for requested year.');
const bytes=await fs.readFile(path.join(root,'bisect_wasm.wasm'));
assert.equal(createHash('sha256').update(bytes).digest('hex'),manifest.wasm_sha256);
const engine=await instantiateEngine(bytes),requests=new Map(),results=new Map();
const fetcher=async ref=>{const data=await fs.readFile(path.join(root,ref));return{ok:true,arrayBuffer:async()=>data.buffer.slice(data.byteOffset,data.byteOffset+data.length)};};
let executions=0;
class EngineWorker {
  postMessage(message){setImmediate(()=>{
    if(this.terminated)return;
    try {
      if(message.type==='initialize')this.onmessage({data:{type:'ready'}});
      else {
        executions++;const key=message.request.graph.state,result=engine.execute(message.request);
        requests.set(key,message.request);results.set(key,result);
        this.onmessage({data:{type:'result',id:message.id,result,memoryBytes:engine.memoryBytes()}});
      }
    } catch(error){this.onmessage({data:{type:'error',id:message.id,error:error.message}});}
  });}
  terminate(){this.terminated=true;}
}
const records=[];
for(const search of ['forest-recom','merge-split','flip','bisection-ensemble']){
  const config={name:`Real input ${search}`,mode:'national',states,year,chamber:'congressional',districts:null,structure:'standard-bisect',weights:'geographic',search,seed:42,seeds:8,steps:20,percentile:0,alpha_county:0,balance_tolerance:0.5,area_swing:1.1,iterations:100,timeout_seconds:300};
  const lab=new WasmCatalog(manifest,fetcher,()=>new EngineWorker());
  const run=lab.submit(config);while(lab.active)await new Promise(resolve=>setTimeout(resolve,5));
  const job=await lab.api(`/api/runs/${run.id}`);assert.equal(job.status,'completed',JSON.stringify(job.states));
  const project=parseLabProject(JSON.stringify(await createLabProject(lab,run.id)));
  const before=executions;
  const openedLab=new WasmCatalog(manifest,fetcher,()=>{throw new Error('Open must not execute the engine.');});
  const opened=await openedLab.restoreProject(project);
  assert.equal(executions,before);assert.equal(openedLab.worker,null);assert.deepEqual(opened.config,config);
  if(search==='flip'){
    // Older projects recorded the requested refinement count even though the
    // native flip initializer always used 100 iterations. Keep those readable.
    const legacy=structuredClone(project);legacy.experiment.config.iterations=1;
    for(const state of legacy.experiment.states){
      state.metrics.structure_evidence=null;
      for(const key of ['requested_options','recorded_options','effective_config'])state.metrics[key].iterations=1;
    }
    await openedLab.restoreProject(parseLabProject(JSON.stringify(legacy)));
    const withoutLedger=structuredClone(project);
    for(const state of withoutLedger.experiment.states)state.metrics.structure_evidence=null;
    await openedLab.restoreProject(parseLabProject(JSON.stringify(withoutLedger)));
    assert.equal(executions,before);
  }
  for(const state of opened.states){
    const requestFile=`target/real-chain-${state.code}-${search}.json`;
    await fs.writeFile(requestFile,JSON.stringify(requests.get(state.code)));
    const native=spawnSync('target/release/examples/execute_request.exe',[requestFile],{encoding:'utf8',maxBuffer:32*1024*1024});
    assert.equal(native.status,0,native.stderr);const reference=JSON.parse(native.stdout);
    assert.deepEqual(results.get(state.code).assignments,reference.assignments);
    assert.deepEqual(results.get(state.code).metrics,reference.metrics);
    assert.deepEqual(project.assignments[state.code],reference.assignments);
    const map=await openedLab.api(`/api/runs/${opened.id}/${state.code}/map`);
    assert.equal(map.features.length,state.metrics.units);
    assert.ok(map.features.every(feature=>feature.properties.district===reference.assignments[feature.properties.geoid]));
    records.push({state:state.code,search,year,districts:state.metrics.district_count,units:state.metrics.units,exact_native_wasm:true,engine_executed_on_open:false,contiguous:state.metrics.contiguous,balance_passed:state.metrics.balance_passed,maximum_deviation_percent:state.metrics.max_deviation_percent});
    openedLab.assets.delete(manifest.geometries[`${state.code}:${year}`]);
  }
  lab.destroy();
}
await fs.writeFile('target/chains-real-input-parity.json',JSON.stringify({wasm_sha256:manifest.wasm_sha256,records},null,2)+'\n');
console.log(`Real standalone chains: ${records.length} exact native/WASM cases, independent Save/Open/map verification; ${records.filter(r=>!r.balance_passed).length} balance misses, ${records.filter(r=>!r.contiguous).length} disconnected plans.`);
