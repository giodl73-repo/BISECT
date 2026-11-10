// Explicit validation prerequisite acquisition; never silently invoked by builds.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {downloadPublicSource} from '../../web/lab/fetch-network.js';
const states=process.argv.slice(2);
if(!states.length||states.some(state=>!/^[A-Z]{2}$/.test(state))||new Set(states).size!==states.length)throw new Error('Supply unique uppercase state codes, for example TX MD.');
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const plan=engine.execute({operation:'plan-fetch-sources',input:{manifest_json:null,states,year:'2020',data_types:[],selected_paths:[]}});
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
for(const state of states){
 const items=plan.items.filter(item=>item.state_code===state&&['tiger','pl94171'].includes(item.kind));
 assert.equal(items.length,2,'Native Fetch must provide both pinned source archives for '+state);
 const root=path.resolve('target/original-state-acquisition',state);await fs.mkdir(root,{recursive:true});
 for(const item of items){
  const filename=path.join(root,item.kind+'.zip'),receiptFile=filename+'.receipt.json';
  try{
   const existing=await fs.readFile(filename),receipt=JSON.parse(await fs.readFile(receiptFile,'utf8'));
   assert.equal(receipt.url,item.url);assert.equal(receipt.bytes,existing.length);assert.equal(receipt.sha256,sha(existing));
   console.log(state,item.kind,'retained verified acquisition',existing.length);continue;
  }catch(error){if(error.code!=='ENOENT')throw error;}
  // Existing archives without a matching receipt are never overwritten.
  try{await fs.stat(filename);throw new Error('Existing archive lacks its matching acquisition receipt: '+filename);}catch(error){if(error.code!=='ENOENT')throw error;}
  console.log(state,item.kind,'acquiring native Fetch URL',item.url);
  const response=await downloadPublicSource(item.url,{maxBytes:(item.kind==='tiger'?64:128)*1024*1024,largeArchive:true,timeoutMs:300000,retryAttempts:2});
  assert.equal(response.status,200);assert(response.bytes?.length);
  const receipt={state,year:'2020',kind:item.kind,url:item.url,bytes:response.bytes.length,sha256:sha(response.bytes),fetch_manifest_sha256:plan.manifest_sha256,attempt_status:response.attempt_status,observed_at_utc:new Date().toISOString(),scope:'Observed retained response bytes; digest is integrity evidence, not source authentication.'};
  await fs.writeFile(filename,response.bytes,{flag:'wx'});await fs.writeFile(receiptFile,JSON.stringify(receipt,null,2)+'\n',{flag:'wx'});
  console.log(state,item.kind,'saved',receipt.bytes,receipt.sha256);
 }
}
