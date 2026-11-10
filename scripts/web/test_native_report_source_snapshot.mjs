import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {planNativeReportSources,readNativeReportSources,nativeReportSourcesFromFiles} from '../../web/lab/native-report-source-snapshot.js';
import {nativeEngineInputRequest} from '../../web/lab/native-adjacency-files.js';

const root=await fs.mkdtemp(path.resolve('target/state-report-sources/node-'));
const wasm=process.env.BISECT_TEST_WASM??'target/state-report-sources/default.wasm',bytes=await fs.readFile(wasm),engine=await instantiateEngine(bytes),sha=b=>createHash('sha256').update(b).digest('hex');
const referencePath=process.env.BISECT_POPULATION_NATIVE_REFERENCE??'target/population-source/native-1793112696653683700/evidence.json',reference=JSON.parse(await fs.readFile(referencePath,'utf8'));
const verify=async ref=>assert.equal(sha(await fs.readFile(ref.path)),ref.sha256,ref.path+' changed');
await verify(reference.native);for(const row of reference.outcomes)for(const ref of row.files)await verify(ref);
const error=(name,message)=>Object.assign(Error(message),{name});
const effects=[];
class ReadOnlyDirectory {
 constructor(folder,base=folder,hook=null){this.folder=path.resolve(folder);this.base=path.resolve(base);this.hook=hook;this.name=path.basename(folder);this.kind='directory';assert(this.folder===this.base||this.folder.startsWith(this.base+path.sep));}
 child(name){assert(name&&!['.','..'].includes(name)&&!/[\\/]/.test(name));const child=path.resolve(this.folder,name);assert(child.startsWith(this.base+path.sep));return child;}
 async stat(name){const p=this.child(name);try{const s=await fs.lstat(p);if(s.isSymbolicLink())throw error('NotAllowedError','Test links refused');return {p,s};}catch(e){if(e.code==='ENOENT')throw error('NotFoundError','Missing source');throw e;}}
 async getDirectoryHandle(name,options){assert.equal(options?.create,false);effects.push('read-directory');const {p,s}=await this.stat(name);if(!s.isDirectory())throw error('TypeMismatchError','Source is not a directory');return new ReadOnlyDirectory(p,this.base,this.hook);}
 async getFileHandle(name,options){assert.equal(options?.create,false);effects.push('read-file');const {p,s}=await this.stat(name);if(!s.isFile())throw error('TypeMismatchError','Source is not a file');const hook=this.hook;return {kind:'file',name,async getFile(){const stat=await fs.stat(p);return {size:stat.size,async arrayBuffer(){const data=await fs.readFile(p);await hook?.(p);return Uint8Array.from(data).buffer;}};}};}
}
const totalCase=reference.outcomes.find(r=>r.name==='total'),planPin=totalCase.files.find(f=>f.path.endsWith('/plan.rplan')),baseline=JSON.parse(await fs.readFile(planPin.path,'utf8')),ids=baseline.plan.units.unit_ids;
const outcomes=[];
for(const row of reference.outcomes){
 const command={arguments_json:JSON.stringify(row.arguments.slice(1))},caseFolder=path.dirname(row.files.find(f=>f.path.endsWith('/stdout.log')).path),selected=new ReadOnlyDirectory(caseFolder);
 const snapshot=await readNativeReportSources(engine,command,selected,{unitIds:ids});
 assert.equal(snapshot.receipt.cvap_missing_warning,row.missing_cvap_warning,row.name);
 assert.equal(snapshot.receipt.requested_population_source,row.requested);
 assert.equal(snapshot.live_folder_authority,false);assert.deepEqual(structuredClone(snapshot),snapshot);
 if(row.expected_native_failure){assert.equal(snapshot.receipt.status,'native-load-failure');assert.equal(snapshot.receipt.report_load_failure_kind,row.presence==='directory'?'directory':'csv');}
 else{
  assert.equal(snapshot.receipt.status,'prepared-report-counts');const context=JSON.parse(await fs.readFile(row.files.find(f=>f.path.endsWith('/context.rctx')).path,'utf8'));
  assert.deepEqual(snapshot.receipt.demographic_context_json===null?null:JSON.parse(snapshot.receipt.demographic_context_json),context.demographics??null,row.name+' native report counts');
  assert.equal(snapshot.receipt.balance_population_source,'selected-graph-populations');assert.equal(snapshot.receipt.graph_binding_verified,false);
 }
 await fs.writeFile(path.join(root,row.name+'.json'),JSON.stringify(snapshot));outcomes.push({name:row.name,native_report_counts_or_failure_match:true,warning_matches:true,requested_metadata_matches:true,native_expected_failure:row.expected_native_failure});
}
const command={arguments_json:JSON.stringify(totalCase.arguments.slice(1))},plan=await planNativeReportSources(engine,command);
const compatibility=[];
const compatFolder=path.join(root,'compat-native'),compatSource=path.join(compatFolder,...plan.report_candidates[0].split('/')),lower=path.join(compatFolder,...plan.report_candidates.at(-1).split('/'));
await fs.mkdir(path.dirname(compatSource),{recursive:true});
const compatibleCsv='GEOID,total_vap,minority_vap\n'+ids[0]+',80,10\n'+ids[1]+',bad,bad\n'+ids[0]+',20,6\n44001000999,8,2\n';
await fs.writeFile(compatSource,compatibleCsv);await fs.writeFile(path.join(root,'compat-original.csv'),compatibleCsv);await fs.writeFile(lower,'GEOID,total_vap,minority_vap\n'+ids.map(id=>id+',100,50\n').join(''));
const nativeEnv={...process.env};delete nativeEnv.BISECT_MANIFEST;
for(const malformed of [false,true]){
 const name=malformed?'first-existing-malformed':'compatibility-defaults',nativeArgs=structuredClone(totalCase.arguments),outputs=path.join(compatFolder,malformed?'failed-outputs':'outputs');nativeArgs[nativeArgs.indexOf('--output-dir')+1]=outputs;
 if(malformed)await fs.writeFile(compatSource,'');
 const native=spawnSync(path.resolve(reference.native.path),nativeArgs,{cwd:compatFolder,env:nativeEnv,encoding:'utf8',windowsHide:true,timeout:120000});assert(!native.error,native.error?.message);await fs.writeFile(path.join(root,name+'.stdout.log'),native.stdout);await fs.writeFile(path.join(root,name+'.stderr.log'),native.stderr);
 const selected=await readNativeReportSources(engine,{arguments_json:JSON.stringify(nativeArgs.slice(1))},new ReadOnlyDirectory(compatFolder),{unitIds:ids});
 if(malformed){assert.equal(native.status,1);assert(native.stderr.includes('VAP demographics load failed'));assert.equal(selected.receipt.status,'native-load-failure');assert.equal(selected.receipt.selected_report_path,plan.report_candidates[0]);}
 else{assert.equal(native.status,0);const context=JSON.parse(await fs.readFile(path.join(outputs,'2020/states/rhode_island/context.rctx'),'utf8'));assert.deepEqual(JSON.parse(selected.receipt.demographic_context_json),context.demographics);assert.deepEqual(context.demographics,{total_vap:[20,0,0,0],minority_vap:[6,0,0,0]});assert.equal(selected.receipt.missing_units_defaulted_to_zero,2);const originalContext=JSON.parse(await fs.readFile(totalCase.files.find(f=>f.path.endsWith('/context.rctx')).path,'utf8'));assert.deepEqual(context.populations,originalContext.populations);}
 await fs.writeFile(path.join(root,name+'.snapshot.json'),JSON.stringify(selected));compatibility.push({name,native_exit:native.status,arguments:nativeArgs,native_counts_or_failure_match:true,lower_priority_source_not_substituted:true});
}
assert.equal(plan.observed,false);assert.equal(plan.cvap_presence_observed,undefined);
await assert.rejects(()=>nativeReportSourcesFromFiles(engine,command,[]),/Explicitly choose/);
const portable=await nativeReportSourcesFromFiles(engine,command,[],{acceptUnselectedAsMissing:true,unitIds:ids});assert.equal(portable.receipt.observation_origin,'explicit-file-bundle');assert.equal(portable.receipt.demographic_context_json,null);
await assert.rejects(()=>readNativeReportSources(engine,command,JSON.parse(JSON.stringify({kind:'directory'}))),/live native working folder/);
const first=plan.report_candidates[0],last=plan.report_candidates.at(-1);
const conflict=await nativeReportSourcesFromFiles(engine,command,[{relative_path:first,bytes:new Uint8Array()},{relative_path:last,bytes:new TextEncoder().encode('GEOID,total_vap,minority_vap\n'+ids[0]+',10,5\n')}],{acceptUnselectedAsMissing:true,unitIds:ids});
assert.equal(conflict.receipt.status,'native-load-failure');assert.equal(conflict.receipt.selected_report_path,first);
const utf8=await nativeReportSourcesFromFiles(engine,command,[{relative_path:first,bytes:Uint8Array.of(0xff)}],{acceptUnselectedAsMissing:true,unitIds:ids});assert.equal(utf8.receipt.report_load_failure_kind,'utf8');
await assert.rejects(()=>nativeReportSourcesFromFiles(engine,command,[{relative_path:'../outside.csv',bytes:new Uint8Array()}],{acceptUnselectedAsMissing:true}),/Unexpected/);
const duplicate=[{relative_path:first,bytes:new Uint8Array()},{relative_path:first,bytes:new Uint8Array()}];await assert.rejects(()=>nativeReportSourcesFromFiles(engine,command,duplicate,{acceptUnselectedAsMissing:true}),/duplicate/);
const mutation=path.join(root,'mutation'),file=path.join(mutation,...first.split('/'));await fs.mkdir(path.dirname(file),{recursive:true});const csv='GEOID,total_vap,minority_vap\n'+ids[0]+',10,5\n';await fs.writeFile(file,csv);
let changed=false;await assert.rejects(()=>readNativeReportSources(engine,command,new ReadOnlyDirectory(mutation,mutation,async p=>{if(p===file&&!changed){changed=true;await fs.unlink(file);}}),{unitIds:ids}),/changed while loading/);
await fs.writeFile(file,csv);const controller=new AbortController();await assert.rejects(()=>readNativeReportSources(engine,command,new ReadOnlyDirectory(mutation,mutation,async()=>controller.abort(Error('Cancelled source load'))),{unitIds:ids,signal:controller.signal}),/Cancelled/);
const changedCommand=structuredClone(command);await assert.rejects(()=>readNativeReportSources(engine,changedCommand,new ReadOnlyDirectory(mutation,mutation,async()=>{const args=JSON.parse(changedCommand.arguments_json);args[args.indexOf('--seed')+1]='32';changedCommand.arguments_json=JSON.stringify(args);}),{unitIds:ids}),/State flags.*changed/);
const denied={kind:'directory',async getDirectoryHandle(){throw error('NotAllowedError','Permission refused');}};await assert.rejects(()=>readNativeReportSources(engine,command,denied),/Permission refused/);
const oversized=path.join(root,'oversized'),large=path.join(oversized,...first.split('/'));await fs.mkdir(path.dirname(large),{recursive:true});await fs.writeFile(large,'');await fs.truncate(large,9*1024*1024);let oversizedReads=0;
await assert.rejects(()=>readNativeReportSources(engine,command,new ReadOnlyDirectory(oversized,oversized,()=>oversizedReads++)),/exceeds 8 MiB/);assert.equal(oversizedReads,0);
const forged={execute(){return {...structuredClone(plan),required_observation_paths:['data/2021/demographics/secret.csv']};}};await assert.rejects(()=>readNativeReportSources(forged,command,new ReadOnlyDirectory(mutation)),/Invalid|differ/);
const fixture=JSON.parse(await fs.readFile('scripts/web/fixtures/native-state-source.json','utf8')),source=nativeEngineInputRequest(fixture.nativeAdjacencyInputs,fixture.nativeAdjacencySettings).input;
for(const [flag,value] of [['--population-source','cvap'],['--state-name','Alias']])assert.throws(()=>engine.execute({operation:'prepare-native-adjacency-state-run',input:{arguments:['--state','RI','--year','2020','--adjacency','fixture.adj.bin','--districts','2','--metis-engine','metis-core','--structure','nway',flag,value],source,label:'guard_probe',chamber:'congressional',created_at:'2026-10-27T00:00:00Z'}}),new RegExp(flag));
assert(effects.every(e=>e.startsWith('read-')));
const evidence={status:'passed-source-snapshot-preparation',wasm:{path:wasm,sha256:sha(bytes),byte_length:bytes.length},native_reference:{path:referencePath,sha256:sha(await fs.readFile(referencePath))},outcomes,compatibility_cases:compatibility,read_only_windows_filesystem_facade:true,complete_observations_required:true,portable_absence_explicit:true,first_existing_failure_not_skipped:true,changed_deleted_source_refused:true,changed_flags_refused:true,post_read_cancel_refused:true,permissions_not_treated_as_absence:true,imported_metadata_grants_no_authority:true,oversized_file_refused_before_read:true,direct_name_and_population_compiler_guards_preserved:true,source_scope_guard:true,execution_integration:false,actual_browser_ui:false,scope:'Eleven retained actual native CLI outcomes match report-source snapshot preparation on this exact WASM. Two further actual native probes compare duplicate/numeric/missing-unit defaults, unchanged population vectors and first-existing precedence/failure against a valid lower-priority source. Folder reads use an owned readonly Windows facade; live browser permission grants and execution/project/UI integration remain open.'};
await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify(evidence,null,2));console.log(JSON.stringify({root,status:evidence.status,cases:outcomes.length}));
