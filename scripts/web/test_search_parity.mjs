import fs from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
import {execFile} from 'node:child_process';
import {promisify} from 'node:util';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const root=process.argv[2]||'dist/wasm-national';
const manifest=JSON.parse(await fs.readFile(path.join(root,'catalog.json'),'utf8'));
const wasm=await fs.readFile(path.join(root,'bisect_wasm.wasm'));
const native=path.resolve('target/release/examples/execute_request'+(process.platform==='win32'?'.exe':''));
const engine=await instantiateEngine(wasm),cases=[];
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
assert.equal(sha(wasm),manifest.wasm_sha256);
for(const code of ['RI','IA'])for(const search of ['multi','percentile']) {
 const graph=JSON.parse(await fs.readFile(path.join(root,manifest.graphs[`${code}:2020`].graph_ref),'utf8'));
 const k=manifest.catalog.states.find(s=>s.code===code).years.find(y=>y.year==='2020').districts.congressional;
 const request={graph,options:{structure:'standard-bisect',weights:'geographic',search,districts:k,seed:42,seeds:8,steps:20,percentile:0.5,alpha_county:0,balance_tolerance:5,area_swing:1.1,iterations:10}};
 const input=path.resolve(`target/parity-${code}-${search}.json`);
 await fs.writeFile(input,JSON.stringify(request));
 const {stdout}=await promisify(execFile)(native,[input],{maxBuffer:16*1024*1024,timeout:120000});
 const reference=JSON.parse(stdout),actual=engine.execute(request);
 assert.deepEqual(actual.assignments,reference.assignments,`${code} ${search} assignments`);
 for(const metric of ['population','contiguous','within_requested_tolerance','max_deviation_percent'])assert.equal(actual.metrics[metric],reference.metrics[metric]);
 assert.equal(Object.keys(actual.assignments).length,graph.geoids.length);
 assert.equal(actual.metrics.contiguous,true);
 if(search==='multi')assert.equal(actual.metrics.within_requested_tolerance,true);
 cases.push({state:code,search,seeds:request.options.seeds,units:graph.geoids.length,
  balanced:actual.metrics.within_requested_tolerance,deviation:actual.metrics.max_deviation_percent,
  assignment_sha256:sha(JSON.stringify(graph.geoids.map(id=>actual.assignments[id])))});
 console.log(JSON.stringify(cases.at(-1)));
}
await fs.writeFile('target/native-wasm-search-parity.json',JSON.stringify({scope:'RI/IA 2020 congressional; geographic standard bisection; multi and percentile searches, seed 42, eight seeds, percentile 0.5, tolerance 5%, iterations 10.',wasm_sha256:sha(wasm),native_sha256:sha(await fs.readFile(native)),cases},null,2));
