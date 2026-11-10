import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {pathToFileURL} from 'node:url';
import {cffiEngineCase} from './fixtures/cffi-engine-case.mjs';
const root=process.env.BISECT_CFFI_EVIDENCE_ROOT??'target/cffi-integration';await fs.mkdir(root,{recursive:true});
const toolkitIndex=process.argv.indexOf('--toolkit');
if(toolkitIndex>=0&&!process.argv[toolkitIndex+1])throw new Error('--toolkit requires an existing package directory');
const toolkit=toolkitIndex>=0?path.resolve(process.argv[toolkitIndex+1]):null;
const modules=toolkit??path.resolve('web/lab'),sidecarRoot=toolkit?path.join(toolkit,'c-metis'):root+'/sidecar';
const {instantiateEngine}=await import(pathToFileURL(path.join(modules,'wasm-engine.js')));
const {loadMetisSidecar,metisSidecarImport}=await import(pathToFileURL(path.join(modules,'metis-c-sidecar.js')));
const wasm=await fs.readFile(toolkit?path.join(toolkit,'bisect_wasm.wasm'):'target/wasm32-unknown-unknown/release/bisect_wasm.wasm'),
 manifest=JSON.parse(await fs.readFile(sidecarRoot+'/metis-sidecar.json','utf8')),
 javascript=await fs.readFile(sidecarRoot+'/metis-sidecar.mjs'),binary=await fs.readFile(sidecarRoot+'/metis-sidecar.wasm');
