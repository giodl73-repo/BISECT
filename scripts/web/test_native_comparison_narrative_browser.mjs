import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import {openWorkbenchProject} from './workbench_project_open.mjs';

const args=process.argv.slice(2);
const arg=name=>{const i=args.indexOf(name);assert(i>=0&&args[i+1],'Supply '+name);return path.resolve(args[i+1]);};
const site=await fs.realpath(arg('--site')),cli=await fs.realpath(arg('--cli')),parent=arg('--output');
const wasm=path.join(site,'bisect_wasm.wasm');
const {chromium}=createRequire(path.join(process.env.BISECT_PLAYWRIGHT_PACKAGES??process.cwd(),'package.json'))('playwright');
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
await fs.mkdir(parent,{recursive:true});
const output=await fs.mkdtemp(path.join(parent,'native-comparison-narrative-'));
const root=path.join(output,'native-fixture');
await fs.mkdir(root);
const native={source_date_epoch:'1793844000',results:[]};
const plans=[
 ['Alpha',{label:'Alpha example',num_districts:2,year:'2020'},{'44001000100':1,'44001000200':1,'44001000300':2,'44001000400':2},[.58,.52],[.42,.61],1],
 ['Beta',{label:'Beta example',num_districts:2,year:'2020',submission_type:'civic_counter_proposal',submitted_by:'Example contributor',submitted_at:'2026-11-05'},{'44001000100':1,'44001000200':2,'44001000300':2,'44001000400':2},[.57,.55],[.55,.63],0]
];
for(const [name,manifest,assignments,shares,compactness,count] of plans){
 for(const [rel,value] of [
  ['manifest.json',manifest],['data/final_assignments.json',assignments],
  ['analysis/partisan.json',{per_district_dem_share:shares}],['analysis/vra_analysis.json',{mm_count:count}],
  ['analysis/compactness.json',{districts:compactness.map(polsby_popper=>({polsby_popper}))}]
 ]){
  const dest=path.join(root,name,rel);
  await fs.mkdir(path.dirname(dest),{recursive:true});
  await fs.writeFile(dest,JSON.stringify(value,null,2),{flag:'wx'});
 }
}
for(const [name,format,signoff] of [['draft','narrative',null],['named','narrative','Example reviewer'],['both','both',null],['html','html','Example reviewer']]){
 const dir=path.join(root,name);
 const argv=['compare','--plan-a','./Alpha','--plan-b','./Beta','--format',format,'--report-dir',dir,'--leaning-threshold','0.55','--close-call-band','0.02'];
 if(signoff)argv.push('--approved-by',signoff);
 const result=spawnSync(cli,argv,{cwd:root,env:{...process.env,SOURCE_DATE_EPOCH:native.source_date_epoch},windowsHide:true,maxBuffer:1024*1024});
 assert.ifError(result.error);assert.equal(result.status,0,result.stderr?.toString());
 await fs.writeFile(path.join(dir,'stdout'),result.stdout,{flag:'wx'});
 await fs.writeFile(path.join(dir,'stderr'),result.stderr,{flag:'wx'});
 const manifest=JSON.parse(await fs.readFile(path.join(dir,'narrative_manifest.json'),'utf8'));
 assert.equal(manifest.approved_by,signoff);
 assert.equal(manifest.civic_counter_proposal_attribution.plan_label,'Beta example');
 const files=[];
 for(const file of (await fs.readdir(dir)).sort()){
  const bytes=await fs.readFile(path.join(dir,file));files.push({file,bytes:bytes.length,sha256:sha(bytes)});
 }
 native.results.push({case:name,format,approved_by:signoff,manifest,argv,files});
}
const mime={'.js':'text/javascript','.css':'text/css','.json':'application/json','.wasm':'application/wasm'};
const server=createServer(async(req,res)=>{try{
 const name=decodeURIComponent(new URL(req.url,'http://localhost').pathname);
 const file=await fs.realpath(path.resolve(site,'.'+name));
 assert(file.startsWith(site+path.sep));
 res.setHeader('Content-Type',mime[path.extname(file)]??'text/html');res.end(await fs.readFile(file));
}catch{res.writeHead(404);res.end();}});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const origin='http://127.0.0.1:'+server.address().port;
const evidence={status:'running',full_parity:false,scope:'Actual supplied toolkit package and Chrome against current native CLI artifact bytes. Build provenance fields are the only platform allowances. No source authentication or certification claim.',
 wasm_sha256:sha(await fs.readFile(wasm)),cli_sha256:sha(await fs.readFile(cli)),native,cases:[],errors:[],external:[]};
