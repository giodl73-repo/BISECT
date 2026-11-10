import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import fsSync from 'node:fs';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {installLabelBuildApplicationControls} from '../../web/lab/label-build-application-ui.js';
import {labelBuildApplicationCase} from './fixtures/label-build-application-case.mjs';
import {nativeOutputDirectoryHarness} from './fixtures/native-output-directory.mjs';
const root=path.resolve('target/build-application/ui-'+Date.now());await fs.mkdir(root,{recursive:true});let serial=0;
const events=[],engine={execute(request){events.push(request.operation);const file=path.join(root,'request-'+serial+++'.json');fsSync.writeFileSync(file,JSON.stringify(request));const r=spawnSync(path.resolve('target/release/examples/execute_tool_request.exe'),[file],{windowsHide:true,encoding:'utf8',maxBuffer:96*1024*1024});if(r.status!==0)throw Error(r.stderr);return JSON.parse(r.stdout);}};
const fixture=await labelBuildApplicationCase(engine),harness=nativeOutputDirectoryHarness(root),folder=await harness.directory('ui-root');let picks=0,closed=0;
globalThis.showDirectoryPicker=async()=>{picks++;return folder;};
const nodes=new Map(),get=id=>{if(!nodes.has(id))nodes.set(id,{disabled:false,checked:false,textContent:''});return nodes.get(id);},document={getElementById:get};
let context={revision:1,inputs:fixture.inputs,result:null,baseline:null,artifact:{},ready:true,busy:false,operation:'run-label-build',cMetis:false},controls;const errors=[];
const changed=()=>{context.revision++;controls?.update();};
controls=installLabelBuildApplicationControls({document,getContext:()=>({...context}),setBaseline:v=>{context.baseline=v;changed();},setResult:v=>{context.result=v;changed();},setRegistry:v=>{context.inputs={...context.inputs,registry:v};context.baseline=null;changed();},setBusy:v=>{context.busy=v;controls?.update();},onError:e=>errors.push(e),createSession:()=>({execute:(...args)=>engine.execute(...args),close(){closed++;}})});
const n=id=>get('label-build-live-'+id);controls.update();assert.equal(picks,0);assert.equal(events.filter(x=>x==='run-label-build').length,0);
await n('review').onclick();assert.equal(picks,1);assert(context.baseline);assert.match(n('summary').textContent,/OUTPUT runs/);assert.equal(events.filter(x=>x==='run-label-build').length,0);assert(n('apply').disabled);n('confirm').checked=true;n('confirm').onchange();assert(!n('apply').disabled);
context.revision++;controls.update();assert(n('apply').disabled);assert.match(n('summary').textContent,/retired/);
await n('review').onclick();n('confirm').checked=true;n('confirm').onchange();await n('apply').onclick();assert.deepEqual(errors,[]);assert.equal(context.result.status,'built');assert.equal(context.result.states.length,1);assert.equal(context.result.years.length,1);assert(n('apply').disabled);assert.match(n('summary').textContent,/completed and read back/);
// Opening saved data calls only update: no picker, compute, or write authority.
const before={picks,ops:events.length,writes:harness.log.length};context={...context,revision:context.revision+1,result:structuredClone(context.result),baseline:structuredClone(context.baseline)};controls.update();assert.deepEqual({picks,ops:events.length,writes:harness.log.length},before);assert(n('apply').disabled);
// Explicit continuation replays retained states before obtaining a new grant.
await n('continue').onclick();assert.deepEqual(errors,[]);n('confirm').checked=true;n('confirm').onchange();const computations=events.filter(x=>x==='run-label-build').length;await n('apply').onclick();assert.equal(events.filter(x=>x==='run-label-build').length,computations);assert.equal(context.result.states.length,1);
await n('registry').onclick();assert(context.inputs.registry);assert.equal(context.baseline,null);assert.deepEqual(errors,[]);assert(closed>0);
n('dry-run').checked=true;n('dry-run').onchange();const dry={ops:events.filter(x=>x==='run-label-build').length,writes:harness.log.length};await n('review').onclick();assert(!n('apply').disabled);await n('apply').onclick();assert.equal(events.filter(x=>x==='run-label-build').length,dry.ops);assert.equal(harness.log.length,dry.writes);assert.match(n('summary').textContent,/Dry-run complete/);
delete globalThis.showDirectoryPicker;controls.update();assert(n('review').disabled);assert(!n('receipt').disabled);
await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({scope:'DOM controller with actual native-host core and injected directory harness. Not actual browser/WASM/OS permission acceptance.',fresh_review_before_compute:true,source_change_retires:true,confirmed_run:true,inert_open:true,verified_continuation:true,dry_run_no_partition_or_writes:true,explicit_registry_read:true,portable_fallback:true},null,2));console.log(JSON.stringify({root,passed:true}));
