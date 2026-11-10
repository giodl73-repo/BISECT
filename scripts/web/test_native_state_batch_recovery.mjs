import {createNativeCommandComparator} from './native_command_comparison.mjs';
let strictComparator;
function compareNative(request,actual,expected){strictComparator??=createNativeCommandComparator(root,'target/wasm32-unknown-unknown/release/bisect_wasm.wasm');strictComparator.compare(request,actual,expected);}
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {writeFileSync} from 'node:fs';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {nativeOutputBatchRecoveryCase} from './fixtures/native-output-batch-recovery-case.mjs';
import {nativeOutputDirectoryHarness} from './fixtures/native-output-directory.mjs';
import {reviewNativeStateInstallation,installNativeStateOutput} from '../../web/lab/native-state-install.js';
import {reviewNativeStateBatchRecovery,installNativeStateBatchOutput} from '../../web/lab/native-state-batch-output.js';
const root=path.resolve('target/native-state-batch-recovery');await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
let nativeReplies=0,partitions=0;const originals=[];
const execute=request=>{
 if(request.operation==='run-prepared-engine'&&request.input.inspect_only===false)partitions++;
 const reply=engine.execute(request),filename=root+'/native-request.json';writeFileSync(filename,JSON.stringify(request));
 const expected=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[filename],{encoding:'utf8',windowsHide:true,maxBuffer:64*1024*1024}));compareNative(request,reply,expected);originals.push({request,native:expected,wasm:reply});nativeReplies++;return reply;
};
const fixture=await nativeOutputBatchRecoveryCase({artifactIdentity:engine.artifactIdentity,execute}),batch=fixture.output,baselinePartitions=partitions;
assert.equal(baselinePartitions,3);assert.equal(batch.outputs.length,3);
const {directory,log}=nativeOutputDirectoryHarness(root),targetPath=(dir,i,name='')=>path.join(dir.dir,batch.outputs[i].target_namespace,name);
async function setup(name,options={}){
 const dir=await directory(name,options);await installNativeStateOutput(execute,await reviewNativeStateInstallation(execute,dir,batch.outputs[0]));
 const interrupted=await reviewNativeStateInstallation(execute,dir,batch.outputs[1]),controller=new AbortController();await assert.rejects(installNativeStateOutput(execute,interrupted,{signal:controller.signal,onProgress:p=>{if(p.path==='plan.rplan')controller.abort(Error('Synthetic interruption'));}}),/interruption/);return dir;
}
async function exact(dir){for(const item of batch.outputs){for(const [name,data]of Object.entries(item.files_base64))assert.deepEqual(await fs.readFile(path.join(dir.dir,item.target_namespace,name)),Buffer.from(data,'base64'));await assert.rejects(fs.stat(path.join(dir.dir,item.target_namespace,'manifest.tmp')));}}
let cases=0;
const success=await setup('mixed');await fs.writeFile(targetPath(success,0,'keep.txt'),'private retained notes');
const priorReceipt=await fs.readFile(targetPath(success,0,'browser/install-receipt.json')),before=log.length,review=await reviewNativeStateBatchRecovery(execute,success,batch);assert.equal(log.length,before);assert.deepEqual(review.display.recovery_summary,{retained:1,resumed:1,installed:1});
const result=await installNativeStateBatchOutput(execute,review);assert.equal(result.status,'completed');assert.deepEqual(result.summary,{total:3,completed:3,failed:0,retained:1,resumed:1,installed:1});await exact(success);assert.deepEqual(await fs.readFile(targetPath(success,0,'browser/install-receipt.json')),priorReceipt);assert.equal(await fs.readFile(targetPath(success,0,'keep.txt'),'utf8'),'private retained notes');
assert(!log.slice(before).some(entry=>entry.includes(batch.outputs[0].target_namespace+'/')));cases++;
// When all are already complete, only a fresh batch receipt is written.
const allBefore=log.length,all=await reviewNativeStateBatchRecovery(execute,success,batch);assert.equal(all.display.files_to_write.length,1);assert.equal((await installNativeStateBatchOutput(execute,all)).summary.retained,3);assert(log.slice(allBefore).every(entry=>entry.includes('/browser/output-install-')));cases++;
// A forged old receipt saying "completed" cannot skip absent state targets.
const forged=await setup('forged-checkpoint');await fs.mkdir(path.join(forged.dir,batch.version,'browser'),{recursive:true});await fs.writeFile(path.join(forged.dir,batch.version,'browser/output-install-'+crypto.randomUUID()+'.json'),JSON.stringify({status:'completed',summary:{completed:3}}));const forgedReview=await reviewNativeStateBatchRecovery(execute,forged,batch);assert.deepEqual(forgedReview.targets.map(t=>t.action),['retained','resumed','installed']);cases++;
const stale=await setup('changed-complete'),staleReview=await reviewNativeStateBatchRecovery(execute,stale,batch),staleLog=log.length;await fs.appendFile(targetPath(stale,0,'plan.rplan'),'altered');await assert.rejects(installNativeStateBatchOutput(execute,staleReview),/neither an exact completed/);assert.equal(log.length,staleLog);cases++;
const changedPartial=await setup('changed-partial'),partialReview=await reviewNativeStateBatchRecovery(execute,changedPartial,batch),partialLog=log.length;await fs.writeFile(targetPath(changedPartial,1,'notes.txt'),'keep');await assert.rejects(installNativeStateBatchOutput(execute,partialReview),/unrelated path/);assert.equal(log.length,partialLog);cases++;
const bound=await setup('bound'),boundReview=await reviewNativeStateBatchRecovery(execute,bound,batch),boundLog=log.length;await assert.rejects(installNativeStateBatchOutput(execute,{...boundReview}),/currently selected folder/);boundReview.root=await directory('different-root');await assert.rejects(installNativeStateBatchOutput(execute,boundReview),/currently selected folder/);assert.equal(log.length,boundLog);cases++;
const cancel=await setup('cancel-recovery'),cancelReview=await reviewNativeStateBatchRecovery(execute,cancel,batch),controller=new AbortController();let cancelled;
try{await installNativeStateBatchOutput(execute,cancelReview,{signal:controller.signal,onProgress:p=>{if(p.state==='VT'&&p.path==='plan.rplan')controller.abort(Error('Cancel recovery'));}});}catch(error){cancelled=error;}
assert(cancelled);assert.equal(cancelled.installation_receipt.status,'cancelled');assert.equal(cancelled.installation_receipt.summary.retained,1);assert(await fs.stat(targetPath(cancel,1,'manifest.tmp')));const retry=await reviewNativeStateBatchRecovery(execute,cancel,batch);await installNativeStateBatchOutput(execute,retry);await exact(cancel);cases++;
const lateOptions={},late=await setup('late-mutation',lateOptions),lateReview=await reviewNativeStateBatchRecovery(execute,late,batch);lateOptions.afterClose=async filename=>{if(filename===targetPath(late,2,'manifest.json'))await fs.appendFile(targetPath(late,0,'plan.rplan'),'other program');};let failed;
try{await installNativeStateBatchOutput(execute,lateReview);}catch(error){failed=error;}
assert(failed);assert.equal(failed.installation_receipt.status,'interrupted');assert.equal(failed.installation_receipt.states[0].status,'failed');assert.equal(failed.installation_receipt.summary.failed,1);cases++;
const checkpoint=await setup('checkpoint-collision'),checkpointReview=await reviewNativeStateBatchRecovery(execute,checkpoint,batch),checkpointLog=log.length;await fs.mkdir(path.dirname(path.join(checkpoint.dir,checkpointReview.checkpoint_path)),{recursive:true});await fs.writeFile(path.join(checkpoint.dir,checkpointReview.checkpoint_path),'another writer');await assert.rejects(installNativeStateBatchOutput(execute,checkpointReview),/receipt target changed/);assert.equal(log.length,checkpointLog);cases++;
assert.equal(partitions,baselinePartitions);await fs.writeFile(root+'/native-replies.json',JSON.stringify(originals));await fs.writeFile(root+'/evidence.json',JSON.stringify({cases,native_replies:nativeReplies,fixture_partitions:baselinePartitions,recovery_partitions:partitions-baselinePartitions,synthetic_states:3,years:['2020','2010','2000'],retained_target_no_writes:true,completed_bytes_reverified:true,forged_checkpoint_ignored:true,final_mutation_revokes_completion:true},null,2));
console.log(`PASS ${cases} batch recovery scenarios, ${nativeReplies} native replies, three source-generated State runs, no recovery partitions.`);
