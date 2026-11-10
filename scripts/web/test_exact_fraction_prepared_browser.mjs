import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import {spawnSync} from 'node:child_process';
import {createProject} from '../../web/lab/project.js';
import {preparedEngineRequest} from '../../web/lab/prepared-engine-files.js';
import {openWorkbenchProject} from './workbench_project_open.mjs';

const arg=name=>{const i=process.argv.indexOf(name);assert(i>=0&&process.argv[i+1],'Supply '+name);return path.resolve(process.argv[i+1]);};
const site=await fs.realpath(arg('--site')),host=await fs.realpath(arg('--host')),parent=arg('--output');
await fs.mkdir(parent,{recursive:true});const out=await fs.mkdtemp(path.join(parent,'exact-fraction-'));
const sha=b=>createHash('sha256').update(b).digest('hex');
const wasm=await fs.readFile(path.join(site,'bisect_wasm.wasm'));
const fraction=1/3,percent=fraction*100;assert(percent/100<fraction);
const graph={schema_version:1,state:'RI',year:'2020',geoids:['44001000100','44001000200'],adjacency:[[1],[0]],population:[2,4],
 edges:[[0,1,1]],areas:[100,100],exterior_perimeters:[40,40],centroids:[[-71,41],[-70.99,41]]};
const options={structure:'standard-bisect',weights:'geographic',search:'single',districts:2,seed:'17',seeds:1,steps:200,
 percentile:0,alpha_county:0,balance_tolerance:percent,area_swing:1.1,iterations:100};
const settings={label:'Exact fractional boundary',chamber:'congressional',created_at:'2026-11-06T00:00:00Z',inspect_only:false};
const cases={exact:fraction,percent:undefined,contradictory:0.25};
const report={status:'running',wasm_sha256:sha(wasm),native_host_sha256:sha(await fs.readFile(host)),fraction,percent,percent_roundtrip:percent/100,
 cases:{},served:{},errors:[],external:[],full_parity:false};
