import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {execFileSync,spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const root='target/package-verification-wasm';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const sha=b=>createHash('sha256').update(b).digest('hex');
let parity=0,cli=0;const cases=[];
async function exact(input){const req={operation:'verify-rplan-package',input},out=engine.execute(req);const path=`${root}/native-request-${parity}.json`;await fs.writeFile(path,JSON.stringify(req));assert.deepEqual(JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[path],{encoding:'utf8',maxBuffer:8*1024*1024})),out);parity++;cases.push({req,out});return out;}
let base,baseResult;
for(const group of ['rplan-golden-packages','rplan-method-packages','rplan-benchmark-packages'])for(const folder of await fs.readdir('docs/examples/'+group,{withFileTypes:true})){
 if(!folder.isDirectory())continue;const dir=`docs/examples/${group}/${folder.name}`;const manifest=await fs.readFile(dir+'/manifest.json','utf8'),parsed=JSON.parse(manifest),files={};for(const entry of parsed.files)files[entry.path]=(await fs.readFile(dir+'/'+entry.path)).toString('base64');
 const input={manifest_json:manifest,files_base64:files},out=await exact(input);assert.equal(out.status,'verified',out.error);assert.equal(out.verification.checked_files,parsed.files.length);assert.equal(out.verification.certificate_result,'pass');
 const process=spawnSync('target/debug/bisect.exe',['verify','--manifest',dir+'/manifest.json'],{encoding:'utf8'});assert.equal(process.status,0,process.stderr);assert.match(process.stderr,/manifest hashes verified/);assert.match(process.stderr,/certificate verified/);cli++;
 if(folder.name==='U.16+branch-and-cut-path8-benchmark'){base=input;baseResult=out;}
}
assert.ok(base);
const upper=JSON.parse(base.manifest_json);for(const file of upper.files)file.sha256=file.sha256.toUpperCase();assert.equal((await exact({...base,manifest_json:JSON.stringify(upper)})).status,'verified');
const extra=await exact({...base,files_base64:{...base.files_base64,'extra.bin':Buffer.from([0,255,128]).toString('base64')}});assert.equal(extra.status,'verified');assert.equal(extra.selected_file_sha256['extra.bin'],sha(Buffer.from([0,255,128])));assert.equal(extra.verification.checked_files,baseResult.verification.checked_files);
const tampered={...base,files_base64:{...base.files_base64,'plan.rplan':Buffer.from(Buffer.from(base.files_base64['plan.rplan'],'base64').toString()+' ').toString('base64')}};assert.match((await exact(tampered)).error,/SHA-256 mismatch/);
const cert=JSON.parse(Buffer.from(base.files_base64['audit-certificate.json'],'base64').toString());cert.certificate_id='changed';const bytes=Buffer.from(JSON.stringify(cert)),certManifest=JSON.parse(base.manifest_json);certManifest.files.find(f=>f.role==='certificate').sha256=sha(bytes);const rehashed={manifest_json:JSON.stringify(certManifest),files_base64:{...base.files_base64,'audit-certificate.json':bytes.toString('base64')}};assert.match((await exact(rehashed)).error,/certificate verification failed/);
const missingFiles={...base.files_base64};delete missingFiles['context.rctx'];assert.match((await exact({...base,files_base64:missingFiles})).error,/not selected/);
const role=JSON.parse(base.manifest_json);role.files.find(f=>f.role==='context').role='unbound';assert.match((await exact({...base,manifest_json:JSON.stringify(role)})).error,/missing required role/);
for(const path of ['../outside','/absolute','C:/drive','folder\\file','__proto__/file']){const manifest=JSON.parse(base.manifest_json);manifest.files[0].path=path;assert.match((await exact({...base,manifest_json:JSON.stringify(manifest)})).error,/Unsafe/);}
assert.equal((await exact({...base,manifest_json:'{bad'})).status,'failed');assert.equal((await exact({...base,manifest_json:'{"schema_version":"unknown"}'})).status,'failed');
const duplicated=JSON.parse(base.manifest_json);duplicated.files.push({...duplicated.files[0]});assert.match((await exact({...base,manifest_json:JSON.stringify(duplicated)})).error,/duplicate/);
// A correctly bound certificate may record failed audit checks: package integrity must
// not silently turn that recorded result into a passed audit.
const disconnected=await fs.readFile('crates/rplan-audit/fixtures/grid3x3-disconnected.rplan','utf8'),context=await fs.readFile('crates/rplan-audit/fixtures/grid3x3.rctx','utf8');
const profile=JSON.parse(await fs.readFile('crates/rplan-audit/fixtures/grid3x3-valid-certificate.json','utf8')).legal_profile;delete profile.legal_disclaimer;profile.contiguity_required=true;profile.nesting_rule={type:'not-evaluated'};
const failedAudit=engine.execute({operation:'audit-rplan-text',document_json:disconnected,context_json:context,profile_json:JSON.stringify(profile),constraints:['contiguity'],generated_at_utc:'2026-10-08T00:00:00Z',lineage_json:null});assert.equal(failedAudit.result,'fail');
const failedFiles={'plan.rplan':disconnected,'context.rctx':context,'audit-certificate.json':failedAudit.certificate_json};
const failedManifest={schema_version:'benchmark-rplan-package-manifest-v1',example_id:'Bound certificate with failed audit',files:Object.entries(failedFiles).map(([path,text])=>({path,sha256:sha(text),role:path==='plan.rplan'?'plan':path==='context.rctx'?'context':'certificate'}))};
const failedInput={manifest_json:JSON.stringify(failedManifest),files_base64:Object.fromEntries(Object.entries(failedFiles).map(([p,t])=>[p,Buffer.from(t).toString('base64')]))};const failedOut=await exact(failedInput);assert.equal(failedOut.status,'verified');assert.equal(failedOut.verification.certificate_result,'fail');
let rejected=0;for(const input of [{...base,manifest_json:'x'.repeat(1024*1024+1)},{...base,files_base64:{'../unsafe':'AA=='}},{...base,files_base64:{'A.bin':'AA==','a.bin':'AA=='}},{...base,files_base64:{'bad.bin':'%%%'}},{...base,files_base64:Object.fromEntries(Array.from({length:101},(_,i)=>[`${i}.bin`,'AA==']))},{...base,files_base64:{'large.bin':Buffer.alloc(16*1024*1024+1).toString('base64')}}]){assert.throws(()=>engine.execute({operation:'verify-rplan-package',input}));rejected++;}
// Invoke the actual native package command on each materialized negative fixture.
for(const [index,input,pattern,exitCode]of [[0,tampered,/SHA-256 mismatch/,1],[1,rehashed,/certificate verification failed/,1],[2,{...base,manifest_json:JSON.stringify(role)},/missing required role/,1],[3,failedInput,/certificate verified/,0]]){
 const dir=`${root}/cli-${index}`;await fs.mkdir(dir,{recursive:true});await fs.writeFile(dir+'/manifest.json',input.manifest_json);for(const [path,encoded]of Object.entries(input.files_base64))await fs.writeFile(dir+'/'+path,Buffer.from(encoded,'base64'));
 const process=spawnSync('target/debug/bisect.exe',['verify','--manifest',dir+'/manifest.json'],{encoding:'utf8'});assert.equal(process.status,exitCode);assert.match(process.stderr,pattern);cli++;
}
await fs.writeFile(root+'/result.json',JSON.stringify(baseResult));await fs.writeFile(root+'/input.json',JSON.stringify(base));await fs.writeFile(root+'/evidence.json',JSON.stringify({native_wasm_cases:parity,actual_cli_cases:cli,boundary_rejections:rejected,rehashed_certificate_tamper_rejected:true,all_public_packages_verified:true,extra_selected_files_disclosed:true},null,2));
console.log(`RPLAN package verification: ${parity} native/WASM results, ${cli} actual CLI cases, ${rejected} boundary rejections; every public package, missing/hash/role failures and rehashed certificate tampering passed.`);

