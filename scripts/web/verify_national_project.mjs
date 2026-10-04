import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {parseLabProject} from '../../web/lab/laboratory-project.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';

const root=process.argv[2]||'dist/wasm-national';
const source=process.argv[3]||'target/wasm-national-browser-project.bisect';
const manifest=JSON.parse(await fs.readFile(path.join(root,'catalog.json'),'utf8'));
const moduleBytes=await fs.readFile(path.join(root,'bisect_wasm.wasm'));
const moduleHash=createHash('sha256').update(moduleBytes).digest('hex');
assert.equal(moduleHash,manifest.wasm_sha256);
const project=parseLabProject(await fs.readFile(source,'utf8'));
const {config,states,status}=project.experiment;
assert.equal(status,'completed');assert.equal(states.length,50);
assert.deepEqual([...config.states].sort(),manifest.catalog.states.map(s=>s.code).sort());
for(const [key,value]of Object.entries({mode:'national',year:'2020',chamber:'congressional',districts:null,structure:'standard-bisect',weights:'geographic',search:'multi',seed:42,seeds:8,steps:200,percentile:0,alpha_county:0,balance_tolerance:5,area_swing:1.1,iterations:10,timeout_seconds:300}))assert.equal(config[key],value,key);
const fetcher=async name=>{
  const bytes=await fs.readFile(path.join(root,name));
  return {ok:true,arrayBuffer:async()=>bytes.buffer.slice(bytes.byteOffset,bytes.byteOffset+bytes.length)};
};
const lab=new WasmCatalog(manifest,fetcher,()=>{throw new Error('Opening a national project must not execute the engine.');});
const restored=await lab.restoreProject(project);
assert.equal(lab.worker,null);assert.equal(restored.restored_from_project,true);
let districts=0,units=0,population=0;
const evidence=[];
for(const state of restored.states){
  const m=state.metrics;
  assert.equal(state.status,'completed');assert.equal(m.contiguous,true);assert.equal(m.balance_passed,true);
  assert.ok(m.max_deviation_percent<=5);assert.equal(m.engine_provenance.wasm_sha256,moduleHash);
  assert.ok(m.districts.every(d=>d.units>0&&d.components===1));
  assert.equal(m.district_count,lab.districtCount(state.code,config));
  assert.ok(Number.isSafeInteger(m.wasm_memory_bytes)&&m.wasm_memory_bytes<=128*1024*1024);
  const map=await lab.api(`/api/runs/${restored.id}/${state.code}/map`);
  assert.equal(map.features.length,m.units);
  assert.ok(map.features.every(f=>f.properties.district===project.assignments[state.code][f.properties.geoid]));
  lab.assets.delete(manifest.geometries[`${state.code}:2020`]);
  districts+=m.district_count;units+=m.units;population+=m.population;
  evidence.push({state:state.code,districts:m.district_count,units:m.units,population:m.population,maximum_deviation_percent:m.max_deviation_percent,contiguous:true,balance_passed:true,map_units:map.features.length,recorded_memory_bytes:m.wasm_memory_bytes,recorded_seconds:state.elapsed_seconds});
}
assert.equal(districts,435);assert.equal(units,84208);
const report={source:path.resolve(source),wasm_sha256:moduleHash,states:50,districts,units,population,maximum_deviation_percent:Math.max(...evidence.map(s=>s.maximum_deviation_percent)),engine_executed_on_open:false,scope:'Independent hash, allocation, assignment metric, connectivity, balance and complete geometry join checks of a browser-downloaded 50-state project. Recorded execution provenance and timing remain unverified claims; this is not an optimality or election certificate.',evidence};
await fs.writeFile('target/wasm-national-browser-verification.json',JSON.stringify(report,null,2));
console.log(JSON.stringify({states:50,districts,units,population,maximum_deviation_percent:report.maximum_deviation_percent,engine_executed_on_open:false,wasm_sha256:moduleHash}));
