import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {selectNativeWorkspaceFiles,selectNativeWorkspaceDirectory,selectNativeWorkspaceArchive,nativeWorkspaceFiles,retireNativeWorkspaceSource,decodeNativeWorkspaceArchive} from '../../web/lab/native-workspace-source.js';
import {nativeFolderSelection,nativeFolderRequest,queueNativeFolderSources,NATIVE_FOLDER_DEFAULTS} from '../../web/lab/native-folder-files.js';
import {stateRunJobs} from '../../web/lab/state-run-batch.js';
import {executePreparedNativeJob} from '../../web/lab/prepared-native-job.js';
import {preparedStateRunMap} from '../../web/lab/prepared-state-job.js';
import {executeNativeStateOutput,nativeStateOutputSource,nativeStateOutputZip} from '../../web/lab/native-state-output.js';
import {reviewNativeStateInstallation,installNativeStateOutput,reviewNativeStateRecovery} from '../../web/lab/native-state-install.js';
import {storedZipBytes} from '../../web/lab/reproducibility-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {nativeWorkspaceCase} from './fixtures/native-workspace-case.mjs';
import {nativeOutputDirectoryHarness} from './fixtures/native-output-directory.mjs';
const root=path.resolve('target/native-workspace-source');await fs.mkdir(root,{recursive:true});
const fixture=nativeWorkspaceCase(),{files,graphInputs,stem}=fixture,folder=path.join(root,'input');await fs.mkdir(folder,{recursive:true});
for(const [name,data]of files){await fs.mkdir(path.dirname(path.join(folder,name)),{recursive:true});await fs.writeFile(path.join(folder,name),data);}
const stored=storedZipBytes([...files],{maxSourceBytes:64*1024*1024});await fs.writeFile(root+'/input-stored.zip',stored);
execFileSync('python',['-c',"import pathlib,zipfile; r=pathlib.Path('target/native-workspace-source'); z=zipfile.ZipFile(r/'input-deflated.zip','w',zipfile.ZIP_DEFLATED); [z.write(p,'outputs/'+p.relative_to(r/'input').as_posix()) for p in sorted((r/'input').rglob('*')) if p.is_file()]; z.close()"]);
const deflated=await fs.readFile(root+'/input-deflated.zip'),engine=await instantiateEngine(await fs.readFile(process.env.BISECT_TEST_WASM||'target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));let nativeMatches=0,partitions=0;
const execute=request=>{const result=engine.execute(request);const file=root+'/native-request.json';execFileSync('python',['-c','import pathlib,sys; pathlib.Path(sys.argv[1]).write_text(sys.argv[2],encoding="utf-8")',file,JSON.stringify(request)]);const native=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[file],{encoding:'utf8',maxBuffer:128*1024*1024,windowsHide:true}));assert.deepEqual(result,native);nativeMatches++;if(request.operation==='run-prepared-engine'&&!request.input.inspect_only)partitions++;return result;};
const {Directory,directory,log}=nativeOutputDirectoryHarness(root),inputRoot=new Directory(folder);let allowed=true;inputRoot.queryPermission=async()=>allowed?'granted':'denied';
const uploaded=selectNativeWorkspaceFiles([...files].map(([name,data])=>({name:path.basename(name),webkitRelativePath:'input/'+name,size:data.length,arrayBuffer:async()=>Uint8Array.from(data).buffer}))),live=await selectNativeWorkspaceDirectory(inputRoot),zip=await selectNativeWorkspaceArchive(deflated,'input-deflated.zip');
const selections=[uploaded,live,zip],runs=[];let baseline,output;
for(const selection of selections){
 const selected=await nativeWorkspaceFiles(selection),inputs=nativeFolderSelection(selected,{...NATIVE_FOLDER_DEFAULTS,states:['RI']}),inspection=execute(nativeFolderRequest(inputs));
 assert.equal(inspection.states[0].status,'binary-selected');
 const queued=await queueNativeFolderSources(selected,inputs,inspection,graphInputs,{jobs:null,workers:1}),job=stateRunJobs(queued.inputs.jobs)[0];job.scope.districts=2;
 const run=executePreparedNativeJob({execute},job);assert.equal(run.result.prepared.executed_request.options.seed,'18446744073709551615');assert.equal(preparedStateRunMap(run.result).units,4);
 if(!baseline)baseline=run.result;else{assert.deepEqual(run.result.prepared.result.assignments,baseline.prepared.result.assignments);assert.equal(run.result.prepared.export.document_json,baseline.prepared.export.document_json);}
 output=await executeNativeStateOutput({execute,artifactIdentity:engine.artifactIdentity},{source:nativeStateOutputSource(run.result),version:'vWorkspace',layout:'state'});runs.push({selection:selection.kind,queued,run,output});
}
// An actual native file reader, separate from the WASM byte adapter, checks the
// source graph. This is not a full CLI filesystem Run orchestration claim.
const nativeGraph=execFileSync('target/debug/examples/native_adjacency_reference.exe',[path.join(folder,stem+'.adj.bin'),'RI','2020'],{encoding:'utf8',windowsHide:true});const browserGraph=JSON.parse(runs[0].run.source_preparation.graph_json),nativeObject=JSON.parse(nativeGraph);assert.equal(browserGraph.unit_kind,'tract');assert(!Object.hasOwn(nativeObject,'unit_kind'));assert.deepEqual(Object.fromEntries(Object.entries(browserGraph).filter(([key])=>key!=='unit_kind')),nativeObject);await fs.writeFile(root+'/native-graph-reference.json',JSON.stringify({contract:'native-file-reader-graph-fields-v1',browser_graph_json:runs[0].run.source_preparation.graph_json,native_graph_json:nativeGraph,browser_only_declared_metadata:{unit_kind:'tract'},native_reader:'crates/bisect-cli/examples/native_adjacency_reference.rs'},null,2));
await assert.rejects(nativeWorkspaceFiles({...live}),/grants no folder authority/);allowed=false;await assert.rejects(nativeWorkspaceFiles(live),/permission/);allowed=true;
await fs.appendFile(path.join(folder,'ignored.txt'),'changed');await assert.rejects(nativeWorkspaceFiles(live),/source changed/);
const switched=await selectNativeWorkspaceDirectory(inputRoot);retireNativeWorkspaceSource(switched);await assert.rejects(nativeWorkspaceFiles(switched),/Select and inspect/);
const aborted=new AbortController();aborted.abort(new Error('source cancelled'));await assert.rejects(nativeWorkspaceFiles(zip,{signal:aborted.signal}),/source cancelled/);
const target=await directory('output'),review=await reviewNativeStateInstallation(execute,target,output),before=partitions,cancel=new AbortController();await assert.rejects(installNativeStateOutput(execute,review,{signal:cancel.signal,onProgress:p=>{if(p.path==='plan.rplan')cancel.abort(new Error('install cancelled'));}}),/install cancelled/);
assert(await fs.stat(path.join(target.dir,output.target_namespace,'manifest.tmp')));const recovery=await reviewNativeStateRecovery(execute,target,output);await installNativeStateOutput(execute,recovery);assert.equal(partitions,before);
for(const [name,b64]of Object.entries(output.files_base64))assert.deepEqual(await fs.readFile(path.join(target.dir,output.target_namespace,name)),Buffer.from(b64,'base64'));
const writes=log.length;await assert.rejects(reviewNativeStateInstallation(execute,target,output),/contains existing files/);assert.equal(log.length,writes);
await fs.writeFile(root+'/native-layout.zip',await nativeStateOutputZip(output));
const initialFiles=await nativeWorkspaceFiles(zip),initial=nativeFolderSelection(initialFiles,{...NATIVE_FOLDER_DEFAULTS,states:['RI']});
const project=createProject({name:'Workspace input',operation:'inspect-native-input-folder',files:{nativeFolderInputs:initial,nativeWorkspaceArchive:zip.archive_source,preparedGraphInputs:graphInputs},constraints:[],result:null,lastOperation:null});assert.deepEqual(parseProject(JSON.stringify(project)),project);await fs.writeFile(root+'/source.bisect',JSON.stringify(project));
// Standard-library ZIP writers produce independent adversarial containers.
const badZip=(entries,mode='normal')=>new Uint8Array(execFileSync('python',['-c',"import io,zipfile,json,sys; b=io.BytesIO(); z=zipfile.ZipFile(b,'w'); entries=json.loads(sys.argv[1]); mode=sys.argv[2]; [(lambda i: (setattr(i,'external_attr',0o120777<<16) if mode=='link' else None,z.writestr(i,b'x')))(zipfile.ZipInfo(n)) for n in entries]; z.close(); sys.stdout.buffer.write(b.getvalue())",JSON.stringify(entries),mode],{windowsHide:true}));
let rejected=0;
for(const [entries,mode]of [[['../outside'],'normal'],[['V3/x','v3/X'],'normal'],[['V3/A/x','V3/a/y'],'normal'],[['V3/'+'a/'.repeat(33)+'x'],'normal'],[['V3/file','V3/file/child'],'normal'],[['V3/CON.txt'],'normal'],[['V3/link'],'link'],[['a/V3/x','b/V4/y'],'normal'],[['V3/a/./b'],'normal'],[['V3/__proto__/x'],'normal'],[['V3/x.'],'normal']]){await assert.rejects(decodeNativeWorkspaceArchive(badZip(entries,mode)));rejected++;}
const directoryOffset=Buffer.from(stored).indexOf(Buffer.from([0x50,0x4b,0x01,0x02]));
for(const mutate of [b=>b[30]^=1,b=>b.writeUInt32LE(64*1024*1024+1,directoryOffset+24),b=>b.writeUInt16LE(1,directoryOffset+8),b=>b.writeUInt32LE(0xffffffff,directoryOffset+42),b=>b.writeUInt16LE(12,directoryOffset+10),b=>b[directoryOffset+16]^=1]){const b=Buffer.from(stored);mutate(b);await assert.rejects(decodeNativeWorkspaceArchive(b));rejected++;}
const corrupt=Buffer.from(deflated);const local=corrupt.indexOf(Buffer.from([0x50,0x4b,0x03,0x04]));corrupt[local+30+corrupt.readUInt16LE(local+26)+corrupt.readUInt16LE(local+28)]^=128;await assert.rejects(decodeNativeWorkspaceArchive(corrupt));rejected++;
assert.equal((await decodeNativeWorkspaceArchive(stored)).files.length,files.size);
await fs.writeFile(root+'/runs.json',JSON.stringify(runs));await fs.writeFile(root+'/evidence.json',JSON.stringify({wasm:engine.artifactIdentity,native_host_matches:nativeMatches,independent_native_graph_fields_exact:true,three_source_modes_exact:true,actual_partition_runs:partitions,recovery_partition_runs:partitions-before,archive_refusals:rejected,original_zip_retained_sha256:createHash('sha256').update(Buffer.from(zip.archive_source.base64,'base64')).digest('hex'),copied_retired_changed_and_revoked_authority_refused:true,exact_installation_and_portable_output:true,source_owned_fixture:true,full_parity:false},null,2)+'\n');
console.log(`PASS native workspace: three source modes, ${nativeMatches} native host replies, independent native graph, exact exports/maps/install/recovery, ${rejected} ZIP refusals; no recovery partition.`);
