import {verifyFreshNativeFollowups} from './fixtures/fresh-native-followup-checks.mjs';
import fs from 'node:fs/promises';import path from 'node:path';import assert from 'node:assert/strict';import {spawnSync} from 'node:child_process';import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';import {loadMetisSidecar} from '../../web/lab/metis-c-sidecar.js';
import {loadFreshNativeMultiscaleCorpus,freshMultiscaleRefusals} from './fixtures/fresh-native-multiscale.mjs';
import {compareCommandReplies} from './native_command_comparison.mjs';import {nativeBinaryWidth} from './fixtures/native-recorded-replay.mjs';
import {createProject,parseProject} from '../../web/lab/project.js';
import {createWorkbenchArchive,parseWorkbenchArchive,createCompressedWorkbenchArchive,parseWorkbenchArchiveBytes} from '../../web/lab/workbench-archive.js';
import {retainFreshNativeStateRun,validateFreshNativeStateSession} from '../../web/lab/fresh-native-state-session.js';
const argv=process.argv.slice(2),arg=(name,required=true)=>{const i=argv.indexOf(name);if(i<0){assert(!required,'Supply '+name);return null;}return path.resolve(argv[i+1]);};
const wasm=arg('--wasm'),host=arg('--host'),parent=arg('--output'),cbase=arg('--c-sidecar',false),legacy=arg('--legacy-native32-preimage',false),mode=cbase?'c':'core';await fs.mkdir(parent,{recursive:true});const root=await fs.mkdtemp(path.join(parent,mode+'-'));
let metisSidecar=null;if(cbase){const manifest=JSON.parse(await fs.readFile(path.join(cbase,'metis-sidecar.json'),'utf8'));metisSidecar=await loadMetisSidecar({manifest,javascript:await fs.readFile(path.join(cbase,manifest.javascript.file)),wasm:await fs.readFile(path.join(cbase,manifest.wasm.file))});}
const bytes=await fs.readFile(wasm),hostBytes=await fs.readFile(host),width=nativeBinaryWidth(hostBytes),engine=await instantiateEngine(bytes,{metisSidecar}),cases=await loadFreshNativeMultiscaleCorpus();
const evidence={status:'running',mode,wasm_sha256:createHash('sha256').update(bytes).digest('hex'),host_sha256:createHash('sha256').update(hostBytes).digest('hex'),sidecar:metisSidecar?.identity??null,cases:[],refusals:[],scope:'Three complete core references and an additional C-profile flag reference; exact unchanged32-BG or16-tract native outputs,64steps/seed17/7.125%. Raw sources only enter fresh operations. Sidecar attachment and actual C bridge use are distinct. No full-state capacity, fresh SDK or full release claim.'};
const save=()=>fs.writeFile(path.join(root,'evidence.json'),JSON.stringify(evidence,null,2));await save();
function native(request,file){const result=spawnSync(host,[file],{windowsHide:true,maxBuffer:64*1024*1024});return result;}
try{
 for(const row of cases){
  if(row.c_profile_required&&!cbase)continue;
  const folder=path.join(root,row.id);await fs.mkdir(folder);
  for(const operation of ['prepare-fresh-native-state-run','run-fresh-native-state']){
   const request={operation,input:row.input},file=path.join(folder,operation+'.request.json');await fs.writeFile(file,JSON.stringify(request));
   const actual=engine.execute(request);await fs.writeFile(path.join(folder,operation+'.wasm.json'),JSON.stringify(actual));assert.equal(actual.execution_performed,operation==='run-fresh-native-state');assert.equal(actual.recorded_paths_opened,false);
   if(!row.c_profile_required){
    const result=native(request,file);await fs.writeFile(path.join(folder,operation+'.host.stdout'),result.stdout??'');await fs.writeFile(path.join(folder,operation+'.host.stderr'),result.stderr??'');assert.equal(result.status,0,result.error?.message??result.stderr?.toString());
    const expected=JSON.parse(result.stdout);assert.equal(actual.input_sha256,expected.input_sha256);compareCommandReplies({operation:'prepare-native-state-graph-run'},actual.prepared,expected.prepared,width);
   }
   if(operation==='run-fresh-native-state'){
    verifyFreshNativeFollowups(actual);
    assert.equal(actual.export_error,null);assert.deepEqual(actual.export.document.plan,JSON.parse(row.document_json).plan);const context=JSON.parse(row.context_json);
    for(const key of ['units','graph','populations','subdivisions','demographics','geometry'])assert.deepEqual(actual.export.context[key],context[key],row.id+':'+key);
    assert.equal(actual.export.context.source_hashes.adjacency,context.source_hashes.adjacency,'Physical adjacency hash role must match the native RCTX');
    if(row.input.multiscale.fine_source)assert.equal(actual.export.context.source_hashes['browser.native-primary-adjacency-v1'],'sha256:'+createHash('sha256').update(Buffer.from(row.input.source.sources.adjacency.base64,'base64')).digest('hex'));
    assert.equal(actual.executed_request.total_steps,64);assert.equal(actual.executed_request.native_sampler,row.workload.native_sampler);assert.equal(actual.export.fresh_native_state_binding.original_context_loaded,false);
    const input=row.input,args=input.arguments.filter((v,i,all)=>!['--adjacency','--coi-weights'].includes(v)&&!['--adjacency','--coi-weights'].includes(all[i-1])),sources=Object.fromEntries(Object.entries(input.source.sources).filter(([,v])=>v!=null));
    if(input.multiscale.fine_source)for(const [key,value]of Object.entries(input.multiscale.fine_source.sources))if(value)sources['fine_'+key]=value;
    const display={type:'FeatureCollection',features:context.units.unit_ids.map((id,i)=>({type:'Feature',properties:{GEOID:id},geometry:{type:'Polygon',coordinates:[[[i,0],[i+.5,0],[i+.5,.5],[i,.5],[i,0]]]}}))};
    const session=validateFreshNativeStateSession({schema_version:'bisect-fresh-native-state-session-v1',fields:{arguments_json:JSON.stringify(args),state:input.source.state,year:input.source.year,label:input.label,chamber:input.chamber,created_at:input.created_at},sources,display_geometry_json:' '.repeat(64*1024)+JSON.stringify(display),retained_run:retainFreshNativeStateRun(actual),runtime_artifact:engine.artifactIdentity,multiscale_sampler:input.multiscale.sampler,load_c_metis:row.c_profile_required});
    const project=createProject({name:'Fresh multiscale',files:{freshNativeStateSession:session},operation:'run-tui-configured',constraints:[],result:null,lastOperation:null});
    assert.deepEqual(parseProject(JSON.stringify(project)).files.freshNativeStateSession,session);assert.deepEqual(parseWorkbenchArchive(createWorkbenchArchive(project)).files.freshNativeStateSession,session);assert.deepEqual((await parseWorkbenchArchiveBytes(await createCompressedWorkbenchArchive(project))).files.freshNativeStateSession,session);
    for(const mutation of [s=>s.multiscale_sampler='native32',s=>s.load_c_metis='true',s=>s.sources.fine_extra={name:'bad',base64:'e30='}]){const bad=structuredClone(session);mutation(bad);assert.throws(()=>validateFreshNativeStateSession(bad));}
    assert.equal(actual.runtime_artifacts?.metis_sidecar!=null,row.c_profile_required);
    if(row.c_profile_required){assert.deepEqual(actual.runtime_artifacts.metis_sidecar,metisSidecar.identity);assert.equal(actual.runtime_artifacts.engine.sha256,evidence.wasm_sha256);}
    evidence.cases.push({id:row.id,status:'passed-native-plan-and-physical-context',units:context.units.unit_ids.length,steps:64,seed:'17',tolerance_percent:7.125,project_and_plain_compressed_archive_exact:true,c_bridge_observed:actual.runtime_artifacts?.metis_sidecar!=null,backend_option:actual.executed_request.request.options.metis_engine});await save();
   }
  }
 }
 const originalControl=freshMultiscaleRefusals(cases.find(row=>row.id==='core-native64-bg').input);
 assert.equal(originalControl.length,5);
 const legacyControl=originalControl.find(row=>row.name==='native32');
 assert(legacyControl);
 const legacyRequest={operation:'run-fresh-native-state',input:legacyControl.input};
 if(legacy){
  const oldWasm=await fs.readFile(legacy),oldHash=createHash('sha256').update(oldWasm).digest('hex');
  assert.equal(oldHash,'bcca6582e455e19ae25ff8447a48af930e778404e88999402dd98a300a6da46a');
  const original=await instantiateEngine(oldWasm);
  assert.throws(()=>original.execute(legacyRequest),/native64 Wilson sampler/);
  evidence.native32_preimage={status:'original-refusal-replayed',wasm_sha256:oldHash,
   request_sha256:createHash('sha256').update(JSON.stringify(legacyRequest)).digest('hex')};
 }else evidence.native32_preimage={status:'historical-witness-not-replayed-in-this-run'};
 const reviewRequest={operation:'prepare-fresh-native-state-run',input:legacyControl.input};
 const review=engine.execute(reviewRequest);
 assert.equal(review.execution_performed,false);
 assert.equal(review.recorded_paths_opened,false);
 const request=JSON.parse(review.prepared.prepared_request_json);
 assert.equal(request.operation,'run-multiscale-build');
 assert.equal(request.input.native_sampler,'rand-0.8.6-smallrng-native32-wilson');
 assert.equal(request.input.total_steps,64);
 assert.equal(String(request.input.request.options.seed),'17');
 const reviewFile=path.join(root,'native32-review.request.json');await fs.writeFile(reviewFile,JSON.stringify(reviewRequest));
 const reviewed=native(reviewRequest,reviewFile);
 await fs.writeFile(path.join(root,'native32-review.host.stdout'),reviewed.stdout??'');
 await fs.writeFile(path.join(root,'native32-review.host.stderr'),reviewed.stderr??'');
 assert.equal(reviewed.status,0,reviewed.stderr?.toString());
 const nativeReview=JSON.parse(reviewed.stdout);
 assert.equal(nativeReview.input_sha256,review.input_sha256);
 compareCommandReplies({operation:'prepare-native-state-graph-run'},review.prepared,nativeReview.prepared,width);
 evidence.native32_preparation={status:'reviewed-explicit-native32-no-execution',input_sha256:review.input_sha256,sampler:request.input.native_sampler};
 const invalid=structuredClone(legacyControl.input);invalid.multiscale.sampler='rand-0.8.6-smallrng-native96-wilson';
 const controls=[{name:'unknown-native-width',input:invalid,message:'unknown variant'},...originalControl.filter(row=>row.name!=='native32')];
 assert.equal(controls.length,5);
 for(const row of controls){
  const request={operation:'run-fresh-native-state',input:row.input},file=path.join(root,row.name+'.request.json');await fs.writeFile(file,JSON.stringify(request));let error;try{engine.execute(request);}catch(e){error=e.message;}
  await fs.writeFile(path.join(root,row.name+'.wasm-error.json'),JSON.stringify({error}));assert.equal(typeof error,'string',row.name);assert(error.toLowerCase().includes(row.message.toLowerCase()),row.name+':'+error);
  const result=native(request,file);await fs.writeFile(path.join(root,row.name+'.host.stderr'),result.stderr??'');assert.equal(result.status,1);assert(result.stderr.toString().toLowerCase().includes(row.message.toLowerCase()));evidence.refusals.push(row.name);await save();
 }
 assert.equal(evidence.cases.length,cbase?4:3);evidence.status='passed-maintained-fresh-multiscale-native-references';await save();
}catch(error){evidence.status='failed';evidence.error={message:error.message,stack:error.stack};await save();throw error;}
console.log(JSON.stringify({root,status:evidence.status,cases:evidence.cases.length,refusals:evidence.refusals.length}));
