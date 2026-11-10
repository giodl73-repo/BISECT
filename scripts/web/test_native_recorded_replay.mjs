// Frozen actual-native references plus current host/WASM adapters. No native path is read from a fixture.
import fs from 'node:fs/promises';import path from 'node:path';import assert from 'node:assert/strict';import{spawnSync}from'node:child_process';
import{instantiateEngine}from'../../web/lab/wasm-engine.js';import{loadMetisSidecar}from'../../web/lab/metis-c-sidecar.js';
import{compareCommandReplies}from'./native_command_comparison.mjs';import{loadNativeReplayCorpus,nativeBinaryWidth,sha256}from'./fixtures/native-recorded-replay.mjs';

const args=process.argv.slice(2),option=name=>{const at=args.indexOf(name);return at<0?null:args[at+1];};
const wasmPath=path.resolve(option('--wasm')||process.env.BISECT_TEST_WASM||'target/wasm32-unknown-unknown/release/bisect_wasm.wasm'),hostPath=option('--host');assert(hostPath,'--host requires the current bisect-wasm execute_tool_request native example');
const sidecarDirectory=option('--c-sidecar'),mode=sidecarDirectory?'c':'core',parent=path.resolve(option('--output')||'target/native-recorded-replay');await fs.mkdir(parent,{recursive:true});const root=await fs.mkdtemp(path.join(parent,mode+'-'));
const wasm=await fs.readFile(wasmPath),hostBytes=await fs.readFile(hostPath),hostWidth=nativeBinaryWidth(hostBytes);let metisSidecar=null;
if(sidecarDirectory){const manifest=JSON.parse(await fs.readFile(path.join(sidecarDirectory,'metis-sidecar.json'),'utf8'));metisSidecar=await loadMetisSidecar({manifest,javascript:await fs.readFile(path.join(sidecarDirectory,manifest.javascript.file)),wasm:await fs.readFile(path.join(sidecarDirectory,manifest.wasm.file))});}
const engine=await instantiateEngine(wasm,{metisSidecar}),fixtures=await loadNativeReplayCorpus(),evidence={schema_version:'bisect-native-replay-gate-v1',status:'running',mode,wasm_sha256:sha256(wasm),host_sha256:sha256(hostBytes),host_width:hostWidth,runtime_artifacts:engine.runtimeArtifacts,cases:[],failures:[],scope:'Maintained selected-byte regression against frozen actual Windows-native assignment references and explicitly labelled synthetic metadata. Current host/core whole replies allow only the strict measured State pointer-width annotation. Trusted C is explicit. Not fresh native filesystem, real-state/capacity, browser or full release acceptance.'};
const save=()=>fs.writeFile(path.join(root,'evidence.json'),JSON.stringify(evidence,null,2));await save();
function compare(actual,native){
 if(actual.schema_version!=='bisect-native-recorded-replay-v1'){assert.deepEqual(actual,native);return;}
 if(!actual.prepared||!native.prepared){assert.deepEqual(actual,native);return;}
 compareCommandReplies({operation:'prepare-native-state-graph-run'},actual.prepared,native.prepared,hostWidth);
 const expected=structuredClone(native);expected.prepared.native_command.target_pointer_width=32;assert.deepEqual(actual,expected);
}
async function run(id,request,expected,reference=null){
 const file=path.join(root,id+'-request.json');await fs.writeFile(file,JSON.stringify(request));const native=spawnSync(hostPath,[file],{encoding:'utf8',windowsHide:true,maxBuffer:32*1024*1024});await fs.writeFile(path.join(root,id+'-host.stdout'),native.stdout);await fs.writeFile(path.join(root,id+'-host.stderr'),native.stderr);assert.equal(native.status,0,native.stderr);
 const host=JSON.parse(native.stdout),actual=engine.execute(request);await fs.writeFile(path.join(root,id+'-wasm.json'),JSON.stringify(actual));assert.equal(actual.status,expected,actual.error);
 if(mode==='core')compare(actual,host);else if(actual.source_check)assert.deepEqual(actual.source_check,host.source_check);else assert.deepEqual(actual,host);
 assert.equal(actual.recorded_paths_opened,false);
 if(actual.execution_performed){
  assert(reference);assert.equal(actual.status,'match');assert.deepEqual(actual.comparison.regenerated_assignments,reference.assignments);assert.deepEqual(actual.comparison.reference_assignments,reference.assignments);assert.equal(actual.replay.export_error,null);
  const r=actual.replay.result;assert(r.metrics?.within_requested_tolerance??r.within_requested_tolerance);assert(r.metrics?.contiguous??r.connected.every(Boolean));
  const exported=actual.replay.export.document.plan;assert.deepEqual(Object.fromEntries(exported.units.unit_ids.map((unit,i)=>[unit,exported.assignment[i]+1])),reference.assignments);
 }
 evidence.cases.push({id,status:actual.status,execution_performed:actual.execution_performed});await save();return actual;
}
for(const fixture of fixtures){try{
 const actual=await run(fixture.id,fixture.request,fixture[mode==='c'?'expected_c':'expected_core'],fixture.native_reference);
 if(fixture.id==='native64-metadata'){assert.equal(actual.binding.recipe_json,fixture.native_reference.recipe_json);assert.equal(actual.binding.recipe_digest,fixture.native_reference.recipe_digest);assert(actual.binding.native64_readonly.unsigned_integers.some(r=>r.pointer==='/state/algorithm/seeds/Multi/seeds'&&r.decimal==='18446744073709551615'));assert.equal(actual.execution_performed,false);}
 if(fixture.id==='c-nway-effective-c'){assert.equal(actual.backend_resolution.engine,'CFfi');if(mode==='core')assert.equal(actual.execution_performed,false);else assert.deepEqual(actual.runtime_artifacts,engine.runtimeArtifacts);}
 if(fixture.id==='area-empty-refusal'){assert.equal(actual.execution_performed,false);assert.equal(actual.prepared,null);assert(actual.error.includes('requires an Area constraint'));}
 }catch(error){evidence.failures.push({id:fixture.id,message:error.message,stack:error.stack});await save();}}
