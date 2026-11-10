import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {spawn} from 'node:child_process';
import {createHash} from 'node:crypto';
import {createNativeAggregateOracle} from './fixtures/native-aggregate-effects.mjs';
const repo=path.resolve(fileURLToPath(new URL('../..',import.meta.url)));
assert.equal(path.resolve(),repo,'Run the suite from its source repository root.');
const wasmPath=path.resolve(process.argv[2]||process.env.BISECT_TEST_WASM||'target/wasm32-unknown-unknown/release/bisect_wasm.wasm'),nativePath=path.resolve(process.argv[3]||process.env.BISECT_TEST_NATIVE_CLI||('target/release/bisect'+(process.platform==='win32'?'.exe':''))),browser=process.argv.includes('--browser'),site=path.resolve(process.env.BISECT_TEST_WEB_SOURCE||'web/lab');
await fs.mkdir('target/native-aggregate-effects',{recursive:true});const out=await fs.mkdtemp(path.resolve('target/native-aggregate-effects/suite-')),wasm=await fs.readFile(wasmPath),sha=b=>createHash('sha256').update(b).digest('hex'),ownedWasm=path.join(out,'engine.wasm');await fs.writeFile(ownedWasm,wasm);
const steps=[],sourceHashes={},status={status:'running',browser_requested:browser,wasm:{source:wasmPath,owned:ownedWasm,sha256:sha(wasm)},native_source:nativePath,site,steps,scope:'Fresh actual native Aggregate fixtures, current-source Node/native-action and optional actual-browser/backup/UI tests. No automatic full parent or host/capacity acceptance.'};
const save=()=>fs.writeFile(path.join(out,'evidence.json'),JSON.stringify({...status,source_hashes:sourceHashes},null,2));
await save();
try{
 const oracle=await createNativeAggregateOracle(nativePath);status.native_oracle=oracle.directory;await save();
 const env={...process.env,BISECT_TEST_WASM:ownedWasm,BISECT_TEST_NATIVE_CLI:nativePath,BISECT_TEST_WEB_SOURCE:site,BISECT_AGGREGATE_ORACLE:oracle.directory};
 async function run(name,args=[]){const source=path.join(repo,'scripts/web',name);sourceHashes['scripts/web/'+name]=sha(await fs.readFile(source));const row={name,status:'running',stdout:path.join(out,name+'.stdout.log'),stderr:path.join(out,name+'.stderr.log')};steps.push(row);await save();console.log('Aggregate suite: '+name);const chunks=[],errors=[];const child=spawn(process.execPath,[source,...args],{cwd:repo,env,windowsHide:true});child.stdout.on('data',b=>chunks.push(b));child.stderr.on('data',b=>errors.push(b));const code=await new Promise((resolve,reject)=>{child.once('error',reject);child.once('close',resolve);});const stdout=Buffer.concat(chunks),stderr=Buffer.concat(errors);await fs.writeFile(row.stdout,stdout);await fs.writeFile(row.stderr,stderr);row.exit_code=code;row.status=code===0?'passed':'failed';await save();assert.equal(code,0,name+' failed; retained '+row.stderr);const roots=stdout.toString('utf8').split(/\r?\n/).flatMap(line=>{try{const v=JSON.parse(line);return typeof v.native_aggregate_evidence_root==='string'?[v.native_aggregate_evidence_root]:[];}catch{return [];}});assert.equal(roots.length,1,'Missing unique output receipt: '+name);row.output=roots[0];await save();return row.output;}
 await run('test_native_aggregate_effect_plan.mjs',[oracle.directory]);
 env.BISECT_AGGREGATE_APPLICATION_EVIDENCE=await run('test_native_aggregate_effect_application.mjs',[oracle.directory]);
 await run('test_native_aggregate_effect_recompute.mjs');
 await run('test_native_aggregate_effect_recovery.mjs',[oracle.directory]);
 await run('test_native_aggregate_workspace_inspection.mjs');
 await run('test_native_aggregate_workspace_backup.mjs');
 if(browser){for(const name of ['test_native_aggregate_effect_io_browser.mjs','test_native_aggregate_effect_coordination_browser.mjs','test_native_aggregate_effect_persistence_browser.mjs','test_native_aggregate_effect_recovery_browser.mjs','test_native_aggregate_workspace_browser.mjs','test_native_aggregate_workspace_backup_browser.mjs','test_native_aggregate_incomplete_workspace_browser.mjs'])await run(name);}
 assert.equal(sha(await fs.readFile(wasmPath)),status.wasm.sha256,'Source WASM changed during suite.');status.status='passed';await save();console.log('PASS native Aggregate action suite '+steps.length+' steps: '+out);
}catch(error){status.status='failed';status.error={message:error.message,stack:error.stack};await save();console.error('Retained failed Aggregate suite: '+out);throw error;}
