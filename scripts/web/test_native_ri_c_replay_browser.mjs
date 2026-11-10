// Isolated localhost and browser for actual recorded C METIS replay.
import fs from 'node:fs/promises';import path from 'node:path';import assert from 'node:assert/strict';
import{createHash}from'node:crypto';import{createServer}from'node:http';import{createRequire}from'node:module';
import{pathToFileURL}from'node:url';import{createProject}from'../../web/lab/project.js';
import{openWorkbenchProject}from'./workbench_project_open.mjs';

const arg=name=>{const i=process.argv.indexOf(name);assert(i>=0&&process.argv[i+1],'Supply '+name);return process.argv[i+1];};
const mode=arg('--profile');assert(['core','c'].includes(mode));
const parent=path.resolve(arg('--output')),nativeCase=await fs.realpath(arg('--native-case'));await fs.mkdir(parent,{recursive:true});
const out=await fs.mkdtemp(path.join(parent,'real-ri-replay-'));
const sha=b=>createHash('sha256').update(b).digest('hex'),json=async p=>JSON.parse(await fs.readFile(p,'utf8'));
const native=await json(path.join(nativeCase,'evidence.json'));assert.equal(native.status,'passed-actual-state-and-replay-native-on-original-ri-tracts');
const manifestPath=await fs.realpath(native.original_manifest);assert(manifestPath.startsWith(nativeCase+path.sep));
const manifest=await fs.readFile(manifestPath),record=JSON.parse(manifest);assert.equal(sha(manifest),native.original_manifest_sha256);
const site=await fs.realpath(arg('--site'));assert.equal(record.native_replay_recipe.recursive_metis_backend,'c-ffi');
assert.equal(native.effective_backend,'c-ffi');assert.equal(native.seed,'42');
const {instantiateEngine}=await import(pathToFileURL(path.join(site,'wasm-engine.js')).href);
const {loadMetisSidecar}=await import(pathToFileURL(path.join(site,'metis-c-sidecar.js')).href);
const report={status:'running',mode,actual_native_cli:native.native.sha256,original_manifest_sha256:sha(manifest),served:{},external:[],errors:[],full_parity:false};
const evidence=()=>fs.writeFile(path.join(out,'evidence.json'),JSON.stringify(report,null,2));await evidence();
const root=path.dirname(native.original_manifest),prefix='plans/original/',files_base64={},originals=[];
for(const f of [record.rplan_path,record.rctx_path,record.audit_certificate_path]){
 assert(['plan.rplan','context.rctx','audit-certificate.json'].includes(f));
 files_base64[prefix+f]=(await fs.readFile(path.join(root,f))).toString('base64');
}
const inputRoot=await fs.realpath(path.join(nativeCase,'inputs'));assert.equal(inputRoot,await fs.realpath(native.input_root));
const names=new Set();for(const s of record.native_replay_recipe.sources){
 const f=await fs.realpath(s.path);assert(f.startsWith(inputRoot+path.sep));const bytes=await fs.readFile(f);assert.equal(sha(bytes),s.sha256.replace(/^sha256:/,''));
 const key=sha(bytes);if(names.has(key))continue;names.add(key);originals.push({name:s.role+'.selected',base64:bytes.toString('base64')});
}
const sources={prechecks:{manifest_json:manifest.toString('utf8'),manifest_path:prefix+'manifest.json',files_base64},sources:originals};
const settings={label:'BrowserReplayRI',created_at:'2026-11-06T00:00:00Z'};
const req={operation:'run-native-recorded-replay',input:{sources,...settings}};
const write=(n,b)=>fs.writeFile(path.join(out,n),typeof b==='string'?b:JSON.stringify(b,null,2));
await write('request.json',req);
const wasm=await fs.readFile(path.join(site,'bisect_wasm.wasm'));report.wasm_sha256=sha(wasm);
let unsupported;
try{
 const noRuntime=await instantiateEngine(wasm);assert.equal(noRuntime.supportsCMetis,mode==='c');
 unsupported=noRuntime.execute(req);await write('unselected-wasm.json',unsupported);
 assert.equal(unsupported.status,'preparation-failed',unsupported.error);
 assert.equal(unsupported.execution_performed,false);assert.equal(unsupported.recorded_paths_opened,false);
 if(mode==='c'){
  const base=path.join(site,'c-metis'),manifest=await json(path.join(base,'metis-sidecar.json'));
  const sidecar=await loadMetisSidecar({manifest,javascript:await fs.readFile(path.join(base,manifest.javascript.file)),wasm:await fs.readFile(path.join(base,manifest.wasm.file))});
  const engine=await instantiateEngine(wasm,{metisSidecar:sidecar});const r=engine.execute(req);await write('current-wasm.json',r);
  assert.equal(r.status,'match',r.error??JSON.stringify(r.source_check));assert.equal(r.execution_performed,true);
  assert.deepEqual(r.comparison.regenerated_assignments,native.assignments);assert.deepEqual(r.comparison.reference_assignments,native.assignments);
  assert.equal(r.recorded_paths_opened,false);assert.deepEqual(r.runtime_artifacts,engine.runtimeArtifacts);
  const identity=r.runtime_artifacts.metis_sidecar;assert.equal(identity.runtime,'msvcrt-v1');
  for(const kind of ['javascript','wasm']){
   const artifact=identity[kind], bytes=await fs.readFile(path.join(base,artifact.file));
   assert.equal(sha(bytes),artifact.sha256);assert.equal(bytes.length,artifact.byte_length);
  }
  report.sidecar_identity=identity;
 }
}catch(error){report.status='failed-node-wasm';report.failure=String(error.stack??error);await evidence();throw error;}
const inputs={packageVerificationInputs:{manifest:{name:prefix+'manifest.json',base64:manifest.toString('base64')},
 files:Object.fromEntries(Object.entries(files_base64).map(([name,base64])=>[name,{name,base64}]))},
 nativeReplaySources:{sources:originals},nativeRecordedReplaySettings:settings};
