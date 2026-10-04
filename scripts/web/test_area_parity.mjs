import fs from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
import {execFile} from 'node:child_process';
import {promisify} from 'node:util';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const root=process.argv[2]||'dist/wasm-national';
const wasm=await fs.readFile(process.argv[3]||'target/wasm32-unknown-unknown/release/bisect_wasm.wasm');
const native=path.resolve('target/release/examples/execute_request'+(process.platform==='win32'?'.exe':''));
const engine=await instantiateEngine(wasm), cases=[];
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const manifest=JSON.parse(await fs.readFile(path.join(root,'catalog.json'),'utf8'));
const requests=[];
for(const swing of [1.1,1.9])requests.push({name:'two-district-fixture-'+swing,request:JSON.parse(await fs.readFile(`target/area-fixture-${swing}.json`,'utf8'))});
for(const state of ['RI','IA']){
  const graph=JSON.parse(await fs.readFile(path.join(root,manifest.graphs[`${state}:2020`].graph_ref),'utf8'));
  const districts=manifest.catalog.states.find(s=>s.code===state).years.find(y=>y.year==='2020').districts.congressional;
  requests.push({name:state,request:{graph,options:{structure:'ratio-optimal-area',weights:'geographic',search:'multi',districts,seed:42,seeds:32,steps:20,percentile:0,alpha_county:0,balance_tolerance:5,area_swing:1.1,iterations:20}}});
}
function near(actual,expected,name){assert.ok(Math.abs(actual-expected)<=Math.max(1e-8,Math.abs(expected)*1e-10),name);}
for(const {name,request}of requests){
  const input=path.resolve(`target/area-parity-${name}.json`);await fs.writeFile(input,JSON.stringify(request));
  const {stdout}=await promisify(execFile)(native,[input],{maxBuffer:16*1024*1024,timeout:120000});
  const reference=JSON.parse(stdout), actual=engine.execute(request);
  assert.deepEqual(actual.assignments,reference.assignments,name+' complete assignments');
  assert.deepEqual(actual.options,reference.options,name+' options');
  assert.deepEqual(actual.metrics.district_metrics,reference.metrics.district_metrics,name+' district metrics');
  for(const field of ['population','contiguous','within_requested_tolerance'])assert.equal(actual.metrics[field],reference.metrics[field]);
  for(const field of Object.keys(reference.metrics.root_split)){
    const expected=reference.metrics.root_split[field], got=actual.metrics.root_split[field];
    if(typeof expected==='number')near(got,expected,name+' '+field);else assert.equal(got,expected,name+' '+field);
  }
  assert.equal(actual.metrics.root_split.area_within_requested_swing,true);assert.equal(actual.metrics.contiguous,true);assert.equal(actual.metrics.within_requested_tolerance,true);
  const graph=request.graph, split=actual.metrics.root_split;
  let pop=0,area=0;for(let i=0;i<graph.geoids.length;i++)if(actual.assignments[graph.geoids[i]]<=split.left_districts){pop+=graph.population[i];area+=graph.areas[i];}
  assert.equal(split.population_left,pop);near(split.area_left_m2,area,name+' independent area');
  assert.ok(pop<=split.population_total*split.population_target_left*1.001+1e-6);
  assert.ok(split.population_total-pop<=split.population_total*(1-split.population_target_left)*1.001+1e-6);
  assert.ok(area<=split.area_total_m2*0.5*request.options.area_swing+1e-6&&split.area_total_m2-area<=split.area_total_m2*0.5*request.options.area_swing+1e-6);
  cases.push({case:name,units:graph.geoids.length,root_split:split,maximum_population_deviation_percent:actual.metrics.max_deviation_percent,assignment_sha256:sha(JSON.stringify(graph.geoids.map(id=>actual.assignments[id])))});
  console.log(JSON.stringify(cases.at(-1)));
}
await fs.writeFile('target/native-wasm-area-parity.json',JSON.stringify({scope:'AreaSection root population/land-area constraints; fixture area-swing variation and RI/IA 2020 congressional Multi, seed42,32seeds,5%final tolerance,20iterations. Exact assignments and district metrics; floating costs checked numerically. Not all states or solver optimality.',wasm_sha256:sha(wasm),native_sha256:sha(await fs.readFile(native)),cases},null,2));
