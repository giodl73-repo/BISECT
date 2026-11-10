// Exercise the exact native municipal split cases in an extracted Wrangler app.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import fs from 'node:fs/promises';
import {createRequire} from 'node:module';
import path from 'node:path';
import {createProject} from '../../web/lab/project.js';
import {openWorkbenchProject} from './workbench_project_open.mjs';

const [address, packageDir, nativeReceipt, out] = process.argv.slice(2);
const origin=new URL(address).origin;
assert.match(origin,/^http:\/\/127\.0\.0\.1:\d+$/);
await fs.mkdir(out);
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const read=async file=>JSON.parse(await fs.readFile(file));
const runtime=await read(nativeReceipt);
const manifest=await read(path.join(packageDir,'assets/offline-manifest.json'));
assert.equal(runtime.status,'passed-explicit-municipal-cli-wasm-and-package');
assert.equal(runtime.wasm.sha256,manifest.assets['toolkit/bisect_wasm.wasm'].sha256);
async function pinned(reference){
  const bytes=await fs.readFile(reference.path);
  assert.equal(bytes.length,reference.bytes);assert.equal(sha(bytes),reference.sha256);
  return JSON.parse(bytes);
}
const {chromium}=createRequire(path.join(process.env.BISECT_PLAYWRIGHT_PACKAGES,'package.json'))('playwright');
const browser=await chromium.launch({headless:true,
  executablePath:process.env.BISECT_TEST_BROWSER||'C:/Program Files/Google/Chrome/Application/chrome.exe',
  args:['--no-first-run','--disable-background-networking','--disable-extensions']});
const timeout=120000;
let page;
const checks=[];
try{
  const context=await browser.newContext({acceptDownloads:true});
  await context.route('**/*',r=>new URL(r.request().url()).origin===origin?r.continue():r.abort());
  await context.addInitScript(()=>{
    window.municipalRequests=[];window.municipalReplies=[];
    const W=Worker;window.Worker=class extends W{
      constructor(url,...args){super(url,...args);this.engine=new URL(url,location.href).pathname==='/toolkit/wasm-worker.js';
        if(this.engine)this.addEventListener('message',({data})=>{if(['result','error'].includes(data?.type))municipalReplies.push(data);});}
      postMessage(m,...args){if(this.engine&&m?.type==='run')municipalRequests.push(m.request);return super.postMessage(m,...args);}
    };
  });
  const errors=[];
  const visit=async()=>{page=await context.newPage();page.on('pageerror',e=>errors.push(e.message));await page.goto(origin+'/toolkit/');};
  const el=id=>page.locator('#'+id);
  const count=()=>page.evaluate(()=>municipalRequests.length);
  await visit();
  await el('offline-package').evaluate(section=>section.open=true);
  await el('offline-install').click({timeout});
  await page.waitForFunction(()=>document.querySelector('#offline-status').textContent.startsWith('Installed '),null,{timeout});
  await page.waitForFunction(()=>!!navigator.serviceWorker.controller,null,{timeout});
  await page.close();await context.setOffline(true);await visit();
  await page.waitForFunction(hash=>!document.querySelector('#execute').disabled&&
    document.querySelector('#runtime-artifact').textContent.includes(hash),runtime.wasm.sha256,{timeout});
  const source=(name,text)=>({name,base64:Buffer.from(text).toString('base64')});
  const refusal={invalid:/invalid explicit municipal identity/i,wrongOrder:/plan\/context ordered unit identity mismatch/i,
    badLength:/subdivision|context hash/i};
  for(const row of runtime.rows){
    const request=await pinned(row.request);
    assert.equal(sha(request.context_json),row.context.sha256);
    const expected=row.wasm_result?await pinned(row.wasm_result):null;
    const project=createProject({name:`Municipal split: ${row.name}`,operation:'analyze-plan-splits',constraints:[],
      files:{auditPlanText:source('custom.rplan',request.document_json),auditContextText:source(`${row.name}.rctx`,request.context_json)},
      result:expected,lastOperation:expected?'analyze-plan-splits':null});
    const projectPath=path.join(out,row.name+'.bisect');
    await fs.writeFile(projectPath,JSON.stringify(project));
    const before=await count();
    await openWorkbenchProject(page,projectPath,{timeout});
    assert.equal(await count(),before);assert(await el('download-splits').isDisabled());
    await el('execute').click();
    await page.waitForFunction(n=>municipalReplies.length===n+1&&!document.querySelector('#execute').disabled,before,{timeout});
    const {observed,reply}=await page.evaluate(i=>({observed:municipalRequests[i],reply:municipalReplies[i]}),before);
    assert.equal(observed.operation,'analyze-plan-splits');assert.equal(observed.context_json,request.context_json);
    assert.equal(observed.document_json,request.document_json);
    if(!expected){
      assert.notEqual(row.native_status,0);assert.equal(reply.type,'error');assert.match(reply.error,refusal[row.name]);
      assert(await el('download-splits').isDisabled());checks.push({name:row.name,outcome:'matching-refusal',error:reply.error});continue;
    }
    assert.equal(reply.type,'result',reply.error);assert.deepEqual(reply.result,expected);
    if(['complete','partial','zero','allUnknown'].includes(row.name)){
      const cells=Object.fromEntries(await page.locator('#result-tables table').first().locator('tbody tr')
        .evaluateAll(rows=>rows.map(row=>[row.cells[0].textContent,row.cells[1].textContent])));
      const municipalities=expected.report.municipalities;
      assert.equal(cells['Units with unknown municipal IDs'],String(municipalities.missing_units));
      assert.equal(cells['Split municipalities'],municipalities.available?String(municipalities.split):'—');
    }
    const waiting=page.waitForEvent('download',{timeout});await el('download-splits').click();
    const file=path.join(out,`${row.name}-splits.json`);await(await waiting).saveAs(file);
    const bytes=await fs.readFile(file);assert.equal(bytes.toString('utf8'),expected.report_json);
    checks.push({name:row.name,outcome:'native-equal-output',report_sha256:sha(bytes),context_sha256:row.context.sha256});
  }
  assert.deepEqual(errors,[]);assert.equal(await count(),runtime.rows.length);
  await fs.writeFile(path.join(out,'evidence.json'),JSON.stringify({status:'passed-packaged-offline-municipal-splits',
    checks,browser:browser.version(),wasm_sha256:runtime.wasm.sha256,source_open_inert:true,
    runtime_receipt_sha256:sha(await fs.readFile(nativeReceipt)),deployed:false},null,2)+'\n');
}catch(error){
  const ui=await page?.evaluate(()=>({status:document.querySelector('#result-status')?.textContent,
    project:document.querySelector('#project-status')?.textContent,count:window.municipalRequests?.length})).catch(()=>null);
  await fs.writeFile(path.join(out,'failure.json'),JSON.stringify({error:String(error.stack),ui,checks},null,2)+'\n');
  throw error;
}finally{await browser.close();}
