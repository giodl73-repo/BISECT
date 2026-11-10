// Exercise actual upload, Run, ZIP, workbench Save/Open and inert Open
// for the >3 MiB real RCTX inputs. Only an owned browser is launched.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import fs from 'node:fs/promises';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {openWorkbenchProject} from './workbench_project_open.mjs';

const arg=name=>{
  const i=process.argv.indexOf(name);assert(i>0&&process.argv[i+1],name);
  return path.resolve(process.argv[i+1]);
};
const site=await fs.realpath(arg('--site')),reference=await fs.realpath(arg('--reference')),root=arg('--output');
assert.equal(path.dirname(root),await fs.realpath('target'));
await fs.mkdir(root);
const mod=name=>import(pathToFileURL(path.join(site,name)).href);
const {createProject}=await mod('project.js');
const {parseWorkbenchArchiveBytes}=await mod('workbench-archive.js');
const sha=b=>createHash('sha256').update(b).digest('hex');
const read=async p=>JSON.parse(await fs.readFile(p,'utf8'));
const measured=await read(path.join(reference,'evidence.json'));
assert.equal(measured.status,'passed-real-retained-state-label-contiguity');
const wasm=await fs.readFile(path.join(site,'bisect_wasm.wasm'));
assert.equal(sha(wasm),measured.wasm_sha256);
const served={},errors=[],external=[];
const server=createServer(async(req,res)=>{
  try{
    const url=new URL(req.url,'http://localhost'),file=path.resolve(site,'.'+decodeURIComponent(url.pathname==='/'?'/index.html':url.pathname));
    assert(file.startsWith(site+path.sep));
    const bytes=await fs.readFile(file);
    served[path.relative(site,file).replaceAll('\\','/')]={sha256:sha(bytes),bytes:bytes.length};
    res.setHeader('Content-Type',file.endsWith('.js')?'application/javascript':file.endsWith('.wasm')?'application/wasm':file.endsWith('.css')?'text/css':'text/html');
    res.end(bytes);
  }catch{res.writeHead(404).end();}
});
const packages=process.env.BISECT_PLAYWRIGHT_PACKAGES;
assert(packages);
const {chromium}=createRequire(path.join(packages,'package.json'))('playwright');
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const origin='http://127.0.0.1:'+server.address().port;
let browser,page,milestone='browser start';
try{
  browser=await chromium.launch({headless:true,executablePath:process.env.BISECT_TEST_BROWSER||'C:/Program Files/Google/Chrome/Application/chrome.exe',
    args:['--no-first-run','--disable-background-networking','--disable-extensions']});
  const tab=await browser.newContext({acceptDownloads:true});
  await tab.route('**/*',route=>{
    if(new URL(route.request().url()).origin===origin)return route.continue();
    external.push(route.request().url());return route.abort();
  });
  page=await tab.newPage();page.on('pageerror',e=>errors.push(e.message));
  await page.addInitScript(()=>{
    window.executions=[];window.pickers=0;
    window.showDirectoryPicker=async()=>{pickers++;throw Error('Local context analysis must not ask for directory access.');};
    const W=Worker;
    window.Worker=class extends W{postMessage(value,...rest){
      if(value?.type==='run')executions.push(value.request.operation);
      return super.postMessage(value,...rest);
    }};
  });
  await page.goto(origin);
  const ready=selector=>page.waitForFunction(s=>!document.querySelector('#execute').disabled&&!document.querySelector(s).disabled,selector,{timeout:240000});
  const capture=()=>page.evaluate(()=>({requests:[...executions],pickers}));
  const download=async(selector,filename)=>{
    const waiting=page.waitForEvent('download',{timeout:240000});
    await page.locator(selector).click();
    await(await waiting).saveAs(filename);
  };
  const rows=[];
  for(const [id,state] of [['ca-tract','california'],['ri-block','rhode_island']]){
    milestone=id+' project Open';
    const record=measured.outcomes.find(r=>r.id===id),dir=path.join(reference,id),label=id.replace('-','_'),year='2020';
    assert(record.context_bytes>3*1024*1024&&record.context_bytes<=8*1024*1024);
    const contextPath=path.join(dir,'source.rctx'),csv=await fs.readFile(path.join(dir,'assigned.csv'));
    const context=await fs.readFile(contextPath);
    assert.equal(context.length,record.context_bytes);assert.equal(sha(context),record.context_sha256);
    const importIndex=await read(path.join(dir,`runs/${label}/${year}/index.json`));
    const indexPath=`analysis/${label}/${year}/index.json`,reportPath=`analysis/${label}/${year}/${state}/contiguity.json`;
    const analysis=await read(path.join(dir,indexPath));
    const recipe=createProject({name:id,operation:'import-label-assignments',files:{labelImportInputs:{
      source:{name:'assigned.csv',base64:csv.toString('base64')},dbf:null,shx:null,registry:null,
      settings:{label,year,format:'auto',created:importIndex.created}}},constraints:[],result:null,lastOperation:null});
    const recipePath=path.join(root,id+'.bisect');
    await fs.writeFile(recipePath,JSON.stringify(recipe));
    const before=await capture();
    await openWorkbenchProject(page,recipePath,{ready:({argument:n,openedStatus})=>openedStatus==='Opened '+n,readyArg:path.basename(recipePath)});
    assert.deepEqual(await capture(),before);
    milestone=id+' assignment import';
    await page.locator('#execute').click();await ready('#use-label-import-context');
    await page.locator('#label-import-state').selectOption(state);
    await page.locator('#use-label-import-context').click();
    milestone=id+' context upload';
    await page.locator('#label-import-context-file').setInputFiles(contextPath);
    await page.waitForFunction(()=>document.querySelector('#label-import-context-name').textContent.includes('source.rctx'));
    milestone=id+' context alignment';
    await page.locator('#execute').click();await ready('#use-label-import-batch');
    await page.locator('#use-label-import-batch').click();
    await page.locator('#label-analysis-created').fill(analysis.created);
    await page.locator('#label-analysis-types').fill('contiguity');
    milestone=id+' contiguity Run';
    await page.locator('#execute').click();await ready('#download-label-analysis');
    assert.deepEqual((await capture()).requests.slice(before.requests.length),
      ['import-label-assignments','prepare-label-import-plan','run-label-analysis']);
    const zip=path.join(root,id+'.zip'),archive=path.join(root,id+'.bisect-workbench');
    milestone=id+' ZIP';
    await download('#download-label-analysis',zip);
    const z=spawnSync('python',['-B','-c',
      'import sys,zipfile,pathlib; z=zipfile.ZipFile(sys.argv[1]); root=pathlib.Path(sys.argv[2]); assert z.testzip() is None; assert all(z.read(n)==(root/n).read_bytes() for n in sys.argv[3:5]); assert z.read(sys.argv[5])==(root/"source.rctx").read_bytes()',
      zip,dir,indexPath,reportPath,`analysis/${label}/${year}/${state}/context.rctx`],{encoding:'utf8',windowsHide:true});
    assert.ifError(z.error);assert.equal(z.status,0,z.stderr);
    milestone=id+' archive Save';
    await download('#save-workbench-archive',archive);
    const bytes=await fs.readFile(archive),saved=await parseWorkbenchArchiveBytes(bytes);
    const batch=JSON.parse(Buffer.from(saved.files.labelAnalysisInputs.source.base64,'base64').toString('utf8'));
    assert.equal(sha(batch.states[state].context_json),record.context_sha256);
    assert.equal(sha(saved.result.files[reportPath]),record.report_sha256);
    assert.equal(sha(saved.result.files[indexPath]),record.index_sha256);
    const beforeOpen=await capture();
    milestone=id+' archive Open';
    await page.locator('#open-workbench-archive').setInputFiles(archive);
    await page.waitForFunction(name=>document.querySelector('#project-status').textContent.startsWith('Opened workbench archive '+name),
      path.basename(archive),{timeout:240000});
    assert.deepEqual(await capture(),beforeOpen);
    assert(await page.locator('#download-label-analysis').isDisabled());
    rows.push({id,units:record.units,context_bytes:context.length,context_sha256:sha(context),
      report_sha256:record.report_sha256,zip_sha256:sha(await fs.readFile(zip)),archive_sha256:sha(bytes),archive_bytes:bytes.length});
  }
  assert.equal((await capture()).pickers,0);
  assert.equal(served['bisect_wasm.wasm'].sha256,sha(wasm));
  assert.deepEqual(errors,[]);assert.deepEqual(external,[]);
  await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({status:'passed-real-large-context-label-browser',
    rows,native_sha256:measured.native_sha256,wasm_sha256:sha(wasm),browser:browser.version(),served,
    inert_archive_open:true,exact_context_and_cli_download:true,partition_performed:false,full_parity:false},null,2)+'\n');
  console.log(JSON.stringify({root,rows}));
}catch(error){
  if(page){
    console.error(milestone,await page.locator('#project-status').innerText(),await page.locator('#result-status').innerText());
    await page.screenshot({path:path.join(root,'failure.png'),fullPage:true}).catch(()=>{});
  }
  throw error;
}finally{
  await browser?.close();
  await new Promise(resolve=>server.close(resolve));
}
