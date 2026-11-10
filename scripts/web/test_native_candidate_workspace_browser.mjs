import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import {openWorkbenchProject} from './workbench_project_open.mjs';
import {spawnSync} from 'node:child_process';

const arg=name=>{const i=process.argv.indexOf(name);assert(i>=0&&process.argv[i+1],'Supply '+name);return path.resolve(process.argv[i+1]);};
const site=await fs.realpath(arg('--site')),cli=await fs.realpath(arg('--cli')),parent=arg('--output');
await fs.mkdir(parent,{recursive:true});
const out=await fs.mkdtemp(path.join(parent,'native-candidate-workspace-')),base=path.join(out,'native');
await fs.mkdir(base);
const fixture={
 'annotations.csv':Buffer.from('candidate_name,party,race,curator,curator_credentials,curator_attestation_date,source,independently_verified,attestation_doc_path,attestation_doc_format\r\n"Adams, J",DEM,Black,curator1,PhD,2026-04-15,public,true,letter.pdf,pdf\r\n"Adams, J",DEM,white,curator2,MS,2026-04-16,public,false,other.txt,txt\r\n'),
 'letter.pdf':Buffer.from([0xff,0,0x25,80,68,70]),'other.txt':Buffer.from('Other document\n')};
const nativeCalls={};
for(const [name,version] of [['create','v7'],['replace-preserves-unrelated','v1'],['directory-refusal','v1']]){
 const dir=path.join(base,name),destination=path.join(dir,'outputs',version,'civic_inputs','candidate_workspace');
 await fs.mkdir(dir);for(const [name,data]of Object.entries(fixture))await fs.writeFile(path.join(dir,name),data);
 if(name==='replace-preserves-unrelated'){
  await fs.mkdir(destination,{recursive:true});
  for(const f of ['candidate_race.csv','letter.pdf','annotations.json','manifest.json','provenance.json','extra.txt'])await fs.writeFile(path.join(destination,f),'previous '+f);
 }
 if(name==='directory-refusal')await fs.mkdir(path.join(destination,'manifest.json'),{recursive:true});
 const argv=['civic','add-candidate-race','annotations.csv','--year','2010','--state','Ri','--submitter','Recorded curator','--label','candidate_workspace','--attestation-doc','letter.pdf','--output-base','outputs','--version',version];
 const p=spawnSync(cli,argv,{cwd:dir,windowsHide:true,timeout:30000});assert.ifError(p.error);
 await fs.writeFile(path.join(dir,'stdout.txt'),p.stdout);await fs.writeFile(path.join(dir,'stderr.txt'),p.stderr);
 assert.equal(p.status,name==='directory-refusal'?1:0,p.stderr.toString());nativeCalls[name]={argv,exit:p.status};
 if(name==='directory-refusal'){
  for(const f of ['candidate_race.csv','letter.pdf','annotations.json'])assert((await fs.stat(path.join(destination,f))).isFile());
  await assert.rejects(fs.stat(path.join(destination,'provenance.json')),{code:'ENOENT'});
 }
}
console.log('Candidate workspace evidence:',out);
const {chromium}=createRequire(path.join(process.env.BISECT_PLAYWRIGHT_PACKAGES??process.cwd(),'package.json'))('playwright');
const hash=b=>createHash('sha256').update(b).digest('hex');
const bytes=await fs.readFile(path.join(site,'bisect_wasm.wasm'));
const report={status:'running',native_cli:hash(await fs.readFile(cli)),nativeCalls,wasm:hash(bytes),served:{},errors:[],network:[],serverFailures:[],full_parity:false,deployed:false};
const server=createServer(async(req,res)=>{
 try{
  const rel=decodeURIComponent(new URL(req.url,'http://localhost').pathname),file=await fs.realpath(path.resolve(site,'.'+rel));
  assert(file.startsWith(site+path.sep));const data=await fs.readFile(file);report.served[rel]=hash(data);
  res.setHeader('Content-Type',{'.js':'text/javascript','.html':'text/html','.css':'text/css','.json':'application/json','.wasm':'application/wasm'}[path.extname(file)]??'application/octet-stream');res.end(data);
 }catch(e){if(!['/offline-manifest.json','/favicon.ico'].includes(req.url))report.serverFailures.push([req.url,e.message]);res.writeHead(404);res.end();}
});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const origin='http://127.0.0.1:'+server.address().port;
let browser;
try{
 browser=await chromium.launch({headless:true,executablePath:process.env.BISECT_CHROME_PATH??'C:/Program Files/Google/Chrome/Application/chrome.exe'});
 report.browser=browser.version();const ctx=await browser.newContext({acceptDownloads:true});
 await ctx.route('**/*',r=>{if(new URL(r.request().url()).origin===origin)return r.continue();report.network.push(r.request().url());return r.abort();});
 await ctx.addInitScript(()=>{
  window.picks=0;window.candidateRequests=0;window.targetFolder='created';
  window.showDirectoryPicker=async()=>{window.picks++;return (await navigator.storage.getDirectory()).getDirectoryHandle(window.targetFolder,{create:true});};
  const W=window.Worker;window.Worker=class extends W{postMessage(m,...rest){if(m?.request?.operation==='ingest-candidate-race')window.candidateRequests++;return super.postMessage(m,...rest);}};
  const original=FileSystemFileHandle.prototype.createWritable;
  FileSystemFileHandle.prototype.createWritable=function(...args){if(window.failManifest&&this.name==='manifest.json')throw new Error('Injected manifest write interruption');return original.apply(this,args);};
 });
 const page=await ctx.newPage();page.setDefaultTimeout(30000);page.on('pageerror',e=>report.errors.push(e.message));
 const g=id=>page.locator('#'+id),c=k=>g('candidate-'+k),w=k=>g('candidate-workspace-'+k);
 const state=()=>page.evaluate(()=>({picks:window.picks,requests:window.candidateRequests}));
 const wait=(k,phrase)=>page.waitForFunction(({k,phrase})=>document.getElementById('candidate-workspace-'+k)?.textContent.includes(phrase),{k,phrase});
 const enabled=k=>page.waitForFunction(k=>!document.getElementById('candidate-workspace-'+k)?.disabled,k);
 const field=async(locator,value)=>{await locator.fill(value);await locator.dispatchEvent('change');};
 async function folder(name){await page.evaluate(n=>window.targetFolder=n,name);}
 async function write(folder,version,name,bytes){await page.evaluate(async({folder,version,name,bytes})=>{
  let p=await navigator.storage.getDirectory();for(const n of [folder,version,'civic_inputs','candidate_workspace'])p=await p.getDirectoryHandle(n,{create:true});
  const h=await p.getFileHandle(name,{create:true}),w=await h.createWritable();await w.write(Uint8Array.from(bytes));await w.close();
 },{folder,version,name,bytes:[...Buffer.from(bytes)]});}
 async function makeDirectory(folder,version,name){await page.evaluate(async({folder,version,name})=>{
  let p=await navigator.storage.getDirectory();for(const n of [folder,version,'civic_inputs','candidate_workspace',name])p=await p.getDirectoryHandle(n,{create:true});
 },{folder,version,name});}
 async function read(folder){return page.evaluate(async folder=>{
  const root=await navigator.storage.getDirectory();let p;
  try{p=await root.getDirectoryHandle(folder);}catch(e){if(e.name==='NotFoundError')return {};throw e;}
  const files={};async function walk(d,prefix=''){for await(const [n,h]of d.entries())if(h.kind==='directory')await walk(h,prefix+n+'/');else files[prefix+n]=[...new Uint8Array(await(await h.getFile()).arrayBuffer())];}await walk(p);return files;
 },folder);}
 async function download(locator,file){const result=page.waitForEvent('download');await locator.click();await(await result).saveAs(path.join(out,file));return JSON.parse(await fs.readFile(path.join(out,file)));}
 async function review(phrase){await w('review').click();await wait('summary',phrase);}
 async function apply(phrase){await w('confirm').check();await w('apply').click();await wait('summary',phrase);}
 const retained='2026-10-07T12:00:00Z';
 await page.goto(origin+'/index.html');await page.waitForFunction(x=>document.querySelector('#runtime-artifact')?.textContent.includes(x),report.wasm);
 await g('operation').selectOption('ingest-candidate-race');assert(await w('review').isDisabled());
 for(const [k,v] of Object.entries({state:'Ri',year:'2010',label:'candidate_workspace',submitter:'Recorded curator',timestamp:retained}))await field(c(k),v);
 const inputs=path.join(base,'create');await c('csv').setInputFiles(path.join(inputs,'annotations.csv'));
 await page.waitForFunction(()=>document.getElementById('candidate-source-name').textContent==='annotations.csv');
 await c('documents').setInputFiles([path.join(inputs,'letter.pdf'),path.join(inputs,'other.txt')]);
 await page.waitForFunction(()=>document.getElementById('candidate-stage').options.length===2);await c('stage').selectOption('letter.pdf');
 await g('execute').click();await enabled('review');assert.deepEqual(await state(),{picks:0,requests:1});
 await download(g('save-project'),'package.bisect');
 await field(w('version'),'v7');await review('create manifest.json');assert(await w('apply').isDisabled());assert.deepEqual(await read('created'),{});
 await openWorkbenchProject(page,path.join(out,'package.bisect'));assert(await w('review').isDisabled());assert(await w('apply').isDisabled());
 assert.deepEqual(await state(),{picks:1,requests:2});assert.deepEqual(await read('created'),{});
 await g('execute').click();await enabled('review');await review('create manifest.json');await apply('Wrote and checked 5 files');
 const createReceipt=await download(w('receipt'),'create-receipt.json');assert.equal(createReceipt.status,'completed');
 assert(createReceipt.entries.every(x=>x.status==='verified'&&x.action==='create'));
 async function compare(folder,kind,version,{omit=[]}={}){
  const result=await read(folder),prefix=version+'/civic_inputs/candidate_workspace/';
  const native=path.join(base,kind,'outputs',prefix);
  const files=(await fs.readdir(native,{withFileTypes:true})).filter(f=>f.isFile()&&!omit.includes(f.name));
  assert.deepEqual(Object.keys(result).sort(),files.map(f=>prefix+f.name).sort());
  const hashes={};
  for(const {name}of files){let want=await fs.readFile(path.join(native,name));const got=Buffer.from(result[prefix+name]);
   if(name==='manifest.json'){const m=JSON.parse(want);want=Buffer.from(want.toString().replace(`"submitted_at": "${m.submitted_at}"`,`"submitted_at": "${retained}"`).replace(`"ingested_at": "${m.ingested_at}"`,`"ingested_at": "${retained}"`));}
   assert.deepEqual(got,want,name);hashes[name]=hash(got);await fs.writeFile(path.join(out,folder+'-'+name),got);
  }return hashes;
 }
 report.created=await compare('created','create','v7');
 await field(w('version'),'v1');await folder('existing');
 for(const name of ['candidate_race.csv','letter.pdf','annotations.json','manifest.json','provenance.json','extra.txt'])await write('existing','v1',name,'previous '+name);
 const existing=await read('existing');await review('Enable replacement');assert.deepEqual(await read('existing'),existing);
 await w('replace').check();await review('replace manifest.json');
 await write('existing','v1','letter.pdf','changed externally');const changed=await read('existing');await apply('destination changed after review');
 assert.deepEqual(await read('existing'),changed);const stale=await download(w('receipt'),'stale-receipt.json');assert.equal(stale.status,'refused');assert.equal(stale.effects_may_have_started,false);
 await review('replace manifest.json');await apply('Wrote and checked 5 files');report.replaced=await compare('existing','replace-preserves-unrelated','v1');
 await folder('blocked');await makeDirectory('blocked','v1','manifest.json');
 await write('blocked','v1','provenance.json','untouched');const blocked=await read('blocked');
 await review('destination is a directory');assert(await w('apply').isDisabled());assert.deepEqual(await read('blocked'),blocked);
 report.knownDirectoryFailurePreflightNoWrite=true;
 await folder('case-alias');await write('case-alias','v1','Manifest.JSON','case-sensitive alias');
 const alias=await read('case-alias');await review('Case-colliding workspace entry');assert(await w('apply').isDisabled());assert.deepEqual(await read('case-alias'),alias);
 report.caseAliasRefused=true;
 await folder('interrupted');await write('interrupted','v1','provenance.json','previous provenance');
 await review('replace provenance.json');await page.evaluate(()=>window.failManifest=true);await apply('Injected manifest write interruption');
 const interrupted=await download(w('receipt'),'interrupted-receipt.json');assert.equal(interrupted.status,'interrupted');
 assert.deepEqual(interrupted.entries.map(r=>r.status),['verified','verified','verified','started','pending']);
 const partial=await read('interrupted');assert.deepEqual(partial['v1/civic_inputs/candidate_workspace/manifest.json'],[]);
 assert.equal(Buffer.from(partial['v1/civic_inputs/candidate_workspace/provenance.json']).toString(),'previous provenance');
 for(const f of ['candidate_race.csv','letter.pdf','annotations.json'])assert.equal(hash(Buffer.from(partial['v1/civic_inputs/candidate_workspace/'+f])),report.created[f]);
 await fs.writeFile(path.join(out,'interrupted-files.json'),JSON.stringify(partial));
 await page.setViewportSize({width:390,height:840});
 assert(await page.evaluate(()=>document.documentElement.scrollWidth<=390));
 assert.deepEqual(report.errors,[]);assert.deepEqual(report.network,[]);assert.deepEqual(report.serverFailures,[]);
 report.opfs=true;report.actual_os_picker=false;report.counts=await state();report.status='passed-candidate-workspace-actual-opfs';
}catch(e){report.status='failed';report.failure=e.stack??String(e);throw e;}
finally{await fs.writeFile(path.join(out,'evidence.json'),JSON.stringify(report,null,2));await browser?.close();await new Promise(resolve=>server.close(resolve));}
