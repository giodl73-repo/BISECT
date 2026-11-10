// Self-contained actual CLI/browser regression; fixtures are generated below.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {createProject} from '../../web/lab/project.js';
import {openWorkbenchProject} from './workbench_project_open.mjs';
const {chromium}=createRequire(path.join(process.env.BISECT_PLAYWRIGHT_PACKAGES,'package.json'))('playwright');
await fs.mkdir(path.resolve('target'),{recursive:true});
const root=await fs.mkdtemp(path.resolve('target/native-aggregate-order-'));
const source=path.resolve(process.env.BISECT_TEST_WEB_SOURCE||'web/lab');
const wasmPath=path.resolve(process.env.BISECT_TEST_WASM||'target/wasm32-unknown-unknown/release/bisect_wasm.wasm');
const cliPath=path.resolve(process.env.BISECT_TEST_NATIVE_CLI||'target/release/bisect.exe');
const wasm=await fs.readFile(wasmPath),cliBytes=await fs.readFile(cliPath),cli=path.join(root,'native.exe');
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
await fs.writeFile(cli,cliBytes);assert.equal(sha(await fs.readFile(cliPath)),sha(cliBytes));
const types=['summary','proportionality','summary'];
const reports={alpha:{summary:'{"analyzer":"summary","districts":[{"district":1,"population":17}]}',proportionality:'{"analyzer":"proportionality","votes":30}'},zeta:{summary:'{"analyzer":"summary","districts":[{"district":2,"population":18446744073709551615}]}',proportionality:'{"analyzer":"proportionality","votes":100}'}};
const served={},errors=[];
const server=createServer(async(req,res)=>{try{
 const pathname=decodeURIComponent(new URL(req.url,'http://localhost').pathname);
 const file=path.resolve(source,'.'+(pathname==='/'?'/workbench.html':pathname));
 assert(file.startsWith(source+path.sep));
 const bytes=pathname==='/bisect_wasm.wasm'?wasm:await fs.readFile(file);
 served[pathname]=sha(bytes);res.setHeader('Content-Type',pathname.endsWith('.js')?'application/javascript':pathname.endsWith('.wasm')?'application/wasm':pathname.endsWith('.css')?'text/css':'text/html');res.end(bytes);
}catch{res.writeHead(404);res.end();}});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const origin='http://127.0.0.1:'+server.address().port;let context;
try{
 context=await chromium.launchPersistentContext(path.join(root,'profile'),{headless:true,executablePath:process.env.BISECT_TEST_BROWSER||'C:/Program Files/Google/Chrome/Application/chrome.exe',args:['--no-first-run','--disable-background-networking','--disable-extensions']});
 await context.route('**/*',route=>new URL(route.request().url()).origin===origin?route.continue():route.abort());
 const page=await context.newPage();page.on('pageerror',error=>errors.push(error.message));
 await page.addInitScript(()=>{window.aggregateRequests=[];const Original=Worker;window.Worker=class extends Original{postMessage(message,...rest){if(message.type==='run')aggregateRequests.push(message.request);return super.postMessage(message,...rest);}};});
 await page.goto(origin);const cases=[];
 for(const force of [false,true]){
  const name=force?'force':'no-force',work=path.join(root,name);await fs.mkdir(work);
  for(const [state,files]of Object.entries(reports))for(const [type,text]of Object.entries(files)){const filename=path.join(work,'outputs/test/2020/states',state,'analysis',type+'.json');await fs.mkdir(path.dirname(filename),{recursive:true});await fs.writeFile(filename,text);}
  const args=['aggregate','--version','test','--year','2020','--types',...types,'--format','csv',...(force?['--force']:[])];
  await fs.writeFile(path.join(work,'arguments.json'),JSON.stringify(args));const native=spawnSync(cli,args,{cwd:work,maxBuffer:16*1024*1024});await fs.writeFile(path.join(work,'native.stdout.bin'),native.stdout);await fs.writeFile(path.join(work,'native.stderr.bin'),native.stderr);assert.equal(native.status,0,native.stderr.toString());assert.equal(native.stderr.toString().includes('[skip] summary'),!force);
  const inputs={settings:{year:'2020',types:[...types],format:'csv',csv:false},completion:{year:'2020',existing_outputs:[],force},reports:Object.fromEntries(Object.entries(reports).map(([state,files])=>[state,Object.fromEntries(Object.entries(files).map(([type,text])=>[type,{name:type+'.json',base64:Buffer.from(text).toString('base64')}]))]))};
  const packet=createProject({name:'Ordered aggregation '+name,operation:'aggregate-native-analysis',constraints:[],files:{nativeAggregateInputs:inputs},result:null,lastOperation:null});const input=path.join(work,'input.bisect');await fs.writeFile(input,JSON.stringify(packet));
  const before=await page.evaluate(()=>aggregateRequests.length);await openWorkbenchProject(page,input,{ready:()=>!document.querySelector('#execute').disabled});assert.equal(await page.evaluate(()=>aggregateRequests.length),before);
  assert((await page.locator('#native-aggregate-names').textContent()).includes('Analyzer order: summary -> proportionality -> summary'));
  await page.locator('#native-aggregate-format').selectOption('json');await page.locator('#native-aggregate-format').selectOption('csv');
  await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#download-native-aggregate').disabled);assert.equal(await page.evaluate(()=>aggregateRequests.length),before+1);const request=await page.evaluate(()=>aggregateRequests.at(-1));await fs.writeFile(path.join(work,'posted-request.json'),JSON.stringify(request));assert.deepEqual(request.input.types,types);assert.equal(request.input.completion.force,force);
  const save=async filename=>{const pending=page.waitForEvent('download');await page.locator('#save-project').click();const dest=path.join(work,filename);await(await pending).saveAs(dest);return {path:dest,value:JSON.parse(await fs.readFile(dest,'utf8'))};};
  const actual=await save('actual.bisect');assert.deepEqual(actual.value.result.summaries.map(row=>row.status),force?['aggregated','aggregated','aggregated']:['aggregated','aggregated','skipped-existing']);
  const national=path.join(work,'outputs/test/2020/national');assert.deepEqual(Object.keys(actual.value.result.files).sort(),(await fs.readdir(national)).sort());for(const [filename,text]of Object.entries(actual.value.result.files)){assert.equal(text,await fs.readFile(path.join(national,filename),'utf8'));assert.equal(actual.value.result.file_sha256[filename],sha(Buffer.from(text)));}
  const pending=page.waitForEvent('download');await page.locator('#download-native-aggregate').click();await(await pending).saveAs(path.join(work,'outputs.zip'));
  await page.locator('#native-aggregate-types').selectOption(['summary']);assert.equal(await page.locator('#download-native-aggregate').isDisabled(),true);const edited=await save('edited.bisect');assert.deepEqual(edited.value.files.nativeAggregateInputs.settings.types,['summary']);assert.deepEqual(edited.value.files.nativeAggregateInputs.reports,inputs.reports);assert.equal(await page.evaluate(()=>aggregateRequests.length),before+1);
  await openWorkbenchProject(page,actual.path);assert.equal(await page.evaluate(()=>aggregateRequests.length),before+1);assert.equal(await page.locator('#download-native-aggregate').isDisabled(),true);const reopened=await save('reopened.bisect');assert.deepEqual(reopened.value.files.nativeAggregateInputs,actual.value.files.nativeAggregateInputs);
  cases.push({name,actual_cli_exit:native.status,worker_runs:1,exact_native_output_files:true,ordered_repeated_types_preserved:true,format_edits_preserve_order:true,explicit_type_edit_replaces_order_and_retires_exports:true,inert_open_and_reopen:true});
 }
 assert.deepEqual(errors,[]);assert.equal(served['/bisect_wasm.wasm'],sha(wasm));await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({status:'passed',browser:context.browser().version(),native_cli_sha256:sha(cliBytes),wasm_sha256:sha(wasm),cases,page_errors:errors,served_assets:served,scope:'Two synthetic actual CLI/real-worker browser cases for imported analyzer order, repetition and Force, source preservation, explicit edits/downloads and inert reopen. Not full Aggregate installation/host/capacity/profile or acceptance-v1 closure.'},null,2));console.log('PASS actual CLI/browser analyzer order, duplicates, Force and explicit edits. Evidence '+root);
}catch(error){await fs.writeFile(path.join(root,'failure.json'),JSON.stringify({message:error.message,stack:error.stack,served_assets:served},null,2));console.error('Retained aggregate order failure: '+root);throw error;}finally{await context?.close();await new Promise(resolve=>server.close(resolve));}
