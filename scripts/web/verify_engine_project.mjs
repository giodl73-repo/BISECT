import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {parseLabProject,verifyLabAssignments} from '../../web/lab/laboratory-project.js';
import {engineOptions} from '../../web/lab/static.js';
const [root,source]=process.argv.slice(2);assert.ok(root&&source,'Usage: verify_engine_project.mjs <catalog directory> <saved project>');
const manifest=JSON.parse(await fs.readFile(path.join(root,'catalog.json'),'utf8')),project=parseLabProject(await fs.readFile(source,'utf8')),c=project.experiment.config;
const bytes=await fs.readFile(path.join(root,'bisect_wasm.wasm')),hash=createHash('sha256').update(bytes).digest('hex');assert.equal(hash,manifest.wasm_sha256);
const engine=await instantiateEngine(bytes),records=[];
for(const state of project.experiment.states){
 if(!state.metrics)continue;
 const entry=manifest.graphs[`${state.code}:${c.year}`],graphBytes=await fs.readFile(path.join(root,entry.graph_ref));assert.equal(createHash('sha256').update(graphBytes).digest('hex'),manifest.assets[entry.graph_ref]);const graph=JSON.parse(graphBytes);
 const options={...engineOptions(c),structure:c.structure,weights:c.weights,search:c.search,districts:state.metrics.district_count,seed:c.seed,seeds:c.seeds,steps:c.steps,percentile:c.percentile,alpha_county:c.alpha_county,balance_tolerance:c.balance_tolerance,area_swing:c.area_swing,iterations:c.iterations},request={graph,options};
 const actual=engine.execute(request),file=`target/replay-${state.code}-${c.search}.json`;await fs.writeFile(file,JSON.stringify(request));
 const native=spawnSync('target/release/examples/execute_request.exe',[file],{encoding:'utf8',maxBuffer:32*1024*1024});assert.equal(native.status,0,native.stderr);const reference=JSON.parse(native.stdout);assert.deepEqual(actual.assignments,reference.assignments);assert.deepEqual(actual.metrics.structure_evidence,reference.metrics.structure_evidence);assert.deepEqual(actual.assignments,project.assignments[state.code]);
 verifyLabAssignments(graph,{metrics:{...actual.metrics,district_count:options.districts,districts:actual.metrics.district_metrics,balance_passed:actual.metrics.within_requested_tolerance}},actual.assignments,options);
 records.push({state:state.code,search:c.search,exact_native_wasm_assignments:true,recorded_browser_assignments_match:true,maximum_deviation_percent:actual.metrics.max_deviation_percent,balance_passed:actual.metrics.within_requested_tolerance,contiguous:actual.metrics.contiguous});
}
assert.ok(records.length,'No completed states to replay.');console.log(JSON.stringify({wasm_sha256:hash,records,execution:'Fresh replay, separate from project Open verification.'}));
