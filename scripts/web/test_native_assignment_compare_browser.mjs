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
const image=path.join(site,'bisect_wasm.wasm');
const require=createRequire(path.join(process.env.BISECT_PLAYWRIGHT_PACKAGES??process.cwd(),'package.json'));
const {chromium}=require('playwright');
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
await fs.mkdir(parent,{recursive:true});
const output=await fs.mkdtemp(path.join(parent,'native-assignment-compare-'));
const root=path.join(output,'native-fixture');
await fs.mkdir(root);
const scale=args.includes('--block-scale-fixture');
const planA=scale?'plan-a.rplan':'plan-a.json',planB='plan-b.json';
const units=scale?200001:4,districts=scale?501:2;
let a,b;
if(scale){
 const left=[],right=[];
 for(let i=0;i<units;i++){
  const key=String(i).padStart(15,'0'),label=i%districts;
  // Both files describe the same groups under different positive IDs.
  left.push('"'+key+'":'+(18446744073709551615n-BigInt(label)));
  right.push('"'+key+'":'+(districts-label));
 }
 a='{"rplan_version":"legacy","assignments":{'+left.join(',')+'}}\n';
 b='{"assignment":{'+right.join(',')+'}}\n';
 assert(Buffer.byteLength(a)>3*1024*1024&&Buffer.byteLength(b)>3*1024*1024);
}else{
 a='{"A":4294967297,"B":4294967297,"C":18446744073709551615,"D":18446744073709551615}\n';
 b='{"assignment":{"A":9007199254740993,"B":7,"C":7,"E":9007199254740993}}\n';
}
await fs.writeFile(path.join(root,planA),a,{flag:'wx'});
await fs.writeFile(path.join(root,planB),b,{flag:'wx'});
const nativeCases=[];
for(const format of ['table','json','csv']){
 const argv=['compare','--plan-a','./'+planA,'--plan-b','./'+planB,'--format',format];
 const native=spawnSync(cli,argv,{cwd:root,windowsHide:true,maxBuffer:1024*1024});
 assert.ifError(native.error);
 await fs.writeFile(path.join(root,format+'.stdout'),native.stdout,{flag:'wx'});
 await fs.writeFile(path.join(root,format+'.stderr'),native.stderr,{flag:'wx'});
 assert.equal(native.status,0,native.stderr.toString('utf8'));
 assert.equal(native.stderr.length,0);
 nativeCases.push({argv,format,stdout_sha256:hash(native.stdout),stdout_bytes:native.stdout.length});
}
const oracle=JSON.parse(await fs.readFile(path.join(root,'json.stdout'),'utf8'));
assert.equal(oracle.metrics.jaccard_similarity,scale?1:1/3);
assert.deepEqual([oracle.plan_a.num_districts,oracle.plan_b.num_districts],[districts,districts]);
assert.deepEqual([oracle.plan_a.num_tracts,oracle.plan_b.num_tracts],[units,units]);
const server=createServer(async (req,res)=>{
 try{
  const name=decodeURIComponent(new URL(req.url,'http://localhost').pathname);
  const asset=await fs.realpath(path.resolve(site,'.'+name));
  assert(asset.startsWith(site+path.sep));
  const data=await fs.readFile(asset);
  res.setHeader('Content-Type',name.endsWith('.js')?'text/javascript':name.endsWith('.wasm')?'application/wasm':name.endsWith('.css')?'text/css':name.endsWith('.json')?'application/json':'text/html');
  res.end(data);
 }catch{res.writeHead(404);res.end();}
});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const origin='http://127.0.0.1:'+server.address().port;
const errors=[],external=[];
const evidence={status:'running',full_parity:false,scope:'Actual supplied workbench package, Worker and project controls. Original direct-file Compare on fresh native CLI fixture inputs. No cross-cycle, layout discovery, authenticated provenance or approved narrative claim.',
 image_sha256:hash(await fs.readFile(image)),cli_sha256:hash(await fs.readFile(cli)),fixture:{scale,units,districts,planA,planB},native_cases:nativeCases,downloads:{}};
