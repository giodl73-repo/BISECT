// Run desktop import and analysis, then feed their real files to the staged
// browser's native summary UI. The browser and OPFS are owned by this test.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import fs from 'node:fs/promises';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import path from 'node:path';
import {createProject,parseProject} from '../../web/lab/project.js';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {executeNativeSummaryBatch} from '../../web/lab/native-label-summary.js';

const arg=name=>{
  const i=process.argv.indexOf(name);
  assert(i>0&&process.argv[i+1],name);
  return path.resolve(process.argv[i+1]);
};
const cli=await fs.realpath(arg('--cli')),site=await fs.realpath(arg('--site')),root=arg('--output');
assert.equal(path.dirname(root),await fs.realpath('target'));
await fs.mkdir(root);
const desktop=path.join(root,'desktop');
await fs.mkdir(desktop);
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const save=(name,value)=>fs.writeFile(path.join(root,name),JSON.stringify(value,null,2)+'\n',{flag:'wx'});
const label='external_handoff',year='2020',state='washington';
const registryPath='.bisect',analysisPath=`analysis/${label}/${year}/index.json`;
const reportPrefix=`reports/${label}/${year}/`,reportPath=reportPrefix+`${label}_${year}_report.json`,indexPath=reportPrefix+'index.json';
const source=await fs.readFile('crates/rplan-audit/fixtures/grid3x3.rctx','utf8');
const ids=JSON.parse(source).units.unit_ids;
await fs.writeFile(path.join(desktop,'supplied.csv'),'GEOID,district\n'+ids.map((id,i)=>`${id},${Math.floor(i/3)+1}\n`).join(''));
const native=[];
function execute(args) {
  const res=spawnSync(cli,args,{cwd:desktop,encoding:'utf8',timeout:25000,windowsHide:true});
  assert.ifError(res.error);
  native.push({args,status:res.status,stdout:res.stdout,stderr:res.stderr});
  assert.equal(res.status,0,res.stderr);
}
const desktopText=name=>fs.readFile(path.join(desktop,name),'utf8');
execute(['label-import',label,'--from','supplied.csv','--year',year]);
execute(['label-analyze',label,'--year',year,'--states',state,'--types','summary']);
const analysis=await desktopText(analysisPath),registry=await desktopText(registryPath);
assert.equal(JSON.parse(analysis).states[state].status,'ok');
assert.deepEqual(JSON.parse(registry)[label],{built:[year],analyzed:[year],reported:[]});
assert.equal(JSON.parse(analysis).run_index_sha256,sha(await desktopText(`runs/${label}/${year}/index.json`)));
execute(['label-report',label,'--year',year,'--format','json']);
const expected={};
for(const name of [registryPath,reportPath,indexPath])expected[name]=await desktopText(name);
const generatedAt=JSON.parse(expected[reportPath]).created;
assert.equal(JSON.parse(expected[indexPath]).analysis_index_sha256,sha(analysis));
assert.deepEqual(JSON.parse(expected[registryPath])[label].reported,[year]);
await save('desktop-execution.json',{native,expected,analysis,registry});

const snapshot={schema_version:'bisect-native-label-summary-sources-v1',label,registry_json:registry,
  sources:{[year]:{analysis_directory_present:true,analysis_index_json:analysis}}};
const settings={generated_at:generatedAt,format:'json',audit_with_report:false,report_variant:'native-summary',
  output_dir:null,native_label:label};
const wasm=await fs.readFile(path.join(site,'bisect_wasm.wasm'));
const engine=await instantiateEngine(wasm);
const input={source:snapshot,year:null,format:'json',generated_at:generatedAt,output_dir:null};
const candidate=await executeNativeSummaryBatch(engine,input);
assert.equal(candidate.execution_status,'completed');
const recipe=createProject({name:'Desktop external analysis',operation:'run-label-report',
  files:{labelReportInputs:{source:{name:'native-analysis.json',base64:Buffer.from(JSON.stringify(snapshot)).toString('base64')},settings}},
  constraints:[],result:candidate,lastOperation:'run-label-report'});
