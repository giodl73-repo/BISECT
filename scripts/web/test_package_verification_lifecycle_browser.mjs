import fs from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';

const argv=process.argv.slice(2),argument=name=>{const i=argv.indexOf(name);assert(i>=0&&argv[i+1],'Supply '+name);return path.resolve(argv[i+1]);};
const workspace=argv.includes('--source-root')?argument('--source-root'):process.cwd();
const site=await fs.realpath(argument('--site')),parent=argument('--output');await fs.mkdir(parent,{recursive:true});
const out=await fs.mkdtemp(path.join(parent,'package-verification-'));
const hash=b=>createHash('sha256').update(b).digest('hex');
const {chromium}=createRequire(path.resolve(process.env.BISECT_PLAYWRIGHT_PACKAGES??process.cwd(),'package.json'))('playwright');
const native=argument('--native-rhist'),nativeCount=argument('--native-rcount');
const nativeBytes=await fs.readFile(native), wasmBytes=await fs.readFile(path.join(site,'bisect_wasm.wasm'));
const served=new Map(),external=[],errors=[],cases=[];
let browser,page;
const server=createServer(async(req,res)=>{try{
 if(req.method!=='GET'){res.writeHead(405);res.end();return;}
 const pathname=decodeURIComponent(new URL(req.url,'http://localhost').pathname);
 const file=await fs.realpath(path.resolve(site,'.'+(pathname==='/'?'/index.html':pathname)));
 assert(file.startsWith(site+path.sep));const servedFile=file;const bytes=await fs.readFile(servedFile);
 served.set(servedFile,{path:servedFile,sha256:hash(bytes),bytes:bytes.length});
 res.setHeader('Cache-Control','no-store');res.setHeader('Content-Type',file.endsWith('.js')?'application/javascript':file.endsWith('.wasm')?'application/wasm':file.endsWith('.css')?'text/css':file.endsWith('.json')?'application/json':'text/html');res.end(bytes);
}catch{res.writeHead(404);res.end();}});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));const origin='http://127.0.0.1:'+server.address().port;
async function collect(dir,prefix=''){
 const files={};for(const entry of await fs.readdir(dir,{withFileTypes:true})){
  const name=prefix+entry.name,p=path.join(dir,entry.name);
  if(entry.isDirectory())Object.assign(files,await collect(p,name+'/'));else files[name]=(await fs.readFile(p)).toString('base64');
 }return files;
}
async function selectFolder(folder){
 await page.evaluate(()=>{const input=document.querySelector('#package-folder'),original=input.onchange;window.historySelectionComplete=false;input.onchange=async function(...args){try{return await original.apply(this,args);}finally{input.onchange=original;window.historySelectionComplete=true;}};});
 await page.locator('#package-folder').setInputFiles(folder);await page.waitForFunction(()=>window.historySelectionComplete);
}
async function downloadProject(label){
 const wait=page.waitForEvent('download');await page.locator('#save-project').click();const item=await wait;
 const file=path.join(out,label+'.bisect');await item.saveAs(file);await fs.writeFile(path.join(out,label+'-worker-requests.json'),JSON.stringify(await page.evaluate(()=>historyRequests),null,2));return {file,project:JSON.parse(await fs.readFile(file,'utf8'))};
}
try{
 // Original hostile mutations from test_history_wasm.mjs; original fixtures remain unchanged.
const originalHistory=['l0-rename','l1-split-merge','l2-three-cycle','real-ri-tract-unchanged','l0-missing-unit','l1-bad-weights'];
const fixtureSources=[];
for(const name of originalHistory){const folder=path.join(workspace,'docs/fixtures/rhist',name);for(const [key,encoded]of Object.entries(await collect(folder))){const bytes=Buffer.from(encoded,'base64');fixtureSources.push({path:path.join(folder,key),sha256:hash(bytes),bytes:bytes.length});}}
const hostileRoot=path.join(out,'hostile-fixtures');await fs.mkdir(hostileRoot);
for(const kind of ['source-tamper','missing-source','hash-drift','malformed-json','invalid-utf8','missing-manifest','unsafe-path']){
 const folder=path.join(hostileRoot,kind);await fs.cp(path.join(workspace,'docs/fixtures/rhist/l0-rename'),folder,{recursive:true,errorOnExist:true});
 const source=JSON.parse(await fs.readFile(path.join(folder,'sources/source-index.json'),'utf8'))[0].path;
 if(kind==='source-tamper'){const p=path.join(folder,source),bytes=await fs.readFile(p);bytes[0]^=1;await fs.writeFile(p,bytes);}
 if(kind==='missing-source')await fs.unlink(path.join(folder,source));
 if(kind==='hash-drift'){const p=path.join(folder,'manifest.json'),value=JSON.parse(await fs.readFile(p,'utf8'));value.package_content_hash='sha256:'+'f'.repeat(64);await fs.writeFile(p,JSON.stringify(value));}
 if(kind==='malformed-json')await fs.writeFile(path.join(folder,'manifest.json'),Buffer.from([123]));
 if(kind==='invalid-utf8')await fs.writeFile(path.join(folder,'units/cycles.ndjson'),Buffer.from([255]));
 if(kind==='missing-manifest')await fs.unlink(path.join(folder,'manifest.json'));
 if(kind==='unsafe-path'){const p=path.join(folder,'sources/source-index.json'),value=JSON.parse(await fs.readFile(p,'utf8'));value[0].path='sources/..\\private';await fs.writeFile(p,JSON.stringify(value));}
}

 browser=await chromium.launch({headless:true,executablePath:process.env.BISECT_TEST_BROWSER||(process.platform==='win32'?'C:/Program Files/Google/Chrome/Application/chrome.exe':undefined),args:['--no-first-run','--disable-background-networking','--disable-extensions']});
 const context=await browser.newContext({acceptDownloads:true});
 await context.route('**/*',route=>new URL(route.request().url()).origin===origin?route.continue():(external.push(route.request().url()),route.abort()));
 await context.addInitScript(()=>{
  localStorage.clear();window.historyRequests=[];const W=Worker;
  window.Worker=class extends W{postMessage(value,...rest){if(value.type==='run')historyRequests.push(structuredClone(value.request));return super.postMessage(value,...rest);}};
 });
 for(const name of ['l0-rename','l1-split-merge','l2-three-cycle','real-ri-tract-unchanged','l0-missing-unit','l1-bad-weights','hostile-source-tamper','hostile-missing-source','hostile-hash-drift','hostile-malformed-json','hostile-invalid-utf8','hostile-missing-manifest','hostile-unsafe-path']){
  const folder=name.startsWith('hostile-')?path.join(out,'hostile-fixtures',name.slice(8)):path.join(workspace,'docs/fixtures/rhist',name),files=await collect(folder);
  const cli=spawnSync(native,['verify',folder,'--format','json'],{encoding:'utf8',maxBuffer:16*1024*1024});
  await fs.writeFile(path.join(out,name+'-native.json'),JSON.stringify({status:cli.status,stdout:cli.stdout,stderr:cli.stderr},null,2));
  assert([0,1].includes(cli.status),cli.stderr);const reference=JSON.parse(cli.stdout);assert.equal(cli.status,reference.status==='pass'?0:1);
  page=await context.newPage();page.setDefaultTimeout(45000);page.on('pageerror',e=>errors.push(e.message));
  await page.goto(origin);await page.waitForFunction(()=>!document.querySelector('#execute').disabled);
  await page.locator('#operation').selectOption('verify-history-files');
  await selectFolder(folder);
  if(name==='hostile-missing-manifest'){
   const status=await page.locator('#result-status').innerText();assert.match(status,/one package folder containing one manifest\.json/);
   assert.equal(reference.status,'fail');assert.equal(await page.evaluate(()=>historyRequests.length),0);assert(await page.locator('#download').isDisabled());
   await page.locator('#result-status').screenshot({path:path.join(out,name+'-status.png')});
   cases.push({name,status:'browser-preflight-refusal',native_status:reference.status,native_exit:cli.status,no_worker_request:true,no_result_download:true,exact_native_transcript:false});
   await page.close();page=undefined;continue;
  }
  assert((await page.locator('#input-status').innerText()).startsWith('Loaded '));
  assert.equal(await page.evaluate(()=>historyRequests.length),0);
  await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#execute').disabled&&!document.querySelector('#download').disabled);
  const first=await downloadProject(name+'-run');assert.deepEqual(first.project.result,reference);
  assert.deepEqual(first.project.files.packageFiles,files);assert.equal(first.project.lastOperation,'verify-history-files');
  const requests=await page.evaluate(()=>historyRequests);assert.equal(requests.length,1);assert.equal(requests[0].operation,'verify-history-files');
  assert.deepEqual(requests[0].files,Object.fromEntries(Object.entries(files).map(([k,v])=>[k,[...Buffer.from(v,'base64')]])));
  await page.reload();await page.waitForFunction(()=>!document.querySelector('#execute').disabled);
  await page.locator('#open-project').setInputFiles(first.file);
  await page.waitForFunction(()=>document.querySelector('#project-status').textContent.includes('Opened'));
  assert.equal(await page.evaluate(()=>historyRequests.length),0);
  const reopened=await downloadProject(name+'-reopened');assert.deepEqual(reopened.project.files.packageFiles,files);assert.deepEqual(reopened.project.result,reference);
  await page.setViewportSize({width:390,height:844});assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1));
  await page.locator('#result-status').screenshot({path:path.join(out,name+'-status.png')});
  cases.push({name,status:reference.status,native_exit:cli.status,files:Object.keys(files).length,exact_native_transcript:true,exact_source_bytes:true,actual_worker:true,inert_open:true,viewport_width:390});
  await page.close();page=undefined;
 }
 await fs.writeFile(path.join(out,'completed-cases.json'),JSON.stringify({cases},null,2));
 page=await context.newPage();page.setDefaultTimeout(45000);page.on('pageerror',e=>errors.push(e.message));
 await page.goto(origin);await page.waitForFunction(()=>!document.querySelector('#execute').disabled);
 await page.locator('#operation').selectOption('verify-history-files');
 await selectFolder(path.join(workspace,'docs/fixtures/rhist/l0-rename'));
 await page.waitForFunction(()=>document.querySelector('#input-status').textContent.startsWith('Loaded '));
 await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#execute').disabled&&!document.querySelector('#download').disabled);
 const priorStatus=await page.locator('#result-status').innerText();
 await selectFolder(path.join(out,'hostile-fixtures/source-tamper'));
 await page.waitForFunction(()=>document.querySelector('#input-status').textContent.startsWith('Loaded '));
 const changedStatus=await page.locator('#result-status').innerText(),downloadDisabled=await page.locator('#download').isDisabled();
 const afterEdit=await downloadProject('changed-source-after-prior-pass');
 assert.deepEqual(afterEdit.project.files.packageFiles,await collect(path.join(out,'hostile-fixtures/source-tamper')));
 const changed={priorStatus,changedStatus,downloadDisabled,requests:await page.evaluate(()=>historyRequests.length),retained_result:afterEdit.project.result};
 await fs.writeFile(path.join(out,'source-edit-retirement.json'),JSON.stringify(changed,null,2));
 assert.equal(changed.requests,1,'Source selection must stay inert');
 assert(downloadDisabled||/unverified|changed|stale|run again|retained/i.test(changedStatus),'Changed package must retire or visibly qualify the prior fresh result');
 await page.close();page=undefined;
 const golden=path.join(workspace,'docs/examples/rcount-golden-packages/summary-basic'),bad=path.join(workspace,'docs/examples/rcount-golden-packages/bad-lineage');
 const goodFiles=await collect(golden),badFiles=await collect(bad),demoFiles=JSON.parse(await fs.readFile(path.join(site,'fixtures/count-package.json'),'utf8'));
 const archive=path.join(out,'bad-lineage.rcount-files.json');await fs.writeFile(archive,JSON.stringify({schema_version:'bisect-rcount-files-v1',files:badFiles}));
 let nativeIndex=0;
 async function verifyNative(files,label){
  const folder=path.join(out,'native-'+nativeIndex+++'-'+label);await fs.mkdir(folder);
  for(const [name,encoded]of Object.entries(files)){const f=path.resolve(folder,name);assert(f.startsWith(folder+path.sep));await fs.mkdir(path.dirname(f),{recursive:true});await fs.writeFile(f,Buffer.from(encoded,'base64'));}
  const cli=spawnSync(nativeCount,['verify',folder,'--format','json'],{encoding:'utf8',maxBuffer:16*1024*1024});assert([0,1].includes(cli.status),cli.stderr);
  await fs.writeFile(path.join(out,label+'-native.json'),JSON.stringify({status:cli.status,stdout:cli.stdout,stderr:cli.stderr},null,2));const result=JSON.parse(cli.stdout);assert.equal(cli.status,result.status==='pass'?0:1);return result;
 }
 async function newPage(){page=await context.newPage();page.setDefaultTimeout(45000);page.on('pageerror',e=>errors.push(e.message));await page.goto(origin);await page.waitForFunction(()=>!document.querySelector('#execute').disabled);}
 async function chooseInput(id,files){
  await page.evaluate(id=>{const input=document.getElementById(id),original=input.onchange;window.inputReadComplete=false;input.onchange=async function(...args){try{return await original.apply(this,args);}finally{input.onchange=original;window.inputReadComplete=true;}};},id);
  await page.locator('#'+id).setInputFiles(files);await page.waitForFunction(()=>window.inputReadComplete);
 }
 async function run(label,expected){await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#execute').disabled&&!document.querySelector('#download').disabled);const saved=await downloadProject(label);if(expected)assert.deepEqual(saved.project.result,expected);return saved.project;}
 async function freshCount(label){await newPage();await page.locator('#operation').selectOption('verify-count-files');await selectFolder(golden);const expected=await verifyNative(goodFiles,label+'-baseline');const saved=await run(label+'-baseline',expected);assert.deepEqual(saved.files.packageFiles,goodFiles);assert(!await page.locator('#count-transcript-review').isDisabled());}
 for(const mode of ['folder','archive','demo']){
  await freshCount(mode);const before=await page.evaluate(()=>historyRequests.length);
  if(mode==='folder')await selectFolder(bad);
  if(mode==='archive')await chooseInput('package-archive',archive);
  if(mode==='demo'){await page.locator('#count-demo').click();await page.waitForFunction(()=>document.querySelector('#input-status').textContent.includes('public synthetic election package'));}
  const expectedFiles=mode==='demo'?demoFiles:badFiles,status=await page.locator('#result-status').innerText();assert.match(status,/retained result is unverified/i);
  assert.equal(await page.evaluate(()=>historyRequests.length),before);assert(await page.locator('#count-transcript-review').isDisabled());assert(await page.locator('#count-transcript-apply').isDisabled());
  const retired=(await downloadProject(mode+'-retired')).project;assert.deepEqual(retired.files.packageFiles,expectedFiles);
  const reference=await verifyNative(expectedFiles,mode+'-replacement');const verified=await run(mode+'-reverified',reference);assert.deepEqual(verified.files.packageFiles,expectedFiles);assert(!await page.locator('#count-transcript-review').isDisabled());
  cases.push({mode,selection_inert:true,retired_status:status,install_review_disabled_after_edit:true,exact_selected_bytes:true,explicit_native_reverification:reference.status,install_review_restored_after_explicit_verification:true});await page.close();page=undefined;
 }
 const metadata={country:'US',state:'RI',county:'Synthetic County',date:'2024-11-05',election_type:'general',scope:'county',status:'canvassed'};
 const header='contest_id,contest_title,vote_for,selection_id,selection_label,selection_kind,reporting_unit_id,reporting_unit_kind,parent_jurisdiction,status,votes,undervotes,overvotes,blank_contests,counted_ballots';
 for(const count of ['80','9007199254740993']){
  await newPage();await page.locator('#operation').selectOption('import-statement-csv');for(const [key,value]of Object.entries(metadata))await page.locator('#import-'+key).fill(value);
  const bytes=Buffer.from(header+'\r\n'+`mayor,Synthetic Mayor,1,a,Candidate A,candidate,p1,precinct,county,canvassed,${count},0,0,0,${count}\r\n`+`mayor,Synthetic Mayor,1,a,Candidate A,candidate,total,jurisdiction-total,county,canvassed,${count},0,0,0,${count}\r\n`);
  await chooseInput('election-source',{name:'original-'+count+'.csv',mimeType:'text/csv',buffer:bytes});const imported=await run('import-'+count);assert.equal(imported.result.verification.status,'pass');assert.equal(imported.result.package_files['sources/statement-of-votes.csv'],bytes.toString('base64'));
  const before=await page.evaluate(()=>historyRequests.length);await page.locator('#use-imported-package').click();assert.equal(await page.evaluate(()=>historyRequests.length),before);assert(await page.locator('#count-transcript-review').isDisabled());
  const selected=(await downloadProject('import-'+count+'-handoff')).project;assert.deepEqual(selected.files.packageFiles,imported.result.package_files);
  const reference=await verifyNative(selected.files.packageFiles,'import-'+count+'-verification');const verified=await run('import-'+count+'-verified',reference);assert.deepEqual(verified.files.packageFiles,imported.result.package_files);assert(!await page.locator('#count-transcript-review').isDisabled());
  cases.push({mode:'imported-package',count,exact_original_csv:true,handoff_inert:true,no_count_install_authority_from_import_result:true,explicit_native_reverification:reference.status});await page.close();page=undefined;
 }
 assert.deepEqual(errors,[]);assert.deepEqual(external,[]);
 await fs.writeFile(path.join(out,'evidence.json'),JSON.stringify({status:'passed-package-verification-lifecycle-browser',site,wasm:{sha256:hash(wasmBytes),bytes:wasmBytes.length},native:{path:native,sha256:hash(nativeBytes),fresh_compilation_claimed:false},browser:browser.version(),cases,fixture_sources:fixtureSources,test_source:{path:new URL(import.meta.url).pathname,sha256:hash(await fs.readFile(new URL(import.meta.url)))},native_count:{path:nativeCount,sha256:hash(await fs.readFile(nativeCount)),fresh_compilation_claimed:false},served_assets:[...served.values()],external_requests:external,page_errors:errors,os_picker_tested:false,full_parity:false,final_package_acceptance:false},null,2));
 console.log(JSON.stringify({out,cases:cases.length}));
}catch(error){
 await fs.writeFile(path.join(out,'failure.txt'),error.stack??String(error));
 if(page){await fs.writeFile(path.join(out,'failure-state.json'),JSON.stringify(await page.evaluate(()=>({status:document.querySelector('#result-status')?.textContent,project:document.querySelector('#project-status')?.textContent,input:document.querySelector('#input-status')?.textContent,requests:window.historyRequests})),null,2));await page.screenshot({path:path.join(out,'failure.png'),fullPage:true});}
 throw error;
}finally{await browser?.close();await new Promise(resolve=>server.close(resolve));}