let browser,page;
try{
 browser=await chromium.launch({headless:true,executablePath:'C:/Program Files/Google/Chrome/Application/chrome.exe'});
 const context=await browser.newContext({acceptDownloads:true});
 await context.route('**/*',route=>new URL(route.request().url()).origin===origin?route.continue():
  (evidence.external.push(route.request().url()),route.abort()));
 await context.addInitScript(()=>{
  const Native=window.Worker;window.narrativeRuns=[];
  window.Worker=class extends Native{postMessage(message,...options){
   if(message?.request?.operation==='render-native-comparison-narrative')
    window.narrativeRuns.push({format:message.request.input.format,signoff:message.request.input.signoff});
   return super.postMessage(message,...options);
  }};
 });
 page=await context.newPage();
 page.on('pageerror',error=>evidence.errors.push(error.message));
 const get=id=>page.locator('#'+id);
 const waitReady=()=>page.waitForFunction(()=>!document.querySelector('#execute').disabled,undefined,{timeout:15000});
 const waitResult=()=>page.waitForFunction(()=>!document.querySelector('#download-native-narrative-manifest').disabled,undefined,{timeout:15000});
 const runs=()=>page.evaluate(()=>window.narrativeRuns);
 const download=async(id,file)=>{
  const pending=page.waitForEvent('download');await get(id).click();
  await (await pending).saveAs(file);return fs.readFile(file);
 };
 await page.goto(origin+'/index.html');
 await page.waitForFunction(hash=>document.querySelector('#runtime-artifact')?.textContent.includes(hash),evidence.wasm_sha256,{timeout:15000});
 await get('operation').selectOption('render-native-comparison-narrative');
 assert(await get('execute').isDisabled());
 for(const [side,label] of [['a','Alpha'],['b','Beta']]){
  await get('native-narrative-'+side).setInputFiles(path.join(root,label));
  await page.waitForFunction(({side,label})=>document.querySelector('#native-narrative-name-'+side).textContent===label,{side,label});
 }
 await get('native-narrative-epoch').fill(native.source_date_epoch);
 assert.deepEqual(await runs(),[]);
 for(const row of native.results){
  const folder=path.join(output,row.case);await fs.mkdir(folder);
  await get('native-narrative-format').selectOption(row.format);
  if(row.approved_by){
   await get('native-narrative-name').fill(row.approved_by);
   const before=(await runs()).length;
   await waitReady();await get('execute').click();
   await page.waitForFunction(()=>document.querySelector('#result-status').textContent.includes('Confirm a plain sign-off name'));
   assert.equal((await runs()).length,before);
   await get('native-narrative-confirm').check();
  }
  await waitReady();await get('execute').click();await waitResult();
  assert.equal(await get('native-narrative-name').inputValue(),'');
  assert(!await get('native-narrative-confirm').isChecked());
  const actualFiles={};
  const exports=[['markdown','narrative.md'],['manifest','narrative_manifest.json']];
  if(row.format==='both')exports.push(['table','comparison.txt']);
  if(row.format==='html')exports.push(['html','comparison.html']);
  for(const [key,name] of exports){
   const bytes=await download('download-native-narrative-'+key,path.join(folder,name));
   const originalName=key==='table'?'stdout':name;
   const oracle=await fs.readFile(path.join(root,row.case,originalName),'utf8');
   let expected=oracle;
   // Only producer-specific desktop provenance is absent in this browser artifact.
   if(key==='manifest'){
    for(const field of ['build_commit','build_commit_short','rustc_version']){
     const original=JSON.stringify(row.manifest[field]);
     const entry='"'+field+'": '+original;
     assert.equal(expected.split(entry).length,2);
     expected=expected.replace(entry,'"'+field+'": ""');
    }
   }else if(key==='html'){
    const entry='(build <code>'+row.manifest.build_commit_short+'</code>)';
    assert.equal(expected.split(entry).length,2);
    expected=expected.replace(entry,'(build <code></code>)');
   }
   assert.deepEqual(bytes,Buffer.from(expected),row.case+':'+name);
   actualFiles[name]={bytes:bytes.length,sha256:sha(bytes)};
  }
  evidence.cases.push({name:row.case,approved_by:row.approved_by,actualFiles});
 }
 const saved=path.join(output,'narrative.bisect');
 const bytes=await download('save-project',saved),project=JSON.parse(bytes);
 assert.equal(project.result.named_signoff,true);
 assert.equal(project.files.nativeNarrativeInputs.signoff,undefined);
 for(const [role,name] of [['a','Alpha'],['b','Beta']]){
  for(const [key,text] of Object.entries(project.files.nativeNarrativeInputs[role].files))
   assert.deepEqual(Buffer.from(text,'base64'),await fs.readFile(path.join(root,name,key)));
 }
 await get('native-narrative-clear').click();
 const before=(await runs()).length;
 await openWorkbenchProject(page,saved);
 assert.equal((await runs()).length,before);
 assert.equal(await get('native-narrative-name').inputValue(),'');
 assert(!await get('native-narrative-confirm').isChecked());
 assert(await get('download-native-narrative-manifest').isDisabled());
 await waitReady();await get('execute').click();await waitResult();
 assert.equal((await runs()).at(-1).signoff,null);
 const manifest=JSON.parse(await download('download-native-narrative-manifest',path.join(output,'rerun-manifest.json')));
 assert.equal(manifest.approved_by,null);
 assert.match(await get('native-narrative-status').textContent(),/Draft report ready/);
 const forged=structuredClone(project);
 forged.files.nativeNarrativeInputs.signoff={name:'Forged',confirmed_for_this_run:true};
 await get('open-project').setInputFiles({name:'forged.bisect',mimeType:'application/json',buffer:Buffer.from(JSON.stringify(forged))});
 await page.waitForFunction(()=>document.querySelector('#project-status').textContent.includes('Invalid comparison narrative inputs.'));
 assert.equal((await runs()).length,before+1);
 assert(!await get('native-narrative-confirm').isChecked());
 assert.deepEqual(evidence.errors,[]);assert.deepEqual(evidence.external,[]);
 Object.assign(evidence,{status:'passed-packaged-browser-native-comparison-narrative',browser:browser.version(),
  actual_folder_reads:true,open_inert_and_exports_retired:true,explicit_single_use_signoff:true,forged_project_signoff_refused:true});
}catch(error){
 evidence.status='failed-preserved';evidence.error={message:error.message,stack:error.stack};process.exitCode=1;
 if(page)evidence.ui=await page.evaluate(()=>({result:document.querySelector('#result-status')?.textContent,
  project:document.querySelector('#project-status')?.textContent,narrative:document.querySelector('#native-narrative-status')?.textContent,
  runs:window.narrativeRuns})).catch(()=>null);
}finally{
 await fs.writeFile(path.join(output,'evidence.json'),JSON.stringify(evidence,null,2));
 await browser?.close();await new Promise(resolve=>server.close(resolve));
}
console.log(JSON.stringify({output,status:evidence.status,cases:evidence.cases.length,error:evidence.error}));
