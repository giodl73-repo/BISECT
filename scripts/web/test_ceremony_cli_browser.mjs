// Dedicated browser/current-source overlay. Use --require-package for final bytes.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import {createProject} from '../../web/lab/project.js';
const value=name=>process.argv.find(arg=>arg.startsWith('--'+name+'='))?.slice(name.length+3);
const contract=value('contract');if(!contract)throw new Error('Supply --contract=<successful actual CLI case directory>.');
const packageRoot=path.resolve(value('toolkit')??'target/report-cffi/package/toolkit'),sourceRoot=path.resolve('web/lab');
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const reference=JSON.parse(await fs.readFile(path.join(contract,'evidence.json'),'utf8'));assert.equal(reference.status,'passed-actual-CLI-ceremony-verification');
const engineFile=value('wasm')??path.join(packageRoot,'bisect_wasm.wasm'),engineBytes=await fs.readFile(engineFile);
assert.equal(hash(engineBytes),reference.sources.find(ref=>ref.path.endsWith('.wasm')).sha256);
const root=await fs.mkdtemp(path.resolve('target/ceremony-native-contract/browser-'));
const completed=await fs.readFile(path.join(contract,'completed.json')),tree=await fs.readFile(path.join(contract,'tree.json'));
const project=createProject({name:'Completed ceremony',operation:'verify-ceremony',files:{ceremonyTranscript:{name:'completed.json',base64:completed.toString('base64')},ceremonySettings:{published_unix_seconds:'',witness_receipts:[],reason:'',include_tree:false}},constraints:[],result:null,lastOperation:null});
const sources=new Map(),requests=[];
const server=createServer(async(req,res)=>{try{
 const url=new URL(req.url,'http://localhost').pathname;requests.push(url);let bytes,file;
 if(url==='/bisect_wasm.wasm')bytes=engineBytes;
 else{const relative=url==='/'?'workbench.html':decodeURIComponent(url.slice(1));file=path.resolve(sourceRoot,relative);if(!file.startsWith(sourceRoot+path.sep))throw Error('Path escaped source');try{bytes=await fs.readFile(file);sources.set(file,hash(bytes));}catch(error){if(error.code!=='ENOENT')throw error;file=path.resolve(packageRoot,relative);if(!file.startsWith(packageRoot+path.sep))throw error;bytes=await fs.readFile(file);}}
 const ext=path.extname(file??url);res.setHeader('Content-Type',['.js','.mjs'].includes(ext)?'application/javascript':ext==='.wasm'?'application/wasm':ext==='.css'?'text/css':ext==='.json'?'application/json':'text/html');res.setHeader('Cache-Control','no-store');res.end(bytes);
}catch{res.writeHead(404);res.end();}});await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const origin='http://127.0.0.1:'+server.address().port;
const {chromium}=createRequire(path.join(process.env.BISECT_PLAYWRIGHT_PACKAGES,'package.json'))('playwright');let browser;
try{
 browser=await chromium.launch({headless:true,executablePath:process.env.BISECT_TEST_BROWSER??'C:/Program Files/Google/Chrome/Application/chrome.exe'});
 const context=await browser.newContext();await context.route('**/*',route=>new URL(route.request().url()).origin===origin?route.continue():route.abort());
 const page=await context.newPage(),errors=[];page.on('pageerror',error=>errors.push(error.message));
 await page.addInitScript(()=>{window.runs=[];window.replies=[];const Original=Worker;window.Worker=class extends Original{constructor(...args){super(...args);this.addEventListener('message',({data})=>replies.push(data));}postMessage(data,...args){if(data.type==='run')runs.push(structuredClone(data));return super.postMessage(data,...args);}};});
 await page.goto(origin);await page.waitForFunction(()=>!document.querySelector('#execute').disabled);
 await page.locator('#open-project').setInputFiles({name:'completed.bisect',mimeType:'application/json',buffer:Buffer.from(JSON.stringify(project))});
 await page.waitForFunction(()=>document.querySelector('#operation').value==='verify-ceremony');assert.equal(await page.evaluate(()=>runs.length),0);
 await page.locator('#execute').click();await page.waitForFunction(()=>document.querySelector('#result-status').textContent.includes('completed ceremony verification requires --tree for final binding'));
 assert(await page.locator('#download-ceremony').isDisabled());
 await page.locator('#operation').selectOption('inspect-ceremony-prefix');assert(await page.locator('#ceremony-tree-field').isHidden());assert(await page.locator('#ceremony-scope-field').isHidden());
 await page.locator('#execute').click();await page.waitForFunction(()=>replies.some(row=>row.type==='result'&&row.result?.verification_scope==='released-prefix-only'));
 const prefix=await page.evaluate(()=>replies.find(row=>row.type==='result'&&row.result?.verification_scope==='released-prefix-only').result);
 assert.equal(prefix.status,'completed');assert.equal(prefix.verified,true);assert.equal(await page.evaluate(()=>Object.hasOwn(runs.at(-1).request,'tree_json')),false);
 const save=page.waitForEvent('download');await page.locator('#save-project').click();const savedFile=path.join(root,'prefix.bisect');await(await save).saveAs(savedFile);
 const saved=JSON.parse(await fs.readFile(savedFile,'utf8'));assert.equal(saved.operation,'inspect-ceremony-prefix');assert.deepEqual(saved.result,prefix);
 const before=await page.evaluate(()=>runs.length);await page.locator('#open-project').setInputFiles(savedFile);await page.waitForFunction(()=>document.querySelector('#result-status').textContent.includes('unverified'));assert.equal(await page.evaluate(()=>runs.length),before);
 await page.locator('#operation').selectOption('verify-ceremony');await page.locator('#ceremony-full-tree').check();
 await page.locator('#ceremonyTree-file').setInputFiles({name:'tree.json',mimeType:'application/json',buffer:tree});await page.waitForFunction(()=>document.querySelector('#ceremonyTree-name').textContent==='tree.json');
 await page.locator('#execute').click();await page.waitForFunction(()=>replies.some(row=>row.type==='result'&&row.result?.verification_scope==='supplied-complete-tree'));
 // Exercise all three transcript-producing actions against the actual CLI files.
 const open=await fs.readFile(path.join(contract,'open.json')),halted=await fs.readFile(path.join(contract,'halted.json')),round0=await fs.readFile(path.join(contract,'round-0.json'));
 await page.locator('#operation').selectOption('start-ceremony');
 await page.locator('#ceremonyConfig-file').setInputFiles({name:'config.json',mimeType:'application/json',buffer:Buffer.from(JSON.stringify(JSON.parse(open).config))});await page.waitForFunction(()=>document.querySelector('#ceremonyConfig-name').textContent==='config.json');
 await page.locator('#execute').click();await page.waitForFunction(()=>replies.some(row=>row.type==='result'&&row.result?.schema_version==='bisect-ceremony-output-v1'));
 assert.deepEqual(JSON.parse(await page.evaluate(()=>replies.filter(row=>row.type==='result').at(-1).result.transcript_json)),JSON.parse(open));
 await page.locator('#use-ceremony').click();assert.equal(await page.locator('#operation').inputValue(),'announce-ceremony');
 await page.locator('#ceremony-time').fill('1010');await page.locator('#ceremony-receipts').fill('round-A\nround-B');
 await page.locator('#execute').click();await page.waitForFunction(()=>replies.filter(row=>row.type==='result'&&row.result?.schema_version==='bisect-ceremony-output-v1').length===2);
 assert.deepEqual(JSON.parse(await page.evaluate(()=>replies.filter(row=>row.type==='result').at(-1).result.transcript_json)),JSON.parse(round0));
 await page.locator('#operation').selectOption('halt-ceremony');await page.locator('#ceremonyTranscript-file').setInputFiles({name:'open.json',mimeType:'application/json',buffer:open});await page.waitForFunction(()=>document.querySelector('#ceremonyTranscript-name').textContent==='open.json');
 await page.locator('#ceremony-reason').fill('Native parity fixture');await page.locator('#execute').click();await page.waitForFunction(()=>replies.some(row=>row.type==='result'&&row.result?.status==='halted'));
 assert.deepEqual(JSON.parse(await page.evaluate(()=>replies.filter(row=>row.type==='result').at(-1).result.transcript_json)),JSON.parse(halted));
 assert.equal(await page.evaluate(()=>runs.length),6);assert.deepEqual(errors,[]);assert(!requests.some(url=>url.startsWith('/c-metis/')));
 await page.screenshot({path:path.join(root,'completed-tree-verified.png'),fullPage:true});
 const refs=[];for(const [file,sha256]of sources){assert.equal(hash(await fs.readFile(file)),sha256);refs.push({path:path.relative(process.cwd(),file).replaceAll('\\','/'),sha256});if(process.argv.includes('--require-package'))assert.equal(hash(await fs.readFile(path.join(packageRoot,path.basename(file)==='workbench.html'?'index.html':path.basename(file)))),sha256);}
 const evidence={status:'passed-ceremony-CLI-browser-contract',actual_workbench_worker:true,browser:browser.version(),engine_sha256:hash(engineBytes),contract_evidence:{path:path.join(contract,'evidence.json'),sha256:hash(await fs.readFile(path.join(contract,'evidence.json')))},sources:refs,test_sha256:hash(await fs.readFile('scripts/web/test_ceremony_cli_browser.mjs')),prefix_inspection:prefix,explicit_runs:6,native_start_announce_halt_transcript_values_exact:true,inert_prefix_save_open:true,scope:'Native-generated k=4 transcripts: desktop-compatible verification refusal/success, separate prefix inspection and actual start/announce/halt transcript comparisons with supplied config. Current-source overlay unless require-package is set; no external publication/authentication or full parity.'};
 await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify(evidence,null,2)+'\n');console.log(JSON.stringify({status:evidence.status,root}));
}finally{await browser?.close();await new Promise(resolve=>server.close(resolve));}