let browser,page;
try{
 browser=await chromium.launch({headless:true,executablePath:process.env.BISECT_CHROME_PATH??'C:/Program Files/Google/Chrome/Application/chrome.exe'});
 const context=await browser.newContext({acceptDownloads:true});
 await context.route('**/*',route=>{
  if(new URL(route.request().url()).origin===origin)return route.continue();
  external.push(route.request().url());return route.abort();
 });
 await context.addInitScript(()=>{
  window.assignmentCompareRuns=0;
  const Native=window.Worker;
  window.Worker=class extends Native{
   postMessage(message,...rest){
    if(message?.request?.operation==='compare-native-assignment-files')window.assignmentCompareRuns++;
    return super.postMessage(message,...rest);
   }
  };
 });
 page=await context.newPage();
 page.on('pageerror',error=>errors.push(error.message));
 await page.goto(origin+'/index.html');
 await page.waitForFunction(sha=>document.querySelector('#runtime-artifact')?.textContent.includes(sha),evidence.image_sha256);
 const get=id=>page.locator('#'+id);
 const download=async (id,name)=>{
  const pending=page.waitForEvent('download');await get(id).click();
  const result=await pending;const file=path.join(output,name);await result.saveAs(file);return fs.readFile(file);
 };
 const ready=()=>page.waitForFunction(()=>!document.querySelector('#execute').disabled);
 const exportReady=()=>page.waitForFunction(()=>!document.querySelector('#download-direct-assignment-json').disabled,undefined,{timeout:8000});
 const pick=async (key,name)=>{
  await get('direct-assignment-'+key).setInputFiles(path.join(root,name));
  await page.waitForFunction(({key,name})=>document.querySelector('#direct-assignment-name-'+key).textContent===name,{key,name});
 };
 await ready();
 await get('operation').selectOption('compare-native-assignment-files');
 assert.match(await get('direct-assignment-fields').textContent(),/uses unit counts/);
 await pick('a',planA);await pick('b',planB);
 assert.equal(await page.evaluate(()=>window.assignmentCompareRuns),0);
 await get('execute').click();await exportReady();
 assert.equal(await page.evaluate(()=>window.assignmentCompareRuns),1);
 for(const format of ['table','json','csv']){
  const file=await download('download-direct-assignment-'+format,format+'.actual');
  const native=await fs.readFile(path.join(root,format+'.stdout'));
  assert.deepEqual(file,native);
  evidence.downloads[format]=hash(file);
 }
 assert.equal(await get('direct-assignment-table').textContent(),await fs.readFile(path.join(root,'table.stdout'),'utf8'));
 const saved=await download('save-project','compare.bisect'),project=JSON.parse(saved);
 for(const [key,name] of [['a',planA],['b',planB]]){
  assert.equal(project.files.directAssignmentInputs[key].name,name);
  assert.deepEqual(Buffer.from(project.files.directAssignmentInputs[key].base64,'base64'),await fs.readFile(path.join(root,name)));
 }
 await get('direct-assignment-clear').click();
 assert(await get('download-direct-assignment-json').isDisabled());
 await openWorkbenchProject(page,path.join(output,'compare.bisect'));
 assert.equal(await get('direct-assignment-name-a').textContent(),planA);
 assert(await get('download-direct-assignment-json').isDisabled());
 assert.equal(await page.evaluate(()=>window.assignmentCompareRuns),1);
 await ready();await get('execute').click();await exportReady();
 assert.equal(await page.evaluate(()=>window.assignmentCompareRuns),2);
 assert.deepEqual(await download('download-direct-assignment-json','json.after-open'),await fs.readFile(path.join(root,'json.stdout')));
 await get('direct-assignment-b').setInputFiles({name:'<markup>.json',mimeType:'application/json',buffer:Buffer.from('{"B":0}')});
 await page.waitForFunction(()=>document.querySelector('#direct-assignment-name-b').textContent==='<markup>.json');
 assert.equal(await get('direct-assignment-name-b').locator('markup').count(),0);
 assert(await get('download-direct-assignment-json').isDisabled());
 await ready();await get('execute').click();
 await page.waitForFunction(()=>document.querySelector('#result-status').textContent.includes('must be a positive integer'));
 assert(await get('download-direct-assignment-json').isDisabled());
 await page.evaluate(()=>{
  const original=File.prototype.arrayBuffer;
  File.prototype.arrayBuffer=async function(){
   if(this.name==='held.json')await new Promise(resolve=>{window.resumeAssignmentRead=resolve;});
   return original.call(this);
  };
 });
 await get('direct-assignment-b').setInputFiles({name:'held.json',mimeType:'application/json',buffer:Buffer.from('{"B":9}')});
 await page.waitForFunction(()=>typeof window.resumeAssignmentRead==='function');
 assert(await get('execute').isDisabled());
 await openWorkbenchProject(page,path.join(output,'compare.bisect'));
 await page.evaluate(()=>window.resumeAssignmentRead());
 await ready();
 assert.equal(await get('direct-assignment-name-b').textContent(),planB);
 assert(await get('download-direct-assignment-json').isDisabled());
 assert.equal(await page.evaluate(()=>window.assignmentCompareRuns),3);
 if(scale){
  await get('direct-assignment-a').setInputFiles({name:'too-large.json',mimeType:'application/json',buffer:Buffer.alloc(16*1024*1024+1,32)});
  await page.waitForFunction(()=>document.querySelector('#result-status').textContent.includes('at most 16 MiB'));
  assert(await get('download-direct-assignment-json').isDisabled());
  assert.equal(await page.evaluate(()=>window.assignmentCompareRuns),3);
  evidence.over_byte_cap_refused_without_run=true;
 }
 assert.deepEqual(errors,[]);assert.deepEqual(external,[]);
 Object.assign(evidence,{status:'passed-packaged-chrome-direct-assignment-comparison',browser:browser.version(),
  saved_sources_exact:true,open_inert:true,rerun_exact:true,bad_id_refused:true,filename_as_text:true,held_read_retired_by_open:true});
}catch(error){
 evidence.status='failed-preserved';evidence.error={message:error.message,stack:error.stack};process.exitCode=1;
 if(page)evidence.browser_state=await page.evaluate(()=>({status:document.querySelector('#result-status')?.textContent,
  operation:document.querySelector('#operation')?.value,requests:window.assignmentCompareRuns,
  runtime:document.querySelector('#runtime-artifact')?.textContent,
  executing:!document.querySelector('#cancel')?.disabled})).catch(()=>null);
}finally{
 await fs.writeFile(path.join(output,'evidence.json'),JSON.stringify({...evidence,errors,external},null,2));
 await browser?.close();await new Promise(resolve=>server.close(resolve));
}
console.log(JSON.stringify({output,status:evidence.status,error:evidence.error}));
