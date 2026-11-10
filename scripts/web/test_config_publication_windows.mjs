import fs from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
import os from 'node:os';
import {spawn,spawnSync} from 'node:child_process';
import {pathToFileURL,fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
const option=n=>process.argv.find(a=>a.startsWith('--'+n+'='))?.slice(n.length+3);
const R=path.resolve(option('repo')||'.'),reference=path.resolve(option('reference')||'');
if(!option('reference'))throw Error('Pass --reference=<actual Config CLI cases>.');
const P=path.dirname(fileURLToPath(import.meta.url));
const {instantiateEngine}=await import(pathToFileURL(path.join(R,'web/lab/wasm-engine.js')));
const {reviewConfigPublication,applyConfigPublication}=await import(pathToFileURL(path.join(R,'web/lab/config-publication.js')));
const {nativeOutputDirectoryHarness}=await import(pathToFileURL(path.join(R,'scripts/web/fixtures/native-output-directory.mjs')));
const sha=raw=>createHash('sha256').update(raw).digest('hex');
await fs.mkdir(path.join(R,'target/config-cli-contract'),{recursive:true});
const output=await fs.mkdtemp(path.join(R,'target/config-cli-contract/windows-sharing-'));
const evidence={status:'running',os:{platform:os.platform(),release:os.release(),version:os.version(),arch:os.arch()},cases:[],scope:'Actual Windows FileShare.Read locking against the owned native CLI and Node directory facade backed by real Windows files. No real browser FSA permission or OS folder grant is exercised. No ACL/security setting is changed.'};
assert.equal(os.platform(),'win32');
try{
 for(const mode of [option('profile')||'selected']){
  const source=JSON.parse(await fs.readFile(path.join(reference,'evidence.json'),'utf8')),base=source.records.find(r=>r.name==='defaults');
  assert.equal(source.status,'passed-actual-CLI-config-contract');
  assert(base);
  const wasmPath=path.resolve(R,source.wasm.path),wasm=await fs.readFile(wasmPath),engine=await instantiateEngine(wasm);assert.equal(sha(wasm),source.wasm.sha256);
  const result=engine.execute(base.request);assert.equal(result.yaml_text,base.result.yaml_text);
  const harness=nativeOutputDirectoryHarness(output),dir=await harness.directory(mode),root=dir.dir,file=path.join(dir.dir,'config.yml'),old=Buffer.from('Original owned test bytes retained through the Windows write lock.\n');
  await fs.writeFile(path.join(root,'owned-test-marker.json'),JSON.stringify({owned_fixture:true,mode}));
  await fs.writeFile(file,old);await fs.writeFile(path.join(root,'before.bin'),old);
  const lock=spawn('pwsh',['-NoProfile','-File',path.join(P,'fixtures/hold-read-share.ps1'),'-Root',root,'-Path',file],{windowsHide:true,stdio:['ignore','pipe','pipe']});
  // The lock target must be inside the same declared ownership root.
  let stdout='',stderr='';lock.stdout.on('data',b=>stdout+=b);lock.stderr.on('data',b=>stderr+=b);let lockCode=null;
  const exited=new Promise(resolve=>lock.on('exit',code=>{lockCode=code;resolve(code);}));
  const row={mode,native_executable_sha256:sha(await fs.readFile(path.resolve(R,source.native.path))),wasm_sha256:source.wasm.sha256};assert.equal(row.native_executable_sha256,source.native.sha256);
  try{
   const deadline=Date.now()+15000;let ready;
   while(Date.now()<deadline){try{ready=JSON.parse((await fs.readFile(path.join(root,'lock-ready.json'),'utf8')).replace(/^\uFEFF/,''));break;}catch(error){if(error.code!=='ENOENT')throw error;}if(lockCode!==null)throw Error('Lock helper exited: '+stderr);await new Promise(resolve=>setTimeout(resolve,50));}
   assert(ready,'Lock helper did not become ready');row.lock=ready;
   const argv=[...base.native.arguments],outIndex=argv.indexOf('--out')+1;argv[outIndex]=file;argv.push('--force');const native=path.resolve(R,source.native.path);
   const denied=spawnSync(native,argv,{windowsHide:true,maxBuffer:4*1024*1024});row.native_denied={argv,exit_code:denied.status,stdout:denied.stdout?.toString(),stderr:denied.stderr?.toString()};assert.equal(denied.status,1,denied.error?.message);assert.deepEqual(await fs.readFile(file),old);
   const review=await reviewConfigPublication(engine,result,base.request.settings,{path:'config.yml',force:true,dry_run:false},dir);let interrupted;
   try{await applyConfigPublication(review,{confirmed:true});}catch(error){interrupted=error.configPublicationReceipt;row.facade_error=error.message;}
   assert(interrupted);assert.equal(interrupted.status,'interrupted');assert.equal(interrupted.readback_verified,false);assert.equal(interrupted.close_completed,false);row.facade_interrupted=interrupted;assert.deepEqual(await fs.readFile(file),old);
   await fs.writeFile(path.join(root,'locked-after.bin'),await fs.readFile(file));
   await fs.writeFile(path.join(root,'release-lock'),'release');assert.equal(await exited,0,stderr);
   const resumed=spawnSync(native,argv,{windowsHide:true,maxBuffer:4*1024*1024});row.native_after_release={exit_code:resumed.status,stdout:resumed.stdout?.toString(),stderr:resumed.stderr?.toString()};assert.equal(resumed.status,0);assert.equal(await fs.readFile(file,'utf8'),result.yaml_text);
   await assert.rejects(applyConfigPublication(review,{confirmed:true}));
   await fs.writeFile(file,old);const fresh=await reviewConfigPublication(engine,result,base.request.settings,{path:'config.yml',force:true,dry_run:false},dir);row.facade_after_fresh_review=await applyConfigPublication(fresh,{confirmed:true});assert.equal(row.facade_after_fresh_review.status,'completed');assert(row.facade_after_fresh_review.readback_verified);assert.equal(await fs.readFile(file,'utf8'),result.yaml_text);
   await fs.writeFile(path.join(root,'completed.bin'),await fs.readFile(file));row.original_bytes_preserved_during_lock=true;row.used_review_refused_after_failure=true;row.status='passed';
  }finally{
   await fs.writeFile(path.join(root,'release-lock'),'release');row.lock_helper_exit_code=await exited;await fs.writeFile(path.join(root,'lock.stdout'),stdout);await fs.writeFile(path.join(root,'lock.stderr'),stderr);
  }
  evidence.cases.push(row);await fs.writeFile(path.join(output,'evidence.json'),JSON.stringify(evidence,null,2));
 }
 evidence.status='passed-real-windows-sharing-lock-and-retry';
}catch(error){evidence.status='failed';evidence.error={message:error.message,stack:error.stack};process.exitCode=1;}
finally{await fs.writeFile(path.join(output,'evidence.json'),JSON.stringify(evidence,null,2)+'\n');}
console.log(JSON.stringify({root:output,status:evidence.status,cases:evidence.cases.length,error:evidence.error}));
