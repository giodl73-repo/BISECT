import fs from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
import {execFile} from 'node:child_process';
import {promisify} from 'node:util';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const root=process.argv[2]||'dist/wasm-national';
const manifest=JSON.parse(await fs.readFile(path.join(root,'catalog.json'),'utf8'));
const moduleBytes=await fs.readFile(path.join(root,'bisect_wasm.wasm'));
const engine=await instantiateEngine(moduleBytes);
const native=path.resolve('target/release/examples/trace_splits'+(process.platform==='win32'?'.exe':''));
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const states=[];
for(const code of ['CA','TX']) {
 const graphPath=path.resolve(root,manifest.graphs[`${code}:2020`].graph_ref);
 const graph=JSON.parse(await fs.readFile(graphPath,'utf8'));
 const k=manifest.catalog.states.find(s=>s.code===code).years.find(y=>y.year==='2020').districts.congressional;
 const {stdout}=await promisify(execFile)(native,[graphPath,String(k)],{maxBuffer:16*1024*1024,timeout:120000});
 const reference=JSON.parse(stdout);
 const actual=engine.execute({graph,options:{structure:'standard-bisect',weights:'geographic',search:'single',districts:k,seed:42,seeds:1,steps:20,percentile:0,alpha_county:0,balance_tolerance:5,area_swing:1.1,iterations:10}});
 assert.deepEqual(actual.assignments,reference.result.assignments,code+' native/WASM assignment parity');
 for(const metric of ['population','contiguous','within_requested_tolerance','max_deviation_percent'])assert.equal(actual.metrics[metric],reference.result.metrics[metric],code+' '+metric);
 states.push({state:code,units:graph.geoids.length,assignment_sha256:sha(JSON.stringify(graph.geoids.map(id=>actual.assignments[id]))),
  contiguous:actual.metrics.contiguous,deviation:actual.metrics.max_deviation_percent,
  failing_splits:reference.splits.filter(s=>s.max_deviation_percent>s.requested_node_tolerance_percent)});
 console.log(JSON.stringify(states.at(-1)));
}
await fs.writeFile('target/native-wasm-split-parity.json',JSON.stringify({scope:'CA/TX 2020 congressional, standard geographic bisection, single seed 42, 5% tolerance, 10 iterations. Assignment parity does not imply valid balance.',wasm_sha256:sha(moduleBytes),native_sha256:sha(await fs.readFile(native)),states},null,2));
