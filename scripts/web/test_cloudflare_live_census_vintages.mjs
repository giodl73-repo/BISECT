// Explicit live diagnostic using the extracted Wrangler project and pinned public URLs.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import fs from 'node:fs/promises';
import {createRequire} from 'node:module';
import path from 'node:path';
import {CENSUS_ARCHIVE_ENDPOINTS} from '../../web/lab/census-endpoints.js';
import {createProject} from '../../web/lab/project.js';
import {pl94Request} from '../../web/lab/pl94-files.js';
import {parseWorkbenchArchiveBytes} from '../../web/lab/workbench-archive.js';
import {openWorkbenchProject} from './workbench_project_open.mjs';

const [address, packageDir, downloaded, native, destination] = process.argv.slice(2);
const origin = new URL(address).origin;
assert.match(origin, /^http:\/\/127\.0\.0\.1:\d+$/);
await fs.mkdir(destination);
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const read = async file => JSON.parse(await fs.readFile(file));
const hosting = await read(path.join(packageDir,'hosting-manifest.json'));
const wasm = hosting.gzip_routes['toolkit/bisect_wasm.wasm'].decoded.sha256;
const cases = [['VT','2000',24824,'608827'],['WA','2010',195574,'6724540'],['RI','2020',25649,'1097379']];
const urls = cases.map(([state,year])=>CENSUS_ARCHIVE_ENDPOINTS[`${state}:${year}:pl94171`]);
const {chromium} = createRequire(path.join(process.env.BISECT_PLAYWRIGHT_PACKAGES,'package.json'))('playwright');
const browser = await chromium.launch({headless:true,
  executablePath:process.env.BISECT_TEST_BROWSER || 'C:/Program Files/Google/Chrome/Application/chrome.exe',
  args:['--no-first-run','--disable-background-networking','--disable-extensions']});
