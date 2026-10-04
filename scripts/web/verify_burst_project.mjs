import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {parseLabProject} from '../../web/lab/laboratory-project.js';
import {engineOptions} from '../../web/lab/static.js';
const [root,source]=process.argv.slice(2),manifest=JSON.parse(await fs.readFile(path.join(root,'catalog.json'),'utf8')),project=parseLabProject(await fs.readFile(source,'utf8')),c=project.experiment.config;
const engine=await instantiateEngine(await fs.readFile(path.join(root,'bisect_wasm.wasm'))),records=[];
for(const state of project.experiment.states){
 const graph=JSON.parse(await fs.readFile(path.join(root,manifest.graphs[`${state.code}:${c.year}`].graph_ref),'utf8'));
 for(const search of ['short-burst','short-burst-forest','short-burst-merge-split']) {
  const options={...engineOptions(c),structure:c.structure,weights:c.weights,search,districts:state.metrics.district_count,seed:c.seed,seeds:c.seeds,steps:c.steps,percentile:c.percentile,alpha_county:c.alpha_county,balance_tolerance:c.balance_tolerance,area_swing:c.area_swing,iterations:c.iterations},request={graph,options};
  const actual=engine.execute(request),file=`target/burst-real-${search}.json`;await fs.writeFile(file,JSON.stringify(request));
  const native=spawnSync('target/release/examples/execute_request.exe',[file],{encoding:'utf8',maxBuffer:16*1024*1024});assert.equal(native.status,0,native.stderr);const reference=JSON.parse(native.stdout);assert.deepEqual(actual.assignments,reference.assignments);assert.deepEqual(actual.metrics.structure_evidence,reference.metrics.structure_evidence);
  if(search===c.search)assert.deepEqual(actual.assignments,project.assignments[state.code]);
  records.push({state:state.code,search,exact_native_wasm_assignments:true,recorded_browser_assignments_match:search===c.search?true:null,maximum_deviation_percent:actual.metrics.max_deviation_percent,balance_passed:actual.metrics.within_requested_tolerance,contiguous:actual.metrics.contiguous});
 }
}
console.log(JSON.stringify({records}));
