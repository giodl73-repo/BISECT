// Turn two native-verified retained graphs into an explicit C batch project.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {loadMetisSidecar} from '../../web/lab/metis-c-sidecar.js';
import {preparedEngineRequest} from '../../web/lab/prepared-engine-files.js';
import {preparedStateJobFromResult,executePreparedStateJob} from '../../web/lab/prepared-state-job.js';
import {appendStateRunJob} from '../../web/lab/state-run-batch.js';
import {createProject} from '../../web/lab/project.js';

const [rawWasm, toolkit, nativeRoot] = process.argv.slice(2);
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const read=async file=>JSON.parse(await fs.readFile(file));
const reference=await read(path.join(nativeRoot,'native-evidence.json'));
assert.equal(reference.status,'passed-actual-c-native-state-on-retained-batch-graphs');
const base=path.join(toolkit,'c-metis');
const manifest=await read(path.join(base,'metis-sidecar.json'));
const sidecar=await loadMetisSidecar({manifest,javascript:await fs.readFile(path.join(base,manifest.javascript.file)),
  wasm:await fs.readFile(path.join(base,manifest.wasm.file))});
const engine=await instantiateEngine(await fs.readFile(rawWasm),{metisSidecar:sidecar});
const expected={};let batch={jobs:null,workers:2};
for(const state of ['RI','NH']){
  const native=reference.states[state], graphBytes=await fs.readFile(path.join(nativeRoot,state,'graph.json'));
  assert.equal(sha(graphBytes),native.graph_sha256);assert.equal(native.effective_backend,'c-ffi');
  const metis=native.native_algorithm.metis;
  const options={structure:'nway',weights:'geographic',search:'single',districts:2,seed:native.seed,seeds:1,
    steps:200,percentile:0,alpha_county:0,balance_tolerance:0.5,area_swing:1.1,iterations:metis.niter,
    metis_engine:'c-ffi',metis_runtime:'msvcrt-v1',metis_ufactor:metis.ufactor};
  const request={graph:JSON.parse(graphBytes),options};
  const input={source:{name:`${state}-explicit-c.json`,text:JSON.stringify(request)},settings:{
    label:`Explicit C ${state}`,chamber:'congressional',created_at:'2026-11-06T00:00:00Z',inspect_only:true}};
  const review=engine.execute(preparedEngineRequest(input));
  const job=await preparedStateJobFromResult(input,review);
  const result=executePreparedStateJob(engine,job).prepared;
  assert.equal(result.result.backend,'c-metis-msvcrt-v1');
  assert.deepEqual(result.result.assignments,native.assignments);
  expected[state]={assignments:native.assignments,runtime:result.runtime_artifacts,request_sha256:job.request_sha256,
    district_populations:native.district_populations};
  batch=appendStateRunJob(batch,job);
}
const packet=createProject({name:'Explicit C batch on retained Rhode Island and New Hampshire tracts',
  files:{stateRunBatchInputs:batch,stateBatchInputs:{year:'2020',states:['RI','NH'],manifest:null}},
  operation:'run-state-batch',constraints:[],result:null,lastOperation:null});
await fs.writeFile(path.join(nativeRoot,'real-c-batch.bisect'),JSON.stringify(packet),{flag:'wx'});
await fs.writeFile(path.join(nativeRoot,'expected.json'),JSON.stringify({status:'passed-real-c-batch-fixture-native-comparison',
  states:expected,wasm_sha256:engine.artifactIdentity.sha256},null,2),{flag:'wx'});
