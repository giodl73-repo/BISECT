// Actual browser batch workers, explicit C selection, and literal desktop assignments.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import fs from 'node:fs/promises';
import {createRequire} from 'node:module';
import path from 'node:path';
import {openWorkbenchProject} from './workbench_project_open.mjs';

const [address,packageDir,fixtureDir,out,profile]=process.argv.slice(2);
const origin=new URL(address).origin;
assert.match(origin,/^http:\/\/127\.0\.0\.1:\d+$/);assert(['c','core'].includes(profile));
const withC=profile==='c',timeout=120000;
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const read=async file=>JSON.parse(await fs.readFile(file));
const expected=await read(path.join(fixtureDir,'expected.json'));
const native=await read(path.join(fixtureDir,'native-evidence.json'));
const manifest=await read(path.join(packageDir,'assets/offline-manifest.json'));
const enginePin=manifest.assets['toolkit/bisect_wasm.wasm'];
assert.equal(enginePin.sha256===expected.wasm_sha256,withC);
const project=path.resolve(fixtureDir,expected.project_file??'real-c-batch.bisect');
const stateCodes=Object.keys(expected.states).sort();assert.equal(stateCodes.length,2);
assert.deepEqual(Object.keys(native.states).sort(),stateCodes);
await fs.mkdir(out);
const {chromium}=createRequire(path.join(process.env.BISECT_PLAYWRIGHT_PACKAGES,'package.json'))('playwright');
const browser=await chromium.launch({headless:true,
  executablePath:process.env.BISECT_TEST_BROWSER||'C:/Program Files/Google/Chrome/Application/chrome.exe',
  args:['--no-first-run','--disable-background-networking','--disable-extensions']});
