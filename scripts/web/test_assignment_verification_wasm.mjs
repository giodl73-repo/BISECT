import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync,spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const root='target/assignment-verification-wasm';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));let parity=0,cli=0;
const cases=[
 ['identical',{a:1,b:2},{a:1,b:2},.99,'match',1],
 ['relabelled',{a:1,b:2},{a:2,b:1},.99,'different',0],
 ['partial-overlap',{a:1,b:2},{a:1,c:2},.5,'match',.5],
 ['threshold-failure',{a:1,b:2},{a:1,c:2},.5001,'different',.5],
 ['larger-reference',{a:0},{a:0,b:2},.5,'match',.5],
 ['empty-reference',{a:1},{},0,'match',0],
 ['empty-reference-fails',{a:1},{},.99,'different',0],
 ['empty-original',{}, {a:1},0,'unavailable',0],
 ['both-empty',{}, {},-1,'unavailable',0],
 ['above-one',{a:1},{a:1},1.1,'different',1],
 ['negative-threshold',{a:1},{b:1},-1,'match',0],
 ['max-district',{a:4294967295},{a:4294967295},1,'match',1],
];
const outputs=[];
async function exact(input){const req={operation:'compare-verify-assignments',input},path=`${root}/native-${parity}.json`,out=engine.execute(req);await fs.writeFile(path,JSON.stringify(req));assert.deepEqual(JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[path],{encoding:'utf8',maxBuffer:8*1024*1024})),out);parity++;return out;}
const manifest=JSON.parse(await fs.readFile('target/manifest-certificate-wasm/input.json','utf8')).manifest_json;const nativeManifest={...JSON.parse(manifest),audit_certificate_path:null,label:'assignment-test'};
for(const [name,original,reference,threshold,status,similarity]of cases){
 const input={original_json:JSON.stringify(original),reference_json:JSON.stringify(reference),min_similarity:threshold},out=await exact(input);assert.equal(out.status,status);assert.equal(out.agreement.similarity,similarity);outputs.push({name,input,out});
 const dir=`${root}/cli-${cli}`,data=dir+'/outputs/v1/2020/plans/assignment-test/data';await fs.mkdir(data,{recursive:true});await fs.writeFile(dir+'/manifest.json',JSON.stringify(nativeManifest));await fs.writeFile(data+'/final_assignments.json',input.original_json);await fs.writeFile(dir+'/reference.json',input.reference_json);
 const proc=spawnSync('target/debug/bisect.exe',['verify','--manifest',dir+'/manifest.json','--output-base',dir+'/outputs','--verify-assignments-only','--plan-ref',dir+'/reference.json',`--min-similarity=${threshold}`],{encoding:'utf8'});assert.equal(proc.status,out.native_exit_code,proc.stderr);if(status==='unavailable')assert.match(proc.stderr,/Original plan not found/);else assert.match(proc.stderr,new RegExp(`similarity: ${similarity.toFixed(4).replace('.','\\.')}`));cli++;
}
const large=JSON.stringify(Object.fromEntries(Array.from({length:100000},(_,i)=>['unit'+i,i%52])));assert.equal((await exact({original_json:large,reference_json:large,min_similarity:1})).agreement.matching_units,100000);
const padded='{"a":1}'+' '.repeat(8*1024*1024-7);assert.equal((await exact({original_json:padded,reference_json:'{"a":1}',min_similarity:1})).status,'match');
let rejected=0;for(const input of [{original_json:'x'.repeat(8*1024*1024+1),reference_json:'{}',min_similarity:1},{original_json:'{"a":4294967296}',reference_json:'{}',min_similarity:1},{original_json:'{"a":-1}',reference_json:'{}',min_similarity:1},{original_json:'[]',reference_json:'{}',min_similarity:1},{original_json:'{bad',reference_json:'{}',min_similarity:1},{original_json:JSON.stringify({['x'.repeat(241)]:1}),reference_json:'{}',min_similarity:1}]){assert.throws(()=>engine.execute({operation:'compare-verify-assignments',input}));rejected++;}
const helpers=await import('../../web/lab/assignment-verification-files.js');const {createProject,parseProject}=await import('../../web/lab/project.js');
for(const {name,input,out}of outputs){const files={original:{name:'original.json',base64:Buffer.from(input.original_json).toString('base64')},reference:{name:'reference.json',base64:Buffer.from(input.reference_json).toString('base64')},minSimilarity:input.min_similarity};assert.deepEqual(helpers.assignmentVerificationRequest(files),{operation:'compare-verify-assignments',input});const project=createProject({name:'Native assignment comparison: '+name,files:{assignmentVerificationInputs:files},operation:'compare-verify-assignments',constraints:[],result:out,lastOperation:'compare-verify-assignments'});assert.deepEqual(parseProject(JSON.stringify(project)),project);await fs.writeFile(root+'/'+name+'.bisect',JSON.stringify(project));await fs.writeFile(root+'/'+name+'-result.json',JSON.stringify(out));assert.deepEqual(JSON.parse(new TextDecoder().decode(helpers.assignmentVerificationReceiptBytes(out))),out);}
const fake=(bytes,name='assignments.json')=>({name,size:bytes.length,arrayBuffer:async()=>Uint8Array.from(bytes).buffer});
const raw='{"a":4294967295}';assert.equal((await helpers.readAssignmentVerificationSource(fake(Buffer.from(raw)))).base64,Buffer.from(raw).toString('base64'));
await assert.rejects(()=>helpers.readAssignmentVerificationSource(fake(Buffer.from([255]))));await assert.rejects(()=>helpers.readAssignmentVerificationSource(fake(Buffer.from('{"a":1,"a":2}'))),/Duplicate/);
let reads=0;await assert.rejects(()=>helpers.readAssignmentVerificationSource({name:'large.json',size:8*1024*1024+1,arrayBuffer:async()=>{reads++;}}));assert.equal(reads,0);await assert.rejects(()=>helpers.readAssignmentVerificationSource({...fake(Buffer.from('{}')),size:3}),/changed/);
assert.throws(()=>helpers.validateAssignmentVerificationInputs({original:null,reference:null,minSimilarity:Infinity}));assert.throws(()=>helpers.assignmentVerificationReceiptBytes({...outputs[0].out,native_exit_code:1}));
await fs.writeFile(root+'/evidence.json',JSON.stringify({native_wasm_cases:parity,actual_cli_cases:cli,boundary_rejections:rejected,full_100000_unit_comparison:true,exact_8MiB_source:true},null,2));console.log(`Assignment verification: ${parity} native/WASM results, ${cli} actual CLI cases, ${rejected} source rejects; native denominator, literal labels, empty maps, thresholds, 100000 units and 8 MiB source passed. Project/raw bytes/receipts/pre-read/UTF-8/duplicate/size guards passed.`);
