import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {fletchStageRequest,fletchStageFromAcquisition,validatedFletchCacheObject} from '../../web/lab/fletch-stage-files.js';
import {fletchHistoryFromEntry} from '../../web/lab/fletch-history.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {createWorkbenchArchive,parseWorkbenchArchive} from '../../web/lab/workbench-archive.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const native=JSON.parse(execFileSync('target/debug/examples/fletch_retry_reference.exe',['500,500,200','2','30000','target/fletch-history-native'],{encoding:'utf8'}));
assert.equal(native.requests,3);
const base=JSON.parse(await fs.readFile('target/fletch-stage-wasm/text.bisect','utf8')).files.fletchStageInputs;
for(const history of [{attempts:native.entry_attempts,retries:native.entry_retries,last_retryable_error:native.attempt_status.last_retryable_error},{attempts:0,retries:0,last_retryable_error:null},{attempts:4294967295,retries:4294967295,last_retryable_error:'historical diagnostic'}]){
 const input={...base,fetch_history:history},result=engine.execute(fletchStageRequest(input));
 assert.deepEqual(fletchHistoryFromEntry(result.entries_json),history);await validatedFletchCacheObject(result,input);
 const project=createProject({name:'Retained history',files:{fletchStageInputs:input},operation:'stage-fletch-cache-source',constraints:[],result,lastOperation:'stage-fletch-cache-source'});
 assert.deepEqual(parseProject(JSON.stringify(project)).files.fletchStageInputs.fetch_history,history);
 assert.deepEqual(parseWorkbenchArchive(createWorkbenchArchive(project)).files.fletchStageInputs.fetch_history,history);
 await assert.rejects(()=>validatedFletchCacheObject(result,{...input,fetch_history:{...history,attempts:history.attempts===0?1:0}}),/metadata/);
}
const acquired=JSON.parse(await fs.readFile('target/fetch-acquisition/retried.bisect','utf8')),input=fletchStageFromAcquisition(acquired.files);
assert.deepEqual(input.fetch_history,{attempts:2,retries:1,last_retryable_error:'HTTP status 500'});
await validatedFletchCacheObject(engine.execute(fletchStageRequest(input)),input);
for(const history of [{attempts:-1,retries:0,last_retryable_error:null},{attempts:4294967296,retries:0,last_retryable_error:null},{attempts:1,retries:0,last_retryable_error:'é'.repeat(8193)},{attempts:1,retries:0,last_retryable_error:null,extra:true}]){
 assert.throws(()=>fletchStageRequest({...base,fetch_history:history}));
 assert.throws(()=>engine.execute({operation:'stage-fletch-cache-source',input:{...fletchStageRequest(base).input,fetch_history:history}}));
}
assert.throws(()=>fletchHistoryFromEntry('[{"fetch_attempts":1,"fetch_attempts":2,"retry_count":0,"last_retryable_error":null}]'));
console.log('Fetch history: actual native retry counts preserved in WASM staging; zero/u32 bounds, diagnostics, project/archive round trips, retried acquisition and mismatched output/invalid metadata guards passed.');
