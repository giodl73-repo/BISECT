// Actual staged worker and file downloads for the native Report width matrix.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import {createProject} from '../../web/lab/project.js';
import {observedProjectOpen} from './fixtures/observed-project-open.mjs';

const arg=name=>{const i=process.argv.indexOf(name);assert(i>=0&&process.argv[i+1],`Supply ${name}`);return process.argv[i+1];};
const site=await fs.realpath(arg('--site')),reference=await fs.realpath(arg('--reference')),out=path.resolve(arg('--output'));
const hash=b=>createHash('sha256').update(b).digest('hex'),read=async p=>JSON.parse(await fs.readFile(p,'utf8'));
const e=await read(path.join(reference,'evidence.json')),wasm=hash(await fs.readFile(path.join(site,'bisect_wasm.wasm')));
assert.equal(e.status,'passed-report-width-native-and-wasm');assert.equal(e.wasm_sha256,wasm);assert.equal(e.outcomes.length,18);
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
 for(const row of e.outcomes){
  const root=path.join(reference,row.id),requests=await read(path.join(root,'requests.json')),results=await read(path.join(root,'result.json'));
  const dest=path.join(out,row.id);await fs.mkdir(dest);
  for(const mode of ['audit','report']){
   const req=requests[mode],expect=results[mode],audit=mode==='audit',button=audit?'download-commission-audit-only':'download-commission-json';
   const files=audit?{commissionAuditSource:source('manifest.json',req.manifest_json)}:{commissionInputs:{files:Object.fromEntries(Object.entries(req.files).map(([name,b])=>[name,source(name,b)])),generated_at:req.generated_at},commissionDeliverySettings:req.delivery};
   if(req.fallback)files.commissionFallbackSettings={enabled:true,...req.fallback};
   const project=(name,fallback)=>createProject({name,operation:req.operation,files:fallback?files:Object.fromEntries(Object.entries(files).filter(([k])=>k!=='commissionFallbackSettings')),constraints:[],result:expect,lastOperation:req.operation});
   if(row.id==='invalid-overflow'){
    const refused=path.join(dest,mode+'-without-fallback.bisect');await fs.writeFile(refused,JSON.stringify(project(mode+'-without-fallback',false)));
    await open(refused,button);const {reply}=await run();assert.equal(reply.type,'error');assert.match(reply.error,/expected u64/);assert(await g(button).isDisabled());
   }
   const input=path.join(dest,mode+'.bisect');await fs.writeFile(input,JSON.stringify(project(row.id+'-'+mode,true)));
   await open(input,button);const {request,reply}=await run();assert.equal(reply.type,'result',reply.error);assert.deepEqual(reply.result,expect);
   assert.deepEqual(request,req);
   const deliveries=audit?[[button,'audit.json',expect.audit_json]]:[['download-commission-json','report.json',expect.report_json],['download-commission-html','report.html',expect.report_html],['download-commission-audit','companion-audit.json',expect.audit_json]];
   for(const [key,name,expected] of deliveries)assert.equal((await download(key,path.join(dest,name))).toString('utf8'),expected);
   if(row.id==='wide-num_districts'&&!audit){
    await g('preview-commission').click();await page.waitForFunction(()=>!!document.querySelector('#commission-preview-host iframe')?.contentDocument?.querySelector('h1'));
    const frame=g('commission-preview-host').locator('iframe');assert(!(await frame.getAttribute('sandbox')).includes('allow-scripts'));
    assert.match(await frame.evaluate(el=>el.contentDocument.body.textContent),/18446744073709551615/);
    await g('close-commission-preview').click();await download('download-commission-zip',path.join(dest,'delivery.zip'));
   }
   const saved=path.join(dest,mode+'-saved.bisect');const s=JSON.parse((await download('save-project',saved)).toString('utf8'));
   assert.deepEqual(s.files,files);assert.deepEqual(s.result,expect);await open(saved,button);
   outcomes.push({id:row.id,mode,inert_open:true,exact_worker_result:true,exact_downloads:true,exact_saved_source:true});
  }
 }
 assert.deepEqual(external,[]);assert.deepEqual(errors,[]);assert.equal(served['/bisect_wasm.wasm'],wasm);
 const receipt={status:'passed-report-width-staged-browser',wasm_sha256:wasm,site,browser:browser.version(),outcomes,served,
  actual_worker_runs:await count(),fallback_refusals:2,exact_u64_in_sandboxed_preview:true,full_parity:false,deployed:false};
 assert.equal(receipt.actual_worker_runs,38);await fs.writeFile(path.join(out,'evidence.json'),JSON.stringify(receipt,null,2)+'\n');
 console.log('PASS report width browser: 36 exact native-based deliveries and two explicit refusals.');
}catch(e){if(page)await fs.writeFile(path.join(out,'failure.json'),JSON.stringify({error:e.message,state:await page.evaluate(()=>({project:document.querySelector('#project-status')?.textContent,result:document.querySelector('#result-status')?.textContent})).catch(()=>null)},null,2));throw e;}
finally{await browser?.close();await new Promise(r=>server.close(r));}
