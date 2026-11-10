import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {createServer} from 'node:http';
import {execFileSync} from 'node:child_process';
import {downloadPublicSource,validateFetchRequestOptions} from '../../web/lab/fetch-network.js';
const root='target/fetch-network';await fs.mkdir(root,{recursive:true});
for(const [name,states,retries,timeout]of [['first',['200'],3,30000],['one-retry',['500','200'],1,30000],['two-retries',['500','500','200'],2,30000],['unauthorized',['401','200'],1,30000],['rate-limit',['429','200'],1,30000],['body-read',['truncate','200'],1,30000],['exhausted',['500','500'],1,30000],['missing',['404','404'],1,30000],['timeout',['delay','200'],1,3000]]){
 const native=JSON.parse(execFileSync('target/debug/examples/fletch_retry_reference.exe',[states.join(','),String(retries),String(timeout),root+'/'+name],{encoding:'utf8'}));
 assert(Array.isArray(native.server_handlers),'Rebuild the native reference with HTTP handler diagnostics');assert.equal(native.server_handlers.length,native.requests,name);
 for(const [index,status]of native.server_handlers.entries()){assert.match(status,/^Ok\(/,'Native HTTP fixture thread failed');if(states[Math.min(index,states.length-1)]!=='delay')assert.equal(status,'Ok(Ok(()))',name+': native HTTP fixture handler failed');}
 let calls=0;const server=createServer((req,res)=>{const state=states[Math.min(calls++,states.length-1)];const finish=()=>{const status=Number(state)||200;res.writeHead(status,{'Content-Length':state==='truncate'?'14':status===200?'4':'0'});res.end(status===200?'body':'');};if(state==='delay')setTimeout(finish,6000);else finish();});await new Promise(r=>server.listen(0,'127.0.0.1',r));
 let result,error;try{result=await downloadPublicSource('http://127.0.0.1:'+server.address().port+'/source',{maxBytes:1024,retryAttempts:retries,timeoutMs:timeout});}catch(e){error=e;}finally{server.closeAllConnections();await new Promise(r=>server.close(r));}
 await fs.writeFile(root+'/'+name+'-observed.json',JSON.stringify({native,browser:result?{status:result.status,attempt_status:result.attempt_status}:{error:error?.message},calls},null,2));
 assert.equal(calls,native.requests,name);
 if(native.attempt_status){assert.equal(result.attempt_status.attempts,native.attempt_status.attempts);assert.equal(result.attempt_status.retries,native.attempt_status.retries);assert.equal(result.attempt_status.last_retryable_error===null,native.attempt_status.last_retryable_error===null);assert.equal(native.entry_attempts,result.attempt_status.attempts);assert.equal(native.entry_retries,result.attempt_status.retries);assert.equal(new TextDecoder().decode(result.bytes),'body');}
 else if(states.every(s=>s==='404'))assert.equal(result.status,404);else assert(error,name+': '+JSON.stringify(native));
 await fs.writeFile(root+'/'+name+'.json',JSON.stringify({native,browser:result?{status:result.status,attempt_status:result.attempt_status}:{error:error.message},calls},null,2));
}
let calls=0;const fail=async()=>{calls++;throw new Error('private request URL &key=secret');};await assert.rejects(()=>downloadPublicSource('https://example.test/',{fetcher:fail,maxBytes:10,retryAttempts:2}),e=>!e.message.includes('secret'));assert.equal(calls,3);
for(const response of [new Response('body',{headers:{'content-length':'100'}}),{redirected:true,url:'https://other.test'}]){calls=0;await assert.rejects(()=>downloadPublicSource('https://example.test/',{fetcher:async()=>{calls++;return response;},maxBytes:10,retryAttempts:3}));assert.equal(calls,1);}
const cancel=new AbortController();calls=0;await assert.rejects(()=>downloadPublicSource('https://example.test/',{signal:cancel.signal,maxBytes:10,retryAttempts:3,fetcher:async()=>{calls++;cancel.abort();throw new Error('cancelled');}}));assert.equal(calls,1);
const started=Date.now();await assert.rejects(()=>downloadPublicSource('https://example.test/',{maxBytes:10,timeoutMs:20,fetcher:()=>new Promise(()=>{})}),/timed out/);assert(Date.now()-started<2000);
for(const options of [{retryAttempts:11},{retryAttempts:-1},{timeoutMs:0},{timeoutMs:NaN}])assert.throws(()=>validateFetchRequestOptions(options));
console.log('Source retry/timeout: nine actual native HTTP/stream/timeout cases match request/attempt/retry counts and outcomes; transport retry, secret-free errors, guard no-retry, cancellation, hung transport deadline and option bounds pass.');
