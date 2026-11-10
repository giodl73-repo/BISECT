// Rebind the original RI k=2 / VT k=1 fixtures to the current packaged C engine.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import fs from 'node:fs/promises';
import path from 'node:path';
import {cffiEngineCase} from './fixtures/cffi-engine-case.mjs';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {loadMetisSidecar} from '../../web/lab/metis-c-sidecar.js';
import {preparedEngineRequest} from '../../web/lab/prepared-engine-files.js';
import {preparedStateJobFromResult,executePreparedStateJob} from '../../web/lab/prepared-state-job.js';
import {appendStateRunJob} from '../../web/lab/state-run-batch.js';
import {createProject} from '../../web/lab/project.js';

const [rawWasm,toolkit,originalEvidence,outDir]=process.argv.slice(2);
const out=path.resolve(outDir),read=async f=>JSON.parse(await fs.readFile(f));
const sha=b=>createHash('sha256').update(b).digest('hex');
const write=async (name,data)=>fs.writeFile(path.join(out,name),JSON.stringify(data,null,2)+'\n',{flag:'wx'});
const pinned=await read(originalEvidence);
const exe=path.resolve(pinned.native_executable.path),exeHash=sha(await fs.readFile(exe));
assert.equal(exeHash,pinned.native_executable.sha256);
const c=path.join(toolkit,'c-metis'),manifest=await read(path.join(c,'metis-sidecar.json'));
const sidecar=await loadMetisSidecar({manifest,javascript:await fs.readFile(path.join(c,manifest.javascript.file)),
  wasm:await fs.readFile(path.join(c,manifest.wasm.file))});
const engine=await instantiateEngine(await fs.readFile(rawWasm),{metisSidecar:sidecar});
await fs.mkdir(out);
const native={},expected={};let batch={jobs:null,workers:2};
for(const [state,districts] of [['RI',2],['VT',1]]){
  const request=cffiEngineCase();
  Object.assign(request.options,{districts,metis_engine:'c-ffi',metis_runtime:'msvcrt-v1'});
  request.graph.state=state;
  if(state==='VT')request.graph.geoids=request.graph.geoids.map(id=>'50'+id.slice(2));
  const state_args=['--state',state,'--metis-engine','c-ffi','--partition-mode','edge-weighted',
    '--structure','standard-bisect','--search','single','--seeds','2','--districts',String(districts),
    '--balance-tolerance','25','--niter','10','--seed','7','--weights-override','geographic'];
  const nativeInput=`${state}-native-request.json`;
  await write(nativeInput,{state_args,graph:request.graph});
  const processResult=spawnSync(exe,[path.join(out,nativeInput)],{encoding:'utf8',windowsHide:true,timeout:120000});
  assert.ifError(processResult.error);
  assert.equal(processResult.status,0,processResult.stderr);
  const {assignments}=JSON.parse(processResult.stdout);
  await write(`${state}-native-reply.json`,{stdout:processResult.stdout,stderr:processResult.stderr,exit_code:processResult.status});
  const district_populations=Array(districts).fill(0);
  assert.equal(Object.keys(assignments).length,64);
  for(const [i,id] of request.graph.geoids.entries()){
    assert(assignments[id]>=1&&assignments[id]<=districts);district_populations[assignments[id]-1]+=request.graph.population[i];
  }
  assert(district_populations.every(n=>n>0));
  const input={source:{name:`explicit-c-${state}.json`,text:JSON.stringify(request)},settings:{
    label:`Explicit C ${state}`,chamber:'congressional',created_at:'2026-10-23T00:00:00Z',inspect_only:true}};
  const job=await preparedStateJobFromResult(input,engine.execute(preparedEngineRequest(input)));
  const result=executePreparedStateJob(engine,job).prepared;
  assert.equal(result.result.backend,'c-metis-msvcrt-v1');
  assert.deepEqual(result.result.assignments,assignments);
  batch=appendStateRunJob(batch,job);
  native[state]={assignments,districts,district_populations};
  assert.equal(Boolean(result.runtime_artifacts),districts>1);
  expected[state]={assignments,request_sha256:job.request_sha256,runtime:result.runtime_artifacts??null,
    loaded_worker_runtime:engine.runtimeArtifacts};
}
const project_file='synthetic-c-batch.bisect';
await write(project_file,createProject({name:'Original explicit C RI/VT state batch',operation:'run-state-batch',
  files:{stateRunBatchInputs:batch,stateBatchInputs:{year:'2020',states:['RI','VT'],manifest:null}},
  constraints:[],result:null,lastOperation:null}));
await write('expected.json',{project_file,wasm_sha256:engine.artifactIdentity.sha256,states:expected});
await write('native-evidence.json',{status:'passed-original-synthetic-native-c-reference',
  native_exe_sha256:exeHash,states:native});