const recipePath=path.join(root,'desktop-analysis.bisect');
await fs.writeFile(recipePath,JSON.stringify(recipe));
const served=new Map(),errors=[],external=[];
const server=createServer(async(req,res)=>{
  try{
    const url=new URL(req.url,'http://localhost');
    const file=path.resolve(site,'.'+decodeURIComponent(url.pathname==='/'?'/index.html':url.pathname));
    if(!file.startsWith(site+path.sep))throw Error('Invalid path');
    const bytes=await fs.readFile(file);
    served.set(path.relative(site,file).replaceAll('\\','/'),{sha256:sha(bytes),bytes:bytes.length});
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
  const context=await browser.newContext({acceptDownloads:true});
  await context.route('**/*',route=>{
    if(new URL(route.request().url()).origin===origin)return route.continue();
    external.push(route.request().url());return route.abort();
  });
  page=await context.newPage();
  page.on('pageerror',error=>errors.push(error.message));
  await page.addInitScript(()=>{
    window.handoff={requests:[],writes:[],pickers:0};
    const W=Worker;
    window.Worker=class extends W{postMessage(value,...rest){
      if(value?.type==='run')handoff.requests.push(value.request.operation);
      return super.postMessage(value,...rest);
    }};
    window.showDirectoryPicker=async()=>{handoff.pickers++;return handoff.root;};
    const create=FileSystemFileHandle.prototype.createWritable;
    FileSystemFileHandle.prototype.createWritable=async function(...args){
      const stream=await create.apply(this,args),close=stream.close.bind(stream);
      stream.close=async()=>{await close();handoff.writes.push(this.name);};
      return stream;
    };
  });
  await page.goto(origin);
  await page.waitForFunction(()=>!document.querySelector('#execute').disabled);
  const original={'.bisect':registry,[analysisPath]:analysis,'unlisted.txt':'preserved'};
  await page.evaluate(async files=>{
    handoff.root=await(await navigator.storage.getDirectory()).getDirectoryHandle('handoff-'+crypto.randomUUID(),{create:true});
    for(const [name,text] of Object.entries(files)){
      const parts=name.split('/');let dir=handoff.root;
      for(const part of parts.slice(0,-1))dir=await dir.getDirectoryHandle(part,{create:true});
      const stream=await(await dir.getFileHandle(parts.at(-1),{create:true})).createWritable();
      await stream.write(text);await stream.close();
    }
    handoff.writes=[];
  },original);
  const counts=()=>page.evaluate(()=>({requests:[...handoff.requests],writes:[...handoff.writes],pickers:handoff.pickers}));
  await page.locator('#open-project').setInputFiles(recipePath);
  await page.waitForFunction(()=>document.querySelector('#result-status').textContent.startsWith('Imported result is unverified'));
  assert.deepEqual(await counts(),{requests:[],writes:[],pickers:0});
  assert(await page.locator('#native-summary-live-review').isDisabled());
  await page.locator('#execute').click();
  await page.waitForFunction(()=>!document.querySelector('#execute').disabled&&!document.querySelector('#download-label-report').disabled);
  assert.deepEqual((await counts()).requests,['run-native-label-summary-batch']);
  const projectDownload=page.waitForEvent('download');
  await page.locator('#save-project').click();
  const savedPath=path.join(root,'browser-report.bisect');
  await(await projectDownload).saveAs(savedPath);
  const saved=parseProject(await fs.readFile(savedPath,'utf8'));
  assert.deepEqual(saved.result,candidate);
  assert.equal(saved.files.labelReportInputs.source.base64,recipe.files.labelReportInputs.source.base64);
  const archive=page.waitForEvent('download');
  await page.locator('#download-label-report').click();
  const zipPath=path.join(root,'browser-report.zip');
  await(await archive).saveAs(zipPath);
  await save('zip-expected.json',candidate.files);
  const zip=spawnSync('python',['-B','-c',
    'import json,sys,zipfile; z=zipfile.ZipFile(sys.argv[1]); files=json.load(open(sys.argv[2],encoding="utf-8")); assert sorted(z.namelist())==sorted(files); assert all(z.read(k)==v.encode("utf-8") for k,v in files.items())',
    zipPath,path.join(root,'zip-expected.json')],{encoding:'utf8',windowsHide:true});
  assert.ifError(zip.error);assert.equal(zip.status,0,zip.stderr);
  await page.locator('#native-summary-live-review').click();
  await page.waitForFunction(()=>!document.querySelector('#native-summary-live-confirm').disabled);
  assert.deepEqual((await counts()).writes,[]);
  await page.locator('#native-summary-live-confirm').check();
  await page.locator('#native-summary-live-apply').click();
  await page.waitForFunction(()=>document.querySelector('#native-summary-live-summary').textContent.startsWith('Summary application completed'));
  const actual=await page.evaluate(async()=>{
    const files={};
    async function visit(dir,prefix=''){
      for await(const [name,h] of dir.entries()){
        if(h.kind==='directory')await visit(h,prefix+name+'/');
        else files[prefix+name]=await(await h.getFile()).text();
      }
    }
    await visit(handoff.root);return files;
  });
  assert.deepEqual(Object.keys(actual).sort(),[...new Set([...Object.keys(original),...Object.keys(expected)])].sort());
  assert.equal(actual[analysisPath],analysis);
  assert.equal(actual['unlisted.txt'],original['unlisted.txt']);
  assert.equal(actual[reportPath],expected[reportPath]);
  assert.deepEqual(JSON.parse(actual[registryPath]),JSON.parse(expected[registryPath]));
  const browserIndex=JSON.parse(actual[indexPath]),desktopIndex=JSON.parse(expected[indexPath]);
  const clock={browser:browserIndex.created,desktop:desktopIndex.created};
  assert.equal(clock.browser,generatedAt);
  assert.deepEqual({...browserIndex,created:null},{...desktopIndex,created:null});
  if(clock.browser===clock.desktop)assert.equal(actual[indexPath],expected[indexPath]);
  assert.equal((await counts()).writes.at(-1),'.bisect');
  assert.equal(served.get('bisect_wasm.wasm').sha256,sha(wasm));
  assert.deepEqual(external,[]);assert.deepEqual(errors,[]);
  await save('evidence.json',{status:'passed-desktop-analysis-to-browser-report',native_sha256:sha(await fs.readFile(cli)),
    wasm_sha256:sha(wasm),browser:browser.version(),analysis_sha256:sha(analysis),clock,
    worker_opfs:true,inert_open:true,portable_zip_exact:true,os_picker_tested:false,actual,served:Object.fromEntries(served)});
  console.log(JSON.stringify({root,status:'passed-desktop-analysis-to-browser-report'}));
}catch(error){
  if(page)await page.screenshot({path:path.join(root,'failure.png'),fullPage:true}).catch(()=>{});
  throw error;
}finally{
  await browser?.close();
  await new Promise(resolve=>server.close(resolve));
}
