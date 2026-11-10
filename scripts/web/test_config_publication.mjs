import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {reviewConfigPublication,applyConfigPublication,retireConfigPublicationReview,validateConfigPublicationSettings} from '../../web/lab/config-publication.js';
import {configTemplateRequest} from '../../web/lab/config-template.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {nativeOutputDirectoryHarness} from './fixtures/native-output-directory.mjs';
const option=n=>process.argv.find(a=>a.startsWith('--'+n+'='))?.slice(n.length+3),reference=option('reference');if(!reference)throw Error('Pass --reference=<actual Config CLI cases>.');
const native=JSON.parse(await fs.readFile(path.join(reference,'evidence.json'),'utf8'));assert.equal(native.status,'passed-actual-CLI-config-contract');
const root=await fs.mkdtemp(path.resolve('target/config-cli-contract/publication-')),harness=nativeOutputDirectoryHarness(root),hash=b=>createHash('sha256').update(b).digest('hex');
const wasm=await fs.readFile(native.wasm.path);assert.equal(hash(wasm),native.wasm.sha256);const engine=await instantiateEngine(wasm),base=native.records.find(r=>r.kind==='create'),settings=base.request.settings,result=engine.execute(configTemplateRequest(settings)),recipe={path:'',force:false,dry_run:false},rows=[];
const saveCase=async(name,fn)=>{const value=await fn();rows.push({name,...value});await fs.writeFile(path.join(root,'progress.json'),JSON.stringify(rows,null,2)+'\n');};
for(const row of native.records.filter(r=>r.kind==='create'))await saveCase('native-'+row.name,async()=>{
 const dir=await harness.directory(row.name),review=await reviewConfigPublication(engine,row.result,row.request.settings,recipe,dir),receipt=await applyConfigPublication(review,{confirmed:true});assert.equal(receipt.status,'completed');assert(receipt.readback_verified);assert.deepEqual(await fs.readFile(path.join(dir.dir,review.path)),Buffer.from(row.result.yaml_text));assert.deepEqual(receipt.created_parents,['configs']);await assert.rejects(applyConfigPublication(review,{confirmed:true}));return {receipt};
});
await saveCase('default-refusal-and-force',async()=>{
 const dir=await harness.directory('collision'),file=path.join(dir.dir,'custom.yml');await fs.writeFile(file,'previous bytes');let refused;
 try{await reviewConfigPublication(engine,result,settings,{...recipe,path:'custom.yml'},dir);}catch(error){refused=error.configPublicationReceipt;}assert.equal(refused.status,'refused');assert.equal(await fs.readFile(file,'utf8'),'previous bytes');
 const review=await reviewConfigPublication(engine,result,settings,{...recipe,path:'custom.yml',force:true},dir);assert.equal(review.previous_sha256,hash('previous bytes'));const receipt=await applyConfigPublication(review,{confirmed:true});assert.equal(await fs.readFile(file,'utf8'),result.yaml_text);return {refused,receipt};
});
await saveCase('dry-run-never-observes-root',async()=>{
 const bomb=new Proxy({},{get(){throw Error('Dry-run touched destination');}}),review=await reviewConfigPublication(engine,result,settings,{path:'C:/ignored/../native-out.yml',force:true,dry_run:true},bomb),receipt=await applyConfigPublication(review,{confirmed:true});assert.equal(receipt.status,'dry-run');assert.equal(receipt.effects_may_have_started,false);return {receipt};
});
await saveCase('fresh-native-and-snapshot',async()=>{
 const dir=await harness.directory('recompute');await assert.rejects(reviewConfigPublication(engine,{...result,yaml_text:'forged'},settings,recipe,dir),/fresh native/);await assert.rejects(reviewConfigPublication(engine,result,{...settings,workers:7},recipe,dir),/fresh native/);assert.deepEqual(await fs.readdir(dir.dir),[]);
 const review=await reviewConfigPublication(engine,result,settings,recipe,dir);await assert.rejects(applyConfigPublication(JSON.parse(JSON.stringify(review)),{confirmed:true}),/imported or consumed/);await assert.rejects(applyConfigPublication(review),/Confirm/);retireConfigPublicationReview(review);await assert.rejects(applyConfigPublication(review,{confirmed:true}),/Review/);return {forged_refused:true};
});
await saveCase('stale-target',async()=>{
 const dir=await harness.directory('stale'),review=await reviewConfigPublication(engine,result,settings,recipe,dir);await fs.mkdir(path.join(dir.dir,'configs'));await fs.writeFile(path.join(dir.dir,review.path),'external');let receipt;try{await applyConfigPublication(review,{confirmed:true});}catch(e){receipt=e.configPublicationReceipt;}assert.equal(receipt.effects_may_have_started,false);assert.equal(await fs.readFile(path.join(dir.dir,review.path),'utf8'),'external');return {receipt};
});
await saveCase('parent-file-and-case-alias',async()=>{
 const dir=await harness.directory('parents');await fs.writeFile(path.join(dir.dir,'configs'),'not a directory');await assert.rejects(reviewConfigPublication(engine,result,settings,recipe,dir),/parent is a file/);
 const alias=await harness.directory('alias');await fs.mkdir(path.join(alias.dir,'Configs'));await assert.rejects(reviewConfigPublication(engine,result,settings,recipe,alias),/Case-colliding/);return {refused:true};
});
await saveCase('permission-loss',async()=>{
 const dir=await harness.directory('permission');dir.queryPermission=async()=> 'granted';const review=await reviewConfigPublication(engine,result,settings,recipe,dir);dir.queryPermission=async()=> 'denied';let receipt;try{await applyConfigPublication(review,{confirmed:true});}catch(e){receipt=e.configPublicationReceipt;}assert.equal(receipt.effects_may_have_started,false);assert.deepEqual(await fs.readdir(dir.dir),[]);return {receipt};
});
await saveCase('cancellation-after-parent',async()=>{
 const dir=await harness.directory('cancel-parent'),review=await reviewConfigPublication(engine,result,settings,recipe,dir),controller=new AbortController();let receipt;try{await applyConfigPublication(review,{confirmed:true,signal:controller.signal,onProgress:r=>{if(r.created_parents.length)controller.abort(Error('cancel'));}});}catch(e){receipt=e.configPublicationReceipt;}assert.equal(receipt.status,'cancelled');assert.deepEqual(receipt.created_parents,['configs']);assert.equal(receipt.write_started,false);await assert.rejects(fs.stat(path.join(dir.dir,review.path)),e=>e.code==='ENOENT');return {receipt};
});
await saveCase('write-failure-and-fresh-retry',async()=>{
 const dir=await harness.directory('disk',{fail:()=>true}),review=await reviewConfigPublication(engine,result,settings,recipe,dir);let receipt;try{await applyConfigPublication(review,{confirmed:true});}catch(e){receipt=e.configPublicationReceipt;}assert.equal(receipt.status,'interrupted');assert(receipt.file_creation_requested);assert(receipt.write_started);assert.equal(receipt.readback_verified,false);assert.equal((await fs.stat(path.join(dir.dir,review.path))).size,0);
 dir.options.fail=null;await assert.rejects(reviewConfigPublication(engine,result,settings,recipe,dir),/already exists/);const retry=await reviewConfigPublication(engine,result,settings,{...recipe,force:true},dir),completed=await applyConfigPublication(retry,{confirmed:true});assert(completed.readback_verified);return {receipt,completed};
});
await saveCase('cancel-after-close',async()=>{
 const controller=new AbortController(),dir=await harness.directory('closed'),review=await reviewConfigPublication(engine,result,settings,recipe,dir);let receipt;try{await applyConfigPublication(review,{confirmed:true,signal:controller.signal,onProgress:r=>{if(r.close_completed)controller.abort(Error('after close'));}});}catch(e){receipt=e.configPublicationReceipt;}assert(receipt.close_completed);assert.equal(receipt.readback_verified,false);assert.equal(receipt.status,'cancelled');assert.equal(await fs.readFile(path.join(dir.dir,review.path),'utf8'),result.yaml_text);return {receipt};
});
await saveCase('readback-corruption',async()=>{
 const dir=await harness.directory('corrupt',{corrupt:()=>true}),review=await reviewConfigPublication(engine,result,settings,recipe,dir);let receipt;try{await applyConfigPublication(review,{confirmed:true});}catch(e){receipt=e.configPublicationReceipt;}assert.equal(receipt.close_completed,true);assert.equal(receipt.readback_verified,false);assert.equal(receipt.status,'interrupted');return {receipt};
});
await saveCase('stale-source-and-observer-isolation',async()=>{
 const dir=await harness.directory('source'),review=await reviewConfigPublication(engine,result,settings,recipe,dir);await assert.rejects(applyConfigPublication(review,{confirmed:true,assertCurrent:()=>{throw Error('source changed');}}),/source changed/);assert.deepEqual(await fs.readdir(dir.dir),[]);
 const next=await reviewConfigPublication(engine,result,settings,recipe,dir),receipt=await applyConfigPublication(next,{confirmed:true,onProgress:()=>{throw Error('observer');}});assert.equal(receipt.status,'completed');return {receipt};
});
for(const value of [{...recipe,path:'../escape.yml'},{...recipe,path:'CON.yml'},{...recipe,path:'dir./x.yml'},{...recipe,root:{}},{...recipe,force:'true'}])assert.throws(()=>validateConfigPublicationSettings(value));
const project=createProject({name:'Config publication recipe',operation:'create-algorithm-config',files:{configTemplate:settings,configPublicationSettings:{path:'chosen/config.yml',force:true,dry_run:false}},constraints:[],result,lastOperation:'create-algorithm-config'});assert.deepEqual(parseProject(JSON.stringify(project)),project);await fs.writeFile(path.join(root,'recipe.bisect'),JSON.stringify(project));
const sources=[];for(const file of ['web/lab/config-publication.js','web/lab/config-template.js','web/lab/project.js','scripts/web/test_config_publication.mjs','scripts/web/fixtures/native-output-directory.mjs'])sources.push({path:file,sha256:hash(await fs.readFile(file))});
const evidence={status:'passed-config-publication-native-reference',sources,wasm:native.wasm,reference:{path:path.join(reference,'evidence.json'),sha256:hash(await fs.readFile(path.join(reference,'evidence.json')))},rows,project_recipe_only:true,scope:'Actual WASM renderer/native CLI YAML bytes and owned Windows files through an injected directory facade; native create/collision/force/dry-run reference plus refusal/stale/cancel/failure/readback guards. No actual browser picker grant or OS-atomic transaction claim.'};await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify(evidence,null,2)+'\n');console.log(JSON.stringify({status:evidence.status,root,cases:rows.length}));
