// Independent headless browser/profile/server; never attach to ambient tabs.
// Feasibility evidence only: no production engine or production UI is loaded.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
const root=path.resolve('target/metis-c-sidecar-matrix');
const packageRoot=process.env.BISECT_PLAYWRIGHT_PACKAGES;
if(!packageRoot)throw new Error('Set BISECT_PLAYWRIGHT_PACKAGES to an installed Playwright package root.');
const {chromium}=createRequire(path.join(packageRoot,'package.json'))('playwright');
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const assets=new Map(),identities=[];
async function add(url,file){
  const bytes=await fs.readFile(file);assets.set(url,bytes);
  identities.push({url,path:path.relative(process.cwd(),file).replaceAll('\\','/'),sha256:hash(bytes),byte_length:bytes.length});
}
for(const name of ['compare-sidecar.mjs','probe-worker.mjs'])await add('/fixture/'+name,path.resolve('scripts/web/fixtures/wasm-cffi-probe',name));
const attempts={};
for(const profile of ['unmodified','msvcrt']){
  const attempt=JSON.parse(await fs.readFile(path.join(root,profile,'evidence.json'),'utf8'));
  assert.equal(attempt.rng_profile,profile);assert.equal(attempt.production_integrated,false);
  assert.equal(attempt.production_unchanged,true);assert.equal(attempt.parity_verified,false);
  assert.equal(attempt.native_platform,'win32','This fixture compares the measured Windows-native runtime.');
  for(const ref of [attempt.runner,attempt.fixture,attempt.comparison_source,attempt.native_executable,...attempt.native_sources,...(attempt.rng_shim?[attempt.rng_shim]:[])]){
    assert.equal(hash(await fs.readFile(path.resolve(ref.path))),ref.sha256,'Reject stale experiment source: '+ref.path);
  }
  for(const [name,key]of [['metis-probe.mjs','sidecar_js'],['metis-probe.wasm','sidecar_wasm'],['native-reference.json','native_reference']]){
    await add('/artifacts/'+profile+'/'+name,path.join(root,profile,name));
    assert.equal(identities.at(-1).sha256,attempt[key].sha256,'Reject stale sidecar/reference assets');
  }
  const reference=JSON.parse(await fs.readFile(path.join(root,profile,'native-reference.json'),'utf8'));
  assert.equal(reference.cases.length,24);assert(reference.cases.some(c=>c.n===300&&c.ncon===2));
  assert(reference.cases.every(c=>c.code===1));assert.equal(reference.invalid_option,-2);assert.equal(reference.disconnected,-4);
  attempts[profile]={reference,record:attempt};
}
assert.deepEqual(attempts.unmodified.reference,attempts.msvcrt.reference,'Both builds must compare the same native inputs/outcomes');
assets.set('/',Buffer.from('<!doctype html><meta charset="utf-8"><title>C METIS feasibility worker</title><p>Isolated C backend experiment</p>'));
const requested=[];
const server=createServer((req,res)=>{
  const url=new URL(req.url,'http://localhost');requested.push(url.pathname);
  const bytes=assets.get(url.pathname);
  if(!bytes){res.writeHead(404);res.end();return;}
  res.setHeader('Content-Type',url.pathname.endsWith('.mjs')?'application/javascript':url.pathname.endsWith('.wasm')?'application/wasm':url.pathname.endsWith('.json')?'application/json':'text/html');
  res.setHeader('Cache-Control','no-store');res.end(bytes);
});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const origin='http://127.0.0.1:'+server.address().port;
let browser;
try{
  browser=await chromium.launch({headless:true,executablePath:process.env.BISECT_TEST_BROWSER||'C:/Program Files/Google/Chrome/Application/chrome.exe',args:['--no-first-run','--disable-background-networking','--disable-extensions']});
  const context=await browser.newContext();await context.route('**/*',route=>new URL(route.request().url()).origin===origin?route.continue():route.abort());
  const page=await context.newPage(),errors=[];page.on('pageerror',error=>errors.push(error.message));
  await page.goto(origin);const results={};
  for(const profile of ['unmodified','msvcrt']){
    const reply=await page.evaluate(profile=>new Promise((resolve,reject)=>{
      const worker=new Worker('/fixture/probe-worker.mjs',{type:'module'});
      const timeout=setTimeout(()=>{worker.terminate();reject(new Error('Worker timed out'));},30000);
      worker.onmessage=({data})=>{clearTimeout(timeout);worker.terminate();resolve(data);};
      worker.onerror=error=>{clearTimeout(timeout);worker.terminate();reject(new Error(error.message));};
      worker.postMessage({profile});
    }),profile);
    assert.equal(reply.error,undefined,reply.stack);assert.equal(reply.profile,profile);
    assert.equal(reply.result.exact_match,profile==='msvcrt');
    assert.equal(reply.result.cases.length,24);
    assert(attempts[profile].record.comparison,'Node comparison must have run');
    assert.deepEqual(reply.result,Object.fromEntries(Object.entries(attempts[profile].record.comparison).filter(([key])=>key!=='imports')),'Browser worker must reproduce retained Node outcomes, including baseline differences');
    results[profile]=reply.result;
  }
  assert.equal(results.msvcrt.mismatches.length,0);
  assert(results.unmodified.mismatches.some(x=>x.name==='native-crt-rng'));
  assert(results.unmodified.mismatches.some(x=>x.name.startsWith('grid-')));
  assert.deepEqual(errors,[]);
  for(const profile of ['unmodified','msvcrt'])assert(requested.includes('/artifacts/'+profile+'/metis-probe.wasm'));
  const evidence={schema:'bisect-metis-c-sidecar-browser-v1',browser:browser.version(),runtime:'dedicated module worker',production_integrated:false,parity_verified:false,case_count:24,rng_values_per_seed:64,rng_seed_count:5,identities,results,page_errors:errors,requested};
  await fs.writeFile(path.join(root,'browser-evidence.json'),JSON.stringify(evidence,null,2)+'\n');
  console.log('PASS C sidecar worker: 24 native cases plus 320 CRT values; explicit MSVCRT profile exact, unmodified baseline mismatch retained; no production integration.');
}finally{await browser?.close();await new Promise(resolve=>server.close(resolve));}
