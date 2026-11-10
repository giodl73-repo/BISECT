// Actual staged worker, ZIP and inert project roundtrip for desktop Export.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {execFileSync} from 'node:child_process';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import {createProject} from '../../web/lab/project.js';
import {observedProjectOpen} from './fixtures/observed-project-open.mjs';

const arg=name=>{const i=process.argv.indexOf(name);assert(i>=0&&process.argv[i+1],`Supply ${name}`);return process.argv[i+1];};
const site=await fs.realpath(arg('--site')),reference=await fs.realpath(arg('--reference')),out=path.resolve(arg('--output'));
const hash=b=>createHash('sha256').update(b).digest('hex'),read=async p=>JSON.parse(await fs.readFile(p,'utf8'));
const e=await read(path.join(reference,'evidence.json')),wasm=hash(await fs.readFile(path.join(site,'bisect_wasm.wasm')));
const legacy=process.argv.includes('--expect-legacy-width-failure');
const sourceWidth=process.argv.includes('--source-width');
const originalRI=process.argv.includes('--original-ri-block');
assert(!originalRI||sourceWidth&&!legacy);
const status=sourceWidth?(legacy?'reproduced-native-export-source-width-gap':'passed-native-export-source-width'):
 (legacy?'reproduced-native64-reproducibility-width-gap':'passed-native-reproducibility-manifest-width');
