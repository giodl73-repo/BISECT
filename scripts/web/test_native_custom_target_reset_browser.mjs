import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import {openWorkbenchProject} from './workbench_project_open.mjs';

const [toolkitArg,referenceArg,outArg]=process.argv.slice(2);
const [toolkit,reference,out]=[toolkitArg,referenceArg,outArg].map(p=>path.resolve(p));
const read=async file=>JSON.parse(await fs.readFile(file));
const expected=await read(path.join(reference,'evidence.json'));
assert.equal(expected.status,'passed-native-custom-target-reset');
await fs.mkdir(out);
const mime={'.js':'application/javascript','.mjs':'application/javascript','.wasm':'application/wasm',
  '.html':'text/html','.css':'text/css','.json':'application/json'};
const server=createServer(async(req,res)=>{
  try{
    const {pathname}=new URL(req.url,'http://localhost'),file=path.resolve(toolkit,'.'+decodeURIComponent(pathname==='/'?'/index.html':pathname));
    if(!file.startsWith(toolkit+path.sep))throw new Error('Outside candidate toolkit');
    const bytes=await fs.readFile(file);res.setHeader('Content-Type',mime[path.extname(file)]??'application/octet-stream');
    res.end(bytes);
  }catch{res.writeHead(404);res.end();}
});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const origin='http://127.0.0.1:'+server.address().port;
const {chromium}=createRequire(path.join(process.env.BISECT_PLAYWRIGHT_PACKAGES,'package.json'))('playwright');
let browser;
try{
  browser=await chromium.launch({headless:true,executablePath:process.env.BISECT_TEST_BROWSER??'C:/Program Files/Google/Chrome/Application/chrome.exe'});
  const context=await browser.newContext({acceptDownloads:true});
  await context.route('**/*',r=>new URL(r.request().url()).origin===origin?r.continue():r.abort());
  await context.addInitScript(()=>{
    window.targetMessages=[];window.runs=[];window.picks=0;window.removals=[];
    const W=Worker;
    window.Worker=class extends W{
      constructor(...args){super(...args);this.addEventListener('message',({data})=>targetMessages.push(data));}
      postMessage(message,...args){if(message.type==='run')runs.push(message.request.operation);return super.postMessage(message,...args);}
    };
    window.showDirectoryPicker=async()=>{picks++;return window.testRoot;};
    const remove=FileSystemDirectoryHandle.prototype.removeEntry;
    FileSystemDirectoryHandle.prototype.removeEntry=function(name,options){removals.push(name);return remove.call(this,name,options);};
  });
  const page=await context.newPage(),errors=[];page.on('pageerror',e=>errors.push(e.message));
  await page.goto(origin);await page.waitForFunction(()=>!document.querySelector('#execute').disabled);
  const save=async name=>{const d=page.waitForEvent('download');await page.locator('#save-project').click();
    const file=path.join(out,name);await(await d).saveAs(file);return read(file);};
  const counters=()=>page.evaluate(()=>({runs:runs.length,picks,removals:removals.length}));
  const results=[];
  for(const row of expected.cases){
    await page.evaluate(async ns=>{
      testRoot=await(await navigator.storage.getDirectory()).getDirectoryHandle('fixture-'+crypto.randomUUID(),{create:true});
      let d=testRoot;
      for(const p of ns.split('/'))d=await d.getDirectoryHandle(p,{create:true});
      window.target=d;const data=await d.getDirectoryHandle('data',{create:true});
      for(const [root,name,value] of [[d,'keep.txt','notes'],[data,'old.txt','old data']]){
        const w=await(await root.getFileHandle(name,{create:true})).createWritable();await w.write(value);await w.close();
      }
    },row.target);
    const before=await counters();await openWorkbenchProject(page,path.join(reference,row.case+'.bisect'));
    assert.deepEqual(await counters(),before);
    const messages=await page.evaluate(()=>targetMessages.length);await page.locator('#execute').click();
    await page.waitForFunction(n=>targetMessages.slice(n).some(m=>m.type==='native-state-target-required'),messages);
    const intent=await page.evaluate(n=>targetMessages.slice(n).find(m=>m.type==='native-state-target-required').intent,messages);
    assert.equal(intent.target_namespace,row.target);assert.equal(intent.reset,row.reset);
    assert.equal((await counters()).removals,before.removals);
    await page.locator('#review-native-state-before-run').click();
    await page.waitForFunction(()=>!document.querySelector('#confirm-native-state-before-run').disabled);
    assert.equal((await counters()).removals,before.removals);
    assert.equal(await page.evaluate(async()=>await(await(await target.getDirectoryHandle('data')).getFileHandle('old.txt')).getFile().then(f=>f.text())),'old data');
    await page.locator('#confirm-native-state-before-run').click();
    await page.waitForFunction(()=>!document.querySelector('#execute').disabled&&!document.querySelector('#download').disabled,null,{timeout:30000});
    const packet=await save(row.case+'.bisect'),run=packet.result;
    assert.deepEqual(run.prepared.result.assignments,row.assignments);
    assert.equal(run.prepared.export.document.metadata.label,row.expectedLabel);
    assert.equal(run.native_state.target_actions.intent.target_namespace,row.target);
    const remaining=await page.evaluate(async()=>{
      let old;
      try{old=await(await(await target.getDirectoryHandle('data')).getFileHandle('old.txt')).getFile().then(f=>f.text());}
      catch(e){if(e.name!=='NotFoundError')throw e;old=null;}
      return {old,notes:await(await target.getFileHandle('keep.txt')).getFile().then(f=>f.text())};
    });
    assert.deepEqual(remaining,{old:row.reset?null:'old data',notes:'notes'});
    assert.equal((await counters()).removals,before.removals+(row.reset?1:0));
    const after=await counters();await openWorkbenchProject(page,path.join(out,row.case+'.bisect'));
    assert.deepEqual(await counters(),after);assert(await page.locator('#confirm-native-state-before-run').isDisabled());
    results.push({case:row.case,target:row.target,label:row.expectedLabel,assignments:row.assignments,reset:row.reset});
  }
  assert.deepEqual(errors,[]);
  await fs.writeFile(path.join(out,'evidence.json'),JSON.stringify({status:'passed-custom-target-real-worker-and-opfs',
    browser:browser.version(),wasm:expected.wasm,cases:results,open_inert:true,actual_os_picker_permission:false,full_parity:false},null,2)+'\n');
}finally{await browser?.close();await new Promise(resolve=>server.close(resolve));}
