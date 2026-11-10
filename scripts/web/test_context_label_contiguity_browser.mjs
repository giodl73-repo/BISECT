// Use exact actual-CLI sources on the staged workbench and read the ZIP back.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import fs from 'node:fs/promises';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import path from 'node:path';
import {createProject,parseProject} from '../../web/lab/project.js';

const arg=name=>{
  const i=process.argv.indexOf(name);assert(i>0&&process.argv[i+1],name);
  return path.resolve(process.argv[i+1]);
};
const site=await fs.realpath(arg('--site')),reference=await fs.realpath(arg('--reference')),root=arg('--output');
assert.equal(path.dirname(root),await fs.realpath('target'));
await fs.mkdir(root);
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const read=async p=>JSON.parse(await fs.readFile(p,'utf8'));
const native=await read(path.join(reference,'evidence.json'));
assert.equal(native.status,'passed-native-context-label-contiguity');
const wasm=await fs.readFile(path.join(site,'bisect_wasm.wasm'));
assert.equal(sha(wasm),native.wasm.sha256);
const served={},errors=[],network=[];
const server=createServer(async(req,res)=>{
  try{
    const url=new URL(req.url,'http://localhost');
    const file=path.resolve(site,'.'+decodeURIComponent(url.pathname==='/'?'/index.html':url.pathname));
    assert(file.startsWith(site+path.sep));
    const bytes=await fs.readFile(file);
    served[path.relative(site,file).replaceAll('\\','/')]={sha256:sha(bytes),bytes:bytes.length};
    res.setHeader('Content-Type',file.endsWith('.js')?'application/javascript':file.endsWith('.wasm')?'application/wasm':file.endsWith('.css')?'text/css':'text/html');
    res.end(bytes);
  }catch{res.writeHead(404).end();}
});
const packages=process.env.BISECT_PLAYWRIGHT_PACKAGES;
assert(packages,'Set BISECT_PLAYWRIGHT_PACKAGES');
const {chromium}=createRequire(path.join(packages,'package.json'))('playwright');
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const origin='http://127.0.0.1:'+server.address().port;
let browser,page;
try{
  browser=await chromium.launch({headless:true,executablePath:process.env.BISECT_TEST_BROWSER||'C:/Program Files/Google/Chrome/Application/chrome.exe',
    args:['--no-first-run','--disable-background-networking','--disable-extensions']});
  const tab=await browser.newContext({acceptDownloads:true});
  await tab.route('**/*',route=>{
    if(new URL(route.request().url()).origin===origin)return route.continue();
    network.push(route.request().url());return route.abort();
  });
  page=await tab.newPage();page.on('pageerror',e=>errors.push(e.message));
  await page.addInitScript(()=>{
    window.executions=[];
    window.showDirectoryPicker=async()=>{throw Error('This analysis cannot request folder access.');};
    const W=Worker;
    window.Worker=class extends W{postMessage(value,...rest){
      if(value?.type==='run')executions.push(value.request);
      return super.postMessage(value,...rest);
    }};
  });
  await page.goto(origin);
  await page.waitForFunction(()=>!document.querySelector('#execute').disabled);
  const open=async(filename,name)=>{
    const before=await page.evaluate(()=>executions.length);
    await page.locator('#open-project').setInputFiles(filename);
    await page.waitForFunction(n=>document.querySelector('#project-name').value===n&&
      document.querySelector('#result-status').textContent.startsWith('Imported result is unverified'),name);
    assert.equal(await page.evaluate(()=>executions.length),before);
    assert(await page.locator('#download-label-analysis').isDisabled());
  };
  const cases=[];
  for(const row of native.outcomes.filter(r=>Object.hasOwn(r,'contiguous'))){
    const dir=path.join(reference,row.id);
    const expected=await read(path.join(dir,'wasm-result.json'));
    const request=await read(path.join(dir,'wasm-request.json'));
    const {types,created,...payload}=request.input;
    const inputs={source:{name:'recorded-analysis.json',base64:Buffer.from(JSON.stringify({...payload,created})).toString('base64')},created,types};
    const recipe=createProject({name:row.id,operation:'run-label-analysis',files:{labelAnalysisInputs:inputs},
      constraints:[],result:expected,lastOperation:'run-label-analysis'});
    const initial=path.join(root,row.id+'.bisect');
    await fs.writeFile(initial,JSON.stringify(recipe));
    await open(initial,row.id);
    const before=await page.evaluate(()=>executions.length);
    await page.locator('#execute').click();
    await page.waitForFunction(()=>!document.querySelector('#execute').disabled&&!document.querySelector('#download-label-analysis').disabled);
    const executed=await page.evaluate(n=>executions.slice(n),before);
    assert.deepEqual(executed,[request]);
    const project=page.waitForEvent('download');
    await page.locator('#save-project').click();
    const savedFile=path.join(root,row.id+'-saved.bisect');
    await(await project).saveAs(savedFile);
    const saved=parseProject(await fs.readFile(savedFile,'utf8'));
    assert.deepEqual(saved.result,expected);
    assert.deepEqual(saved.files.labelAnalysisInputs,inputs);
    const download=page.waitForEvent('download');
    await page.locator('#download-label-analysis').click();
    const zip=path.join(root,row.id+'.zip');
    await(await download).saveAs(zip);
    const report='analysis/context_fixture/2020/washington/contiguity.json',index='analysis/context_fixture/2020/index.json';
    const p=spawnSync('python',['-B','-c',
      'import sys, zipfile, pathlib; z=zipfile.ZipFile(sys.argv[1]); root=pathlib.Path(sys.argv[2]); assert z.testzip() is None; assert len(z.namelist())==len(set(z.namelist())); assert all(z.read(n)==(root/n).read_bytes() for n in sys.argv[3:])',
      zip,dir,report,index],{encoding:'utf8',windowsHide:true});
    assert.ifError(p.error);assert.equal(p.status,0,p.stderr);
    cases.push({id:row.id,contiguous:row.contiguous,report_sha256:sha(await fs.readFile(path.join(dir,report))),
      analysis_index_sha256:sha(await fs.readFile(path.join(dir,index))),zip_sha256:sha(await fs.readFile(zip))});
    await open(savedFile,row.id);
  }
  assert.equal(cases.length,3);
  assert.equal(served['bisect_wasm.wasm'].sha256,native.wasm.sha256);
  assert.deepEqual(errors,[]);assert.deepEqual(network,[]);
  await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({
    status:'passed-actual-browser-context-label-contiguity',native_sha256:native.native.sha256,wasm_sha256:native.wasm.sha256,
    browser:browser.version(),cases,served,inert_open:true,actual_worker:true,exact_native_zip_report_and_index:true,
    os_picker_tested:false,full_parity:false,
  },null,2)+'\n');
  console.log(JSON.stringify({root,cases:cases.length}));
}catch(e){
  if(page)await page.screenshot({path:path.join(root,'failure.png'),fullPage:true}).catch(()=>{});
  throw e;
}finally{
  await browser?.close();
  await new Promise(resolve=>server.close(resolve));
}