let page;
try{
  const context=await browser.newContext({acceptDownloads:true});
  const sidecar=[],errors=[];
  context.on('response',r=>{
    const u=new URL(r.url());if(u.origin===origin&&u.pathname.startsWith('/toolkit/c-metis/metis-sidecar.'))
      sidecar.push({path:u.pathname,status:r.status(),cache:r.fromServiceWorker()});
  });
  await context.route('**/*',r=>new URL(r.request().url()).origin===origin?r.continue():r.abort());
  await context.addInitScript(()=>{
    Object.defineProperty(navigator,'hardwareConcurrency',{get:()=>2});
    window.batchCalls=[];window.batchReplies=[];window.injectMixedRuntime=false;
    window.selectionReplies=0;window.mixedRuntimeInjected=0;
    let id=0;const W=Worker;
    window.Worker=class extends W{
      constructor(url,...rest){super(url,...rest);this.workerId=++id;
        if(new URL(url,location.href).pathname==='/toolkit/wasm-worker.js')this.addEventListener('message',({data})=>{
          if(injectMixedRuntime&&this.selections?.has(data?.id)&&data?.type==='result'&&data.runtimeArtifacts?.metis_sidecar){
            if(++selectionReplies===2){data.runtimeArtifacts.metis_sidecar.wasm.sha256='0'.repeat(64);mixedRuntimeInjected++;}
          }
          if(['result','error'].includes(data?.type))batchReplies.push({worker:this.workerId,id:data.id,type:data.type,
            error:data.error,runtime:data.runtimeArtifacts});
        });}
      postMessage(m,...rest){if(m?.type==='run')batchCalls.push({worker:this.workerId,id:m.id,
        operation:m.request.operation,cMetis:m.cMetis??false,state:m.request.input?.scope?.state});
        if(m?.type==='run'&&m.request.operation==='inspect-state-batch-selection'){
          this.selections??=new Set();this.selections.add(m.id);
        }
        return super.postMessage(m,...rest);}
    };
  });
  const el=id=>page.locator('#'+id);
  const calls=()=>page.evaluate(()=>batchCalls.slice());
  async function visit(){
    page=await context.newPage();page.on('pageerror',e=>errors.push(e.message));
    await page.goto(origin+'/toolkit/');
    await page.waitForFunction(hash=>!document.querySelector('#execute').disabled &&
      document.querySelector('#runtime-artifact').textContent.includes(hash),enginePin.sha256,{timeout});
  }
  async function open(file){
    const previous=await calls(), assets=sidecar.length;
    await openWorkbenchProject(page,file,{timeout});
    assert.deepEqual(await calls(),previous);assert.equal(sidecar.length,assets);
    assert.equal(await el('c-metis-runtime').isChecked(),false);
    assert(await el('download-state-run-batch').isDisabled());
  }
  async function save(id,file){
    const waiting=page.waitForEvent('download',{timeout});await el(id).click({timeout});await(await waiting).saveAs(file);
  }
  async function run(label,selected){
    const previous=(await calls()).length,assets=sidecar.length;
    if(selected){await el('c-metis-runtime').check();assert.equal((await calls()).length,previous);assert.equal(sidecar.length,assets);}
    await el('execute').click({timeout});
    await page.waitForFunction(n=>batchCalls.length>n&&!document.querySelector('#execute').disabled,previous,{timeout});
    const file=path.join(out,label+'.bisect');await save('save-project',file);
    const {result}=await read(file);const dispatched=(await calls()).slice(previous);
    const partitions=dispatched.filter(c=>c.operation==='run-prepared-state-job');
    assert.deepEqual(partitions.map(c=>c.state).sort(),stateCodes);
    assert.equal(new Set(partitions.map(c=>c.worker)).size,2);
    assert(dispatched.every(c=>c.cMetis===selected));
    assert.equal(result.workers.effective,2);
    if(!selected){
      assert.equal(sidecar.length,assets);assert.equal(result.summary.failed,2);assert.equal(result.summary.succeeded,0);
      assert(result.states.every(row=>row.status==='failed'&&!row.result&&/C METIS sidecar/.test(row.error)));
      return {failures:result.states.map(row=>row.error),slots:2};
    }
    assert.equal(result.summary.succeeded,2);assert.equal(result.summary.failed,0);
    const replies=await page.evaluate(()=>batchReplies.slice());
    const states={};
    for(const row of result.states){
      const state=row.state_code, actual=row.result.prepared, want=expected.states[state];
      assert.equal(actual.request_sha256,want.request_sha256);
      assert.equal(actual.result.backend,'c-metis-msvcrt-v1');
      assert.deepEqual(actual.result.assignments,want.assignments);
      assert.deepEqual(actual.result.assignments,native.states[state].assignments);
      assert.deepEqual(actual.runtime_artifacts??null,want.runtime);
      const dispatch=partitions.find(c=>c.state===state);
      const reply=replies.find(r=>r.worker===dispatch.worker&&r.id===dispatch.id);
      assert.equal(reply.type,'result');assert.deepEqual(reply.runtime,want.loaded_worker_runtime??want.runtime);
      states[state]={units:Object.keys(actual.result.assignments).length,request_sha256:actual.request_sha256,
        assignments_sha256:sha(JSON.stringify(actual.result.assignments)),runtime:actual.runtime_artifacts??null,
        loaded_worker_runtime:reply.runtime};
    }
    const delivered=sidecar.slice(assets);
    for(const ext of ['json','mjs','wasm'])assert(delivered.some(r=>r.status===200&&r.path.endsWith('.'+ext)));
    if(label==='offline')assert(delivered.every(r=>r.cache));
    const zip=path.join(out,label+'.zip');await save('download-state-run-batch',zip);
    return {states,sidecar:delivered,project_sha256:sha(await fs.readFile(file)),zip_sha256:sha(await fs.readFile(zip))};
  }
  await visit();await open(project);
  const cold=await run('cold',false);
  let online=null,offline=null,mixedRuntimeRefusal=false;
  if(withC){
    online=await run('online',true);
    await open(path.resolve(out,'online.bisect'));
    await el('offline-package').evaluate(el=>el.open=true);await el('offline-install').click({timeout});
    await page.waitForFunction(()=>document.querySelector('#offline-status').textContent.startsWith('Installed '),null,{timeout});
    await page.waitForFunction(()=>!!navigator.serviceWorker.controller,null,{timeout});
    await page.close();await context.setOffline(true);await visit();await open(project);
    offline=await run('offline',true);
    assert.deepEqual(offline.states,online.states);assert.equal(offline.zip_sha256,online.zip_sha256);
    await open(project);
    const beforeMixed=(await calls()).length;
    await page.evaluate(()=>{injectMixedRuntime=true;});
    await el('c-metis-runtime').check();await el('execute').click({timeout});
    await page.waitForFunction(()=>!document.querySelector('#execute').disabled&&
      document.querySelector('#result-status').textContent.includes('different C runtime artifacts'),null,{timeout});
    assert.equal(await page.evaluate(()=>mixedRuntimeInjected),1);
    assert((await calls()).slice(beforeMixed).every(c=>c.operation==='inspect-state-batch-selection'));
    assert(await el('download-state-run-batch').isDisabled());
    mixedRuntimeRefusal=true;
  }else assert(await el('c-metis-runtime-control').isHidden());
  assert.deepEqual(errors,[]);
  await fs.writeFile(path.join(out,'evidence.json'),JSON.stringify({status:'passed-packaged-real-c-batch',profile,
    browser:browser.version(),wasm:enginePin,native_sha256:native.native_exe_sha256,cold,online,offline,
    open_inert:true,mixed_runtime_refused_before_state_dispatch:mixedRuntimeRefusal,
    full_parity:false,deployed:false},null,2)+'\n',{flag:'wx'});
}catch(error){
  const ui=await page?.evaluate(()=>({status:document.querySelector('#result-status')?.textContent,
    project:document.querySelector('#project-status')?.textContent,calls:window.batchCalls})).catch(()=>null);
  await fs.writeFile(path.join(out,'failure.json'),JSON.stringify({error:String(error.stack),ui},null,2)+'\n');throw error;
}finally{await browser.close();}
