import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {gzipSync} from 'node:zlib';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {fletchStageRequest,fletchStageFromAcquisition,fletchStagedCacheZip,fletchStagedUpdatesSource,readFletchStageSource} from '../../web/lab/fletch-stage-files.js';
import {fletchMergeRequest} from '../../web/lab/fletch-merge-files.js';
import {verifyFletchCacheFiles} from '../../web/lab/fletch-verify-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {createWorkbenchArchive,parseWorkbenchArchive} from '../../web/lab/workbench-archive.js';
const root='target/fletch-stage-wasm';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const cases=[['empty',Buffer.alloc(0),'0','dataset',null],['text',Buffer.from('actual raw source\n'),'123','dataset','2020'],['binary',Buffer.from([0,255,7,128]),'9007199254740993','dataset',null],['unicode',Buffer.from('原始 source'),'18446744073709551615','元数据','2020'],['gzip',gzipSync('raw compressed source'),'5','dataset','2010'],['quoted-version',Buffer.from('source'),'3','dataset','quoted "\nversion'],['large',Buffer.alloc(5*1024*1024+17,31),'7','dataset','2000']];
for(const [name,bytes,time,id,version]of cases){
 await fs.writeFile(root+'/source.bin',bytes);const cache=path.resolve(root,name);await fs.mkdir(cache,{recursive:true});
 const native=execFileSync('target/debug/examples/fletch_stage_reference.exe',[root+'/source.bin',cache,time,id,version??'none'],{encoding:'utf8'}),entry=JSON.parse(native)[0];
 const input={source:{name:name+'.bin',base64:bytes.toString('base64')},dataset_id:id,version,source_url:entry.source_url,fetched_at_ms:time,expected_source_sha256:null};
 const result=engine.execute(fletchStageRequest(input));assert.equal(result.entries_json,native,name+' actual HTTP native entry bytes');
 assert.equal(result.fetched_at_ms_text,time);assert.equal(result.source_sha256,createHash('sha256').update(bytes).digest('hex'));assert.equal(result.byte_count_text,String(bytes.length));
 assert.deepEqual(await fs.readFile(path.join(cache,entry.relative_path)),bytes);
 const update=fletchStagedUpdatesSource(result),base={name:'base.json',base64:Buffer.from(JSON.stringify({schema_version:'fletch.cache-manifest.v1',generated_by:'fixture',cache_root:'cache',entries:[]})).toString('base64')};
 const merged=engine.execute(fletchMergeRequest(base,update));assert.equal(merged.manifest_json,result.manifest_json);
 const verified=await verifyFletchCacheFiles(engine,{operation:'verify-fletch-cache-files',cache_manifest_json:merged.manifest_json,files:[{path:result.relative_path,file:new Blob([bytes])}]});assert.equal(verified.verified_count,1);
 await fs.writeFile(root+'/'+name+'.zip',await fletchStagedCacheZip(result,input));
 const project=createProject({name:'Raw cache '+name,files:{fletchStageInputs:input},operation:'stage-fletch-cache-source',constraints:[],result,lastOperation:'stage-fletch-cache-source'});assert.deepEqual(parseProject(JSON.stringify(project)),project);const archive=parseWorkbenchArchive(createWorkbenchArchive(project));assert.deepEqual(archive.files,project.files);assert.deepEqual(archive.result,project.result);
 await fs.writeFile(root+'/'+name+'.bisect',JSON.stringify(project));await fs.writeFile(root+'/'+name+'-result.json',JSON.stringify(result));
 await assert.rejects(()=>fletchStagedCacheZip({...result,manifest_json:result.manifest_json+' '},input),/manifest bytes changed/);
 await assert.rejects(()=>fletchStagedCacheZip(result,{...input,source_url:input.source_url+'changed'}),/fresh/);
 if(bytes.length)await assert.rejects(()=>fletchStagedCacheZip(result,{...input,source:{...input.source,base64:Buffer.from('changed').toString('base64')}}),/bytes changed/);
 assert.throws(()=>engine.execute(fletchStageRequest({...input,expected_source_sha256:'0'.repeat(64)})),/acquisition byte identity/);
}
for(const name of ['acs-housing','lodes-wac']){
 const acquired=JSON.parse(await fs.readFile('target/fetch-acquisition/'+name+'.bisect','utf8'));
 const input=fletchStageFromAcquisition(acquired.files,'123'),result=engine.execute(fletchStageRequest(input));assert.equal(result.source_sha256,acquired.files.fetchAcquisitionReceipt.source_sha256);assert.equal(result.dataset_id,'bisect.2020.ri.'+name);
 const files={...acquired.files,fletchStageInputs:input};const project=createProject({name:'Acquired raw cache '+name,files,operation:'stage-fletch-cache-source',constraints:[],result,lastOperation:'stage-fletch-cache-source'});assert.deepEqual(parseProject(JSON.stringify(project)),project);await fs.writeFile(root+'/acquired-'+name+'.bisect',JSON.stringify(project));await fs.writeFile(root+'/acquired-'+name+'.zip',await fletchStagedCacheZip(result,input));
 const inputKey=name==='acs-housing'?'acsHousingInputs':'lodesInputs',wrong=structuredClone(acquired.files);wrong[inputKey].settings.state='NC';assert.throws(()=>fletchStageFromAcquisition(wrong),/scope/);
 const corrupt=structuredClone(acquired.files);corrupt[inputKey].source.base64=Buffer.alloc(Number(acquired.files.fetchAcquisitionReceipt.byte_length),37).toString('base64');assert.throws(()=>engine.execute(fletchStageRequest(fletchStageFromAcquisition(corrupt))),/byte identity/);
}
for(const name of ['tiger','pl94171']){
 const acquired=JSON.parse(await fs.readFile('target/census-zip-acquisition/'+name+'.bisect','utf8')),files=acquired.files;
 const input=fletchStageFromAcquisition(files,'9007199254740993');assert.equal(input.source,null);assert.equal(input.source_ref,'fetchArchiveSource');
 const result=engine.execute(fletchStageRequest(input,files)),bytes=Buffer.from(files.fetchArchiveSource.source.base64,'base64');
 assert.equal(result.source_sha256,files.fetchAcquisitionReceipt.source_sha256);assert.equal(result.dataset_id,'bisect.2020.ri.'+name);
 await fs.writeFile(root+'/source.bin',bytes);const cache=path.resolve(root,'archive-native-'+name);await fs.mkdir(cache,{recursive:true});
 const native=execFileSync('target/debug/examples/fletch_stage_reference.exe',[root+'/source.bin',cache,input.fetched_at_ms,input.dataset_id,input.version],{encoding:'utf8'}),entry=JSON.parse(native)[0];
 const request=fletchStageRequest(input,files);assert.equal(engine.execute({...request,input:{...request.input,source_url:entry.source_url}}).entries_json,native);
 assert.deepEqual(await fs.readFile(path.join(cache,entry.relative_path)),bytes);
 const project=createProject({name:'Archive cache '+name,files:{...files,fletchStageInputs:input},operation:'stage-fletch-cache-source',constraints:[],result,lastOperation:'stage-fletch-cache-source'});
 assert.deepEqual(parseProject(JSON.stringify(project)),project);const text=createWorkbenchArchive(project),archive=parseWorkbenchArchive(text);assert.deepEqual(archive.files,project.files);assert.equal(text.split(files.fetchArchiveSource.source.base64).length-1,1,'Raw ZIP retained once');
 await fs.writeFile(root+'/acquired-'+name+'.bisect',JSON.stringify(project));await fs.writeFile(root+'/acquired-'+name+'.zip',await fletchStagedCacheZip(result,input,files));
 const wrong=structuredClone(files);wrong.fetchArchiveSource.state='NC';assert.throws(()=>fletchStageRequest(input,wrong));
 const corrupt=structuredClone(files);corrupt.fetchArchiveSource.source.base64=Buffer.alloc(bytes.length,37).toString('base64');assert.throws(()=>engine.execute(fletchStageRequest(input,corrupt)),/byte identity/);await assert.rejects(()=>fletchStagedCacheZip(result,input,corrupt),/bytes changed/);
 assert.throws(()=>parseProject(JSON.stringify({...project,files:{...project.files,fetchArchiveSource:undefined}})));
 assert.throws(()=>parseProject(JSON.stringify({...project,files:{...project.files,fletchStageInputs:{...input,source_ref:'file:///arbitrary'}}})));
}
const input=JSON.parse(await fs.readFile(root+'/text.bisect','utf8')).files.fletchStageInputs;
for(const change of [{fetched_at_ms:'01'},{fetched_at_ms:'18446744073709551616'},{fetched_at_ms:123},{dataset_id:''},{source_url:''},{source_base64:'%%%'}])assert.throws(()=>engine.execute({operation:'stage-fletch-cache-source',input:{...fletchStageRequest(input).input,...change}}));
await assert.rejects(()=>readFletchStageSource({name:'huge',size:64*1024*1024+1,arrayBuffer(){throw new Error('Must not read');}}),/64 MiB/);
console.log('Raw FLETCH staging: nine actual native HTTP downloads with exact WASM entry bytes/object identities, key/path/verified/retry/u64 semantics, merge-to-file-verification, exact portable ZIP source/project/archive preservation; four receipt-bound acquisition handoffs, single-copy archive references and source/scope/hash/size/metadata/reference rejection guards passed.');