const timeout = 180000;
const results = {}, requests = [], errors = [];
let page;
try {
  const context = await browser.newContext({acceptDownloads:true});
  await context.route('**/*', route=>{
    const req=route.request();
    return new URL(req.url()).origin===origin || req.method()==='HEAD' && urls.includes(req.url()) ? route.continue() : route.abort();
  });
  context.on('request',req=>{
    const url=new URL(req.url());
    if(url.pathname==='/api/source'||urls.includes(req.url()))requests.push({url:req.url(),method:req.method()});
  });
  await context.addInitScript(()=>{
    window.sourceOps=[];window.sourceResults=[];
    const Original=Worker;
    window.Worker=class extends Original {
      constructor(url,...rest){super(url,...rest);if(new URL(url,location.href).pathname==='/toolkit/wasm-worker.js'){
        this.engine=true;this.addEventListener('message',({data})=>{if(data?.type==='result')sourceResults.push(data.result);});
      }}
      postMessage(message,...rest){if(this.engine&&message?.type==='run')sourceOps.push(message.request.operation);return super.postMessage(message,...rest);}
    };
  });
  page=await context.newPage();page.on('pageerror',e=>errors.push(e.message));
  await page.goto(origin+'/toolkit/');
  await page.waitForFunction(hash=>!document.querySelector('#execute').disabled &&
    document.querySelector('#runtime-artifact').textContent.includes(hash),wasm,{timeout});
  const el = id=>page.locator('#'+id);
  const operations = ()=>page.evaluate(()=>sourceOps.slice());
  async function run(op){
    const before=(await operations()).length;
    await el('execute').click();
    await page.waitForFunction(n=>sourceResults.length===n+1 && !document.querySelector('#execute').disabled,before,{timeout});
    assert.equal((await operations()).at(-1),op);
    return page.evaluate(()=>sourceResults.at(-1));
  }
  for (const [i,[state,year,blocks,population]] of cases.entries()) {
    const scope=`${state}:${year}`, url=urls[i], filename=url.split('/').at(-1);
    const providerBody=await fs.readFile(path.join(downloaded,filename));
    const direct=await page.evaluate(async url=>{
      try{const r=await fetch(url,{method:'HEAD',mode:'cors',credentials:'omit',redirect:'error',signal:AbortSignal.timeout(15000)});
        return {browser_readable:true,status:r.status};}
      catch(e){return {browser_readable:false,error:e.name};}
    },url);
    const selection={manifest_json:null,states:[state],year,data_types:['redistricting'],selected_paths:[]};
    const project=createProject({name:`Live Census ${scope}`,operation:'plan-fetch-sources',constraints:[],result:null,lastOperation:null,
      files:{fetchPlanInputs:{source:{name:'selection.json',base64:Buffer.from(JSON.stringify(selection)).toString('base64')}}}});
    const planFile=path.join(destination,`${state}-${year}.bisect`);await fs.writeFile(planFile,JSON.stringify(project));
    const before=await operations(), networkBefore=requests.length;
    await openWorkbenchProject(page,planFile,{timeout});
    assert.deepEqual(await operations(),before);assert.equal(requests.length,networkBefore);
    assert.equal(await el('fetch-source-transport').inputValue(),'direct');
    const plan=await run('plan-fetch-sources');assert.equal(plan.items.length,1);assert.equal(plan.items[0].url,url);
    await el('fetch-source-transport').selectOption('site-relay');
    const response=page.waitForResponse(res=>{
      const target=new URL(res.url());return target.origin===origin&&target.pathname==='/api/source'&&target.searchParams.get('url')===url;
    },{timeout});
    await el('fetch-acquire-source').click();
    const actual=await response;
    assert.equal(actual.status(),200,`Relay HTTP ${actual.status()} for ${scope}`);
    assert.equal(await actual.headerValue('x-bisect-source-relay'),'v1');
    await page.waitForFunction(()=>document.querySelector('#operation').value==='prepare-pl94-source' &&
      !document.querySelector('#execute').disabled,null,{timeout});
    assert.deepEqual((await operations()).slice(before.length),['plan-fetch-sources']);
    const prepared=await run('prepare-pl94-source');
    assert.equal(prepared.block_count,blocks);assert.equal(prepared.total_population,population);
    const download=page.waitForEvent('download',{timeout});await el('save-workbench-archive').click();
    const saved=await parseWorkbenchArchiveBytes(new Uint8Array(await fs.readFile(await(await download).path())));
    assert.deepEqual(saved.result,prepared);
    assert.deepEqual(Buffer.from(saved.files.fetchArchiveSource.source.base64,'base64'),providerBody);
    const receipt=saved.files.fetchAcquisitionReceipt;
    assert.equal(receipt.transport,'site-relay');assert.equal(receipt.source_sha256,sha(providerBody));
    assert.equal(receipt.byte_length,providerBody.length);
    const requestFile=path.join(destination,`${state}-${year}-native-request.json`);
    await fs.writeFile(requestFile,JSON.stringify(pl94Request(saved.files.pl94Inputs)));
    const reference=spawnSync(path.resolve(native),[path.resolve(requestFile)],{encoding:'utf8',windowsHide:true,
      timeout,maxBuffer:64*1024*1024});
    assert.ifError(reference.error);assert.equal(reference.status,0,reference.stderr);
    assert.deepEqual(prepared,JSON.parse(reference.stdout));
    const expectedRelay=origin+'/api/source?url='+encodeURIComponent(url);
    assert.deepEqual(requests.slice(networkBefore),[{url:expectedRelay,method:'GET'}]);
    results[scope]={url,archive_sha256:receipt.source_sha256,archive_bytes:providerBody.length,
      blocks,population,whole_native_reply_equal:true,relay_requests:1,direct_browser_head:direct};
    console.log(`${scope}: exact live ZIP, ${blocks} blocks, matching native reply`);
  }
  assert.deepEqual(errors,[]);
  await fs.writeFile(path.join(destination,'evidence.json'),JSON.stringify({status:'passed-packaged-live-census-vintages',
    browser:browser.version(),wasm_sha256:wasm,native_sha256:sha(await fs.readFile(native)),results,
    requests,engine_operations:await operations(),deployed:false},null,2)+'\n');
} catch(error) {
  const ui=await page?.evaluate(()=>({result:document.querySelector('#result-status')?.textContent,operations:window.sourceOps})).catch(()=>null);
  await fs.writeFile(path.join(destination,'failure.json'),JSON.stringify({status:'failed',error:String(error.stack),results,requests,ui},null,2)+'\n');
  throw error;
} finally { await browser.close(); }