const helpers=await import('../../web/lab/package-verification-files.js');
const {createProject,parseProject}=await import('../../web/lab/project.js');
const sources=input=>({manifest:{name:'manifest.json',base64:Buffer.from(input.manifest_json).toString('base64')},files:Object.fromEntries(Object.entries(input.files_base64).map(([path,base64])=>[path,{name:path.split('/').at(-1),base64}]))});
const fake=(path,bytes)=>({name:path.split('/').at(-1),webkitRelativePath:'package/'+path,size:bytes.length,arrayBuffer:async()=>Uint8Array.from(bytes).buffer});
for(const [name,input,out]of [['project',base,baseResult],['recorded-fail-project',failedInput,failedOut],['tampered-project',rehashed,cases.find(c=>c.req.input===rehashed).out]]){
 const files=sources(input);assert.deepEqual(helpers.packageVerificationRequest(files),{operation:'verify-rplan-package',input});
 const project=createProject({name:'RPLAN package verification',files:{packageVerificationInputs:files},operation:'verify-rplan-package',constraints:[],result:out,lastOperation:'verify-rplan-package'});
 assert.deepEqual(parseProject(JSON.stringify(project)),project);await fs.writeFile(root+'/'+name+'.bisect',JSON.stringify(project));await fs.writeFile(root+'/'+name+'-result.json',JSON.stringify(out));
 assert.deepEqual(JSON.parse(new TextDecoder().decode(helpers.packageVerificationReceiptBytes(out))),out);
}
const nestedManifest=JSON.parse(base.manifest_json);for(const f of nestedManifest.files)f.path='nested/'+f.path;
const nestedInput={manifest_json:JSON.stringify(nestedManifest),files_base64:Object.fromEntries(Object.entries(base.files_base64).map(([p,b])=>['nested/'+p,b]))};
const folder=await helpers.readPackageVerificationFolder([fake('manifest.json',Buffer.from(nestedInput.manifest_json)),...Object.entries(nestedInput.files_base64).map(([p,b])=>fake(p,Buffer.from(b,'base64')))]);
assert.deepEqual(helpers.packageVerificationRequest(folder).input,nestedInput);assert.equal(engine.execute(helpers.packageVerificationRequest(folder)).status,'verified');
const binary=await helpers.readPackageVerificationFiles([fake('binary.bin',Buffer.from([0,255,128])),fake('empty.bin',Buffer.alloc(0))]);assert.equal(binary['binary.bin'].base64,'AP+A');assert.equal(binary['empty.bin'].base64,'');
const raw='{'+base.manifest_json.trim().slice(1,-1)+',"metadata":18446744073709551615}';assert.equal(helpers.packageVerificationRequest(sources({...base,manifest_json:raw})).input.manifest_json,raw);
let reads=0;const oversized={name:'oversized.bin',size:16*1024*1024+1,arrayBuffer:async()=>{reads++;return new ArrayBuffer(0);}};await assert.rejects(()=>helpers.readPackageVerificationFiles([oversized]));assert.equal(reads,0);
await assert.rejects(()=>helpers.readPackageVerificationFiles([fake('A.bin',Buffer.alloc(0)),fake('a.bin',Buffer.alloc(0))]),/colliding/);
await assert.rejects(()=>helpers.readPackageVerificationFiles([{...fake('unsafe',Buffer.alloc(0)),name:'../unsafe'}]),/Unsafe/);
await assert.rejects(()=>helpers.readPackageVerificationFolder([fake('nested/manifest.json',Buffer.from('{}'))]),/root/);
await assert.rejects(()=>helpers.readPackageVerificationManifest(fake('manifest.json',Buffer.from([255]))));
await assert.rejects(()=>helpers.readPackageVerificationFiles([{...fake('changed.bin',Buffer.alloc(0)),size:1}]),/changed/);
assert.throws(()=>helpers.packageVerificationReceiptBytes({...baseResult,native_exit_code:1}));
console.log('Package sources: nested folder, exact binary/empty bytes, raw u64 metadata, project roundtrips, receipt content and pre-read/path/UTF-8/stale-size guards passed.');