const metisSidecar=await loadMetisSidecar({javascript,wasm:binary,manifest});
const engine=await instantiateEngine(wasm,{metisSidecar}),core=await instantiateEngine(wasm);
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const golden=JSON.parse(await fs.readFile('scripts/web/fixtures/cffi-core-baseline.json','utf8'));
assert.equal(hash(await fs.readFile('scripts/web/fixtures/cffi-engine-case.mjs')),golden.fixture_sha256);
const previousIndex=process.argv.indexOf('--previous-wasm');
if(previousIndex>=0){const previous=await fs.readFile(process.argv[previousIndex+1]);assert.equal(hash(previous),golden.wasm_sha256);assert.deepEqual((await instantiateEngine(previous)).execute(cffiEngineCase()),golden.result);}
await fs.copyFile(process.env.BISECT_CFFI_NATIVE_REFERENCE??'target/release/examples/state_signal_preset_reference.exe',root+'/native-state-reference.exe');
const nativeIdentity={path:root+'/native-state-reference.exe',sha256:hash(await fs.readFile(root+'/native-state-reference.exe'))};
assert.equal(metisSidecar.identity.runtime,'msvcrt-v1');
await assert.rejects(loadMetisSidecar({javascript,wasm:Buffer.concat([binary,Buffer.from([0])]),manifest}),/identity/);
const changedJS=Buffer.from(javascript),space=changedJS.indexOf(32);assert(space>=0);changedJS[space]=9;
await assert.rejects(loadMetisSidecar({javascript:changedJS,wasm:binary,manifest}),/identity/,'Same-size valid JavaScript must still match its hash');
const changedWasm=Buffer.from(binary);changedWasm[0]^=1;
await assert.rejects(loadMetisSidecar({javascript,wasm:changedWasm,manifest}),/identity/,'Same-size WASM corruption is rejected before compilation');
assert.throws(()=>metisSidecarImport({...metisSidecar},()=>new WebAssembly.Memory({initial:1})),/freshly loaded/);
const request=cffiEngineCase();
assert.deepEqual(engine.execute(request),golden.result,'Default metis-core whole result must remain byte-semantically unchanged');
const records=[];let nativeMatches=0,nativeFailures=0;
for(const [width,height]of [[8,8],[20,15]])for(const structure of ['standard-bisect','nway','prime-factor','ratio-optimal','ratio-optimal-area'])for(const districts of structure==='prime-factor'?[4,5]:[4])for(const search of structure==='standard-bisect'?['single','multi']:['single'])for(const weights of ['unweighted','geographic'])for(const seed of ['7','4294967297']){
  const test=cffiEngineCase(width,height);Object.assign(test.options,{structure,search,weights,seed,metis_engine:'c-ffi',metis_runtime:'msvcrt-v1'});
  if(structure==='nway')test.options.metis_ufactor=5; // actual State default, independent of final-plan tolerance
  test.options.districts=districts;
  if(structure==='prime-factor')test.options.balance_tolerance=1; // k=5 also exercises asymmetric prime fallback
  const tokens=['--state','RI','--metis-engine','c-ffi','--partition-mode','edge-weighted','--structure',structure,'--search',search,'--seeds','2','--districts',String(districts),'--balance-tolerance',String(test.options.balance_tolerance),'--niter','10','--seed',seed,'--weights-override',weights];
  // Use the actual State parser/compositor and independent native C dispatcher.
  const nativeInput={state_args:tokens,graph:test.graph};
  const file=root+'/native-request.json';await fs.writeFile(file,JSON.stringify(nativeInput));
  const native=spawnSync(path.resolve(nativeIdentity.path),[file],{encoding:'utf8',windowsHide:true,maxBuffer:32*1024*1024,timeout:120000});
  if(native.status!==0){
    const lines=native.stderr.trim().split(/\r?\n/).filter(line=>line.startsWith('Error: '));assert.equal(lines.length,1,native.stderr);
    const nativeMessage=JSON.parse(lines[0].slice(7));
    assert.throws(()=>engine.execute(test),error=>error.message===nativeMessage,native.stderr);
    assert.deepEqual(engine.execute(request),golden.result,'Failed C execution must also restore the default Rust backend');
    records.push({request:test,original_native_arguments:tokens,original_native_request_json:JSON.stringify(nativeInput),outcome:'expected-native-failure',native_stdout:native.stdout,native_stderr:native.stderr,native_exit_code:native.status});nativeFailures++;continue;
  }
  const reference=JSON.parse(native.stdout);
  const result=engine.execute(test);assert.equal(result.backend,'c-metis-msvcrt-v1');
  assert.deepEqual(result.assignments,reference.assignments,`${width}x${height}/${structure}/${search}/${weights}/${seed}`);
  assert.equal(result.metrics.weighted_boundary,reference.weighted_boundary);
  assert.equal(result.runtime_artifacts.metis_sidecar.runtime,'msvcrt-v1');
  assert.deepEqual(engine.execute(request),golden.result,'C selection must restore metis-core after execution');
  let integerCut=0;for(const [u,v,w]of test.graph.edges)if(result.assignments[test.graph.geoids[u]]!==result.assignments[test.graph.geoids[v]])integerCut+=weights==='unweighted'?1:Math.max(1,Math.trunc(w*100));
  records.push({request:test,original_native_arguments:tokens,original_native_request_json:JSON.stringify(nativeInput),native_stdout:native.stdout,native_stderr:native.stderr,native_exit_code:native.status,native:reference,result,independent_integer_cut:integerCut});nativeMatches++;
}
const cRequest=structuredClone(request);Object.assign(cRequest.options,{metis_engine:'c-ffi',metis_runtime:'msvcrt-v1'});
assert.throws(()=>core.execute(cRequest),/Load the verified/);
for(const change of [{metis_runtime:'unknown'},{metis_engine:'gpmetis'},{metis_engine:'metis-core',metis_runtime:'msvcrt-v1'},{metis_runtime:null}]){
  const bad=structuredClone(cRequest);Object.assign(bad.options,change);assert.throws(()=>engine.execute(bad));
}
const memory=new WebAssembly.Memory({initial:2}),call=metisSidecarImport(metisSidecar,()=>memory);
assert.equal(call(0,2,1,2,0,0,0,0,0,0,0,0,0),-2);
new Int32Array(memory.buffer,4,3).set([0,1,2]);new Int32Array(memory.buffer,20,2).set([1,0]);new Int32Array(memory.buffer,100,40).fill(-1);
assert.equal(call(0,2,1,2,4,20,0,0,0,0,100,300,304),1);
assert.equal(call(0,2,1,2,4,20,0,0,0,0,100,304,304),-2,'Overlapping outputs rejected');
assert.equal(call(0,2,1,2,4,131072,0,0,0,0,100,300,304),-2,'Out of bounds input rejected before C');
assert.equal(call(0,2,1,2,4,20,0,0,0,0,100,300,304),1,'ABI error does not poison subsequent C calls');
await fs.writeFile(root+'/engine-records.json',JSON.stringify(records,null,2)+'\n');
assert(records.some(r=>r.request.options.structure==='prime-factor'&&r.result));
const prepared=engine.execute({operation:'prepare-native-state-graph-run',input:{arguments:['--state','RI','--metis-engine','c-ffi','--structure','standard-bisect','--search','single','--districts','4','--balance-tolerance','25','--niter','10','--seed','7'],graph_json:JSON.stringify(request.graph),graph_source_name:'grid.json',label:'C State',chamber:'congressional',created_at:'2026-10-22T00:00:00Z'}});
const envelope=JSON.parse(prepared.prepared_request_json);
const compiled=envelope.request??envelope;
assert.equal(compiled.options.metis_engine,'c-ffi');assert.equal(compiled.options.metis_runtime,'msvcrt-v1');
assert.deepEqual(engine.execute(compiled).assignments,engine.execute(cRequest).assignments);
const yaml='name: C backend\nalgorithm:\n  structure: standard-bisect\n  engine: c-ffi\n  weights: geographic\n  search: single\n  seed: 7\n  balance_tolerance: 25\n';
const validated=engine.execute({operation:'validate-algorithm-config',yaml_text:yaml,source_name:'c.yml',profile:'open'});
assert.equal(validated.browser_engine_available,true);assert.equal(validated.browser_execution_backend,'c-metis-msvcrt-v1');
const configured=engine.execute({operation:'run-algorithm-config',input:{yaml_text:yaml,source_name:'c.yml',profile:'open',graph:request.graph,chamber:'congressional',districts:4}});
assert.equal(configured.engine_request.options.metis_runtime,'msvcrt-v1');assert.equal(configured.configuration.browser_execution_backend,'c-metis-msvcrt-v1');
assert.deepEqual(configured.result.assignments,engine.execute({...cRequest,options:{...cRequest.options,iterations:100}}).assignments);
const numericSource=JSON.stringify(cRequest).replace('"seed":"7"','"seed":18446744073709551615');
const numericRun=engine.execute({operation:'run-prepared-engine',input:{request_json:numericSource,inspect_only:false,label:'Exact C seed',chamber:'congressional',created_at:'2026-10-22T00:00:00Z'}});
assert.equal(numericRun.executed_request.options.seed,'18446744073709551615');assert.equal(numericRun.request_sha256,hash(Buffer.from(numericSource)));
assert.deepEqual(numericRun.result.assignments,engine.execute({...cRequest,options:{...cRequest.options,seed:'18446744073709551615'}}).assignments);
assert.deepEqual(engine.execute(request),golden.result);
await fs.writeFile(root+'/engine-evidence.json',JSON.stringify({schema:'bisect-cffi-engine-integration-v1',tested_toolkit:toolkit,test_source_sha256:hash(await fs.readFile('scripts/web/test_cffi_engine.mjs')),native_matches:nativeMatches,expected_native_failures:nativeFailures,native_executable:nativeIdentity,loaded_artifacts:engine.runtimeArtifacts,default_core_exact:true,core_after_c_exact:true,asset_identity_refusal:true,invalid_profiles_refused:true,abi_pointer_refusals:true,actual_browser:false,full_parity:false},null,2)+'\n');
console.log(`PASS C backend: ${nativeMatches} State compositor/native C assignment matches, ${nativeFailures} matching failures, default/post-C core equality, asset/profile/ABI refusals.`);
