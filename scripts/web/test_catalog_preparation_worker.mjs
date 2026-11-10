import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {runInNewContext} from 'node:vm';
import {prepareBuildCatalogJobs} from '../../web/lab/config-catalog-preparation.js';
import {readConfigCatalog,loadConfigCatalogJobs} from '../../web/lab/config-catalog.js';
import {validateLabelBuildInputs,LABEL_BUILD_DEFAULTS} from '../../web/lab/label-build-files.js';
const root=process.argv[2]||'dist/wasm-lab-build';
const fetcher=async p=>new Response(await fs.readFile(root+'/'+p.replace(/^\.\.\//,'')));
const catalog=await readConfigCatalog(fetcher),national=process.argv.includes('--national'),keys=(national?catalog.entries:catalog.entries.slice(0,2)).map(e=>e.key),chamber=national?'congressional':'house';
if(national)assert.equal(keys.length,50);
const inputs={config:null,jobs:null,registry:null,settings:{...LABEL_BUILD_DEFAULTS}};
const code=(await fs.readFile('web/lab/config-catalog-worker.js','utf8')).replace(/^import .*;\r?\n/gm,'');
const messages=[];
const scope={self:{postMessage:m=>messages.push(m)},loadConfigCatalogJobs:(c,k,ch)=>loadConfigCatalogJobs(c,k,ch,fetcher),validateLabelBuildInputs};
runInNewContext(code,scope);
await scope.self.onmessage({data:{type:'prepare-build-jobs',catalog,keys,chamber,inputs}});
assert.equal(messages[0].ok,true);
const direct=await loadConfigCatalogJobs(catalog,keys,chamber,fetcher);
assert.deepEqual(messages[0].value.source,direct.source);
assert.deepEqual(messages[0].value.inputs.jobs,direct.source);
assert.deepEqual(messages[0].value.inputs.settings.years,direct.years);
assert.equal(messages[0].value.inputs.config,null);
await scope.self.onmessage({data:{type:'prepare-build-jobs',catalog,keys,chamber:'house',inputs}});
assert.match(messages[1].error,/already used/);
let fetched=false;
const invalid={self:{postMessage:m=>messages.push(m)},loadConfigCatalogJobs:()=>{fetched=true;},validateLabelBuildInputs};
runInNewContext(code,invalid);
await invalid.self.onmessage({data:{type:'prepare-build-jobs',catalog,keys,chamber:'house',inputs,extra:true}});
assert.equal(fetched,false);assert.match(messages.at(-1).error,/Invalid/);
let instances=[];
class WorkerFixture{
 constructor(url,options){assert.match(url.href,/config-catalog-worker.js\?v=build-catalog-worker-v1$/);assert.equal(options.type,'module');instances.push(this);}
 postMessage(message){this.message=message;}
 terminate(){this.terminated=true;}
}
const options={WorkerClass:WorkerFixture};
let pending=prepareBuildCatalogJobs(catalog,keys,'house',inputs,options),w=instances.at(-1);
assert.equal(w.message.type,'prepare-build-jobs');w.onmessage({data:messages[0]});assert.deepEqual(await pending,messages[0].value);assert.equal(w.terminated,true);
const controller=new AbortController();pending=prepareBuildCatalogJobs(catalog,keys,'house',inputs,{...options,signal:controller.signal});w=instances.at(-1);controller.abort();await assert.rejects(pending,/cancelled/);assert.equal(w.terminated,true);
// A late success after cancellation cannot revive the rejected result.
w.onmessage({data:messages[0]});
const before=instances.length;await assert.rejects(()=>prepareBuildCatalogJobs(catalog,keys,'house',inputs,{...options,signal:controller.signal}),/cancelled/);assert.equal(instances.length,before);
pending=prepareBuildCatalogJobs(catalog,keys,'house',inputs,{...options,timeoutMs:5});w=instances.at(-1);await assert.rejects(pending,/timed out/);assert.equal(w.terminated,true);
pending=prepareBuildCatalogJobs(catalog,keys,'house',inputs,options);w=instances.at(-1);w.onerror({});await assert.rejects(pending,/worker failed/);assert.equal(w.terminated,true);
pending=prepareBuildCatalogJobs(catalog,keys,'house',inputs,options);w=instances.at(-1);w.onmessage({data:{ok:false,error:'hash mismatch'}});await assert.rejects(pending,/hash mismatch/);assert.equal(w.terminated,true);
await assert.rejects(()=>prepareBuildCatalogJobs(catalog,keys,'house',inputs,{...options,timeoutMs:300001}),/timeout/);
await fs.mkdir('target/label-build-catalog',{recursive:true});await fs.writeFile('target/label-build-catalog/'+(national?'national-worker-evidence.json':'worker-evidence.json'),JSON.stringify({states:keys.length,chamber,exact_real_graph_sources:true,worker_validated_inputs:true,engine_execution:false,cancellation_cleanup:true,timeout_cleanup:true,late_reply_ignored:true},null,2));
console.log('Catalog preparation worker: exact real graph sources, validated inputs, no engine, cancellation/late reply, timeout and failure cleanup passed.');
