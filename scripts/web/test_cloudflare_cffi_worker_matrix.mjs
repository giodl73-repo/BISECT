// Run the maintained C METIS fixtures through the shipped browser Worker.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import fs from 'node:fs/promises';
import {createRequire} from 'node:module';
import path from 'node:path';
import {cffiEngineCase} from './fixtures/cffi-engine-case.mjs';

const [address,packageDir,recordsPath,out,profile]=process.argv.slice(2);
const origin=new URL(address).origin;
assert.match(origin,/^http:\/\/127\.0\.0\.1:\d+$/);
assert(['core','c'].includes(profile));
const withC=profile==='c',timeout=120000;
const matrixBytes=await fs.readFile(recordsPath),rows=JSON.parse(matrixBytes);
const read=async file=>JSON.parse(await fs.readFile(file));
const manifest=await read(path.join(packageDir,'assets/offline-manifest.json'));
const engine=manifest.assets['toolkit/bisect_wasm.wasm'];
const baseline=(await read('scripts/web/fixtures/cffi-core-baseline.json')).result;
const {chromium}=createRequire(path.join(process.env.BISECT_PLAYWRIGHT_PACKAGES,'package.json'))('playwright');
const browser=await chromium.launch({headless:true,
  executablePath:process.env.BISECT_TEST_BROWSER??'C:/Program Files/Google/Chrome/Application/chrome.exe',
  args:['--no-first-run','--disable-extensions','--disable-background-networking']});
const sidecar=[];
await fs.mkdir(out);
try{
  const context=await browser.newContext();
  await context.route('**/*',r=>new URL(r.request().url()).origin===origin?r.continue():r.abort());
  context.on('response',r=>{
    const url=new URL(r.url());
    if(url.origin===origin&&url.pathname.startsWith('/toolkit/c-metis/metis-sidecar.')){
      sidecar.push({path:url.pathname,status:r.status(),service_worker:r.fromServiceWorker()});
    }
  });
  const shell=await context.newPage();
  await shell.goto(origin+'/toolkit/');
  await shell.waitForFunction(()=>!document.querySelector('#execute').disabled,null,{timeout});
  await shell.locator('#offline-package').evaluate(el=>el.open=true);
  await shell.locator('#offline-install').click({timeout});
  await shell.waitForFunction(()=>document.querySelector('#offline-status').textContent.startsWith('Installed '),null,{timeout});
  await shell.waitForFunction(()=>!!navigator.serviceWorker.controller,null,{timeout});
  await shell.close();await context.setOffline(true);
  const page=await context.newPage(),errors=[];page.on('pageerror',e=>errors.push(e.message));
  await page.goto(origin+'/toolkit/');
  await page.waitForFunction(()=>!!navigator.serviceWorker.controller,null,{timeout});
  const before=sidecar.length;
  const ready=await page.evaluate(async ({engine,timeout})=>{
    const worker=new Worker('/toolkit/wasm-worker.js',{type:'module'});let id=0;
    function send(message,accept){
      return new Promise((resolve,reject)=>{
        const timer=setTimeout(()=>{clean();reject(new Error('Worker reply timed out'));},timeout);
        function clean(){clearTimeout(timer);worker.removeEventListener('message',receive);worker.removeEventListener('error',failed);}
        function receive({data}){if(accept(data)){clean();resolve(data);}}
        function failed(e){clean();reject(new Error(e.message));}
        worker.addEventListener('message',receive);worker.addEventListener('error',failed);
        worker.postMessage(message);
      });
    }
    window.runMatrixCase=(request,cMetis)=>{
      const token=++id;
      return send({type:'run',id:token,request,cMetis},m=>m.id===token&&['result','error'].includes(m.type));
    };
    return send({type:'initialize',url:'/toolkit/bisect_wasm.wasm',sha256:engine.sha256},m=>['ready','error'].includes(m.type));
  },{engine,timeout});
  assert.equal(ready.type,'ready');assert.equal(ready.cMetisAvailable,withC);
  assert.equal(ready.runtimeArtifact.sha256,engine.sha256);
  const run=(request,cMetis)=>page.evaluate(({request,cMetis})=>window.runMatrixCase(request,cMetis),{request,cMetis});
  const standard=cffiEngineCase();
  const first=await run(standard,false);
  assert.equal(first.type,'result');assert.deepEqual(first.result,baseline);
  const cold=await run(rows[0].request,false);
  assert.equal(cold.type,'error');
  assert.match(cold.error,withC?/Load the verified C METIS sidecar/:/no C METIS sidecar bridge/);
  assert.equal(sidecar.length,before);
  const results=[];let matches=0,failures=0;
  if(withC){
    const assets=await read(path.join(packageDir,'assets/toolkit/c-metis/metis-sidecar.json'));
    for(const [index,row] of rows.entries()){
      const reply=await run(row.request,true);
      if(row.native_exit_code!==0){
        const reason=JSON.parse(row.native_stderr.trim().split('Error: ')[1]);
        assert.equal(reply.type,'error');assert.equal(reply.error,reason);
        failures++;
      }else{
        assert.equal(reply.type,'result');assert.equal(reply.result.backend,'c-metis-msvcrt-v1');
        assert.deepEqual(reply.result.assignments,row.native.assignments);
        assert.equal(reply.result.metrics.weighted_boundary,row.native.weighted_boundary);
        assert.equal(reply.runtimeArtifacts.engine.sha256,engine.sha256);
        assert.deepEqual(reply.runtimeArtifacts,row.result.runtime_artifacts);
        for(const key of ['javascript','wasm'])assert.deepEqual(reply.runtimeArtifacts.metis_sidecar[key],assets[key]);
        matches++;
      }
      results.push({index,reply});
    }
    assert.equal(matches,52);assert.equal(failures,4);
    for(const ext of ['json','mjs','wasm'])assert(sidecar.slice(before).some(r=>r.status===200&&r.service_worker&&r.path.endsWith('.'+ext)));
  }else{
    const refused=await run(rows[0].request,true);
    assert.equal(refused.type,'error');assert.match(refused.error,/does not include C METIS support/);
    assert.equal(sidecar.length,before);
    results.push({index:0,reply:refused});
  }
  const after=await run(standard,false);
  assert.equal(after.type,'result');assert.deepEqual(after.result,baseline);
  assert.equal(after.runtimeArtifacts.metis_sidecar,null);
  assert.deepEqual(errors,[]);
  await fs.writeFile(path.join(out,'worker-replies.json'),JSON.stringify({first,cold,results,after}),{flag:'wx'});
  await fs.writeFile(path.join(out,'evidence.json'),JSON.stringify({status:'passed-offline-packaged-cffi-worker-matrix',
    profile,browser:browser.version(),engine,matrix_sha256:createHash('sha256').update(matrixBytes).digest('hex'),
    matches,expected_native_failures:failures,default_and_post_c_core_exact:true,cold_refused:true,
    sidecar_after_offline_navigation:sidecar.slice(before),full_parity:false},null,2)+'\n',{flag:'wx'});
}finally{await browser.close();}
