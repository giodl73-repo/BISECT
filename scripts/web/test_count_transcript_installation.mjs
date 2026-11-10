import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {encodeBytes} from '../../web/lab/package-files.js';
import {reviewCountTranscriptInstallation,applyCountTranscriptInstallation,nativeCountTranscriptBytes} from '../../web/lab/count-transcript-install.js';

const root=path.resolve('target/count-transcript-installation/run-'+Date.now());await fs.mkdir(root,{recursive:true});
const wasm=path.resolve(process.env.BISECT_TEST_WASM||'target/wasm32-unknown-unknown/release/bisect_wasm.wasm'),native=path.resolve(process.env.BISECT_TEST_RCOUNT||'target/release/rcount.exe');
const hash=value=>createHash('sha256').update(value).digest('hex');
const engine=await instantiateEngine(await fs.readFile(wasm));
const source='docs/examples/rcount-golden-packages/summary-basic',target='transcripts/verify-transcript.json';
async function collect(folder,prefix=''){const out={};for(const entry of await fs.readdir(folder,{withFileTypes:true})){const name=prefix+entry.name;if(entry.isDirectory())Object.assign(out,await collect(path.join(folder,entry.name),name+'/'));else out[name]=encodeBytes(await fs.readFile(path.join(folder,entry.name)));}return out;}
function handle(folder,hooks={}){return {kind:'directory',name:path.basename(folder),queryPermission:async()=>hooks.permission||'granted',async *entries(){for(const entry of await fs.readdir(folder,{withFileTypes:true}))yield [entry.name,entry.isDirectory()?handle(path.join(folder,entry.name),hooks):file(path.join(folder,entry.name),hooks)];},async getDirectoryHandle(name,{create=false}={}){const p=path.join(folder,name);if(create)await fs.mkdir(p,{recursive:true});else await fs.stat(p);return handle(p,hooks);},async getFileHandle(name,{create=false}={}){const p=path.join(folder,name);if(create){const stream=await fs.open(p,'a');await stream.close();}else await fs.stat(p);return file(p,hooks);}};}
function file(name,hooks){return {kind:'file',async getFile(){const b=await fs.readFile(name);return {size:b.length,arrayBuffer:async()=>Uint8Array.from(b).buffer};},async createWritable(){let data;return {async write(value){data=value;},async close(){await fs.writeFile(name,data);hooks.afterWrite?.();},async abort(){}};}};}
async function fixture(name){const dir=path.join(root,name);await fs.cp(source,dir,{recursive:true});return dir;}
function cli(dir){try{return {code:0,stdout:execFileSync(native,['verify',dir,'--write-transcript'],{encoding:'utf8'})};}catch(error){return {code:error.status,stdout:error.stdout,stderr:error.stderr};}}
const outcomes=[];
for(const [name,tamper,remove]of [['add',false,true],['replace',false,false],['failed-verification',true,false]]){
 const dir=await fixture(name),oracle=await fixture(name+'-native');if(remove){await fs.unlink(path.join(dir,target));await fs.unlink(path.join(oracle,target));}
 if(tamper){for(const folder of [dir,oracle]){const all=await collect(folder),key=Object.keys(all).find(x=>x.startsWith('sources/')&&x!=='sources/source-index.json');await fs.appendFile(path.join(folder,key),'\nchanged');}}
 const inputs=await collect(dir),result=engine.execute({operation:'verify-count-files',files:Object.fromEntries(Object.entries(inputs).map(([k,v])=>[k,[...Buffer.from(v,'base64')]]))});
 const expected=cli(oracle);assert.equal(expected.code,tamper?1:0);assert.equal(result.status,tamper?'fail':'pass');
 const original=await collect(dir),review=await reviewCountTranscriptInstallation(engine,result,inputs,handle(dir));assert.equal(review.action,remove?'add':'replace');
 await assert.rejects(()=>applyCountTranscriptInstallation({...review},{confirmed:true}),/Review/);
 await assert.rejects(()=>applyCountTranscriptInstallation(review),/Confirm/);
 const receipt=await applyCountTranscriptInstallation(review,{confirmed:true});assert.equal(receipt.readback_verified,true);
 const actual=await fs.readFile(path.join(dir,target)),nativeBytes=await fs.readFile(path.join(oracle,target));assert.deepEqual(actual,nativeBytes);assert.deepEqual(nativeCountTranscriptBytes(result),Uint8Array.from(nativeBytes));
 const final=await collect(dir);for(const key of Object.keys(original).filter(k=>k!==target))assert.equal(final[key],original[key]);
 await assert.rejects(()=>applyCountTranscriptInstallation(review,{confirmed:true}),/Review/);
 await fs.writeFile(path.join(root,name+'-result.json'),JSON.stringify({result,receipt,native_exit:expected.code,sha256:hash(actual)},null,2));outcomes.push({name,native_exit:expected.code,transcript_sha256:hash(actual)});
}
const dir=await fixture('guards'),inputs=await collect(dir),request={operation:'verify-count-files',files:Object.fromEntries(Object.entries(inputs).map(([k,v])=>[k,[...Buffer.from(v,'base64')]]))},result=engine.execute(request);
await assert.rejects(()=>reviewCountTranscriptInstallation(engine,result,inputs,handle(dir,{permission:'denied'})),/permission/);
const review=await reviewCountTranscriptInstallation(engine,result,inputs,handle(dir));await fs.writeFile(path.join(dir,target),'concurrent edit');await assert.rejects(()=>applyCountTranscriptInstallation(review,{confirmed:true}),/changed/);assert.equal(await fs.readFile(path.join(dir,target),'utf8'),'concurrent edit');
const packageReview=await reviewCountTranscriptInstallation(engine,result,inputs,handle(dir));await fs.writeFile(path.join(dir,'unexpected.txt'),'extra');await assert.rejects(()=>applyCountTranscriptInstallation(packageReview,{confirmed:true}),/roster/);
const altered=await fixture('altered-source');await fs.writeFile(path.join(altered,'manifest.json'),'{}');await assert.rejects(()=>reviewCountTranscriptInstallation(engine,result,inputs,handle(altered)),/differ/);
const blocked=await fixture('blocked-destination');await fs.unlink(path.join(blocked,target));await fs.mkdir(path.join(blocked,target));const blockedCli=cli(blocked);assert.equal(blockedCli.code,2);await assert.rejects(()=>reviewCountTranscriptInstallation(engine,result,inputs,handle(blocked)),/destination is a directory/);await fs.writeFile(path.join(root,'native-write-failure.json'),JSON.stringify(blockedCli,null,2));
const cancelled=await fixture('cancelled'),controller=new AbortController(),cancelReview=await reviewCountTranscriptInstallation(engine,result,inputs,handle(cancelled,{afterWrite:()=>controller.abort(new Error('test after committed write'))}));
await assert.rejects(()=>applyCountTranscriptInstallation(cancelReview,{confirmed:true,signal:controller.signal}),error=>{assert.equal(error.installationReceipt.status,'cancelled');assert.equal(error.installationReceipt.readback_verified,false);return true;});assert.deepEqual(await fs.readFile(path.join(cancelled,target)),Buffer.from(nativeCountTranscriptBytes(result)));
const evidence={wasm:{path:wasm,sha256:hash(await fs.readFile(wasm))},native:{path:native,sha256:hash(await fs.readFile(native))},outcomes,exact_native_transcript_cases:3,native_write_failure_exit:blockedCli.code,browser_directory_destination_refused:true,imported_review_refused:true,confirmation_required:true,consumed_review_refused:true,changed_destination_refused:true,changed_package_refused:true,unexpected_file_refused:true,permission_refused:true,cancel_after_real_write_retained:true,certification_authority:false,full_parity:false};
await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify(evidence,null,2)+'\n');console.log(JSON.stringify({root,evidence}));
