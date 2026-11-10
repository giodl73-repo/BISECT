import fs from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import {pathToFileURL} from 'node:url';
import {runFaultCases} from './fixtures/cffi-fault-cases.mjs';
import {cffiEngineCase} from './fixtures/cffi-engine-case.mjs';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const bridgePath=path.resolve(process.env.BISECT_CFFI_FAULT_BRIDGE??'web/lab/metis-c-sidecar.js');
const {loadMetisSidecar,metisSidecarImport}=await import(pathToFileURL(bridgePath));
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const root=await fs.mkdtemp(path.resolve('target/cffi-faults/run-'));
const toolkitArgument=process.argv.find(arg=>arg.startsWith('--toolkit='));
const toolkit=toolkitArgument?path.resolve(toolkitArgument.slice(10)):null;
const sidecar=toolkit?path.join(toolkit,'c-metis'):path.resolve(process.env.BISECT_CFFI_FAULT_SIDECAR??'target/cffi-integration/sidecar');
const wasmPath=toolkit?path.join(toolkit,'bisect_wasm.wasm'):path.resolve(process.env.BISECT_CFFI_FAULT_ENGINE??'target/state-report-workflow/c-sidecar.wasm');
// The harness serves source modules; a package check requires identical bridge
// bytes, never silently treating an old packaged bridge as the current one.
if(toolkit)for(const name of ['metis-c-sidecar.js','wasm-engine.js'])assert.equal(hash(await fs.readFile(path.join(toolkit,name))),hash(await fs.readFile(path.join('web/lab',name))),'Packaged runtime must match tested source: '+name);
const javascript=await fs.readFile(path.join(sidecar,'metis-sidecar.mjs')),binary=await fs.readFile(path.join(sidecar,'metis-sidecar.wasm')),engineBytes=await fs.readFile(wasmPath);
const manifest=JSON.parse(await fs.readFile(path.join(sidecar,'metis-sidecar.json'),'utf8'));
for(const [bytes,ref]of [[javascript,manifest.javascript],[binary,manifest.wasm]])assert.equal(hash(bytes),ref.sha256,'Start with exact pinned real sidecar bytes');
const beforeFiles=[bridgePath,'web/lab/wasm-engine.js','scripts/web/fixtures/cffi-fault-cases.mjs','scripts/web/test_cffi_fault_recovery.mjs'];
const sources=await Promise.all(beforeFiles.map(async file=>({path:file,sha256:hash(await fs.readFile(file))})));
const result=await runFaultCases({loadMetisSidecar,metisSidecarImport,instantiateEngine,javascript,binary,engineBytes,cffiEngineCase,sha:hash});
let browserResult=null,browserVersion=null;
if(process.argv.includes('--browser')){
  assert.equal(bridgePath,path.resolve('web/lab/metis-c-sidecar.js'),'Browser mode checks the current product bridge');
  const {chromium}=createRequire(path.join(process.env.BISECT_PLAYWRIGHT_PACKAGES,'package.json'))('playwright');
  const assets=new Map([['/sidecar.mjs',javascript],['/sidecar.wasm',binary],['/engine.wasm',engineBytes]]);
  const rootDir=process.cwd();
  const server=createServer(async(req,res)=>{try{
    const url=new URL(req.url,'http://localhost').pathname;
    if(url==='/'){res.setHeader('Content-Type','text/html');res.end('<!doctype html><title>C bridge failure checks</title>');return;}
    let bytes=assets.get(url);
    if(!bytes){const file=path.resolve(rootDir,'.'+url);if(!file.startsWith(rootDir+path.sep)||!['.js','.mjs'].includes(path.extname(file)))throw new Error('Unknown fixture');bytes=await fs.readFile(file);}
    res.setHeader('Content-Type',url.endsWith('.wasm')?'application/wasm':'application/javascript');res.end(bytes);
  }catch{res.writeHead(404);res.end();}});
  await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
  let browser;
  try{
    const origin='http://127.0.0.1:'+server.address().port;
    browser=await chromium.launch({headless:true,executablePath:process.env.BISECT_TEST_BROWSER??'C:/Program Files/Google/Chrome/Application/chrome.exe'});
    const context=await browser.newContext();await context.route('**/*',route=>new URL(route.request().url()).origin===origin?route.continue():route.abort());
    const page=await context.newPage();await page.goto(origin);
    browserResult=await page.evaluate(()=>new Promise((resolve,reject)=>{
      const source=`import {runFaultCases} from '/scripts/web/fixtures/cffi-fault-cases.mjs';
        import {cffiEngineCase} from '/scripts/web/fixtures/cffi-engine-case.mjs';
        import {instantiateEngine} from '/web/lab/wasm-engine.js';
        import {loadMetisSidecar,metisSidecarImport} from '/web/lab/metis-c-sidecar.js';
        const sha=async bytes=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>b.toString(16).padStart(2,'0')).join('');
        try{const [javascript,binary,engineBytes]=await Promise.all(['/sidecar.mjs','/sidecar.wasm','/engine.wasm'].map(async url=>new Uint8Array(await(await fetch(url)).arrayBuffer())));
        postMessage({result:await runFaultCases({loadMetisSidecar,metisSidecarImport,instantiateEngine,javascript,binary,engineBytes,cffiEngineCase,sha})});}catch(error){postMessage({error:error.stack});}`;
      // Worker imports resolve against the server, while its source stays local.
      const workerURL=URL.createObjectURL(new Blob([source.replaceAll("from '/","from '"+location.origin+"/").replace("fetch(url)","fetch("+JSON.stringify(location.origin)+"+url)")],{type:'application/javascript'}));
      const worker=new Worker(workerURL,{type:'module'}),timer=setTimeout(()=>{worker.terminate();URL.revokeObjectURL(workerURL);reject(new Error('Fault worker timeout'));},60000);
      worker.onmessage=({data})=>{clearTimeout(timer);worker.terminate();URL.revokeObjectURL(workerURL);data.error?reject(new Error(data.error)):resolve(data.result);};
      worker.onerror=event=>{clearTimeout(timer);worker.terminate();URL.revokeObjectURL(workerURL);reject(new Error(event.message));};
    }));
    browserVersion=browser.version();assert.deepEqual(browserResult,result,'Node and actual dedicated Chrome worker outcomes must agree');
  }finally{await browser?.close();await new Promise(resolve=>server.close(resolve));}
}
for(const ref of sources)assert.equal(hash(await fs.readFile(ref.path)),ref.sha256,'Source changed during execution');
const evidence={status:'passed-controlled-C-fault-recovery',toolkit,sources,engine:{path:wasmPath,sha256:hash(engineBytes)},real_sidecar:manifest,node:process.version,result,browser:browserVersion,browser_result:browserResult};
await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify(evidence,null,2)+'\n');
const receiptArgument=process.argv.find(arg=>arg.startsWith('--receipt='));
if(receiptArgument){
  const evidencePath=path.join(root,'evidence.json');
  await fs.writeFile(path.resolve(receiptArgument.slice(10)),JSON.stringify({status:evidence.status,evidence:{path:path.relative(process.cwd(),evidencePath).replaceAll('\\','/'),sha256:hash(await fs.readFile(evidencePath))}},null,2)+'\n');
}
console.log('PASS controlled C fault checks: unchanged outputs on error, refusal recovery, trap retirement, Rust backend restoration, fresh C recovery. '+root);
