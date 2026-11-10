import fs from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {createCompressedWorkbenchArchive,parseWorkbenchArchiveBytes} from '../../web/lab/workbench-archive.js';
import {freshNativeAuxiliarySources,freshNativeAuxiliaryJson} from '../../web/lab/fresh-native-state-auxiliary.js';
const argv=process.argv.slice(2),arg=name=>{const i=argv.indexOf(name);assert(i>=0,'Supply '+name);return path.resolve(argv[i+1]);};
const site=arg('--site'),parent=arg('--output'),cli=arg('--cli');await fs.mkdir(parent,{recursive:true});
const out=await fs.mkdtemp(path.join(parent,'auxiliary-browser-'));
const rows=JSON.parse(await fs.readFile(new URL('./fixtures/fresh-native-absence.json',import.meta.url),'utf8'));
const engine=await instantiateEngine(await fs.readFile(path.join(site,'bisect_wasm.wasm')));
const require=createRequire(path.join(process.env.BISECT_PLAYWRIGHT_PACKAGES,'package.json')),{chromium}=require('playwright');
const html='<!doctype html><html><head><meta charset="utf-8"></head><body><main><section id="practitioner-flow"></section></main><script type="module" src="./fresh-native-state-panel.js"></script></body></html>';
const server=createServer(async(req,res)=>{try{const name=decodeURIComponent(new URL(req.url,'http://localhost').pathname);if(name==='/panel.html'){res.setHeader('Content-Type','text/html');res.end(html);return;}const file=path.resolve(site,'.'+name);assert(file.startsWith(site+path.sep));res.setHeader('Content-Type',file.endsWith('.wasm')?'application/wasm':file.endsWith('.js')?'text/javascript':file.endsWith('.css')?'text/css':'text/html');res.end(await fs.readFile(file));}catch{res.writeHead(404);res.end();}});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));const origin='http://127.0.0.1:'+server.address().port;
const evidence={status:'running',native_cli_sha256:createHash('sha256').update(await fs.readFile(cli)).digest('hex'),runtime_artifact:engine.artifactIdentity,cases:[],refusals:[],scope:'Three actual State CLI partition comparisons, exact raw-source selection, actual Chrome worker Run and source-preserving inert Save/Open/project/archive. Explicit missing-source claims; no new Rust compilation or C computation, report observations or full release claim.'};
let browser,context;const errors=[],external=[];
try{
 browser=await chromium.launch({headless:true,executablePath:'C:/Program Files/Google/Chrome/Application/chrome.exe'});context=await browser.newContext({acceptDownloads:true});
 await context.route('**/*',route=>{if(new URL(route.request().url()).origin!==origin){external.push(route.request().url());return route.abort();}return route.continue();});
 await context.addInitScript(()=>{const Native=Worker;window.operations=[];window.Worker=class extends Native{postMessage(v,...args){if(v?.request?.operation)window.operations.push(v.request.operation);return super.postMessage(v,...args);}};});
 const page=await context.newPage();page.on('pageerror',e=>errors.push(e.message));
 const get=name=>page.locator('[data-fresh="'+name+'"]');
 const waitRun=()=>page.waitForFunction(()=>!document.querySelector('[data-fresh="plan"]').disabled);
 const download=async(name,file)=>{const pending=page.waitForEvent('download');await get(name).click();const d=await pending;await d.saveAs(file);return fs.readFile(file,'utf8');};
 for(const row of rows){
  const nativeRoot=path.join(out,row.kind+'-native');await fs.mkdir(nativeRoot);
  for(const [kind,file]of Object.entries(row.files)){
   const destination=path.resolve(nativeRoot,row.native_source_paths[kind]);assert(destination.startsWith(nativeRoot+path.sep));
   await fs.mkdir(path.dirname(destination),{recursive:true});await fs.writeFile(destination,Buffer.from(file.base64,'base64'));
  }
  const command=['state',...row.input.arguments,'--output-dir',path.join(nativeRoot,'outputs'),'--version','vAux','--dpi','72'];
  const run=spawnSync(cli,command,{cwd:nativeRoot,windowsHide:true,maxBuffer:16*1024*1024});
  await fs.writeFile(path.join(nativeRoot,'stdout.txt'),run.stdout??'');await fs.writeFile(path.join(nativeRoot,'stderr.txt'),run.stderr??'');
  await fs.writeFile(path.join(nativeRoot,'command.json'),JSON.stringify(command));assert.equal(run.status,0,run.error?.message??run.stderr?.toString());
  const native=JSON.parse(await fs.readFile(path.join(nativeRoot,'outputs/2020/states/alabama/plan.rplan'),'utf8'));assert.deepEqual(native.plan,row.reference_plan);
  const direct=engine.execute({operation:'run-fresh-native-state',input:row.input});assert.equal(direct.export_error,null);assert.deepEqual(direct.export.document.plan,native.plan);
  await fs.writeFile(path.join(out,row.kind+'.direct.json'),JSON.stringify(direct));
  await page.goto(origin+'/panel.html');await page.locator('#fresh-native-state summary').click();
  for(const [kind,file]of Object.entries(row.files)){await get(kind).setInputFiles({name:file.name,mimeType:'application/octet-stream',buffer:Buffer.from(file.base64,'base64')});await page.waitForFunction(()=>!document.querySelector('[data-fresh="review"]').disabled);}
  await get(row.kind+'-missing-name').fill(row.missing.name);await get(row.kind+'-mark-missing').click();
  const args=[];for(let i=0;i<row.input.arguments.length;i++){if(['--adjacency','--partisan-shares','--coi-weights'].includes(row.input.arguments[i])){i++;continue;}args.push(row.input.arguments[i]);}
  for(const [name,value]of [['arguments',JSON.stringify(args)],['state','AL'],['year','2020'],['created',row.input.created_at]])await get(name).fill(value);
  await get('review').click();await page.waitForFunction(()=>!document.querySelector('[data-fresh="run"]').disabled);
  const saved=JSON.parse(await download('save',path.join(out,row.kind+'.inputs.json')));
  const restoredSources=freshNativeAuxiliarySources(saved.input.auxiliary_json);assert.deepEqual(restoredSources,{[row.kind]:{name:row.missing.name,missing:true}});
  assert.deepEqual(saved.input.source.sources,row.input.source.sources);
  const expected=engine.execute({operation:'run-fresh-native-state',input:saved.input});assert.deepEqual(expected.export.document.plan,native.plan);
  await get('run').click();await waitRun();const record=JSON.parse(await download('report',path.join(out,row.kind+'.run.json')));assert.deepEqual(record.input,saved.input);assert.deepEqual(record.result,expected);
  const session=await page.evaluate(async()=>{const m=await import('./fresh-native-state-panel.js');return m.freshNativeStatePanel.snapshot();});
  assert.deepEqual(session.sources[row.kind],{name:row.missing.name,missing:true});
  const project=createProject({name:'Auxiliary sources',operation:'run-tui-configured',constraints:[],files:{freshNativeStateSession:session},result:null,lastOperation:null});
  assert.deepEqual(parseProject(JSON.stringify(project)).files.freshNativeStateSession,session);
  const archive=await createCompressedWorkbenchArchive(project);assert.deepEqual((await parseWorkbenchArchiveBytes(archive)).files.freshNativeStateSession,session);
  await fs.writeFile(path.join(out,row.kind+'.bisect-workbench.gz'),archive);
  const before=await page.evaluate(()=>window.operations.length);await get('open').setInputFiles({name:'inputs.json',mimeType:'application/json',buffer:Buffer.from(JSON.stringify(saved))});
  await page.waitForFunction(()=>document.querySelector('[data-fresh="status"]').textContent.startsWith('Inputs opened.'));
  assert.equal(await page.evaluate(()=>window.operations.length),before);assert(await get('run').isDisabled());assert(await get('plan').isDisabled());
  await get('review').click();await page.waitForFunction(()=>!document.querySelector('[data-fresh="run"]').disabled);await get('run').click();await waitRun();
  assert.deepEqual(JSON.parse(await download('report',path.join(out,row.kind+'.reopened-run.json'))).result,expected);
  const beforeEdit=await page.evaluate(()=>window.operations.length);
  await get(row.kind+'-missing-name').fill('changed-missing.csv');assert(await get('run').isDisabled());assert(await get('plan').isDisabled());
  assert.equal(await page.evaluate(async kind=>(await import('./fresh-native-state-panel.js')).freshNativeStatePanel.snapshot().sources[kind],row.kind),undefined);
  await get(row.kind+'-mark-missing').click();assert.deepEqual(await page.evaluate(async kind=>(await import('./fresh-native-state-panel.js')).freshNativeStatePanel.snapshot().sources[kind],row.kind),{name:'changed-missing.csv',missing:true});
  await get(row.kind).setInputFiles({name:'replacement.csv',mimeType:'text/plain',buffer:Buffer.from('x')});await page.waitForFunction(()=>!document.querySelector('[data-fresh="review"]').disabled);
  assert.deepEqual(await page.evaluate(async kind=>(await import('./fresh-native-state-panel.js')).freshNativeStatePanel.snapshot().sources[kind],row.kind),{name:'replacement.csv',base64:'eA=='});
  await page.evaluate(async value=>(await import('./fresh-native-state-panel.js')).freshNativeStatePanel.restore(value),session);assert(await get('run').isDisabled());assert(await get('plan').isDisabled());assert.equal(await page.evaluate(()=>window.operations.length),beforeEdit);
  assert.deepEqual(await page.evaluate(async kind=>(await import('./fresh-native-state-panel.js')).freshNativeStatePanel.snapshot().sources[kind],row.kind),{name:row.missing.name,missing:true});
  await get(row.kind).setInputFiles([]);assert(await get('run').isDisabled());assert(await get('plan').isDisabled());
  evidence.cases.push({kind:row.kind,actual_cli_plan_exact:true,whole_worker_reply_exact:true,raw_sources_exact:true,explicit_runs:2,save_open_inert:true,project_archive_exact:true,source_edit_retires_result:true,source_edit_transitions:true,inert_typed_session_restore:true});
 }
 for(const row of JSON.parse(await fs.readFile(new URL('./fixtures/fresh-native-absence-failures.json',import.meta.url),'utf8'))){
  const nativeRoot=path.join(out,row.kind+'-native-refusal');await fs.mkdir(nativeRoot);
  for(const [kind,file]of Object.entries(row.files)){const destination=path.resolve(nativeRoot,row.native_source_paths[kind]);assert(destination.startsWith(nativeRoot+path.sep));await fs.mkdir(path.dirname(destination),{recursive:true});await fs.writeFile(destination,Buffer.from(file.base64,'base64'));}
  const command=['state',...row.input.arguments,'--output-dir',path.join(nativeRoot,'outputs'),'--version','vAbsent','--dpi','72'];const run=spawnSync(cli,command,{cwd:nativeRoot,windowsHide:true,maxBuffer:16*1024*1024});assert.equal(run.status,1);await fs.writeFile(path.join(nativeRoot,'command.json'),JSON.stringify(command));await fs.writeFile(path.join(nativeRoot,'stderr.txt'),run.stderr??'');
  let error;try{engine.execute({operation:'run-fresh-native-state',input:row.input});}catch(e){error=e.message;}assert.equal(typeof error,'string');
  await page.goto(origin+'/panel.html');await page.locator('#fresh-native-state summary').click();
  for(const [kind,file]of Object.entries(row.files)){await get(kind).setInputFiles({name:file.name,mimeType:'application/octet-stream',buffer:Buffer.from(file.base64,'base64')});await page.waitForFunction(()=>!document.querySelector('[data-fresh="review"]').disabled);}
  await get(row.kind+'-missing-name').fill(row.missing.name);await get(row.kind+'-mark-missing').click();const args=[];for(let i=0;i<row.input.arguments.length;i++){if(['--adjacency','--partisan-shares','--coi-weights'].includes(row.input.arguments[i])){i++;continue;}args.push(row.input.arguments[i]);}
  for(const [name,value]of [['arguments',JSON.stringify(args)],['state','AL'],['year','2020'],['created',row.input.created_at]])await get(name).fill(value);
  await get('review').click();await page.waitForFunction(()=>!document.querySelector('[data-fresh="run"]').disabled);await get('run').click();await page.waitForFunction(message=>document.querySelector('[data-fresh="status"]').textContent===message&&!document.querySelector('[data-fresh="review"]').disabled,error);assert(await get('run').isDisabled());assert(await get('plan').isDisabled());assert.deepEqual(await page.evaluate(()=>window.operations),['prepare-fresh-native-state-run','run-fresh-native-state']);
  evidence.refusals.push({kind:row.kind,native_exit:run.status,native_stderr:run.stderr.toString(),wasm_error:error,browser_run_refused:true,no_completed_plan:true});
 }
 const source={source_name:'selected.csv',bytes_base64:'eA=='};
 for(const [name,aux]of [['derived',{population:[]}],['unknown',{native_auxiliary:{unknown:source}}],['ambiguous-absence',{native_auxiliary:{economic:{...source,bytes_base64:null,missing:true}}}],['unsafe-pad',{native_auxiliary:{housing:{...source,bytes_base64:'eB=='}}}],['empty',{native_auxiliary:{}}],['metadata',{native_auxiliary:{partisan:{...source,grant:true}}}]]){let message;try{freshNativeAuxiliarySources(JSON.stringify(aux));}catch(error){message=error.message;}assert.equal(typeof message,'string');
  const before=await page.evaluate(()=>window.operations.length),invalid={schema_version:'bisect-fresh-native-state-inputs-v1',input:{...rows.at(-1).input,auxiliary_json:JSON.stringify(aux)}};
  await get('open').setInputFiles({name:name+'.json',mimeType:'application/json',buffer:Buffer.from(JSON.stringify(invalid))});
  await page.waitForFunction(message=>document.querySelector('[data-fresh="status"]').textContent===message&&!document.querySelector('[data-fresh="review"]').disabled,message);
  assert.equal(await page.evaluate(()=>window.operations.length),before);assert(await get('run').isDisabled());assert(await get('plan').isDisabled());evidence.refusals.push(name);}
 assert.equal(freshNativeAuxiliaryJson({}),null);assert.deepEqual(errors,[]);assert.deepEqual(external,[]);evidence.browser=browser.version();evidence.status='passed-three-native-cli-absence-ui-workflows';
}catch(error){evidence.status='failed';evidence.error={message:error.message,stack:error.stack};process.exitCode=1;}
finally{await fs.writeFile(path.join(out,'evidence.json'),JSON.stringify(evidence,null,2));await context?.close();await browser?.close();await new Promise(resolve=>server.close(resolve));}
console.log(JSON.stringify(evidence));
