import fs from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
import {fileURLToPath} from 'node:url';
import {createRequire} from 'node:module';
import {createServer} from 'node:http';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {loadMetisSidecar} from '../../web/lab/metis-c-sidecar.js';
import {createProject} from '../../web/lab/project.js';
import {NATIVE_MAP_DEFAULTS} from '../../web/lab/map-export-files.js';
import {openWorkbenchProject} from './workbench_project_open.mjs';
import {addNativeMapToNationalBatch,nationalMapBatchInput,executeNationalMapBatch} from '../../web/lab/national-map-batch.js';

import {loadFineUnitMapCorpus}from'./fixtures/fine-unit-map.mjs';
const args=process.argv.slice(2),option=n=>{const i=args.indexOf(n);return i<0?null:args[i+1];};
const site=path.resolve(option('--site')),wasm=path.resolve(option('--wasm')),cbase=option('--c-sidecar')&&path.resolve(option('--c-sidecar')),mode=cbase?'c':'core',corpus=await loadFineUnitMapCorpus();
const parent=path.resolve(option('--output')||'target/fine-unit-map-browser');await fs.mkdir(parent,{recursive:true});const root=await fs.mkdtemp(path.join(parent,mode+'-'));
const sha=b=>createHash('sha256').update(b).digest('hex');
let metisSidecar=null;
if(mode==='c'){
 const base=cbase,manifest=JSON.parse(await fs.readFile(path.join(base,'metis-sidecar.json'),'utf8'));
 metisSidecar=await loadMetisSidecar({manifest,javascript:await fs.readFile(path.join(base,manifest.javascript.file)),wasm:await fs.readFile(path.join(base,manifest.wasm.file))});
}
const engine=await instantiateEngine(await fs.readFile(wasm),{metisSidecar});
const valid=corpus.valid;
const assignment=Buffer.from(corpus.cases.find(c=>c.name==='bg-full-districts').request.input.assignments_base64,'base64');
const raw={...NATIVE_MAP_DEFAULTS,source_mode:'native-assignments',raw_state:'AL',raw_year:2020,geometry_source:'tiger',tiger:valid.tiger,assignments:{name:'final_assignments.json',base64:assignment.toString('base64')},assignment_index:null,assignment_key_basis:'unit-id',dpi:72};
const rawRequest={operation:'render-native-tiger-assignment-map',input:{state:'AL',tiger:valid.tiger,assignments_base64:raw.assignments.base64,key_basis:'unit-id',dpi:72,map_type:'districts'}};
const prep=engine.execute({operation:'prepare-native-map-tiger',input:{state:'AL',tiger:valid.tiger,geometry_profile:'source-exact'}}),expectedRaw=engine.execute(rawRequest);
const planInputs={...NATIVE_MAP_DEFAULTS,plan:{name:'original-native-bg.rplan',base64:Buffer.from(valid.document_json).toString('base64')},geometry:{name:'exact-bg-geometry.json',base64:Buffer.from(prep.geometry_json).toString('base64')},dpi:72};
const planRequest={operation:'render-native-district-map',input:{document_json:valid.document_json,unit_geometry_json:prep.geometry_json,dpi:72,map_type:'districts',dissolve_method:'sequential'}},expectedPlan=engine.execute(planRequest);
const invalid=structuredClone(rawRequest);invalid.input.tiger.kind='tracts';let invalidError;try{engine.execute(invalid);}catch(error){invalidError=error.message;}assert(invalidError);
const original=corpus.cases.find(c=>c.name==='bg-full-districts').request;
const overlap=engine.execute({operation:'prepare-native-map-tiger',input:{state:'AL',tiger:original.input.tiger,geometry_profile:'source-exact'}});let overlapError;try{engine.execute({...planRequest,input:{...planRequest.input,unit_geometry_json:overlap.geometry_json}});}catch(error){overlapError=error.message;}assert.match(overlapError,/overlap/);
const national=await addNativeMapToNationalBatch(null,raw),expectedNational=await executeNationalMapBatch(engine,nationalMapBatchInput(national.source));
assert.equal(expectedNational.png_sha256,corpus.valid_national);
async function project(name,operation,files,result){const value=createProject({name,operation,files,constraints:[],result,lastOperation:result?operation:null}),file=path.join(root,name+'.bisect');await fs.writeFile(file,JSON.stringify(value));return {file,value};}
const prepareProject=await project('prepare-bg','prepare-native-map-tiger',{tigerInputs:valid.tiger,tractPreparationSettings:{state:'AL',projection:null,geometry_profile:'source-exact'},nativeMapInputs:raw},prep);
const planProject=await project('map-rplan','render-native-district-map',{nativeMapInputs:planInputs},expectedPlan);
const overlapProject=await project('overlap-refusal','render-native-district-map',{nativeMapInputs:{...planInputs,geometry:{name:'original-overlap.json',base64:Buffer.from(overlap.geometry_json).toString('base64')}}},null);
const assets={},errors=[],external=[],evidence={status:'running',mode,cases:[],scope:'Actual owned Chrome against staged index.html/core or C package. BG source selection, inert Open/Save/Open, explicit Run, wrong-unit refusal/recovery, native exact PNG, supplied-state national handoff, strict RPLAN map and retained invalid-source refusal. Synthetic geometry only; no full state/capacity, original geometry authenticity or complete fine export acceptance.'};
let context,page;
const save=()=>fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({...evidence,served_assets:assets,page_errors:errors,external_requests:external},null,2));await save();
const before={};for(const file of await fs.readdir(site)){const full=path.join(site,file);if((await fs.stat(full)).isFile())before[file]=sha(await fs.readFile(full));}
const server=createServer(async(req,res)=>{try{const pathname=new URL(req.url,'http://localhost').pathname,file=path.resolve(site,'.'+(pathname==='/'?'/index.html':decodeURIComponent(pathname)));assert(file.startsWith(site+path.sep));const bytes=await fs.readFile(file);assets[pathname]={sha256:sha(bytes),bytes:bytes.length};res.setHeader('Content-Type',file.endsWith('.wasm')?'application/wasm':/\.m?js$/.test(file)?'application/javascript':file.endsWith('.json')?'application/json':file.endsWith('.css')?'text/css':'text/html');res.end(bytes);}catch{res.writeHead(404);res.end();}});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));const origin='http://127.0.0.1:'+server.address().port;
try{
 const {chromium}=createRequire(path.join(process.env.BISECT_PLAYWRIGHT_PACKAGES,'package.json'))('playwright');
 context=await chromium.launchPersistentContext(path.join(root,'profile'),{headless:true,executablePath:process.env.BISECT_CHROME||'C:/Program Files/Google/Chrome/Application/chrome.exe',viewport:{width:1440,height:1000},args:['--no-first-run','--disable-background-networking','--disable-extensions']});
 await context.route('**/*',route=>new URL(route.request().url()).origin===origin?route.continue():(external.push(route.request().url()),route.abort()));
 page=await context.newPage();page.on('pageerror',error=>errors.push(error.stack));
 await page.addInitScript(()=>{window.mapRequests=[];window.mapReplies=[];window.pickerCalls=0;window.showDirectoryPicker=()=>{pickerCalls++;throw Error('Unexpected OS picker');};const Original=Worker;window.Worker=class extends Original{constructor(...args){super(...args);this.addEventListener('message',({data})=>{if(['result','error'].includes(data?.type))mapReplies.push(data);});}postMessage(message,...args){if(message?.type==='run')mapRequests.push(message);return super.postMessage(message,...args);}};});
 await page.goto(origin);
 async function open(file){const count=await page.evaluate(()=>mapRequests.length);await openWorkbenchProject(page,file,{ready:()=>!document.querySelector('#execute').disabled});assert.equal(await page.evaluate(()=>mapRequests.length),count);assert.equal(await page.locator('#c-metis-runtime').isChecked(),false);}
 async function run(name,expected,error=false){if(mode==='c')await page.locator('#c-metis-runtime').check();const count=await page.evaluate(()=>mapReplies.length);await page.locator('#execute').click();await page.waitForFunction(n=>mapReplies.length>n&&!document.querySelector('#execute').disabled,count,{timeout:120000});const reply=await page.evaluate(()=>mapReplies.at(-1));await fs.writeFile(path.join(root,name+'-reply.json'),JSON.stringify(reply));if(error){assert.equal(reply.type,'error');assert.equal(reply.error,expected);}else{assert.equal(reply.type,'result',reply.error);assert.deepEqual(reply.result,expected);}evidence.cases.push({name,status:error?'refused':'exact-node-wasm-match'});await save();return reply;}
 await open(prepareProject.file);assert.equal(await page.locator('#tiger-kind').inputValue(),'block-groups');assert.equal(await page.locator('#use-native-map-tiger').isDisabled(),true);
 await run('prepare-bg',prep);assert((await page.locator('#result-tables').innerText()).includes('Block-group geometry'));
 const beforeHandoff=await page.evaluate(()=>mapRequests.length);await page.locator('#use-native-map-tiger').click();assert.equal(await page.locator('#operation').inputValue(),'render-native-district-map');assert.equal(await page.evaluate(()=>mapRequests.length),beforeHandoff);assert.equal(await page.locator('#native-map-state-tiger-kind').inputValue(),'block-groups');
 await run('render-bg-raw',expectedRaw);assert((await page.locator('#result-tables').innerText()).includes('Census block groups'));assert(await page.locator('#native-map-preview').locator('img,iframe').count());
 const downloading=page.waitForEvent('download');await page.locator('#download-native-map-png').click();const downloaded=path.join(root,'native-exact-bg.png');await(await downloading).saveAs(downloaded);assert.equal(sha(await fs.readFile(downloaded)),valid.native_png_sha256);await page.locator('#native-map-preview').screenshot({path:path.join(root,'bg-map-workbench.png')});
 const saving=page.waitForEvent('download');await page.locator('#save-project').click();const saved=path.join(root,'saved-bg.bisect');await(await saving).saveAs(saved);const value=JSON.parse(await fs.readFile(saved,'utf8'));assert.deepEqual(value.files.nativeMapInputs.tiger,valid.tiger);assert.equal(value.files.nativeMapInputs.tiger.kind,'block-groups');await open(saved);assert.equal(await page.locator('#native-map-state-tiger-kind').inputValue(),'block-groups');assert.equal(await page.locator('#download-native-map-png').isDisabled(),true);assert.equal(await page.locator('#native-map-preview').locator('img,iframe').count(),0);evidence.inert_source_preserving_save_open=true;
 await page.locator('#native-map-state-tiger-kind').selectOption('tracts');await run('wrong-units-refusal',invalidError,true);await page.locator('#native-map-state-tiger-kind').selectOption('block-groups');await run('correct-units-recovery',expectedRaw);
 const count=await page.evaluate(()=>mapRequests.length);await page.locator('#native-map-add-national').click();await page.waitForFunction(()=>document.querySelector('#operation').value==='render-native-national-map-batch');assert.equal(await page.evaluate(()=>mapRequests.length),count);assert((await page.locator('#national-map-batch-states').innerText()).includes('block groups (TIGER)'));await run('national-bg',expectedNational);
 await open(planProject.file);await run('strict-valid-bg-rplan',expectedPlan);await open(overlapProject.file);await run('strict-original-overlap-refusal',overlapError,true);
 assert.deepEqual(errors,[]);assert.deepEqual(external,[]);assert.equal(await page.evaluate(()=>pickerCalls),0);for(const[file,digest]of Object.entries(before))assert.equal(sha(await fs.readFile(path.join(site,file))),digest);evidence.package_bytes_unchanged=true;evidence.browser=context.browser()?.version();evidence.status='passed-maintained-fine-map-browser';
}catch(error){evidence.status='failed-maintained-fine-map-browser';evidence.error={message:error.message,stack:error.stack};process.exitCode=1;}finally{await save();await context?.close();await new Promise(resolve=>server.close(resolve));}
console.log(JSON.stringify({root,status:evidence.status,cases:evidence.cases.length,error:evidence.error}));