const base=fixtures.find(c=>c.id==='core-nway-empty');
const mutations=[['changed-source',r=>{const s=r.input.sources.sources[0],b=Buffer.from(s.base64,'base64');b[0]^=1;s.base64=b.toString('base64');},'source-check-failed'],['changed-recorded-seed',r=>{const p=r.input.sources.prechecks,m=JSON.parse(p.manifest_json);m.native_replay_recipe.state.algorithm.metis.seed='18';p.manifest_json=JSON.stringify(m);},'source-check-failed'],['missing-plan',r=>{delete r.input.sources.prechecks.files_base64['plans/original/plan.rplan'];},'source-check-failed'],['unsafe-source-name',r=>{r.input.sources.sources[0].name='../outside.bin';},'source-check-failed']];
for(const[id,mutate,status]of mutations){try{const request=structuredClone(base.request);mutate(request);const actual=await run(id,request,status);assert.equal(actual.execution_performed,false);}catch(error){evidence.failures.push({id,message:error.message,stack:error.stack});}}
for(const rename of [true,false]){const id=rename?'safe-source-labels':'success-after-refusals';try{const request=structuredClone(base.request);if(rename)request.input.sources.sources.forEach((s,i)=>s.name=`renamed-${i}.selected`);await run(id,request,'match',base.native_reference);}catch(error){evidence.failures.push({id,message:error.message,stack:error.stack});}}
evidence.status=evidence.failures.length?'failed-maintained-native-replay-gate':'passed-maintained-native-replay-gate';await save();console.log(JSON.stringify({root,status:evidence.status,cases:evidence.cases.length,failures:evidence.failures}));if(evidence.failures.length)process.exitCode=1;
