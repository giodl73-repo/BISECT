import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import {openWorkbenchProject} from './workbench_project_open.mjs';

const arg=key=>{const i=process.argv.indexOf(key);assert(i>=0&&process.argv[i+1],'Supply '+key);return path.resolve(process.argv[i+1]);};
const site=await fs.realpath(arg('--site')),cli=await fs.realpath(arg('--cli')),parent=arg('--output');
const hash=b=>createHash('sha256').update(b).digest('hex');
const {chromium}=createRequire(path.join(process.env.BISECT_PLAYWRIGHT_PACKAGES??process.cwd(),'package.json'))('playwright');
await fs.mkdir(parent,{recursive:true});
const out=await fs.mkdtemp(path.join(parent,'civic-desktop-folder-'));
const tree=path.join(out,'outputs','v1','civic_inputs');
const original=path.join(out,'source.csv');
await fs.writeFile(original,'geoid,comment_id,label,source,source_url,confidence,submitted_at,extra\n01001000100,c1,Port,public,,high,recorded,"a,b"\n');
const native={};
const run=async(name,argv)=>{
 const r=spawnSync(cli,['civic',...argv,'--output-base','outputs'],{cwd:out,windowsHide:true,timeout:30000});
 assert.ifError(r.error);
 await fs.writeFile(path.join(out,name+'.stdout'),r.stdout);await fs.writeFile(path.join(out,name+'.stderr'),r.stderr);
 native[name]={args:['civic',...argv,'--output-base','outputs'],code:r.status,stdout_sha256:hash(r.stdout)};
 assert.equal(r.status,0,r.stderr.toString());return r.stdout;
};
await run('native-ingest',['ingest','source.csv','--label','ordinary','--state','AL','--year','2020']);
const clean=await fs.readFile(path.join(tree,'ordinary','manifest.json'));
const specimens={'ordinary/manifest.json':clean,'malformed/manifest.json':Buffer.from('{'),
 'extra/nested/manifest.json':clean};
