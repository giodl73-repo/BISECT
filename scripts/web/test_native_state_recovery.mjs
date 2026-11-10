import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {nativeOutputRecoveryCase} from './fixtures/native-output-recovery-case.mjs';
import {nativeOutputDirectoryHarness} from './fixtures/native-output-directory.mjs';
import {reviewNativeStateInstallation,reviewNativeStateRecovery,installNativeStateOutput} from '../../web/lab/native-state-install.js';
const root=path.resolve('target/native-state-recovery');await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const fixture=await nativeOutputRecoveryCase(engine),{output}=fixture,{directory,log}=nativeOutputDirectoryHarness(root);
await fs.writeFile(root+'/native-request.json',JSON.stringify(fixture.request));
const native=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[root+'/native-request.json'],{encoding:'utf8',windowsHide:true,maxBuffer:64*1024*1024}));assert.deepEqual(fixture.run,native);
let nativeGuards=0;
// Synchronous host comparisons retain each exact live-folder observation.
const {writeFileSync}=await import('node:fs');
const guard=request=>{assert(!request.operation.startsWith('run-'));const result=engine.execute(request);const file=root+'/guard-request.json';writeFileSync(file,JSON.stringify(request));const expected=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[file],{encoding:'utf8',windowsHide:true,maxBuffer:64*1024*1024}));assert.deepEqual(result,expected);nativeGuards++;return result;};
let cases=0;
async function equalFiles(dir){for(const [name,data]of Object.entries(output.files_base64))assert.deepEqual(await fs.readFile(path.join(dir.dir,output.target_namespace,name)),Buffer.from(data,'base64'));await assert.rejects(fs.stat(path.join(dir.dir,output.target_namespace,'manifest.tmp')));}
async function interrupted(name,options={},stage='plan.rplan'){
 const dir=await directory(name,options),review=await reviewNativeStateInstallation(guard,dir,output),controller=new AbortController();
 await assert.rejects(installNativeStateOutput(guard,review,{signal:controller.signal,onProgress:p=>{if(p.path===stage)controller.abort(Error('Deliberate interruption'));}}),/interruption/);
 assert(await fs.stat(path.join(dir.dir,output.target_namespace,'manifest.tmp')));return dir;
}
for(const stage of ['plan.rplan','manifest.json']){
 const dir=await interrupted('stage',{},stage),before=log.length,review=await reviewNativeStateRecovery(guard,dir,output);assert.equal(log.length,before);assert.equal(review.recovery,true);
 const receipt=await installNativeStateOutput(guard,review);assert.equal(receipt.recovery.mode,'explicit-reviewed-same-package');assert.equal(receipt.force,false);await equalFiles(dir);assert(!log.slice(before).some(s=>s.startsWith('remove:')&&!s.endsWith('/manifest.tmp')));cases++;
}
// Recover a final manifest whose marker removal failed; never delete accepted files.
const fail={removeFail:p=>p.endsWith('manifest.tmp')},committed=await directory('marker-removal',fail),committedReview=await reviewNativeStateInstallation(guard,committed,output);
await assert.rejects(installNativeStateOutput(guard,committedReview),/marker removal/);fail.removeFail=null;
assert(await fs.stat(path.join(committed.dir,output.target_namespace,'manifest.json')));await installNativeStateOutput(guard,await reviewNativeStateRecovery(guard,committed,output));await equalFiles(committed);cases++;
const stale=await interrupted('stale'),staleReview=await reviewNativeStateRecovery(guard,stale,output),staleBefore=log.length;
await fs.writeFile(path.join(stale.dir,output.target_namespace,'plan.rplan'),'changed after review');await assert.rejects(installNativeStateOutput(guard,staleReview),/changed after review/);assert.equal(log.length,staleBefore);cases++;
const owned=await interrupted('unknown'),ownedPath=path.join(owned.dir,output.target_namespace);await fs.writeFile(path.join(ownedPath,'keep.txt'),'private notes');
await assert.rejects(reviewNativeStateRecovery(guard,owned,output),/unrelated path/);assert.equal(await fs.readFile(path.join(ownedPath,'keep.txt'),'utf8'),'private notes');cases++;
const foreign=await interrupted('wrong-package'),changed=structuredClone(output);changed.source_name='different accepted receipt';await assert.rejects(reviewNativeStateRecovery(guard,foreign,changed),/exact output package/);cases++;
const invalid=await interrupted('invalid-marker'),marker=path.join(invalid.dir,output.target_namespace,'manifest.tmp');await fs.writeFile(marker,'incomplete');await assert.rejects(reviewNativeStateRecovery(guard,invalid,output),/valid browser interruption marker/);cases++;
const bound=await interrupted('bound'),boundReview=await reviewNativeStateRecovery(guard,bound,output),beforeForgery=log.length;
await assert.rejects(installNativeStateOutput(guard,{...boundReview}),/currently selected folder/);const other=await directory('another-root');boundReview.root=other;await assert.rejects(installNativeStateOutput(guard,boundReview),/currently selected folder/);assert.equal(log.length,beforeForgery);cases++;
const changedSource=await interrupted('source-change'),changeReview=await reviewNativeStateRecovery(guard,changedSource,output);let changedGeneration=false;
await assert.rejects(installNativeStateOutput(guard,changeReview,{assertCurrent:()=>{if(changedGeneration)throw Error('Changed source generation');},onProgress:p=>{if(p.path==='manifest.json')changedGeneration=true;}}),/source generation/);assert(await fs.stat(path.join(changedSource.dir,output.target_namespace,'manifest.tmp')));await assert.rejects(fs.stat(path.join(changedSource.dir,output.target_namespace,'manifest.json')));cases++;
const pausedOptions={},late=await directory('late-change',pausedOptions),lateReview=await reviewNativeStateInstallation(guard,late,output);let lateChange=false;pausedOptions.afterClose=async p=>{if(p.endsWith('manifest.json'))lateChange=true;};
await assert.rejects(installNativeStateOutput(guard,lateReview,{assertCurrent:()=>{if(lateChange)throw Error('Source changed during final close');}}),/during final close/);assert(await fs.stat(path.join(late.dir,output.target_namespace,'manifest.tmp')));cases++;
const pre=await directory('pre-cancel'),preReview=await reviewNativeStateInstallation(guard,pre,output),aborted=new AbortController();aborted.abort(Error('Before writing'));const preLog=log.length;await assert.rejects(installNativeStateOutput(guard,preReview,{signal:aborted.signal}),/Before writing/);assert.equal(log.length,preLog);cases++;
await fs.writeFile(root+'/evidence.json',JSON.stringify({cases,native_prepared_reply_exact:true,native_guard_replies_exact:nativeGuards,recovery_partition_executions:0,fixture_source:'scripts/web/fixtures/native-output-recovery-case.mjs',journal_bound_to_exact_package:true,unknown_paths_preserved:true,imported_review_refused:true},null,2));
console.log(`PASS ${cases} native output interruption/recovery scenarios; exact native prepared reply and ${nativeGuards} native guard replies, no recovery partitions.`);
