import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {nativeOutputRecoveryCase} from './fixtures/native-output-recovery-case.mjs';
import {nativeOutputDirectoryHarness} from './fixtures/native-output-directory.mjs';
import {executeNativeStateOutput,nativeStateOutputSource} from '../../web/lab/native-state-output.js';
import {reviewNativeStateDataReset,resetNativeStateData} from '../../web/lab/native-state-install.js';
import {createProject} from '../../web/lab/project.js';
const root=path.resolve('target/native-state-data-reset');await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile(process.env.BISECT_TEST_WASM||'target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const fixture=await nativeOutputRecoveryCase(engine),outputs={};
for(const layout of ['state','labeled'])outputs[layout]=await executeNativeStateOutput(engine,{source:nativeStateOutputSource(fixture.run),version:'vReset',layout});
await fs.copyFile('target/release/bisect.exe',root+'/native-bisect-reference.exe');
const executable=path.resolve(root+'/native-bisect-reference.exe'),hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const nativeBinary={path:'target/native-state-data-reset/native-bisect-reference.exe',sha256:hash(await fs.readFile(executable))};
const {directory,log}=nativeOutputDirectoryHarness(root),records=[];let comparisons=0,refusals=0;
async function write(target,name,bytes){const p=path.resolve(target,name);assert(p.startsWith(root+path.sep),p);await fs.mkdir(path.dirname(p),{recursive:true});await fs.writeFile(p,bytes);}
async function fileTree(target){const files={};async function visit(dir,prefix=''){for(const entry of await fs.readdir(dir,{withFileTypes:true})){if(entry.isDirectory())await visit(path.join(dir,entry.name),prefix+entry.name+'/');else{assert(entry.isFile());files[prefix+entry.name]=(await fs.readFile(path.join(dir,entry.name))).toString('base64');}}}try{await visit(target);}catch(e){if(e.code!=='ENOENT')throw e;}return files;}
async function seed(target,output,{manifest=true,incomplete=false}={}){for(const [name,data]of Object.entries(output.files_base64)){if(name==='manifest.json'&&!manifest)continue;await write(path.join(target,output.target_namespace),name,Buffer.from(data,'base64'));}await write(path.join(target,output.target_namespace),'keep.txt','preserve root notes');await write(path.join(target,output.target_namespace),'maps/extra.svg','preserve optional map');await write(path.join(target,output.target_namespace),'data/old.txt','remove only with reset');if(incomplete)await write(path.join(target,output.target_namespace),'manifest.tmp','native interrupted target');}
const execute=request=>engine.execute(request);
for(const layout of ['state','labeled'])for(const manifest of [false,true])for(const incomplete of [false,true])for(const force of [false,true])for(const reset of [false,true]){
 const output=outputs[layout],id=[layout,manifest,incomplete,force,reset].join('-'),nativeRoot=await directory('native-'+id),browserRoot=await directory('browser-'+id);await seed(nativeRoot.dir,output,{manifest,incomplete});await seed(browserRoot.dir,output,{manifest,incomplete});
 const nativeVersion=path.resolve(nativeRoot.dir,output.version),missing=path.join(nativeRoot.dir,'missing.adj.bin');assert(nativeVersion.startsWith(root+path.sep));assert(missing.startsWith(root+path.sep));
 const args=['state','--state','WA','--year','2020','--version',output.version,'--output-dir',nativeVersion,'--adjacency',missing,'--districts','2','--metis-engine','metis-core','--balance-tolerance','25',...(layout==='labeled'?['--label',output.label]:[]),...(force?['--force']:[]),...(reset?['--reset']:[])];
 const initial=await fileTree(path.join(nativeRoot.dir,output.target_namespace)),native=spawnSync(executable,args,{encoding:'utf8',windowsHide:true,maxBuffer:16*1024*1024,timeout:30000});assert.equal(native.status,1,native.stdout+native.stderr);
 // Native preflight must win over reset and the deliberately missing graph.
 const expectedRefusal=incomplete?'incomplete':layout==='labeled'&&manifest&&!force?'collision':null;
 if(expectedRefusal==='incomplete')assert.match(native.stderr,/Incomplete plan/);else if(expectedRefusal==='collision')assert.match(native.stderr,/already exists/);else assert.match(native.stderr,/adjacency load failed/);
 const nativeAfter=await fileTree(path.join(nativeRoot.dir,output.target_namespace));
 if(expectedRefusal){assert.deepEqual(nativeAfter,initial);await assert.rejects(reviewNativeStateDataReset(execute,browserRoot,output,{force}),expectedRefusal==='incomplete'?/Incomplete plan/:/already exists/);refusals++;}
 else{const review=await reviewNativeStateDataReset(execute,browserRoot,output,{force});assert.deepEqual(await fileTree(path.join(browserRoot.dir,output.target_namespace)),initial);if(reset){const receipt=await resetNativeStateData(execute,review);assert.equal(receipt.status,'completed');assert.equal(receipt.partition_execution_performed,false);}assert.deepEqual(await fileTree(path.join(browserRoot.dir,output.target_namespace)),nativeAfter);}
 // Native proceeds to recreate data/ before failing graph load. This separate
 // browser reset action creates nothing and never starts that subsequent Run.
 const expectedFiles=Object.fromEntries(Object.entries(initial).filter(([name])=>!(reset&&!expectedRefusal&&name.startsWith('data/'))));assert.deepEqual(nativeAfter,expectedFiles);
 records.push({layout,manifest,incomplete,force,reset,native_arguments:args,native_stdout:native.stdout,native_stderr:native.stderr,native_exit_code:native.status,initial_files:initial,native_after_files:nativeAfter,expected_refusal:expectedRefusal,browser_after_files:await fileTree(path.join(browserRoot.dir,output.target_namespace)),comparison:'exact file bytes; native recreates empty data/ before its later graph-load failure, browser standalone action creates no directories'});comparisons++;
}
const output=outputs.state;
const stale=await directory('stale');await seed(stale.dir,output);const staleReview=await reviewNativeStateDataReset(execute,stale,output);await write(path.join(stale.dir,output.target_namespace),'new.txt','changed');const staleBefore=await fileTree(stale.dir);await assert.rejects(resetNativeStateData(execute,staleReview),/changed after review/);assert.deepEqual(await fileTree(stale.dir),staleBefore);refusals++;
const authority=await directory('authority');await seed(authority.dir,output);const authorityReview=await reviewNativeStateDataReset(execute,authority,output);await assert.rejects(resetNativeStateData(execute,{...authorityReview}),/currently selected folder/);const other=await directory('different-root');authorityReview.root=other;await assert.rejects(resetNativeStateData(execute,authorityReview),/currently selected folder/);refusals+=2;
const cancelled=await directory('cancelled');await seed(cancelled.dir,output);const cancelledReview=await reviewNativeStateDataReset(execute,cancelled,output),controller=new AbortController();controller.abort(new Error('reset cancelled'));const cancelledBefore=await fileTree(cancelled.dir);await assert.rejects(resetNativeStateData(execute,cancelledReview,{signal:controller.signal}),/reset cancelled/);assert.deepEqual(await fileTree(cancelled.dir),cancelledBefore);refusals++;
const failed=await directory('failed',{removeFail:p=>p.endsWith(path.sep+'data')});await seed(failed.dir,output);const failedReview=await reviewNativeStateDataReset(execute,failed,output),failedBefore=await fileTree(failed.dir);await assert.rejects(resetNativeStateData(execute,failedReview),error=>error.native_data_reset?.status==='interrupted'&&error.native_data_reset.data_removal_attempted);assert.deepEqual(await fileTree(failed.dir),failedBefore);refusals++;
const changed=await directory('revision');await seed(changed.dir,output);const changedReview=await reviewNativeStateDataReset(execute,changed,output),changedBefore=await fileTree(changed.dir);await assert.rejects(resetNativeStateData(execute,changedReview,{assertCurrent:()=>{throw Error('project changed');}}),/project changed/);assert.deepEqual(await fileTree(changed.dir),changedBefore);refusals++;
const absent=await directory('absent'),absentBefore=log.length,noData=await reviewNativeStateDataReset(execute,absent,output);assert.equal((await resetNativeStateData(execute,noData)).removed_entries.length,0);assert.equal(log.length,absentBefore);
const dataFile=await directory('data-file');await write(path.join(dataFile.dir,output.target_namespace),'data','not a directory');await assert.rejects(reviewNativeStateDataReset(execute,dataFile,output),/not removed/);refusals++;
assert.throws(()=>engine.execute({operation:'prepare-native-state-graph-run',input:{arguments:['--state','WA','--metis-engine','metis-core','--districts','2','--reset'],graph_json:JSON.stringify(fixture.run.executed_request.graph),graph_source_name:'reset-guard.json',label:'Recovery',chamber:'congressional',created_at:'2026-10-23T00:00:00Z'}}),/--reset/);
await fs.writeFile(root+'/cli-records.json',JSON.stringify(records,null,2)+'\n');
const input={source:nativeStateOutputSource(fixture.run),version:'vReset',layout:'state'};
await fs.writeFile(root+'/source.bisect',JSON.stringify(createProject({name:'Native data reset',files:{nativeStateOutputInputs:input,stateCommandInputs:{arguments_json:JSON.stringify(['--state','WA','--reset','--force'])}},operation:'prepare-native-state-output',constraints:[],result:outputs.state,lastOperation:'prepare-native-state-output'})));
await fs.writeFile(root+'/output.json',JSON.stringify(outputs.state));
await fs.writeFile(root+'/evidence.json',JSON.stringify({schema:'bisect-native-state-data-reset-tests-v1',wasm:engine.artifactIdentity,native_executable:nativeBinary,native_cli_filesystem_comparisons:comparisons,refusals,shared_native_incomplete_then_collision_guards:true,reset_removes_only_data_files:true,force_without_reset_preserves_existing_files:true,original_cli_stdout_stderr_and_trees_retained:true,native_empty_directory_recreation_disclosed:true,reset_partitions:0,full_flag_execution_wiring:false,full_parity:false},null,2)+'\n');
console.log(`PASS native data reset: ${comparisons} actual CLI filesystem comparisons, ${refusals} refusals, exact preserved files and data-only removal; no browser reset partition.`);