for(const [rel,bytes] of Object.entries(specimens)){
 const p=path.join(tree,rel);await fs.mkdir(path.dirname(p),{recursive:true});await fs.writeFile(p,bytes);
}
await fs.writeFile(path.join(tree,'loose.json'),clean);
const list=(await run('native-list',['list'])).toString().replace(/\r?\n$/,'').split(/\r?\n/);
const show=(await run('native-show',['show','--label','ordinary'])).toString().replace(/\r?\n$/,'');
assert.equal(list.length,3);assert.match(list[2],/^ordinary\s/);
const wasm=hash(await fs.readFile(path.join(site,'bisect_wasm.wasm')));
const errors=[],external=[],served={};
const server=createServer(async(req,res)=>{
 try{
  const rel=decodeURIComponent(new URL(req.url,'http://localhost').pathname);
  const file=await fs.realpath(path.resolve(site,'.'+rel));assert(file.startsWith(site+path.sep));
  const data=await fs.readFile(file);
  served[rel]=hash(data);
  const mime={'.js':'text/javascript','.mjs':'text/javascript','.wasm':'application/wasm','.html':'text/html','.css':'text/css','.json':'application/json'};
  res.setHeader('Content-Type',mime[path.extname(rel)]??'application/octet-stream');res.end(data);
 }catch{res.writeHead(404);res.end();}
});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const origin='http://127.0.0.1:'+server.address().port;
const evidence={status:'running',native,wasm_sha256:wasm,cli_sha256:hash(await fs.readFile(cli)),served,errors,external,modes:{}};
let browser;
try{
 browser=await chromium.launch({headless:true,executablePath:process.env.BISECT_CHROME_PATH??'C:/Program Files/Google/Chrome/Application/chrome.exe'});
 evidence.browser=browser.version();
 const context=await browser.newContext({acceptDownloads:true});
 await context.route('**/*',route=>{
  if(new URL(route.request().url()).origin===origin)return route.continue();
  external.push(route.request().url());return route.abort();
 });
 await context.addInitScript(()=>{
  window.civicReads=[];window.civicRuns=0;
  const a=File.prototype.arrayBuffer,W=window.Worker;
  File.prototype.arrayBuffer=function(...args){if(this.webkitRelativePath)window.civicReads.push(this.webkitRelativePath);return a.apply(this,args);};
  window.Worker=class extends W{postMessage(m,...args){
   if(m?.request?.operation==='inspect-civic-manifests')window.civicRuns++;
   return super.postMessage(m,...args);
  }};
 });
 for(const [mode,nested] of [['native-root',false],['folder',true]]){
  const page=await context.newPage();page.setDefaultTimeout(60000);
  page.on('pageerror',e=>errors.push(e.message));
  const get=id=>page.locator('#'+id),runs=()=>page.evaluate(()=>window.civicRuns);
  const ready=()=>page.waitForFunction(()=>!document.querySelector('#download-civic-inventory').disabled);
  const download=async(id,name)=>{
   const event=page.waitForEvent('download');await get(id).click();
   const p=path.join(out,mode+'-'+name);await (await event).saveAs(p);return fs.readFile(p);
  };
  await page.goto(origin+'/index.html');
  await page.waitForFunction(x=>document.querySelector('#runtime-artifact')?.textContent.includes(x),wasm);
  await get('operation').selectOption('inspect-civic-manifests');
  const chosen=get('civic-manifest-'+mode);await chosen.setInputFiles(tree);
  const all=await chosen.evaluate(el=>Array.from(el.files,f=>f.webkitRelativePath));
  assert(all.includes('civic_inputs/extra/nested/manifest.json')&&all.includes('civic_inputs/loose.json'));
  const selected=Object.keys(specimens).filter(p=>nested||!p.includes('/nested/')).map(p=>'civic_inputs/'+p);
  await page.waitForFunction(names=>names.every(x=>document.querySelector('#civic-manifest-names')?.textContent.includes(x)),selected);
  const reads=await page.evaluate(()=>window.civicReads);
  assert.deepEqual(reads.sort(),selected.sort());
  assert.equal(await runs(),0);assert(await get('download-civic-inventory').isDisabled());
  await get('execute').click();await ready();assert.equal(await runs(),1);
  const data=await download('save-project','inventory.bisect'),project=JSON.parse(data);
  const sources=project.files.civicManifestSources.sources;
  assert.equal(sources.length,selected.length);
  for(const [i,s] of sources.entries()){
   const rel=s.name.replace(/^\d+: civic_inputs\//,'');
   assert.deepEqual(Buffer.from(s.base64,'base64'),specimens[rel]);
   const result=project.result.entries[i],valid=!rel.startsWith('malformed/');
   assert.equal(result.native_list_included,valid);
   assert.equal(result.native_show_json,valid?show:null);
  }
  assert.deepEqual(project.result.native_list_text.split('\n'),nested?[...list,list[2]]:list);
  const zip=await download('download-civic-inventory','inventory.zip');
  await get('civic-manifest-clear').click();
  assert.equal(await chosen.evaluate(el=>el.files.length),0);
  await openWorkbenchProject(page,path.join(out,mode+'-inventory.bisect'));
  assert.equal(await runs(),1);assert(await get('download-civic-inventory').isDisabled());
  await get('execute').click();await ready();assert.equal(await runs(),2);
  assert.deepEqual(await download('download-civic-inventory','inventory-replay.zip'),zip);
  evidence.modes[mode]={selected,reads,zip_sha256:hash(zip),inert_open:true,explicit_replay_exact:true};
  await page.close();
 }
 assert.deepEqual(errors,[]);assert.deepEqual(external,[]);
 evidence.status='passed-civic-desktop-folder-and-recursive-browser';
}catch(e){evidence.status='failed-preserved';evidence.error=e.stack;process.exitCode=1;}
finally{
 await browser?.close();await new Promise(resolve=>server.close(resolve));
 await fs.writeFile(path.join(out,'evidence.json'),JSON.stringify(evidence,null,2)+'\n',{flag:'wx'});
}
console.log(JSON.stringify({status:evidence.status,out,error:evidence.error}));
