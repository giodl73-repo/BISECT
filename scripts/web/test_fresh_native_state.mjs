import {verifyFreshNativeFollowups} from './fixtures/fresh-native-followup-checks.mjs';
import fs from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {loadNativeUnitIdentityCorpus} from './fixtures/native-unit-identity.mjs';
import {freshNativeStateInput,freshNativeStateRefusals} from './fixtures/fresh-native-state.mjs';
import {nativeBinaryWidth} from './fixtures/native-recorded-replay.mjs';
import {compareCommandReplies} from './native_command_comparison.mjs';
const argv=process.argv.slice(2),arg=name=>{const i=argv.indexOf(name);assert(i>=0,'Supply '+name);return path.resolve(argv[i+1]);};
const wasm=arg('--wasm'),host=arg('--host'),parent=arg('--output');await fs.mkdir(parent,{recursive:true});
const root=await fs.mkdtemp(path.join(parent,'fresh-')),bytes=await fs.readFile(wasm),hostBytes=await fs.readFile(host),width=nativeBinaryWidth(hostBytes);
const engine=await instantiateEngine(bytes),rows=await loadNativeUnitIdentityCorpus(),sha=b=>createHash('sha256').update(b).digest('hex');
const evidence={status:'running',wasm_sha256:sha(bytes),host_sha256:sha(hostBytes),cases:[],scope:'Complete twelve raw-source ordinary State cases. Exact native DistrictPlan and physical context; explicitly new browser source/request hashes. No original RCTX/plan supplied to fresh operations. Both binaries use metis-core; no C-sidecar, DOM, fine, capacity or release claim.'};
const save=()=>fs.writeFile(path.join(root,'evidence.json'),JSON.stringify(evidence,null,2));await save();
try{
 for(const row of rows){
  const input=freshNativeStateInput(row),folder=path.join(root,row.name);await fs.mkdir(folder);
  for(const [name,operation]of [['prepare','prepare-fresh-native-state-run'],['run','run-fresh-native-state']]){
   const request={operation,input},file=path.join(folder,name+'.request.json');await fs.writeFile(file,JSON.stringify(request));
   const native=spawnSync(host,[file],{windowsHide:true,maxBuffer:64*1024*1024});
   await fs.writeFile(path.join(folder,name+'.host.stdout'),native.stdout??'');await fs.writeFile(path.join(folder,name+'.host.stderr'),native.stderr??'');
   assert.equal(native.status,0,native.error?.message??native.stderr?.toString());
   const expected=JSON.parse(native.stdout),actual=engine.execute(request);await fs.writeFile(path.join(folder,name+'.wasm.json'),JSON.stringify(actual));
   assert.equal(actual.execution_performed,name==='run');assert.equal(actual.recorded_paths_opened,false);
   assert.equal(actual.input_sha256,expected.input_sha256);
   compareCommandReplies({operation:'prepare-native-state-graph-run'},actual.prepared,expected.prepared,width);
   if(name==='run'){
    verifyFreshNativeFollowups(actual);
    assert.equal(actual.export_error,null);assert.deepEqual(actual.export.document.plan,JSON.parse(row.document_json).plan);
    const context=JSON.parse(row.context_json);
    for(const field of ['units','graph','populations','subdivisions','demographics','geometry'])assert.deepEqual(actual.export.context[field],context[field],row.name+':'+field);
    assert.equal(actual.export.fresh_native_state_binding.original_context_loaded,false);
    assert(!Object.hasOwn(actual.export.context.source_hashes,'native_replay_recipe'));
    assert.deepEqual(actual.export.document.provenance.source_hashes,actual.export.context.source_hashes);
    if(row.name==='absent-index-coi')assert.equal(actual.result.metrics.native_coi_source_evidence.changed_edges,0);
    if(row.name==='sparse-index-coi')assert.equal(actual.result.metrics.native_coi_source_evidence.changed_edges,1);
   }
  }
  evidence.cases.push({name:row.name,status:'passed'});await save();
 }
 evidence.refusals=[];
 const refusals=freshNativeStateRefusals(freshNativeStateInput(rows.find(row=>row.name==='tract')));
 assert.equal(refusals.length,15);
 for(const refusal of refusals){
  const request={operation:'run-fresh-native-state',input:refusal.input},file=path.join(root,refusal.name+'.request.json');await fs.writeFile(file,JSON.stringify(request));
  const native=spawnSync(host,[file],{windowsHide:true,maxBuffer:64*1024*1024});
  await fs.writeFile(path.join(root,refusal.name+'.host.stdout'),native.stdout??'');await fs.writeFile(path.join(root,refusal.name+'.host.stderr'),native.stderr??'');
  assert.equal(native.status,1,native.error?.message);assert.equal(native.stdout.length,0);assert(native.stderr.toString().toLowerCase().includes(refusal.message.toLowerCase()));
  let message;try{engine.execute(request);}catch(error){message=error.message;}
  await fs.writeFile(path.join(root,refusal.name+'.wasm-error.json'),JSON.stringify({message}));
  assert.equal(typeof message,'string',refusal.name);assert(message.toLowerCase().includes(refusal.message.toLowerCase()));
  evidence.refusals.push(refusal.name);await save();
 }
 evidence.status='passed-twelve-fresh-native-state-cases-and-fifteen-refusals';await save();console.log(JSON.stringify({root,status:evidence.status,cases:evidence.cases.length,refusals:evidence.refusals.length}));
}catch(error){evidence.status='failed';evidence.error=String(error);await save();throw error;}
