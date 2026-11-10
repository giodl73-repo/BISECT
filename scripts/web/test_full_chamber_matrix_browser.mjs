// Real GUI/worker integration against a completed, immutable-input matrix run.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import {createProject} from '../../web/lab/project.js';
import {digest} from './fixtures/real-data-matrix-case.mjs';
import {preflightMatrixPrerequisites} from './fixtures/matrix-prerequisites.mjs';
async function main(){
const packages=process.env.BISECT_PLAYWRIGHT_PACKAGES;assert(packages,'Set BISECT_PLAYWRIGHT_PACKAGES.');
const {chromium}=createRequire(path.join(packages,'package.json'))('playwright');
assert(process.argv[2],'Supply the completed matrix run directory.');
const prerequisiteRoot=path.resolve('target/full-chamber-matrix/browser-prerequisites-'+Date.now());
if(!await preflightMatrixPrerequisites(prerequisiteRoot,[path.join(process.argv[2],'declared-matrix.json'),path.join(process.argv[2],'evidence.json')]))return;
const run=path.resolve(process.argv[2]),toolkit=path.resolve(process.argv[3]||'web/lab');
const manifestBytes=await fs.readFile(path.join(run,'declared-matrix.json')),originalManifest=JSON.parse(manifestBytes),baseline=JSON.parse(await fs.readFile(path.join(run,'evidence.json'),'utf8'));
assert.equal(baseline.manifest_sha256,digest(manifestBytes));assert(baseline.rows.every(r=>r.status==='passed-successful-full-chamber-wasm'));const campaign=originalManifest.followup_campaigns.find(c=>c.id===baseline.campaign_id),manifest={...originalManifest,rows:campaign.rows},sourceTree=toolkit===path.resolve('web/lab');
const wasm=sourceTree?path.resolve(baseline.wasm.path):path.join(toolkit,'bisect_wasm.wasm');if(!await preflightMatrixPrerequisites(prerequisiteRoot,[wasm,...baseline.rows.map(r=>path.join(run,r.state,'request.json')),...campaign.rows.map(r=>r.baseline_native.path)],campaign.rows))return;assert.equal(digest(await fs.readFile(wasm)),baseline.wasm.sha256);
const root=path.join(run,'browser-'+Date.now());await fs.mkdir(root);const assets=new Map();
const server=createServer(async(req,res)=>{try{const url=new URL(req.url,'http://localhost'),file=url.pathname==='/bisect_wasm.wasm'?wasm:path.resolve(toolkit,'.'+decodeURIComponent(url.pathname==='/'?(sourceTree?'/workbench.html':'/index.html'):url.pathname));if(file!==wasm&&!file.startsWith(toolkit+path.sep))throw new Error('Path');const bytes=await fs.readFile(file);assets.set(file,{path:file,sha256:digest(bytes),byte_length:bytes.length});res.setHeader('Content-Type',file.endsWith('.js')?'application/javascript':file.endsWith('.wasm')?'application/wasm':file.endsWith('.css')?'text/css':'text/html');res.end(bytes);}catch{res.writeHead(404);res.end();}});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));const origin='http://127.0.0.1:'+server.address().port;let browser;
const rows=[];
try{
 browser=await chromium.launch({headless:true,executablePath:process.env.BISECT_TEST_BROWSER||'C:/Program Files/Google/Chrome/Application/chrome.exe',args:['--no-first-run','--disable-background-networking','--disable-extensions']});
 for(const prior of baseline.rows){
  const row=manifest.rows.find(row=>row.id===prior.id),requestText=await fs.readFile(path.join(run,row.state,'request.json'),'utf8');assert.equal(digest(requestText),row.request_sha256);const request=JSON.parse(requestText),r=request.input.run;
  const files={configSource:{name:r.source_name,base64:Buffer.from(r.yaml_text).toString('base64')},configSettings:{profile:r.profile},configRunInputs:{graph:{name:row.state+'-'+row.year+'-graph.json',base64:Buffer.from(JSON.stringify(r.graph)).toString('base64')}},configRunSettings:{chamber:r.chamber,districts:r.districts,native_build:true,capture_rounds:false},tuiRunSettings:{form:request.input.form,created_at:request.input.created_at}};
  const project=createProject({name:'Matrix '+row.state,files,operation:request.operation,constraints:[],result:null,lastOperation:null}),file=path.join(root,row.state+'.bisect');await fs.writeFile(file,JSON.stringify(project));
  const context=await browser.newContext({acceptDownloads:true});await context.route('**/*',route=>new URL(route.request().url()).origin===origin?route.continue():route.abort());const page=await context.newPage(),errors=[];page.on('pageerror',error=>errors.push(error.message));
  await page.addInitScript(()=>{window.matrixRequests=[];window.matrixReplies=[];const Original=Worker;window.Worker=class extends Original{constructor(...args){super(...args);this.addEventListener('message',event=>{if(event.data?.type==='ready')matrixReplies.push(event.data);});}postMessage(message,...rest){if(message?.type==='run')matrixRequests.push(message.request);return super.postMessage(message,...rest);}};});
  console.log(row.state+' actual packaged GUI');await page.goto(origin);await page.locator('#open-project').setInputFiles(file);await page.waitForFunction(state=>document.querySelector('#project-name').value==='Matrix '+state&&document.querySelector('#tui-run-seed').value==='42'&&!document.querySelector('#execute').disabled,row.state,{timeout:120000});
  assert.equal(await page.evaluate(()=>matrixRequests.length),0,'Open executed a request');assert.equal(await page.locator('#download-tui-run').isDisabled(),true);
  await page.locator('#execute').click();await page.waitForFunction(()=>window.matrixRequests.length>0&&!document.querySelector('#execute').disabled,null,{timeout:300000});
  const requests=await page.evaluate(()=>matrixRequests);assert.equal(requests.length,1);assert.equal(digest(JSON.stringify(requests[0])),row.request_sha256,'GUI request differs from predeclared native request');
  const status=await page.locator('#result-status').innerText();
  if(row.expected_native.status==='failed'){assert(status.includes(row.expected_native.error),status);assert.equal(await page.locator('#download-tui-run').isDisabled(),true);}
  else{
   assert.equal(await page.locator('#download-tui-run').isDisabled(),false,status);const expected=JSON.parse(await fs.readFile(row.baseline_native.path,'utf8'));
   const waiting=page.waitForEvent('download');await page.locator('#save-project').click();const download=await waiting,savedFile=path.join(root,row.state+'-completed.bisect');await download.saveAs(savedFile);const saved=JSON.parse(await fs.readFile(savedFile,'utf8'));assert.deepEqual(saved.result,expected);
   assert((await page.locator('#result-tables').innerText()).includes('42'));const before=await page.evaluate(()=>matrixRequests.length);await page.locator('#open-project').setInputFiles(savedFile);await page.waitForFunction(()=>document.querySelector('#result-status').textContent.startsWith('Imported result is unverified'));assert.equal(await page.evaluate(()=>matrixRequests.length),before);assert.equal(await page.locator('#download-tui-run').isDisabled(),true);
   // Reopening never runs. A second explicit Run reproduces the accepted output.
   await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#download-tui-run').disabled,null,{timeout:300000});const repeatDownload=page.waitForEvent('download');await page.locator('#save-project').click();const repeatFile=path.join(root,row.state+'-replayed.bisect');await(await repeatDownload).saveAs(repeatFile);assert.deepEqual(JSON.parse(await fs.readFile(repeatFile,'utf8')).result,expected);
   await page.setViewportSize({width:390,height:844});assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1));await page.screenshot({path:path.join(root,row.state+'-reopened-390.png')});
  }
  assert.deepEqual(errors,[]);rows.push({id:row.id,status:'passed-full-chamber-gui-run-save-open-replay',request_sha256:row.request_sha256,open_inert:true,explicit_replay_exact:true,seed_visible:row.seed,result_status:status,map_report_recovery_install:'not-run'});await context.close();
 }
 await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({schema_version:'bisect-full-chamber-matrix-browser-v1',campaign_id:campaign.id,source_tree:sourceTree,package:toolkit,manifest_sha256:baseline.manifest_sha256,wasm_sha256:baseline.wasm.sha256,browser:browser.version(),rows,served_assets:[...assets.values()],scope:'Actual full-chamber NC/CA GUI and worker, exact native/CLI-validated baseline outputs, inert Save/Open and explicit replay. Source modules or exact supplied package runtime as recorded. No provider acquisition, fresh derivation, maps/reports, recovery, installation or national capacity claim.'},null,2)+'\n');console.log('Browser evidence '+path.join(root,'evidence.json'));
}finally{await browser?.close();await new Promise(resolve=>server.close(resolve));}

}
await main();
