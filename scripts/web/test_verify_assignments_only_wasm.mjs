import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import {execFileSync,spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {verifyAssignmentsRequest,verifyAssignmentsReceiptBytes} from '../../web/lab/verify-assignments-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const root='target/verify-assignments-only-wasm';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const source=(name,text)=>({name,base64:Buffer.from(text).toString('base64')});
let parity=0,cli=0;
async function check(name,expected,{pre='not-recorded',label='chosen',version='verify',original=' {"a":1,"b":2} \n',reference='{"a":1,"b":2}',threshold=1,modify=null,context=true}={}){
 const selected=parseProject(await fs.readFile(`target/manifest-prechecks-wasm/${pre}.bisect`,'utf8')).files.packageVerificationInputs;
 const manifest=JSON.parse(Buffer.from(selected.manifest.base64,'base64').toString());if(pre==='not-recorded')manifest.label='fallback';selected.manifest=source('manifest.json',JSON.stringify(manifest));
 const files={};if(original!==null){const dir=`${version}/2020/plans/${label||'fallback'}`;files[dir+'/data/final_assignments.json']=source('final_assignments.json',original);if(context)files[dir+'/manifest.json']=selected.manifest;}
 const discovery={manifest:null,files,explicitLabel:label||''},settings={reference:reference===null?null:source('reference.json',reference),minSimilarity:threshold};
 // Malformed reference cases deliberately bypass the browser's source inspector.
 const safeSettings={...settings,reference:reference==='broken'?source('reference.json','{}'):settings.reference};
 const request=verifyAssignmentsRequest(selected,discovery,safeSettings);if(reference==='broken')request.input.reference_json=reference;if(modify)modify(request);
 const result=engine.execute(request);await fs.writeFile(root+'/request.json',JSON.stringify(request));const native=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[root+'/request.json'],{encoding:'utf8',maxBuffer:32*1024*1024}));assert.deepEqual(native,result);parity++;
 assert.deepEqual(result.prechecks,engine.execute({operation:'verify-manifest-prechecks',input:request.input.prechecks}));
 if(name==='verified-sidecars'){assert.equal(result.prechecks.status,'verified');assert.equal(result.prechecks.ilp_summary.status,'verified');assert.equal(result.prechecks.certificate.status,'verified');}
 assert.equal(result.status,expected);assert.equal(result.native_exit_code,expected==='match'?0:1);assert.equal(result.execution_performed,false);
 assert.deepEqual(JSON.parse(new TextDecoder().decode(verifyAssignmentsReceiptBytes(result))),result);
 const dir=await fs.mkdtemp(path.join(os.tmpdir(),'bisect-assignment-branch-')),manifestPath=path.join(dir,'package',request.input.prechecks.manifest_path),outputRoot=path.join(dir,'outputs');
 async function write(relative,bytes){await fs.mkdir(path.dirname(relative),{recursive:true});await fs.writeFile(relative,bytes);}
 for(const [p,b]of Object.entries(request.input.prechecks.files_base64))await write(path.join(dir,'package',p),Buffer.from(b,'base64'));await write(manifestPath,request.input.prechecks.manifest_json);
 for(const [p,b]of Object.entries(request.input.output_files_base64))await write(path.join(outputRoot,p),Buffer.from(b,'base64'));
 const args=['verify','--manifest',manifestPath,'--output-base',outputRoot,'--verify-assignments-only',`--min-similarity=${threshold}`];if(request.input.explicit_label)args.push('--label',request.input.explicit_label);
 if(request.input.reference_json!==null){const ref=path.join(dir,'reference.json');await write(ref,request.input.reference_json);args.push('--plan-ref',ref);}
 const proc=spawnSync('target/debug/bisect.exe',args,{encoding:'utf8'});assert.equal(proc.status,result.native_exit_code,proc.stderr);cli++;
 if(['match','different'].includes(expected))assert.match(proc.stderr,new RegExp(`similarity: ${result.comparison.agreement.similarity.toFixed(4).replace('.','\\.')}`));
 if(expected==='unavailable'){assert.equal(result.comparison,null);assert.match(proc.stderr,/Original plan not found/);}
 if(expected==='precheck-failed'){assert.equal(result.discovery,null);assert.equal(result.comparison,null);assert.doesNotMatch(proc.stderr,/assignments only/);}
 if(!modify&&reference!=='broken'){
 const project=createProject({name:'Assignments-only: '+name,files:{packageVerificationInputs:selected,assignmentDiscoveryInputs:discovery,verifyAssignmentsSettings:settings},operation:'verify-assignments-only',constraints:[],result,lastOperation:'verify-assignments-only'});assert.deepEqual(parseProject(JSON.stringify(project)),project);await fs.writeFile(`${root}/${name}.bisect`,JSON.stringify(project));
 }
 await fs.writeFile(`${root}/${name}-result.json`,JSON.stringify(result));
}
await check('match','match');
await check('verified-sidecars','match',{pre:'bound-lineage'});
await check('different','different',{reference:'{"a":2,"b":1}'});
await check('partial-overlap','match',{reference:'{"a":1,"c":2}',threshold:.5});
await check('empty-reference','match',{reference:'{}',threshold:0});
await check('negative-threshold','match',{reference:'{}',threshold:-1});
await check('above-one','different',{threshold:1.1});
await check('fallback','match',{label:null,version:'v1',context:false});
await check('international','match',{version:'international'});
await check('missing','unavailable',{original:null,reference:'broken'});
await check('empty','unavailable',{original:'{}',reference:'broken'});
await check('bad-original','discovery-failed',{original:'broken'});
await check('missing-reference','reference-required',{reference:null});
await check('bad-reference','reference-failed',{reference:'broken'});
await check('precheck-stop','precheck-failed',{pre:'bad-digest',reference:'broken'});
await check('model-stop','precheck-failed',{pre:'changed-model',original:'broken'});
const over={operation:'verify-assignments-only',input:JSON.parse(await fs.readFile(root+'/request.json','utf8')).input};over.input.output_files_base64={'a.json':Buffer.alloc(16*1024*1024,32).toString('base64')};assert.throws(()=>engine.execute(over),/Combined assignments-only sources/);
console.log(`Assignments-only composition: ${parity} native-host/WASM and ${cli} actual CLI cases passed; native stage ordering, missing/empty stops, literal agreement and receipt/project preservation.`);