const project=createProject({name:'Actual Rhode Island State replay',operation:req.operation,files:inputs,constraints:[],result:null,lastOperation:null});
await write('actual-ri.bisect',project);
const tampered=structuredClone(project);const buf=Buffer.from(tampered.files.nativeReplaySources.sources[0].base64,'base64');buf[buf.length-1]^=1;
tampered.files.nativeReplaySources.sources[0].base64=buf.toString('base64');await write('changed-ri.bisect',tampered);
const server=createServer(async(req,res)=>{try{
 const rel=decodeURIComponent(new URL(req.url,'http://localhost').pathname),f=await fs.realpath(path.resolve(site,'.'+(rel==='/'?'/index.html':rel)));
 assert(f.startsWith(site+path.sep));const data=await fs.readFile(f);report.served[rel]=sha(data);
 res.setHeader('Content-Type',{'.js':'text/javascript','.html':'text/html','.css':'text/css','.json':'application/json','.wasm':'application/wasm'}[path.extname(f)]??'application/octet-stream');res.end(data);
}catch(e){if(!['/favicon.ico','/offline-manifest.json'].includes(req.url))report.errors.push([req.url,e.message]);res.writeHead(404);res.end();}});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));const origin='http://127.0.0.1:'+server.address().port;let browser;
try{
 const {chromium}=createRequire(path.join(process.env.BISECT_PLAYWRIGHT_PACKAGES,'package.json'))('playwright');
 browser=await chromium.launch({headless:true,executablePath:process.env.BISECT_CHROME_PATH??'C:/Program Files/Google/Chrome/Application/chrome.exe'});
 const ctx=await browser.newContext({acceptDownloads:true});report.browser=browser.version();
 await ctx.route('**/*',r=>new URL(r.request().url()).origin===origin?r.continue():(report.external.push(r.request().url()),r.abort()));
 await ctx.addInitScript(()=>{window.riRequests=[];window.riReplies=[];window.pickerCalls=0;window.showDirectoryPicker=()=>{window.pickerCalls++;throw Error('Replay must not request directory access');};
 const W=window.Worker;window.Worker=class extends W{constructor(url,...args){super(url,...args);this.engine=new URL(url,location.href).pathname.endsWith('/wasm-worker.js');if(this.engine)this.addEventListener('message',({data})=>{if(['result','error'].includes(data?.type))window.riReplies.push(data);});}
 postMessage(m,...args){if(this.engine&&m?.type==='run')window.riRequests.push(m);return super.postMessage(m,...args);}};});
 const page=await ctx.newPage();page.on('pageerror',e=>report.errors.push(e.message));const g=id=>page.locator('#'+id),count=()=>page.evaluate(()=>window.riRequests.length);
 await page.goto(origin);await page.waitForFunction(x=>document.getElementById('runtime-artifact').textContent.includes(x),report.wasm_sha256);
 async function open(n){const c=await count();await openWorkbenchProject(page,path.join(out,n));assert.equal(await count(),c);assert(await g('recorded-replay-plan').isDisabled());assert.equal(await g('c-metis-runtime').isChecked(),false);}
 async function run(explicit=false){const n=await count();await g('execute').click();await page.waitForFunction(n=>window.riReplies.length>n&&!document.getElementById('execute').disabled,n);const {m,r}=await page.evaluate(i=>({m:riRequests[i],r:riReplies[i]}),n);assert.equal(m.cMetis??false,explicit);assert.equal(r.type,'result',r.error);return {req:m.request,result:r.result};}
 async function download(id,name){const downloading=page.waitForEvent('download');await g(id).click();await(await downloading).saveAs(path.join(out,name));return fs.readFile(path.join(out,name),'utf8');}
 await open('actual-ri.bisect');const unselected=await run();
 assert.deepEqual(unselected.req,req);assert.deepEqual(unselected.result,unsupported);
 assert(await g('recorded-replay-plan').isDisabled());await write('unselected-browser.json',unselected.result);
 assert(!Object.keys(report.served).some(p=>p.startsWith('/c-metis/')));
 report.unselected_refusal=unsupported.error;
 if(mode==='c'){
  const want=await json(path.join(out,'current-wasm.json'));
  await g('c-metis-runtime').check();const first=await run(true);assert.deepEqual(first.req,req);assert.deepEqual(first.result,want);
  await write('actual-browser.json',first.result);
  assert.equal(await download('recorded-replay-plan','replay.rplan'),want.replay.export.document_json);
  assert.equal(await download('recorded-replay-context','replay.rctx'),want.replay.export.context_json);
  const saved=JSON.parse(await download('save-project','saved.bisect'));assert.deepEqual(saved.files.nativeReplaySources,inputs.nativeReplaySources);assert.deepEqual(saved.files.packageVerificationInputs,inputs.packageVerificationInputs);
  await open('saved.bisect');const afterOpen=await run(false);assert.deepEqual(afterOpen.result,unsupported);assert(await g('recorded-replay-plan').isDisabled());await write('unchecked-after-open.json',afterOpen.result);
  await g('c-metis-runtime').check();assert.deepEqual((await run(true)).result,want);
  await open('changed-ri.bisect');await g('c-metis-runtime').check();const refusal=await run(true);await write('changed-browser.json',refusal.result);
  assert.equal(refusal.result.status,'source-check-failed');assert.equal(refusal.result.execution_performed,false);assert.equal(refusal.result.recorded_paths_opened,false);
  assert(await g('recorded-replay-plan').isDisabled());assert.equal(await count(),5);report.unchecked_after_cached_c_refused=true;
  for(const kind of ['javascript','wasm']){const asset=report.sidecar_identity[kind];assert.equal(report.served['/c-metis/'+asset.file],asset.sha256);}
  assert(report.served['/c-metis/metis-sidecar.json']);
  report.selected_runtime_exact_native_assignments=true;report.changed_source_no_execution=true;
 }else{assert.equal(await count(),1);assert(await g('c-metis-runtime-control').isHidden());}
 assert.equal(await page.evaluate(()=>window.pickerCalls),0);assert.deepEqual(report.errors,[]);assert.deepEqual(report.external,[]);
 report.status='passed-real-ri-c-backend-browser-boundary';report.units=Object.keys(native.assignments).length;
 report.open_inert=true;report.recorded_backend='c-ffi';
}catch(err){report.status='failed-browser';report.failure=String(err.stack??err);throw err;}
finally{await evidence();await browser?.close();await new Promise(resolve=>server.close(resolve));}
console.log(JSON.stringify({mode,status:report.status,out}));
