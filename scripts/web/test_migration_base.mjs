import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {observeMigrationDirectory,retainMigrationFiles,migrationBaseRequest,migrationBaseZip} from '../../web/lab/migration-base-files.js';
const root=path.resolve('target/migration-base');await fs.mkdir(root,{recursive:true});const fsRoot=path.join(root,'cases-'+Date.now());const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const manifest=await fs.readFile('data/manifest.json','utf8'),time='2026-10-17T12:00:00Z',raw=s=>Buffer.from(s),assigned=raw('{"a":18446744073709551615,"b":4294967296,"c":18446744073709551615}');
const wide=JSON.parse(manifest);wide.states.WA.districts['2020']='u64-boundary';const wideManifest=JSON.stringify(wide).replace('\"u64-boundary\"','18446744073709551615');
const cases=[
 ['resolved-source',['states/washington'],{'states/washington/final_assignments.json':assigned}],
 ['u64-resolution-counts',['states/washington'],{}, {manifest_json:wideManifest}],
 ['direct-precedence',['states/wa','states/washington'],{'states/wa/final_assignments.json':raw('{"a":7}'),'states/washington/final_assignments.json':assigned}],
 ['empty-direct-wins',['states/wa','states/washington'],{'states/washington/final_assignments.json':assigned}],
 ['empty-source',['states/washington'],{}],
 ['nested-empty',['states/washington/empty/deeper'],{'states/washington/maps/raw.bin':Buffer.from([0,255,1,2]),'states/washington/readme.txt':raw('original')}],
 ['full-name',['states/washington'],{'states/washington/final_assignments.json':assigned},{state_identifier:'washington'}],
 ['invalid-assignments',['states/washington'],{'states/washington/final_assignments.json':raw('broken')}],
 ['source-manifests-replaced',['states/washington'],{'states/washington/manifest.json':raw('old source'),'states/washington/manifest.tmp':raw('old temp')}],
 ['target-without-manifest',['states/washington','plans/draft/old-empty'],{'states/washington/new.bin':raw('new'),'plans/draft/keep.bin':raw('preserved'),'plans/draft/final_assignments.json':assigned}],
 ['target-force',['states/washington','plans/draft/old-empty'],{'states/washington/overlap.bin':raw('new'),'plans/draft/overlap.bin':raw('old'),'plans/draft/keep.bin':raw('preserved'),'plans/draft/manifest.json':raw('{"created_at":"old-clock"}')},{force:true}],
 ['target-collision',['states/washington'],{'plans/draft/manifest.json':raw('{"created_at":"old-clock"}')},{},'collision'],
 ['target-invalid-collision',['states/washington'],{'plans/draft/manifest.json':Buffer.from([255])},{},'collision'],
 ['target-temp-only',['states/washington'],{'plans/draft/manifest.tmp':raw('incomplete marker'),'plans/draft/keep.bin':raw('preserved')}],
 ['direct-ignores-malformed-manifest',['states/wa'],{}, {manifest_json:'not JSON'}],
 ['missing-source',['states/other'],{}, {},'missing'],
 ['source-is-file',[],{'states/wa':raw('not a directory')},{},'missing'],
 ['plans-is-file',['states/washington'],{'plans':raw('not a directory')},{},'missing'],
];
async function inventory(base,prefix=''){const dirs=[],files={};for(const item of await fs.readdir(base,{withFileTypes:true})){const key=prefix+item.name;if(item.isDirectory()){dirs.push(key);const nested=await inventory(path.join(base,item.name),key+'/');dirs.push(...nested.dirs);Object.assign(files,nested.files);}else files[key]=await fs.readFile(path.join(base,item.name));}return {dirs,files};}
const native=(request,p)=>{return spawnSync(path.resolve('target/release/examples/execute_tool_request.exe'),[p],{encoding:'utf8',maxBuffer:64*1024*1024});};
let matches=0,selections=0,refusals=0;
for(const [name,dirs,files,override={},expected='ok']of cases){
 const artifact=path.join(root,name);await fs.mkdir(artifact,{recursive:true});const dir=path.join(fsRoot,name),base=path.join(dir,'outputs/v1/2020');await fs.mkdir(base,{recursive:true});for(const p of dirs)await fs.mkdir(path.join(base,p),{recursive:true});for(const [p,b]of Object.entries(files)){await fs.mkdir(path.dirname(path.join(base,p)),{recursive:true});await fs.writeFile(path.join(base,p),b);}
 const before=await inventory(base),resolution=override.manifest_json??manifest,resolutionPath=path.join(artifact,'resolution.json');await fs.writeFile(resolutionPath,resolution);
 const selection={label:'draft',state_identifier:'WA',force:false,observation_json:JSON.stringify({directories:before.dirs.sort(),files:Object.keys(before.files).sort()}),manifest_json:resolution,...override};
 const inspection={operation:'inspect-native-migration-base',input:selection},inspectPath=path.join(artifact,'inspect.json');await fs.writeFile(inspectPath,JSON.stringify(inspection));let chosen;
 if(expected==='missing'){assert.throws(()=>engine.execute(inspection));assert.notEqual(native(inspection,inspectPath).status,0);}else{chosen=engine.execute(inspection);const host=native(inspection,inspectPath);assert.equal(host.status,0,host.stderr);assert.deepEqual(chosen,JSON.parse(host.stdout));selections++;}
 const selected=chosen?[...chosen.source_files,...chosen.target_files]:[],input={selection,created_at:time,files_base64:Object.fromEntries(selected.map(p=>[p,before.files[p].toString('base64')]))},request={operation:'stage-native-migration-base',input},requestPath=path.join(artifact,'request.json');await fs.writeFile(requestPath,JSON.stringify(request));
 let actual;if(expected==='collision'){assert.throws(()=>engine.execute(request),/already exists/);assert.notEqual(native(request,requestPath).status,0);}else if(expected==='ok'){actual=engine.execute(request);const host=native(request,requestPath);assert.equal(host.status,0,host.stderr);assert.deepEqual(actual,JSON.parse(host.stdout));}
 const args=['migrate','--state',selection.state_identifier,'--label','draft'];if(selection.force)args.push('--force');const cli=spawnSync(path.resolve('target/debug/bisect.exe'),args,{cwd:dir,env:{...process.env,BISECT_MANIFEST:resolutionPath},encoding:'utf8'});
 if(expected!=='ok'){assert.notEqual(cli.status,0,name);refusals++;continue;}assert.equal(cli.status,0,cli.stderr);
 const target=await inventory(path.join(base,'plans/draft')),expectedFiles=Object.fromEntries(Object.entries(target.files).map(([p,b])=>['plans/draft/'+p,b]));assert.deepEqual(Object.keys(actual.files_base64).sort(),Object.keys(expectedFiles).sort());
 for(const [p,b]of Object.entries(expectedFiles)){let data=b;if(p==='plans/draft/manifest.json'){const m=JSON.parse(b);data=Buffer.from(b.toString().replace(JSON.stringify(m.created_at),JSON.stringify(time)));}assert.deepEqual(Buffer.from(actual.files_base64[p],'base64'),data,name+'/'+p);}
 const expectedDirs=['plans','plans/draft',...target.dirs.map(p=>'plans/draft/'+p)].sort();assert.deepEqual(actual.directories,expectedDirs,name);matches++;
 await fs.writeFile(path.join(artifact,'result.json'),JSON.stringify(actual));await fs.writeFile(path.join(artifact,'files.json'),JSON.stringify(actual.files_base64));await fs.writeFile(path.join(artifact,'dirs.json'),JSON.stringify(actual.directories));await fs.writeFile(path.join(artifact,'migration.zip'),await migrationBaseZip(actual));
 const project=createProject({name:'Native base migration '+name,files:{migrationBaseInputs:input},operation:'stage-native-migration-base',constraints:[],result:actual,lastOperation:'stage-native-migration-base'});assert.deepEqual(parseProject(JSON.stringify(project)),project);await fs.writeFile(path.join(artifact,'project.bisect'),JSON.stringify(project));
}
// Real host adapters retain only referenced bodies and cannot be recreated from
// serialized observations. This fake handle implements the same read interface.
const reads=[];function directory(name,values){return {name,kind:'directory',async *entries(){for(const [key,value]of Object.entries(values))yield [key,value];}};}function file(name,bytes){return {name,kind:'file',getFile:async()=>({size:bytes.length,lastModified:10,arrayBuffer:async()=>{reads.push(name);return new Uint8Array(bytes).buffer;}})};}
const source=directory('base',{states:directory('states',{washington:directory('washington',{'final_assignments.json':file('assignments',assigned),empty:directory('empty',{})})}),ignored:file('ignored',raw('not read'))});
const capture=await observeMigrationDirectory(source);assert.deepEqual(reads,[]);const selection={label:'draft',state_identifier:'WA',force:false,observation_json:capture.observation_json,manifest_json:null},inputs={selection,created_at:time,files_base64:null},chosen=engine.execute(migrationBaseRequest(inputs));const retained=await retainMigrationFiles(capture,inputs,chosen);assert.deepEqual(reads,['assignments']);assert(engine.execute(migrationBaseRequest(retained,true)).directories.includes('plans/draft/empty'));await assert.rejects(()=>retainMigrationFiles(JSON.parse(JSON.stringify(capture)),inputs,chosen));
const controller=new AbortController();controller.abort();await assert.rejects(()=>retainMigrationFiles(capture,inputs,chosen,{signal:controller.signal}));const changed=structuredClone(inputs);changed.selection.force=true;await assert.rejects(()=>retainMigrationFiles(capture,changed,chosen));
const staged=engine.execute(migrationBaseRequest(retained,true)),tampered=structuredClone(staged);tampered.files_base64[Object.keys(tampered.files_base64)[0]]='Zm9yZ2Vk';await assert.rejects(()=>migrationBaseZip(tampered),/hash mismatch/);
for(const observation_json of ['{"directories":["../escape"],"files":[]}','{"directories":["states","states/washington"],"files":["states"]}','{"directories":["states","states/washington"],"files":["states/washington/A","states/washington/a"]}'])assert.throws(()=>engine.execute({operation:'inspect-native-migration-base',input:{...selection,observation_json}}));
assert.throws(()=>engine.execute({operation:'stage-native-migration-base',input:{...retained,selection:retained.selection,files_base64:{...retained.files_base64,'ignored':Buffer.from('not selected').toString('base64')}}}));
let modified=1,driftReads=0;const drifting=directory('base',{states:directory('states',{washington:directory('washington',{file:{kind:'file',getFile:async()=>({size:1,lastModified:modified,arrayBuffer:async()=>{driftReads++;return new Uint8Array([1]).buffer;}})}})})});const driftCapture=await observeMigrationDirectory(drifting),driftInput={...inputs,selection:{...selection,observation_json:driftCapture.observation_json}},driftSelection=engine.execute(migrationBaseRequest(driftInput));modified=2;await assert.rejects(()=>retainMigrationFiles(driftCapture,driftInput,driftSelection),/changed after inventory/);assert.equal(driftReads,0);
await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({wasm_sha256:engine.artifactIdentity.sha256,actual_cli_file_and_directory_matches:matches,full_native_selection_matches:selections,matching_native_refusals:refusals,clock_normalization_only:true,direct_precedence_empty_directories_and_force_merge:true,native_distinct_u64_assignment_counts:true,metadata_capture_reads_no_bodies:true,referenced_reads_only:true,serialized_capture_has_no_read_authority:true,cancel_settings_and_metadata_drift_guards:true,unsafe_observation_unexpected_bytes_and_edited_output_refused:true,zip_checks_pending:true},null,2));console.log(`Migration base: ${matches} actual CLI file/directory matches, ${selections} Native selections, ${refusals} matching refusals, empty folders/force merge, u64 counts, exact snapshots and host read guards passed.`);