const write=(name,b)=>fs.writeFile(path.join(out,name),typeof b==='string'||Buffer.isBuffer(b)?b:JSON.stringify(b,null,2));
const inputs={};
for(const [name,exact] of Object.entries(cases)){
 const source={name:name+'.json',text:JSON.stringify({operation:'run-engine-build',request:{graph,options:{...options,...(exact===undefined?{}:{balance_tolerance_fraction:exact})}}})};
 const files={preparedEngineInputs:{source,settings}};inputs[name]=files;
 await write(name+'.bisect',createProject({name:'Fractional tolerance '+name,operation:'run-prepared-engine',files,constraints:[],result:null,lastOperation:null}));
}
const server=createServer(async(req,res)=>{try{
 const rel=decodeURIComponent(new URL(req.url,'http://localhost').pathname),file=await fs.realpath(path.resolve(site,'.'+(rel==='/'?'/index.html':rel)));
 assert(file.startsWith(site+path.sep));const data=await fs.readFile(file);report.served[rel]=sha(data);
 res.setHeader('Content-Type',{'.wasm':'application/wasm','.js':'text/javascript','.html':'text/html','.css':'text/css','.json':'application/json'}[path.extname(file)]??'application/octet-stream');res.end(data);
}catch(e){if(!['/offline-manifest.json','/favicon.ico'].includes(req.url))report.errors.push([req.url,e.message]);res.writeHead(404);res.end();}});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));const origin='http://127.0.0.1:'+server.address().port;
let browser;
try{
 const {chromium}=createRequire(path.join(process.env.BISECT_PLAYWRIGHT_PACKAGES??process.cwd(),'package.json'))('playwright');
 browser=await chromium.launch({headless:true,executablePath:process.env.BISECT_CHROME_PATH??'C:/Program Files/Google/Chrome/Application/chrome.exe'});
 report.browser=browser.version();const ctx=await browser.newContext({acceptDownloads:true});
 await ctx.route('**/*',r=>new URL(r.request().url()).origin===origin?r.continue():(report.external.push(r.request().url()),r.abort()));
 await ctx.addInitScript(()=>{window.fractionRequests=[];window.fractionReplies=[];const W=window.Worker;window.Worker=class extends W{
  constructor(url,...rest){super(url,...rest);this.engine=new URL(url,location.href).pathname.endsWith('/wasm-worker.js');if(this.engine)this.addEventListener('message',({data})=>{if(['result','error'].includes(data?.type))window.fractionReplies.push(data);});}
  postMessage(m,...rest){if(this.engine&&m?.type==='run')window.fractionRequests.push(m.request);return super.postMessage(m,...rest);}
 };});
 const page=await ctx.newPage();page.on('pageerror',e=>report.errors.push(e.message));
 const get=id=>page.locator('#'+id),count=()=>page.evaluate(()=>window.fractionRequests.length);
 await page.goto(origin);await page.waitForFunction(x=>document.getElementById('runtime-artifact').textContent.includes(x),report.wasm_sha256);
 async function opened(name){const n=await count();await openWorkbenchProject(page,path.join(out,name));assert.equal(await count(),n);assert(await get('download-prepared-engine-plan').isDisabled());}
 async function run(){const n=await count();await get('execute').click();await page.waitForFunction(n=>window.fractionReplies.length>n&&!document.getElementById('execute').disabled,n);return page.evaluate(n=>({request:window.fractionRequests[n],reply:window.fractionReplies[n]}),n);}
 async function native(name,request){
  const file=path.join(out,name+'-request.json');await write(name+'-request.json',request);
  const result=spawnSync(host,[file],{encoding:'utf8',windowsHide:true,timeout:30000});assert.ifError(result.error);
  await write(name+'-native.stdout',result.stdout);await write(name+'-native.stderr',result.stderr);return result;
 }
 async function download(id,file){const event=page.waitForEvent('download');await get(id).click();await(await event).saveAs(path.join(out,file));return fs.readFile(path.join(out,file),'utf8');}
 await opened('exact.bisect');await get('prepared-engine-inspect_only').check();
 const preview=await run();assert.equal(preview.reply.type,'result',preview.reply.error);assert.equal(preview.reply.result.execution_performed,false);
 assert.equal(preview.reply.result.engine_request.options.balance_tolerance_fraction,fraction);assert(await get('download-prepared-engine-plan').isDisabled());
 assert.deepEqual(preview.reply.result,JSON.parse((await native('review',preview.request)).stdout));
 await get('prepared-engine-inspect_only').uncheck();
 for(const name of Object.keys(cases)){
  if(name!=='exact')await opened(name+'.bisect');
  const {request,reply}=await run();assert.deepEqual(request,preparedEngineRequest(inputs[name].preparedEngineInputs));
  await write(name+'-browser.json',reply);const n=await native(name,request);
  if(name==='exact'){
   assert.equal(n.status,0,n.stderr);assert.deepEqual(reply.result,JSON.parse(n.stdout));
   const r=reply.result;assert.equal(r.result.build_report.accepted_attempt,0);assert.equal(r.result.metrics.within_requested_tolerance,true);
   assert.equal(r.executed_request.options.balance_tolerance_fraction,fraction);assert.deepEqual(r.result.metrics.district_metrics.map(d=>d.population).sort((a,b)=>a-b),[2,4]);
   assert.equal(await download('download-prepared-engine-plan','exact.rplan'),r.export.document_json);
   assert.equal(await download('download-prepared-engine-context','exact.rctx'),r.export.context_json);
   const saved=JSON.parse(await download('save-project','saved.bisect'));assert.deepEqual(saved.files,inputs[name]);
   await opened('saved.bisect');const replay=await run();assert.deepEqual(replay.request,request);
   assert.equal(replay.reply.type,'result',replay.reply.error);assert.deepEqual(replay.reply.result,r);
   report.cases[name]={accepted_attempt:0,exact_plan_context:true,inert_open:true,explicit_rerun:true};
  }else{
   assert.notEqual(n.status,0);assert.equal(reply.type,'error');const error=JSON.parse(n.stderr.trim().replace(/^Error: /,''));
   assert.equal(reply.error,error);assert.match(error,name==='percent'?/population balance retry policy exhausted/:/Invalid engine option bounds/);
   assert(await get('download-prepared-engine-plan').isDisabled());report.cases[name]={error,exact_native_refusal:true};
  }
 }
 assert.equal(await count(),5);assert.equal(report.served['/bisect_wasm.wasm'],report.wasm_sha256);assert.deepEqual(report.errors,[]);assert.deepEqual(report.external,[]);
 report.status='passed-exact-fraction-prepared-browser';
}catch(e){report.status='failed';report.failure=String(e.stack??e);throw e;}
finally{await write('evidence.json',report);await browser?.close();await new Promise(resolve=>server.close(resolve));}
console.log('Exact fractional tolerance native/browser evidence:',out);
