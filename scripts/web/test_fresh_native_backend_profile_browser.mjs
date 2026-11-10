import fs from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
const argv=process.argv.slice(2),arg=name=>{const i=argv.indexOf(name);assert(i>=0,'Supply '+name);return path.resolve(argv[i+1]);};
const site=arg('--site'),parent=arg('--output'),mode=argv.includes('--c-profile')?'c-ffi':'core';await fs.mkdir(parent,{recursive:true});
const root=parent,out=await fs.mkdtemp(path.join(parent,'backend-browser-'));
const read=async p=>JSON.parse(await fs.readFile(p,'utf8')),hash=b=>createHash('sha256').update(b).digest('hex');
const corpusBytes=await fs.readFile(new URL('./fixtures/fresh-native-backend-profile.json',import.meta.url));assert.equal(hash(corpusBytes),'96cafcd30f7004914347ddb70d2d1b023e6a7d2e66d85ff11dfc5360bf4c371a');const corpus=JSON.parse(corpusBytes);
const definition={cases:corpus.rows},native=Object.fromEntries(['core','c-ffi'].map(profile=>[profile,corpus.rows.map(row=>({id:row.id,...row.native[profile]}))]));
const images={[mode]:await fs.readFile(path.join(site,'bisect_wasm.wasm'))};
const originalIds=['bisect-metis-core','bisect-gpmetis','nway-c-ffi','ratio-optimal-omitted'];
const groups=[{wasm_image:mode,native_reference:'core',native_recursive_backend:'metis-core',ids:originalIds,kind:'original'}];
if(mode==='c-ffi')groups.push({wasm_image:mode,native_reference:'c-ffi',native_recursive_backend:'c-ffi',ids:originalIds,kind:'original'});
groups.push({wasm_image:mode,native_reference:'c-ffi',native_recursive_backend:'c-ffi',ids:['bfs-growth-gpmetis','capacity-clustering-c-ffi'],kind:'pure'},{wasm_image:mode,native_reference:'c-ffi',native_recursive_backend:'c-ffi',ids:['ilp-error-c-ffi'],kind:'ilp-error'});
const require=createRequire(path.join(process.env.BISECT_PLAYWRIGHT_PACKAGES??process.cwd(),'package.json')),{chromium}=require('playwright');
let image=mode,browser;const external=[],errors=[],requests=[],cases=[];
const server=createServer(async(req,res)=>{try{
 if(req.method!=='GET'){res.writeHead(405);res.end();return;}
 const name=decodeURIComponent(new URL(req.url,'http://localhost').pathname);requests.push(name);res.setHeader('Cache-Control','no-store');
 if(name==='/'){res.setHeader('Content-Type','text/html');res.end('<!doctype html><meta charset="utf-8"><section id="anchor"></section><script type="module" src="./bootstrap.js"></script>');return;}
 if(name==='/bootstrap.js'){res.setHeader('Content-Type','text/javascript');res.end("import {mountFreshNativeStatePanel} from './fresh-native-state-panel.js';window.profilePanel=mountFreshNativeStatePanel(document.querySelector('#anchor'));");return;}
 if(name==='/bisect_wasm.wasm'){res.setHeader('Content-Type','application/wasm');res.end(images[image]);return;}
 const base=name.startsWith('/c-metis/')?site:site;
 const f=await fs.realpath(path.resolve(base,'.'+name));if(!f.startsWith(base+path.sep)){res.writeHead(403);res.end();return;}
 res.setHeader('Content-Type',f.endsWith('.wasm')?'application/wasm':/\.m?js$/.test(f)?'text/javascript':'application/json');res.end(await fs.readFile(f));
}catch{res.writeHead(404);res.end();}});
await new Promise(r=>server.listen(0,'127.0.0.1',r));const origin='http://127.0.0.1:'+server.address().port;
try{
 browser=await chromium.launch({headless:true,executablePath:'C:/Program Files/Google/Chrome/Application/chrome.exe'});const context=await browser.newContext({acceptDownloads:true});
 await context.route('**/*',route=>{if(new URL(route.request().url()).origin!==origin){external.push(route.request().url());return route.abort();}return route.continue();});
 await context.addInitScript(()=>{const Native=window.Worker;window.profileWorkers=0;window.profileMessages=[];window.Worker=class extends Native{constructor(...a){super(...a);window.profileWorkers++;}postMessage(value,...a){window.profileMessages.push({type:value.type,operation:value.request?.operation,cMetis:value.cMetis});return super.postMessage(value,...a);}};});
 const page=await context.newPage();page.on('pageerror',e=>errors.push(e.message));page.setDefaultTimeout(45000);
 const get=k=>page.locator('[data-fresh="'+k+'"]');
 const download=async(key,filename)=>{const pending=page.waitForEvent('download');await get(key).click();const d=await pending;await d.saveAs(path.join(out,filename));return read(path.join(out,filename));};
 for(const group of groups){image=group.wasm_image;
  for(const id of group.ids){
   const c=definition.cases.find(c=>c.id===id),n=native[group.native_reference].find(n=>n.id===id);assert.equal(n.exit,group.kind==='ilp-error'?1:0);
   const label=image+'-'+group.native_recursive_backend+'-'+id,input={...c.request.input,native_recursive_backend:group.native_recursive_backend},loadC=group.kind==='original'&&group.native_recursive_backend==='c-ffi';
   await page.goto(origin);await page.locator('#fresh-native-state summary').click();
   const open=async payload=>{await get('open').setInputFiles({name:'inputs.json',mimeType:'application/json',buffer:Buffer.from(JSON.stringify(payload))});await page.waitForFunction(()=>document.querySelector('[data-fresh="status"]').textContent.startsWith('Inputs opened.'));};
   await open({schema_version:'bisect-fresh-native-state-inputs-v1',input,load_c_metis:loadC});assert.equal(await page.evaluate(()=>window.profileWorkers),0);assert.equal(await get('run').isDisabled(),true);
   const start=requests.length;await get('review').click();await page.waitForFunction(()=>!document.querySelector('[data-fresh="run"]').disabled);
   assert.equal(requests.slice(start).some(x=>x.startsWith('/c-metis/')),false,'Review must not load C');
   const review=JSON.parse(await get('review-text').textContent());assert.equal(review.native_backend.recursive_backend,group.native_recursive_backend);assert.equal(review.engine.sha256,hash(images[image]));
   assert.equal((await page.evaluate(()=>window.profileMessages)).some(m=>m.operation==='run-fresh-native-state'),false);
   const saved=await download('save',label+'.inputs.json');
   // The panel moves filename flags to the end after restoring selected raw bytes.
   // Bind that documented UI transformation explicitly; keep the native fixture untouched.
   const expectedArgs=[];for(let i=0;i<input.arguments.length;i++){if(['--adjacency','--partisan-shares'].includes(input.arguments[i])){i++;continue;}expectedArgs.push(input.arguments[i]);}expectedArgs.push('--adjacency',input.source.sources.adjacency.name,'--partisan-shares','shares.tsv');
   assert.deepEqual(saved.input.arguments,expectedArgs);assert.deepEqual(saved.input.source,input.source);
   for(const key of ['label','chamber','created_at','native_recursive_backend','auxiliary_source_name'])assert.equal(saved.input[key],input[key]);assert.deepEqual(JSON.parse(saved.input.auxiliary_json),JSON.parse(input.auxiliary_json));assert.equal(saved.load_c_metis,loadC);
   await fs.writeFile(path.join(out,label+'.original-fixture.json'),JSON.stringify(c.request,null,2));
   if(group.kind==='ilp-error'){
    assert.equal(review.native_backend.metis_used,false);assert.equal(review.native_backend.effective_backend,null);assert.equal(review.native_backend.runtime_profile,null);
    await get('run').click();await page.waitForFunction(()=>document.querySelector('[data-fresh="status"]').textContent.includes('--ilp-fallback=error'));
    const failure=await get('status').textContent();for(const key of ['plan','context','report'])assert.equal(await get(key).isDisabled(),true);
    assert.equal(requests.slice(start).some(x=>x.startsWith('/c-metis/')),false);assert.equal((await page.evaluate(()=>window.profileMessages)).filter(m=>m.operation==='run-fresh-native-state').length,1);
    await page.screenshot({path:path.join(out,label+'.png'),fullPage:true});await page.goto(origin);await page.locator('#fresh-native-state summary').click();await open(saved);assert.equal(await page.evaluate(()=>window.profileWorkers),0);for(const key of ['run','plan','context','report'])assert.equal(await get(key).isDisabled(),true);
    cases.push({id:label,natural_failure:failure,no_completed_exports:true,open_inert:true,no_c_runtime_requested:true});await fs.writeFile(path.join(out,'cases.json'),JSON.stringify(cases,null,2));continue;
   }
   await get('run').click();await page.waitForFunction(()=>document.querySelector('[data-fresh="status"]').textContent==='Run completed. Plan and physical context are ready to download.');
   const plan=await download('plan',label+'.rplan'),ctx=await download('context',label+'.rctx'),record=await download('report',label+'.run.json');
   assert.deepEqual(plan.plan,n.plan);assert.deepEqual(record.result.export.document.plan,n.plan);assert.deepEqual(ctx,JSON.parse(record.result.export.context_json));assert.equal(record.engine.sha256,hash(images[image]));assert.equal(record.result.execution_performed,true);assert.equal(record.result.recorded_paths_opened,false);assert.equal(record.result.export_error,null);
   assert.deepEqual(record.result.native_backend,review.native_backend);assert.deepEqual(record.result.export.fresh_native_state_binding.native_backend,review.native_backend);assert.deepEqual(record.result.export.fresh_native_state_binding.arguments,saved.input.arguments);assert.equal(record.result.native_backend.effective_backend,group.kind==='pure'?null:group.native_recursive_backend);if(group.kind==='pure'){assert.equal(record.result.native_backend.metis_used,false);assert.equal(record.result.native_backend.runtime_profile,null);assert.equal(record.result.native_backend.configuration_role,'unused');assert.equal(requests.slice(start).some(x=>x.startsWith('/c-metis/')),false);}
   for(const [kind,source]of Object.entries(input.source.sources))if(source)assert.equal(record.result.selected_sources[kind].sha256,hash(Buffer.from(source.base64,'base64')));
   const acceptedSeed=(record.result.executed_request.request??record.result.executed_request).options.seed;assert.equal(await get('seed').textContent(),'Accepted seed: '+acceptedSeed);
   const messages=await page.evaluate(()=>window.profileMessages);assert.equal(messages.filter(m=>m.operation==='run-fresh-native-state').length,1);
   await page.screenshot({path:path.join(out,label+'.png'),fullPage:true});
   const operationCount=(await page.evaluate(()=>window.profileMessages)).length;await get('backend-profile').selectOption(group.native_recursive_backend==='metis-core'?'c-ffi':'metis-core');
   for(const key of ['run','save','plan','context','report'])assert.equal(await get(key).isDisabled(),true);assert.equal(await get('result').textContent(),'');assert.equal(await get('review-text').textContent(),'');assert.equal((await page.evaluate(()=>window.profileMessages)).length,operationCount,'Profile edit must not execute');
   await page.goto(origin);await page.locator('#fresh-native-state summary').click();await open(saved);assert.equal(await page.evaluate(()=>window.profileWorkers),0);for(const key of ['run','plan','context','report'])assert.equal(await get(key).isDisabled(),true);
   assert.equal(await get('backend-profile').inputValue(),group.native_recursive_backend);assert.equal(await get('load-c').isChecked(),loadC);
   cases.push({id:label,native_binary_sha256:n.binary_sha256,engine_sha256:hash(images[image]),native_profile:group.native_reference,selected_profile:group.native_recursive_backend,complete_plan_match:true,source_export_binding_match:true,open_inert:true,seed_visible:true,accepted_seed:acceptedSeed,completed_profile_edit_invalidates:true,review_did_not_load_c:true,messages});await fs.writeFile(path.join(out,'cases.json'),JSON.stringify(cases,null,2));
  }
 }
 assert.equal(cases.length,mode==='c-ffi'?11:7);assert.deepEqual(external,[]);assert.deepEqual(errors,[]);
 const evidence={status:'passed-maintained-native-backend-profile-browser',scope:'Seven core or eleven C-image real Chrome Worker journeys using the supplied package and retained actual native reference plans. Includes pure-method selected C profile without a sidecar and natural ILP no-fallback failures. Native reference compilation is retained, not freshly performed by this browser gate. Review/Run, full plan/context/run-record downloads, requested profile/source binding, seed visibility, saved input restoration and inert Open. No all-state capacity, geometry map, permission, publisher authentication or broad parity acceptance claim.',cases,requests,external,page_errors:errors,main_changed:false,active_r22_changed:false,acceptance_promoted:false};await fs.writeFile(path.join(out,'evidence.json'),JSON.stringify(evidence,null,2));console.log(JSON.stringify({status:evidence.status,cases:cases.length,evidence:path.join(out,'evidence.json')}));
}catch(error){await fs.writeFile(path.join(out,'failure.json'),JSON.stringify({status:'failed',error:error.stack,cases,requests,external,page_errors:errors},null,2));throw error;
}finally{if(browser)await browser.close();await new Promise(r=>server.close(r));}