assert.equal(e.status,status);
assert.equal(e.wasm_sha256,wasm);assert.equal(e.records.length,sourceWidth?(originalRI?25:24):18);
await fs.mkdir(out);
const served={},external=[],errors=[],mimes={'.js':'text/javascript','.html':'text/html','.css':'text/css','.wasm':'application/wasm','.json':'application/json'};
const server=createServer(async(req,res)=>{try{
 const route=decodeURIComponent(new URL(req.url,'http://localhost').pathname),f=await fs.realpath(path.resolve(site,'.'+(route==='/'?'/index.html':route)));
 assert(f.startsWith(site+path.sep));const bytes=await fs.readFile(f);served[route]=hash(bytes);
 res.setHeader('Content-Type',mimes[path.extname(f)]??'application/octet-stream');res.setHeader('Cache-Control','no-store');res.end(bytes);
}catch{res.writeHead(404);res.end();}});
await new Promise(r=>server.listen(0,'127.0.0.1',r));const origin='http://127.0.0.1:'+server.address().port;
const source=(name,bytes)=>({name:path.basename(name),base64:Buffer.from(bytes).toString('base64')});
let browser,page;
try{
 const packages=process.env.BISECT_PLAYWRIGHT_PACKAGES;if(!packages)throw Error('Set BISECT_PLAYWRIGHT_PACKAGES.');
 const {chromium}=createRequire(path.join(packages,'package.json'))('playwright');
 browser=await chromium.launch({headless:true,executablePath:process.env.BISECT_TEST_BROWSER||'C:/Program Files/Google/Chrome/Application/chrome.exe',args:['--no-first-run','--disable-background-networking','--disable-extensions']});
 const ctx=await browser.newContext({acceptDownloads:true});await ctx.route('**/*',r=>new URL(r.request().url()).origin===origin?r.continue():(external.push(r.request().url()),r.abort()));
 page=await ctx.newPage();page.on('pageerror',e=>errors.push(e.message));
 await page.addInitScript(()=>{window.reportRuns=[];window.reportReplies=[];const W=window.Worker;window.Worker=class extends W{
  constructor(url,...rest){super(url,...rest);this.engine=new URL(url,location.href).pathname.endsWith('/wasm-worker.js');if(this.engine)this.addEventListener('message',({data})=>{if(['result','error'].includes(data?.type))reportReplies.push(data);});}
  postMessage(message,...args){if(this.engine&&message?.type==='run')reportRuns.push(message);return super.postMessage(message,...args);}
 };});
 await page.goto(origin);await page.waitForFunction(sha=>document.querySelector('#runtime-artifact').textContent.includes(sha),wasm);
 const count=()=>page.evaluate(()=>reportRuns.length),g=id=>page.locator('#'+id);
 async function open(file,button){const before=await count();await observedProjectOpen(page,file);assert.equal(await count(),before);assert(await g(button).isDisabled());}
 async function run(){const before=await count();await g('execute').click();await page.waitForFunction(i=>reportRuns.length===i+1&&reportReplies.some(r=>r.id===reportRuns[i].id)&&!document.querySelector('#execute').disabled,before);return page.evaluate(i=>({request:reportRuns[i].request,reply:reportReplies.find(r=>r.id===reportRuns[i].id)}),before);}
 async function download(button,file){const d=page.waitForEvent('download');await g(button).click();await(await d).saveAs(file);return fs.readFile(file);}
 const outcomes=[];
 const cases=originalRI?['original-ri-block']:sourceWidth?['baseline','manifest-num_districts','legacy-both','safe-js-count','overflow-assignments']:
  ['baseline','wide-num_districts','invalid-overflow'];
 const modes=sourceWidth?['folder']:['direct','folder'];
 for(const id of cases){
  const input=path.join(reference,id),dest=path.join(out,id);await fs.mkdir(dest);
  const requests=await read(path.join(input,'requests.json')),expected=await read(path.join(input,'result.json'));
  for(const mode of modes){
   const req=requests[mode],expectedMode=expected[mode],want=expectedMode.result;
   const button=mode==='folder'?'download-export-folder':'download-reproducibility';
   const files=mode==='folder'?{exportFolderInputs:req.input}:{auditPlanText:source(req.label+'.rplan',req.document_json),
    reproducibilityInputs:{label:req.label,manifest:source('manifest.json',req.manifest_json),analysis:
     Object.fromEntries(Object.entries(req.analysis_files).map(([n,v])=>[n,source(n,v)]))}};
   const fixture=path.join(dest,mode+'.bisect');
   await fs.writeFile(fixture,JSON.stringify(createProject({name:id+'-'+mode,operation:req.operation,files,
    constraints:[],result:want??null,lastOperation:want?req.operation:null})));
   await open(fixture,button);
   const {request,reply}=await run();assert.deepEqual(request,req);
   const saved=path.join(dest,mode+'-saved.bisect');
   if(want){
    assert.equal(reply.type,'result',reply.error);assert.deepEqual(reply.result,want);
    if(sourceWidth&&typeof want.declared_districts==='string'){
     const row=g('result-tables').getByRole('row').filter({hasText:want.declared_districts});
     assert.equal(await row.count(),1);
     assert(await row.getByRole('cell',{name:want.declared_districts,exact:true}).isVisible());
    }
    const zipped=path.join(dest,mode+'.zip');await download(button,zipped);
    execFileSync('python',['-B','-c',
     'import json,sys,zipfile; files=json.load(open(sys.argv[2],encoding="utf8"))[sys.argv[3]]["result"]["files"]; z=zipfile.ZipFile(sys.argv[1]); names=z.namelist(); assert len(names)==len(set(names)) and set(names)==set(files); assert all(z.getinfo(n).file_size==len(v.encode()) and z.read(n)==v.encode() for n,v in files.items())',
     zipped,path.join(input,'result.json'),mode],{encoding:'utf8',windowsHide:true});
   }else{
    assert.equal(reply.type,'error');assert.match(reply.error,legacy?/expected usize/:/expected u64/);
    assert(await g(button).isDisabled());
   }
   const savedProject=JSON.parse((await download('save-project',saved)).toString('utf8'));
   const {workerFailureReceipt, ...savedSources}=savedProject.files;
   assert.deepEqual(savedSources,files);
   if(want){assert.equal(workerFailureReceipt,undefined);assert.deepEqual(savedProject.result,want);}
   else{
    assert.equal(workerFailureReceipt.runtime_artifact.sha256,wasm);
    assert.equal(workerFailureReceipt.request_sha256,hash(workerFailureReceipt.request_json));
    assert.deepEqual(JSON.parse(workerFailureReceipt.request_json),req);
   }
   await open(saved,button);
   outcomes.push({id,mode,delivery:want?.status??(want?'complete':'refused'),source_retained:true,inert_open:true});
  }
 }
 assert.deepEqual(external,[]);assert.deepEqual(errors,[]);assert.equal(served['/bisect_wasm.wasm'],wasm);
 assert.equal(await count(),cases.length*modes.length);
 const receipt={status:legacy?'reproduced-export-width-in-staged-browser':'passed-export-width-in-staged-browser',source_width:sourceWidth,
  wasm_sha256:wasm,site,browser:browser.version(),outcomes,served,actual_worker_runs:await count(),full_parity:false,deployed:false};
 await fs.writeFile(path.join(out,'evidence.json'),JSON.stringify(receipt,null,2)+'\n');
 console.log(receipt.status);
}catch(e){if(page)await fs.writeFile(path.join(out,'failure.json'),JSON.stringify({error:e.message,state:await page.evaluate(()=>({project:document.querySelector('#project-status')?.textContent,result:document.querySelector('#result-status')?.textContent})).catch(()=>null)},null,2));throw e;}
finally{await browser?.close();await new Promise(r=>server.close(r));}
