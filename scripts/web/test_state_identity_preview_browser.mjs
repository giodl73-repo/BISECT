import {openWorkbenchProject} from './workbench_project_open.mjs';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import {createProject} from '../../web/lab/project.js';
const {chromium}=createRequire(path.join(process.env.BISECT_PLAYWRIGHT_PACKAGES,'package.json'))('playwright');
const directory=path.resolve(process.argv[2]),root=path.resolve('target/state-identity/browser-'+Date.now());await fs.mkdir(root,{recursive:true});
const server=createServer(async(req,res)=>{try{const name=new URL(req.url,'http://localhost').pathname,file=path.resolve(directory,'.'+decodeURIComponent(name==='/'?'/index.html':name));if(!file.startsWith(directory+path.sep))throw Error('Path');res.setHeader('Content-Type',file.endsWith('.js')?'application/javascript':file.endsWith('.wasm')?'application/wasm':file.endsWith('.css')?'text/css':'text/html');res.end(await fs.readFile(file));}catch{res.writeHead(404);res.end();}});await new Promise(r=>server.listen(0,'127.0.0.1',r));
const stagePath=path.resolve(directory,'../../evidence.json'),stageBytes=await fs.readFile(stagePath),stage=JSON.parse(stageBytes),hash=b=>createHash('sha256').update(b).digest('hex');assert.equal(stage.status,'passed-stage-build');
const verifyPackage=async()=>{for(const pin of stage.package_files){const bytes=await fs.readFile(pin.path);assert.equal(bytes.length,pin.byte_length);assert.equal(hash(bytes),pin.sha256,pin.path);}};await verifyPackage();
const browser=await chromium.launch({headless:true,executablePath:process.env.BISECT_TEST_BROWSER||'C:/Program Files/Google/Chrome/Application/chrome.exe'});
let page;const errors=[],outcomes=[];
try{
 const context=await browser.newContext({acceptDownloads:true}),origin='http://127.0.0.1:'+server.address().port;await context.route('**/*',r=>new URL(r.request().url()).origin===origin?r.continue():r.abort());page=await context.newPage();page.setDefaultTimeout(120000);page.on('pageerror',e=>errors.push(e.message));
 await page.addInitScript(()=>{window.identityRequests=[];const W=Worker;window.Worker=class extends W{postMessage(m,...a){if(m.type==='run')identityRequests.push(m.request.operation);return super.postMessage(m,...a);}};});
 await page.goto(origin);await page.waitForFunction(()=>!document.querySelector('#execute').disabled);
 const open=async file=>{const n=await page.evaluate(()=>identityRequests.length);await openWorkbenchProject(page,file,{ready:({argument:__bisectOpenArgument2539,openedStatus:__bisectOpenedStatus2539})=>(()=>__bisectOpenedStatus2539.startsWith('Opened'))(__bisectOpenArgument2539),readyArg:undefined});assert.equal(await page.evaluate(()=>identityRequests.length),n);assert.equal(await page.locator('#native-state-identity-preview').count(),0);};
 const base=['--state','RI','--year','2020','--adjacency','not-opened.adj.bin','--districts','2'];
 const cases=[['portable',[...base,'--state-name','Research_Area'],'2020/states/Research_Area'],['spaces',[...base,'--state-name','Research Area'],'2020/states/Research Area'],['explicit-label',[...base,'--state-name','Research_Area','--label','chosen_plan'],'2020/plans/chosen_plan'],['ignored',['--state','RI','--state-name','Ignored'],'2020/states/rhode_island'],['unsafe',[...base,'--state-name','../outside'],null],['markup',[...base,'--state-name','<img src=x onerror=alert(1)>'],null],['bad-year',['--state','RI','--year','2024','--adjacency','not-opened.adj.bin'],null]];
 for(const [name,args,relative] of cases){
  const file=path.join(root,name+'.bisect');await fs.writeFile(file,JSON.stringify(createProject({name,files:{stateCommandInputs:{arguments_json:JSON.stringify(args)}},operation:'inspect-native-state-command',constraints:[],result:null,lastOperation:null})));
  await open(file);await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#execute').disabled&&document.querySelector('#native-state-identity-preview'));
  const text=await page.locator('#native-state-identity-preview').innerText();assert.match(text,/No output folder was observed or opened/);
  if(relative)assert(text.includes(relative),text);else assert.match(text,/Naming review refused/);
  if(name==='ignored')assert.match(text,/ignores --state-name/);assert.equal(await page.locator('#native-state-identity-preview img').count(),0);
  assert((await page.locator('#state-command-output').textContent()).includes('--state-name')||name==='bad-year');
  if(name==='spaces'){
   await page.setViewportSize({width:390,height:844});assert(await page.locator('#native-state-identity-preview').evaluate(n=>n.scrollWidth<=n.clientWidth+1));await page.locator('#native-state-identity-preview').screenshot({path:path.join(root,'preview-390.png')});
   const download=page.waitForEvent('download');await page.locator('#save-project').click();const saved=path.join(root,'saved-spaces.bisect');await(await download).saveAs(saved);const retained=JSON.parse(await fs.readFile(saved,'utf8'));assert.equal(retained.result.browser_identity_preview.execution_authorized,false);assert.equal(retained.result.browser_identity_preview.relative_target,relative);await open(saved);await page.locator('#execute').click();await page.waitForFunction(()=>document.querySelector('#native-state-identity-preview'));assert((await page.locator('#native-state-identity-preview').innerText()).includes(relative));
  }
  outcomes.push({name,relative_target:relative,passed:true});
 }
 const requests=await page.evaluate(()=>identityRequests);assert(requests.every(r=>r==='inspect-native-state-command'));assert.deepEqual(errors,[]);
 await verifyPackage();await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({status:'passed',profile:stage.profile,package:directory,package_validation:{path:stagePath,sha256:hash(stageBytes)},wasm_sha256:stage.wasm.sha256,package_unchanged:true,cases:outcomes,requests,actual_worker_ui:true,inert_open_and_explicit_reinspection:true,portable_names_preserve_case_and_spaces:true,markup_rendered_as_text:true,mobile_preview_width:390,no_partition_or_folder_action:true,scope:'Actual selected-profile naming-review UI only. State name execution mapping, live target/source grants, full package/parent acceptance and deployment remain open.'},null,2));console.log(JSON.stringify({root,status:'passed',cases:outcomes.length}));
}catch(error){await fs.writeFile(path.join(root,'failure.json'),JSON.stringify({error:error.stack,errors,outcomes},null,2));if(page)await page.screenshot({path:path.join(root,'failure.png'),fullPage:true}).catch(()=>{});throw error;}finally{await browser.close();await new Promise(r=>server.close(r));}
