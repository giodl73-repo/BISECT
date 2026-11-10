// Real browser smoke test of portable workflows from an extracted Wrangler project.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import fs from 'node:fs/promises';
import {createRequire} from 'node:module';
import {createServer,request as forward} from 'node:http';
import path from 'node:path';
import {openWorkbenchProject} from './workbench_project_open.mjs';

const [kind, address, packageDir, fixtureDir, priorFile, out, cReplayDir] = process.argv.slice(2);
const backend = new URL(address).origin;
assert.match(backend, /^http:\/\/127\.0\.0\.1:\d+$/);
assert(['firefox','edge'].includes(kind));
await fs.mkdir(out);
const playwright = createRequire(path.join(process.env.BISECT_PLAYWRIGHT_PACKAGES,'package.json'))('playwright');
const engine = kind==='firefox'?playwright.firefox:playwright.chromium;
const executable = kind==='firefox'?engine.executablePath():process.env.BISECT_TEST_EDGE ||
  'C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe';
const sha = bytes=>createHash('sha256').update(bytes).digest('hex');
const read = async file=>JSON.parse(await fs.readFile(file));
const manifest=await read(path.join(packageDir,'assets','offline-manifest.json'));
const prior=await read(priorFile);
const reference=await read(path.join(fixtureDir,'evidence.json'));
const fixture='disconnected-one-based';
assert.equal(manifest.assets['bisect_wasm.wasm'].sha256,reference.wasm_sha256);
const expectedZip=reference.cases.find(item=>item.id===fixture).zip_sha256;
const timeout=120000;
let context,page,proxy,disconnected=false,refused=0,origin=backend;
try {
  // Firefox's automation offline toggle can reject navigation before the
  // service worker handles it. Cut access to the real loopback origin instead.
  if(kind==='firefox'){
    proxy=createServer((req,res)=>{
      if(disconnected){refused++;res.destroy();return;}
      const url=new URL(req.url,backend);
      if(url.origin!==backend){res.writeHead(400).end();return;}
      const next=forward(url,{method:req.method,headers:{...req.headers,host:url.host}},upstream=>{
        res.writeHead(upstream.statusCode,upstream.headers);upstream.pipe(res);
      });
      next.on('error',()=>res.destroy());req.pipe(next);
    });
    await new Promise(resolve=>proxy.listen(0,'127.0.0.1',resolve));
    origin=`http://127.0.0.1:${proxy.address().port}`;
  }
  context=await engine.launchPersistentContext(path.resolve(out,'profile'),{
    headless:true,executablePath:executable,acceptDownloads:true,
    ...(kind==='edge'?{args:['--no-first-run','--disable-background-networking','--disable-extensions']}:{}),
  });
  const errors=[];
  await context.route('**/*',route=>new URL(route.request().url()).origin===origin?route.continue():route.abort());
  await context.addInitScript(()=>{
    window.engineRuns=[];window.cReplayReply=null;
    const Original=Worker;
    window.Worker=class extends Original {
      constructor(url,...args){super(url,...args);
        this.addEventListener('message',({data})=>{if(data?.type==='result' && data.result?.runtime_artifacts?.metis_sidecar &&
          data.result?.comparison?.regenerated_assignments)cReplayReply=data.result;});}
      postMessage(m,...args){if(m?.type==='run')engineRuns.push(m.request.operation??'state-run');return super.postMessage(m,...args);}
    };
  });
  async function visit(suffix){
    page=await context.newPage();page.on('pageerror',e=>errors.push(e.message));
    await page.goto(origin+suffix);
  }
  const el=id=>page.locator('#'+id);
  async function download(id,file){
    const waiting=page.waitForEvent('download',{timeout});await el(id).click({timeout});
    const d=await waiting;if(file)await d.saveAs(file);
    return fs.readFile(file??await d.path());
  }
  const ops=()=>page.evaluate(()=>engineRuns.slice());
  await visit('/');
  await page.waitForFunction(()=>document.querySelector('#connection').textContent.includes('WASM'),null,{timeout});
  const capabilities=await page.evaluate(()=>({secure:isSecureContext,worker:typeof Worker==='function',
    wasm:typeof WebAssembly==='object',sha256:typeof crypto.subtle?.digest==='function',
    serviceWorker:'serviceWorker'in navigator,opfs:typeof navigator.storage?.getDirectory==='function',
    directoryPicker:typeof window.showDirectoryPicker==='function',indexedDB:'indexedDB'in window,
    compression:typeof CompressionStream==='function',decompression:typeof DecompressionStream==='function'}));
  assert(capabilities.secure&&capabilities.worker&&capabilities.wasm&&capabilities.sha256&&capabilities.serviceWorker);
  await el('offline-package').evaluate(section=>section.open=true);
  await el('offline-states').selectOption('RI:2020');await el('offline-install').click({timeout});
  await page.waitForFunction(()=>!document.querySelector('#offline-install').disabled,null,{timeout});
  const installed=await el('offline-status').textContent();assert.match(installed,/^Installed .*RI:2020/);
  await page.waitForFunction(()=>!!navigator.serviceWorker.controller,null,{timeout});
  await page.close();
  if(proxy){disconnected=true;proxy.closeAllConnections();}
  else await context.setOffline(true);
  await visit('/');
  await page.waitForFunction(()=>document.querySelector('#connection').textContent.includes('WASM'),null,{timeout});
  assert.equal(await page.evaluate(async()=>{
    try{await fetch('/__missing_offline_browser_probe__',{cache:'no-store'});return false;}
    catch{return true;}
  }),true,'An uncached request must fail while disconnected');
  await el('state').selectOption('RI');await el('seed').fill('18446744073709551615');await el('run').click({timeout});
  await page.waitForFunction(()=>document.querySelector('#run-status').textContent==='completed' &&
    !document.querySelector('#download-map').disabled && document.querySelector('#map-empty').hidden,null,{timeout});
  const map=await download('download-map');assert.deepEqual({bytes:map.length,sha256:sha(map)},
    {bytes:prior.state_runs['RI:2020'].geojson.bytes,sha256:prior.state_runs['RI:2020'].geojson.sha256});
  const stateProject=path.resolve(out,'offline-state.bisect');
  const stateBytes=await download('save-lab-project',stateProject);
  const stateSaved=JSON.parse(stateBytes);
  assert.equal(stateSaved.experiment.config.seed,'18446744073709551615');
  assert.equal(Object.keys(stateSaved.assignments.RI).length,250);
  assert.deepEqual(await ops(),['state-run']);
  await el('open-lab-project').setInputFiles(stateProject);
  await page.waitForFunction(()=>document.querySelector('#lab-project-status').textContent.startsWith('Opened project.'),null,{timeout});
  assert.deepEqual(await ops(),['state-run']);
  await page.close();await visit('/toolkit/');
  await page.waitForFunction(()=>!document.querySelector('#execute').disabled,null,{timeout});
  await openWorkbenchProject(page,path.join(fixtureDir,fixture+'.bisect'),{timeout});
  assert.deepEqual(await ops(),[]);
  await el('execute').click({timeout});
  await page.waitForFunction(()=>!document.querySelector('#execute').disabled &&
    !document.querySelector('#download-label-analysis').disabled,null,{timeout});
  const zip=await download('download-label-analysis');assert.equal(sha(zip),expectedZip);
  const workbenchArchive=path.resolve(out,'offline-workbench.bisect-workbench');
  const archive=await download('save-workbench-archive',workbenchArchive);
  await el('open-workbench-archive').setInputFiles(workbenchArchive);
  await page.waitForFunction(name=>document.querySelector('#project-status').textContent.startsWith('Opened workbench archive '+name),
    path.basename(workbenchArchive),{timeout});
  assert.deepEqual(await ops(),['run-label-analysis']);
  let cReplay=null;
  if(cReplayDir){
    const expected=await read(path.join(cReplayDir,'current-wasm.json'));
    const earlier=await read(path.join(cReplayDir,'evidence.json'));
    assert.equal(earlier.wasm_sha256,reference.wasm_sha256);
    assert.equal(earlier.selected_runtime_exact_native_assignments,true);
    await openWorkbenchProject(page,path.join(cReplayDir,'actual-ri.bisect'),{timeout});
    assert.deepEqual(await ops(),['run-label-analysis']);
    assert.equal(await el('c-metis-runtime').isChecked(),false);
    await el('c-metis-runtime').check();
    assert.deepEqual(await ops(),['run-label-analysis']);
    await el('execute').click({timeout});
    await page.waitForFunction(()=>!document.querySelector('#execute').disabled &&
      !document.querySelector('#recorded-replay-plan').disabled,null,{timeout});
    assert.deepEqual(await ops(),['run-label-analysis','run-native-recorded-replay']);
    const actual=await page.evaluate(()=>cReplayReply);
    assert.equal(actual.execution_performed,true);
    assert.deepEqual(actual.runtime_artifacts.metis_sidecar,earlier.sidecar_identity);
    assert.deepEqual(actual.comparison.regenerated_assignments,expected.comparison.regenerated_assignments);
    const plan=await download('recorded-replay-plan'),contextBytes=await download('recorded-replay-context');
    assert.equal(plan.toString('utf8'),expected.replay.export.document_json);
    assert.equal(contextBytes.toString('utf8'),expected.replay.export.context_json);
    cReplay={units:earlier.units,runtime:earlier.sidecar_identity,plan_sha256:sha(plan),context_sha256:sha(contextBytes),
      matches_native_bound_reference:true};
  }
  assert.deepEqual(errors,[]);
  await fs.writeFile(path.join(out,'evidence.json'),JSON.stringify({status:'passed-portable-browser-offline',kind,
    browser:context.browser().version(),capabilities,wasm_sha256:reference.wasm_sha256,installed,
    state:'RI:2020',state_map_sha256:sha(map),state_project_sha256:sha(stateBytes),
    workbench_zip_sha256:sha(zip),workbench_archive_sha256:sha(archive),inert_open:true,
    tested_with_network_disabled:true,offline_method:proxy?'loopback-origin-unreachable':'browser-offline',
    refused_origin_requests:refused,folder_permission_required:false,c_metis_replay:cReplay,deployed:false},null,2)+'\n',{flag:'wx'});
} catch(error) {
  const ui=await page?.evaluate(()=>Object.fromEntries(['connection','offline-status','run-status','lab-project-status','project-status','result-status']
    .map(id=>[id,document.getElementById(id)?.textContent]))).catch(()=>null);
  await fs.writeFile(path.join(out,'failure.json'),JSON.stringify({error:String(error.stack),ui},null,2)+'\n');
  throw error;
} finally {await context?.close();if(proxy)await new Promise(resolve=>proxy.close(resolve));}
