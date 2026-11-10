import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import {spawnSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {openWorkbenchProject} from './workbench_project_open.mjs';

const arg=name=>{const i=process.argv.indexOf(name);assert(i>=0&&process.argv[i+1],'Supply '+name);return path.resolve(process.argv[i+1]);};
const site=await fs.realpath(arg('--site')),cli=await fs.realpath(arg('--cli')),parent=arg('--output');
const sha=b=>createHash('sha256').update(b).digest('hex');
await fs.mkdir(parent,{recursive:true});
const output=await fs.mkdtemp(path.join(parent,'native-civic-conflicts-'));
const h='geoid,comment_id,label,source,source_url,confidence,submitted_at,extra\n';
const specs={
 alpha:h+'01001000100,shared,Harbor,public,,high,recorded,A\n01001000100,other,Harbor,public,,low,recorded,A\nbad,invalid-a,Harbor,public,,low,recorded,A\n',
 beta:h+'01001000100,shared,Hill,public,,high,recorded,B\nbad,invalid-b,Hill,public,,low,recorded,B\n',
};
const native={sha256:sha(await fs.readFile(cli)),calls:{}};
async function invoke(name,argv,expected=0){
 const args=['civic',...argv,'--output-base','outputs'];
 const r=spawnSync(cli,args,{cwd:output,windowsHide:true,timeout:30000});
 assert.ifError(r.error);
 await fs.writeFile(path.join(output,name+'.stdout'),r.stdout);
 await fs.writeFile(path.join(output,name+'.stderr'),r.stderr);
 native.calls[name]={argv:args,exit:r.status,stdout_sha256:sha(r.stdout),stderr_sha256:sha(r.stderr)};
 assert.equal(r.status,expected,r.stderr.toString('utf8'));
 return r;
}
const tree=path.join(output,'outputs','v1','civic_inputs');
for(const [label,text] of Object.entries(specs)){
 const file=label+'.csv';
 await fs.writeFile(path.join(output,file),text);
 await invoke('ingest-'+label,['ingest',file,'--label',label,'--state','AL','--year','2020','--validate','advisory']);
 const manifest=JSON.parse(await fs.readFile(path.join(tree,label,'manifest.json')));
 assert.equal(manifest.validation.errors,1);
}
const [a,b]=await Promise.all(['alpha','beta'].map(label=>fs.readFile(path.join(tree,label,'normalized.csv'))));
const want={};
for(const name of ['alpha-beta','alpha-alpha','beta-alpha']){
 const [first,second]=name.split('-');
 want[name]=(await invoke(name,['conflicts','--label',first,'--label',second])).stdout;
}
assert.deepEqual(JSON.parse(want['alpha-beta']).coi_overlap.map(x=>x.geoid),['01001000100','bad']);
assert.match((await invoke('single',['conflicts','--label','alpha'],1)).stderr.toString(),/requires --label twice/);
const wasm=await fs.readFile(path.join(site,'bisect_wasm.wasm'));
const {instantiateEngine}=await import(pathToFileURL(path.join(site,'wasm-engine.js')).href);
const profile=(await instantiateEngine(wasm)).supportsCMetis?'c':'core';
const build={profiles:{[profile]:{site,image:{sha256:sha(wasm)}}}};
const {chromium}=createRequire(path.join(process.env.BISECT_PLAYWRIGHT_PACKAGES??process.cwd(),'package.json'))('playwright');
const report={status:'running',native,profiles:{},full_parity:false,deployed:false};
let browser;
try{
 browser=await chromium.launch({headless:true,executablePath:process.env.BISECT_CHROME_PATH??'C:/Program Files/Google/Chrome/Application/chrome.exe'});
 for(const [profile,entry] of Object.entries(build.profiles)){
  assert.equal(site,entry.site);
  const p=path.join(output,profile);await fs.mkdir(p);
  const evidence=report.profiles[profile]={wasm_sha256:entry.image.sha256,served:{},errors:[],external:[],failures:[]};
  assert.equal(sha(await fs.readFile(path.join(site,'bisect_wasm.wasm'))),entry.image.sha256);
  const server=createServer(async(req,res)=>{
   try{
    const name=decodeURIComponent(new URL(req.url,'http://localhost').pathname);
    const file=await fs.realpath(path.resolve(site,'.'+name));assert(file.startsWith(site+path.sep));
    const bytes=await fs.readFile(file),hash=sha(bytes);
    if(name==='/bisect_wasm.wasm')assert.equal(hash,entry.image.sha256);
    evidence.served[name]=hash;
    const types={'.js':'text/javascript','.mjs':'text/javascript','.css':'text/css','.json':'application/json','.html':'text/html','.wasm':'application/wasm'};
    res.setHeader('Content-Type',types[path.extname(name)]??'application/octet-stream');res.end(bytes);
   }catch(error){if(!['/offline-manifest.json','/favicon.ico'].includes(req.url))evidence.failures.push([req.url,error.message]);res.writeHead(404);res.end();}
  });
  await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
  const origin='http://127.0.0.1:'+server.address().port;
  const context=await browser.newContext({acceptDownloads:true});
  try{
   await context.route('**/*',r=>{
    if(new URL(r.request().url()).origin===origin)return r.continue();
    evidence.external.push(r.request().url());return r.abort();
   });
   await context.addInitScript(()=>{
    window.civicRuns=0;
    const W=window.Worker;window.Worker=class extends W{postMessage(m,...args){
     if(m?.request?.operation==='inspect-native-civic-conflicts')window.civicRuns++;
     return super.postMessage(m,...args);
    }};
    const read=File.prototype.arrayBuffer;
    File.prototype.arrayBuffer=async function(...args){
     if(this.name==='held.csv')await new Promise(resolve=>window.releaseHeldCivic=resolve);
     return read.apply(this,args);
    };
   });
   const page=await context.newPage();page.setDefaultTimeout(60000);
   page.on('pageerror',e=>evidence.errors.push(e.message));
   const get=id=>page.locator('#'+id),control=k=>get('native-civic-conflict-'+k);
   const roster=control('roster').locator('li');
   const runs=()=>page.evaluate(()=>window.civicRuns);
   const waitRows=count=>page.waitForFunction(count=>document.querySelector('#native-civic-conflict-roster').children.length===count,count);
   const ready=()=>page.waitForFunction(()=>!document.querySelector('#native-civic-conflict-download').disabled);
   const save=async (loc,name)=>{
    const event=page.waitForEvent('download');await loc.click();
    const file=path.join(p,name);await(await event).saveAs(file);return fs.readFile(file);
   };
   const select=async(label,bytes,name='normalized.csv')=>{
    await control('label').fill(label);
    await control('file').setInputFiles({name,mimeType:'text/csv',buffer:bytes});
   };
   await page.goto(origin+'/index.html');
   await page.waitForFunction(x=>document.querySelector('#runtime-artifact')?.textContent.includes(x),entry.image.sha256);
   await get('operation').selectOption('inspect-native-civic-conflicts');
   assert(await control('download').isDisabled());
   await select('alpha',a);await waitRows(1);assert.equal(await runs(),0);
   await get('execute').click();
   await page.waitForFunction(()=>document.querySelector('#result-status')?.textContent.includes('at least two'));
   assert.equal(await runs(),0);
   await select('beta',b);await waitRows(2);assert.equal(await runs(),0);
   await get('execute').click();await ready();assert.equal(await runs(),1);
   assert.deepEqual(await save(control('download'),'alpha-beta.json'),want['alpha-beta']);
   const project=await save(get('save-project'),'conflicts.bisect');
   const saved=JSON.parse(project);
   assert.equal(saved.operation,'inspect-native-civic-conflicts');
   assert.deepEqual(saved.files.nativeCivicConflictInputs.sources.map(x=>x.label),['alpha','beta']);
   assert.deepEqual(saved.files.nativeCivicConflictInputs.sources.map(x=>Buffer.from(x.source.base64,'base64')),[a,b]);
   await select('<img src=x>',a,'held.csv');
   await page.waitForFunction(()=>typeof window.releaseHeldCivic==='function');
   assert(await get('execute').isDisabled());assert(await control('download').isDisabled());
   await openWorkbenchProject(page,path.join(p,'conflicts.bisect'));
   await page.evaluate(()=>window.releaseHeldCivic());
   await waitRows(2);assert.equal(await runs(),1);assert(await control('download').isDisabled());
   assert.deepEqual(await roster.allTextContents(),['alpha — normalized.csv','beta — normalized.csv']);
   await get('execute').click();await ready();assert.equal(await runs(),2);
   assert.deepEqual(await save(control('download'),'after-open.json'),want['alpha-beta']);
   await control('remove').click();await waitRows(1);
   await control('repeat').click();await waitRows(2);
   await get('execute').click();await ready();assert.equal(await runs(),3);
   assert.deepEqual(await save(control('download'),'alpha-alpha.json'),want['alpha-alpha']);
   await control('clear').click();await waitRows(0);
   await select('beta',b);await waitRows(1);
   await select('alpha',a);await waitRows(2);
   await get('execute').click();await ready();assert.equal(await runs(),4);
   assert.deepEqual(await save(control('download'),'beta-alpha.json'),want['beta-alpha']);
   await select('alpha',b);
   await page.waitForFunction(()=>document.querySelector('#result-status')?.textContent.includes('identical normalized CSV'));
   assert.equal(await roster.count(),2);assert.equal(await runs(),4);assert(await control('download').isDisabled());
   const claimed=structuredClone(saved);
   claimed.result.conflicts_json='{"unverified":true}';
   const claimedPath=path.join(p,'claimed-result.bisect');
   await fs.writeFile(claimedPath,JSON.stringify(claimed));
   await openWorkbenchProject(page,claimedPath);
   assert.equal(await runs(),4);assert(await control('download').isDisabled());
   await get('execute').click();await ready();assert.equal(await runs(),5);
   assert.deepEqual(await save(control('download'),'after-claimed-result.json'),want['alpha-beta']);
   assert.deepEqual(evidence.errors,[]);assert.deepEqual(evidence.external,[]);assert.deepEqual(evidence.failures,[]);
   Object.assign(evidence,{browser:browser.version(),downloads:5,imported_claim_never_downloadable:true,exact_native_bytes:true,open_inert:true,held_file_retired:true,source_bytes_exact:true,repeat_and_reverse_exact:true,invalid_repeat_refused:true});
  }finally{await context.close();await new Promise(resolve=>server.close(resolve));}
 }
 report.status='passed-native-civic-conflicts-packaged-browser';
}catch(e){report.status='failed-preserved';report.error=e.stack;process.exitCode=1;}
finally{await browser?.close();await fs.writeFile(path.join(output,'evidence.json'),JSON.stringify(report,null,2)+'\n',{flag:'wx'});}
console.log(JSON.stringify({status:report.status,output,profile,error:report.error}));
