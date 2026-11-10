import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {textComparisonRequest,validateTextComparisonInputs,readTextComparisonSource} from '../../web/lab/text-comparison-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const root='target/text-comparison';await fs.mkdir(root,{recursive:true});const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm')),fixture=JSON.parse(await fs.readFile('target/comparison-fixtures/contexts.json','utf8'));
const source=(name,text)=>({name,base64:Buffer.from(text).toString('base64')}),hash=text=>createHash('sha256').update(text).digest('hex');let cases=0;
const a=fixture.plan,b=structuredClone(a);b.plan.assignment=[0,1,2,0,1,2,0,1,2];b.metadata.label='Text comparison B';
const aText=JSON.stringify(a,null,2)+'\n',bText=JSON.stringify(b,null,2)+'\n';let project;
for(const [name,context]of Object.entries({absent:null,...fixture.contexts})){
 const contextText=context?JSON.stringify(context,null,2)+'\n':null,inputs={a:source('a.rplan',aText),b:source('b.rplan',bText),context:contextText?source('context.rctx',contextText):null},request=textComparisonRequest(inputs);
 const actual=engine.execute(request);await fs.writeFile(root+'/request.json',JSON.stringify(request));const native=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[path.resolve(root+'/request.json')],{encoding:'utf8',maxBuffer:16*1024*1024}));assert.deepEqual(actual,native);
 const {source_sha256,...metrics}=actual;assert.deepEqual(metrics,engine.execute({operation:'compare-plans',plan_a:a,plan_b:b,context}));assert.deepEqual(source_sha256,{plan_a:hash(aText),plan_b:hash(bText),context:contextText?hash(contextText):null});cases++;
 if(context?.populations&&context.graph&&context.populations.some(p=>BigInt(p)>0n))project=createProject({name:'Exact native plan comparison',files:{textComparisonInputs:inputs},operation:'compare-rplan-text',constraints:[],result:actual,lastOperation:'compare-rplan-text'});
}
assert(project);assert.deepEqual(parseProject(JSON.stringify(project)),project);await fs.writeFile(root+'/comparison.bisect',JSON.stringify(project));for(const [key,s]of Object.entries(project.files.textComparisonInputs))if(s)await fs.writeFile(root+'/'+s.name,Buffer.from(s.base64,'base64'));
const large=structuredClone(a);large.provenance.producer.large_counter='__RAW_U64__';const largeText=JSON.stringify(large).replace('"__RAW_U64__"','18446744073709551615'),request={operation:'compare-rplan-text',plan_a_json:largeText,plan_b_json:bText,context_json:null},largeResult=engine.execute(request);assert.equal(largeResult.source_sha256.plan_a,hash(largeText));
let reads=0;await assert.rejects(()=>readTextComparisonSource({name:'big.rplan',size:3*1024*1024+1,arrayBuffer:async()=>{reads++;}},'a'));assert.equal(reads,0);
for(const bad of [{a:null,b:null,context:null,extra:true},{...project.files.textComparisonInputs,a:{name:'a.rplan',base64:'//8='}}])assert.throws(()=>validateTextComparisonInputs(bad));
for(const bad of [{...request,extra:true},{...request,plan_a_json:'x'.repeat(3*1024*1024+1)},{...request,plan_b_json:JSON.stringify({...b,plan:{...b.plan,assignment:[99,...b.plan.assignment.slice(1)]}})}])assert.throws(()=>engine.execute(bad));
await fs.writeFile(root+'/evidence.json',JSON.stringify({whole_native_wasm_matches:cases,existing_measurements_unchanged:true,exact_original_byte_hashes:true,raw_max_u64_retained:true,inert_project_roundtrip:true,pre_read_size_and_invalid_source_guards:true},null,2));console.log('Exact comparison: native/WASM equality, unchanged measured/proxy/missing-context behavior, original byte hashes, raw u64 retention and safe project/source limits passed.');
