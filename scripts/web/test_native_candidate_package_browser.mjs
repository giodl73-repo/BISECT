import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import {openWorkbenchProject} from './workbench_project_open.mjs';

const arg=name=>{const i=process.argv.indexOf(name);assert(i>=0&&process.argv[i+1],'Supply '+name);return path.resolve(process.argv[i+1]);};
const [site,cli,parent]=[await fs.realpath(arg('--site')),await fs.realpath(arg('--cli')),arg('--output')];
await fs.mkdir(parent,{recursive:true});
const out=await fs.mkdtemp(path.join(parent,'native-candidate-package-'));
console.log('Candidate package evidence:',out);
const hash=b=>createHash('sha256').update(b).digest('hex');
const write=(name,bytes)=>fs.writeFile(path.join(out,name),bytes,{flag:'wx'});
const json=b=>JSON.parse(b.toString('utf8'));
const h='candidate_name,party,race,curator,curator_credentials,curator_attestation_date,source,independently_verified,attestation_doc_path,attestation_doc_format';
const csv=Buffer.from(h+'\r\n"Adams, J",DEM,Black,curator1,PhD,2026-04-15,public,true,letter.pdf,pdf\r\n"Adams, J",DEM,white,curator2,MS,2026-04-16,public,false,other.txt,txt\r\n');
const letter=Buffer.from([0xff,0,0x25,80,68,70]);
const other=Buffer.from('Other document\n');
const fixedTime='2026-10-07T12:00:00Z',state='Ri',year='2010';
const automatic='candidate_race_2010_ri', explicit='my_review';
const report={status:'running',wasm:hash(await fs.readFile(path.join(site,'bisect_wasm.wasm'))),native_cli:hash(await fs.readFile(cli)),calls:{},served:{},external:[],errors:[],server_failures:[],comparisons:[]};
try{
 await Promise.all([write('annotations.csv',csv),write('letter.pdf',letter),write('other.txt',other)]);
 function run(file,staged,output,extra,expected){
  const args=['civic','add-candidate-race',file,'--year',year,'--state',state,'--submitter','Recorded curator','--attestation-doc',staged,'--output-base',output,...extra];
  const r=spawnSync(cli,args,{cwd:out,windowsHide:true,timeout:30000});assert.ifError(r.error);
  report.calls[output]={args,exit:r.status,stdout:hash(r.stdout),stderr:hash(r.stderr)};
  assert.equal(r.status,expected,r.stderr.toString());return r;
 }
 const cases=[['default',automatic,'letter.pdf',[]],['explicit',explicit,'other.txt',['--label',explicit]]];
 for(const [name,,staged,extra] of cases){
  const r=run('annotations.csv',staged,name,extra,0);await Promise.all([write(name+'.stdout',r.stdout),write(name+'.stderr',r.stderr)]);
 }
 const refuse=run('annotations.csv','absent.pdf','absent',[],1);
 await write('absent.stderr',refuse.stderr);assert.match(refuse.stderr.toString(),/attestation document not found/);
 const server=createServer(async(req,res)=>{
  try{
   const relative=decodeURIComponent(new URL(req.url,'http://localhost').pathname);
   const file=await fs.realpath(path.resolve(site,'.'+relative));assert(file.startsWith(site+path.sep));
   const bytes=await fs.readFile(file);report.served[relative]=hash(bytes);
   const mime={'.js':'text/javascript','.mjs':'text/javascript','.json':'application/json','.html':'text/html','.css':'text/css','.wasm':'application/wasm'};
   res.setHeader('Content-Type',mime[path.extname(file)]??'application/octet-stream');res.end(bytes);
  }catch(e){if(!['/offline-manifest.json','/favicon.ico'].includes(req.url))report.server_failures.push([req.url,e.message]);res.writeHead(404);res.end();}
 });
 await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
 const origin='http://127.0.0.1:'+server.address().port;
 let browser;
 try{
  const {chromium}=createRequire(path.join(process.env.BISECT_PLAYWRIGHT_PACKAGES??process.cwd(),'package.json'))('playwright');
  browser=await chromium.launch({headless:true,executablePath:process.env.BISECT_CHROME_PATH??'C:/Program Files/Google/Chrome/Application/chrome.exe'});
  report.browser=browser.version();
  const ctx=await browser.newContext({acceptDownloads:true});
  await ctx.route('**/*',r=>{if(new URL(r.request().url()).origin===origin)return r.continue();report.external.push(r.request().url());return r.abort();});
  await ctx.addInitScript(()=>{
   window.candidateRuns=[];
   const W=window.Worker;window.Worker=class extends W{postMessage(m,...args){if(m?.request?.operation==='ingest-candidate-race')window.candidateRuns.push(structuredClone(m.request));return super.postMessage(m,...args);}};
  });
  const page=await ctx.newPage();page.setDefaultTimeout(30000);page.on('pageerror',e=>report.errors.push(e.message));
  const get=id=>page.locator('#'+id),candidate=id=>get('candidate-'+id),ready=()=>page.waitForFunction(()=>!document.querySelector('#download-candidate').disabled);
  const runs=()=>page.evaluate(()=>window.candidateRuns);
  const field=async(k,v)=>{await candidate(k).fill(v);await candidate(k).dispatchEvent('change');};
  async function download(id,name){const downloading=page.waitForEvent('download');await get(id).click();const item=await downloading;await item.saveAs(path.join(out,name));return item.suggestedFilename();}
  await page.goto(origin+'/index.html');await page.waitForFunction(x=>document.querySelector('#runtime-artifact')?.textContent.includes(x),report.wasm);
  await get('operation').selectOption('ingest-candidate-race');
  await field('state',state);await field('year',year);await field('submitter','Recorded curator');await field('timestamp',fixedTime);
  await candidate('csv').setInputFiles(path.join(out,'annotations.csv'));
  await page.waitForFunction(()=>document.querySelector('#candidate-source-name').textContent==='annotations.csv');
  await candidate('documents').setInputFiles([path.join(out,'letter.pdf'),path.join(out,'other.txt')]);
  await page.waitForFunction(()=>document.querySelector('#candidate-stage').options.length===2);
  await candidate('stage').selectOption('letter.pdf');assert.equal((await runs()).length,0);
  await get('execute').click();await ready();
  const actualLabel=(await runs())[0]?.settings?.label;
  await write('automatic-label-observation.json',JSON.stringify({state,year,native:automatic,browser:actualLabel}));
  assert.equal(actualLabel,automatic,'Default candidate package label differs from desktop');
  assert.equal(await download('download-candidate','default.zip'),automatic+'-candidate-race.zip');
  await download('save-project','automatic.bisect');
  const saved=json(await fs.readFile(path.join(out,'automatic.bisect')));
  const inputs=saved.files.candidateInputs;
  assert.equal(inputs.settings.label,'');assert.equal(inputs.staged_path,'letter.pdf');
  for(const [actual,want] of [[inputs.csv.base64,csv],[inputs.documents['letter.pdf'],letter],[inputs.documents['other.txt'],other]])assert.deepEqual(Buffer.from(actual,'base64'),want);
  const forged=structuredClone(saved);forged.result.files['annotations.json']='["forged"]';
  await write('forged.bisect',JSON.stringify(forged));
  await openWorkbenchProject(page,path.join(out,'forged.bisect'));
  assert(await get('download-candidate').isDisabled());assert.equal((await runs()).length,1);
  assert.equal(await candidate('label').inputValue(),'');
  await get('execute').click();await ready();assert.equal((await runs()).length,2);
  await download('download-candidate','replayed.zip');
  assert.deepEqual(await fs.readFile(path.join(out,'default.zip')),await fs.readFile(path.join(out,'replayed.zip')));
  await field('label',explicit);await candidate('stage').selectOption('other.txt');
  assert(await get('download-candidate').isDisabled());
  await get('execute').click();await ready();assert.equal((await runs())[2]?.settings?.label,explicit);
  assert.equal(await download('download-candidate','explicit.zip'),explicit+'-candidate-race.zip');
  await openWorkbenchProject(page,path.join(out,'automatic.bisect'));
  await field('state','NC');assert(await get('download-candidate').isDisabled());
  await get('execute').click();await ready();const requests=await runs();assert.equal(requests[3]?.settings?.label,'candidate_race_2010_nc');
  // Compare the fresh serialized worker results directly to the native tree;
  // a separate reader verifies every member of the downloaded ZIPs.
  const savedResult=json(await fs.readFile(path.join(out,'automatic.bisect'))).result;
  for(const [name,label,doc] of cases){
   const d=path.join(out,name,'v1','civic_inputs',label);
   const manifest=json(await fs.readFile(path.join(d,'manifest.json')));
   assert.equal(manifest.label,label);assert.equal(manifest.attestation_doc,doc);
   for(const key of ['original_csv_sha256','annotations_sha256','provenance_sha256'])assert.equal(manifest[key],savedResult.manifest[key]);
   if(name==='default'){
    manifest.submitted_at=fixedTime;manifest.ingested_at=fixedTime;
    assert.deepEqual(manifest,savedResult.manifest);
    for(const file of ['annotations.json','provenance.json'])assert.equal(await fs.readFile(path.join(d,file),'utf8'),savedResult.files[file]);
   }
  }
  assert.equal(report.served['/bisect_wasm.wasm'],report.wasm);
  assert.deepEqual(report.errors,[]);assert.deepEqual(report.external,[]);assert.deepEqual(report.server_failures,[]);
  report.runs=(await runs()).map(x=>({settings:x.settings,staged_path:x.staged_path}));
  report.status='passed-candidate-package-browser-lifecycle';
 }finally{await browser?.close();await new Promise(resolve=>server.close(resolve));}
}catch(e){report.status='failed';report.failure=String(e.stack??e);throw e;}
finally{await write('browser-evidence.json',JSON.stringify(report,null,2));}
