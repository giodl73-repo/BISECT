import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {columnContextRequest} from '../../web/lab/package-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {countResultTables} from '../../web/lab/count-results.js';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const root='target/branch-price-context-wasm';await fs.mkdir(root,{recursive:true});
const context=JSON.parse(await fs.readFile('crates/rplan-audit/fixtures/grid3x3.rctx','utf8'));
const settings={k:3,tolerance:0,formulation_only:false,exact_fixture_limit:12,label:'Synthetic grid column benchmark',chamber:'congressional',created_at:'2026-10-04T00:00:00Z'};
const canonical=v=>typeof v==='bigint'?v.toString():Array.isArray(v)?'['+v.map(canonical).join(',')+']':v&&typeof v==='object'?'{'+Object.keys(v).sort().map(k=>JSON.stringify(k)+':'+canonical(v[k])).join(',')+'}':JSON.stringify(v);
const hash=v=>'sha256:'+createHash('sha256').update(canonical(v)).digest('hex');
const rehash=c=>{const projection={units:c.units,graph:c.graph??null,populations:c.populations??null,source_hashes:c.source_hashes};for(const k of ['subdivisions','demographics','geometry'])if(c[k]!=null)projection[k]=c[k];c.context_hash=hash(projection);return c;};
let parity=0,rejections=0;
async function native(request){const path=root+'/request.json';await fs.writeFile(path,JSON.stringify(request));return JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[path],{encoding:'utf8',maxBuffer:64*1024*1024,stdio:['ignore','pipe','pipe']}));}
const request=(c,s=settings)=>({operation:'solve-branch-price-context',context_json:typeof c==='string'?c:canonical(c),settings:s});
async function exact(c,s=settings){const r=request(c,s),output=engine.execute(r);assert.deepEqual(output,await native(r));assert.equal(output.context_json,r.context_json);assert.equal(output.context_bytes_sha256,'sha256:'+createHash('sha256').update(r.context_json).digest('hex'));parity++;return output;}
async function reject(c,s=settings){const r=request(c,s);assert.throws(()=>engine.execute(r));await assert.rejects(()=>native(r));rejections++;}
assert.equal(rehash(structuredClone(context)).context_hash,context.context_hash);
for(const k of [1,2,3,4,9])for(const tolerance of [0,0.25]){
 const s={...settings,k,tolerance},out=await exact(context,s);
 const instance={adjacency:context.graph.adjacency.map(edges=>edges.map(e=>e.to)),populations:context.populations,k,tolerance,formulation_only:false,exact_fixture_limit:12};
 assert.deepEqual(out.report,engine.execute({operation:'solve-branch-price',instance_json:JSON.stringify(instance)}).report);
 if(out.report.solution){
  const document=JSON.parse(out.document_json);assert.deepEqual(document.plan.units,context.units);assert.deepEqual(document.plan.assignment,out.report.solution.assignment);assert.equal(document.plan.k,k);
  assert.deepEqual(document.provenance.source_hashes,context.source_hashes);assert.equal(document.provenance.producer.context_hash,context.context_hash);
  const validated=engine.execute({operation:'validate-rplan',document});assert.deepEqual(validated,await native({operation:'validate-rplan',document}));
  assert.equal(document.metadata.label,s.label);
 }else{assert.equal(out.document_json,null);assert.equal(out.plan_hash,null);}
}
const weighted=structuredClone(context);for(const edges of weighted.graph.adjacency)for(const e of edges){e.kind='bridge';e.weight=7.5;}
const weightedOut=await exact(rehash(weighted));assert.deepEqual(weightedOut.report,(await exact(context)).report);assert.equal(JSON.parse(weightedOut.context_json).graph.adjacency[0][0].kind,'bridge');
const large=structuredClone(context);large.populations=Array(9).fill(9007199254740993n);rehash(large);
assert.equal((await exact(canonical(large))).total_population,'81064793292668937');
for(const s of [{...settings,formulation_only:true},{...settings,exact_fixture_limit:8}]){const out=await exact(context,s);assert.equal(out.report.status,'formulation-only');assert.equal(out.document_json,null);assert.equal(out.report.lower_bound,null);}
for(const mutation of [c=>c.context_hash='sha256:wrong',c=>{c.populations[0]++;},c=>{delete c.graph;rehash(c);},c=>{delete c.populations;rehash(c);},c=>{c.graph.adjacency[0][0].to=99;rehash(c);},c=>{c.graph.adjacency[0].push(c.graph.adjacency[0][0]);rehash(c);},c=>{c.units.unit_ids.reverse();},c=>{c.populations[0]=0;rehash(c);},c=>{c.graph.adjacency[0].push({to:0,kind:'boundary'});rehash(c);}]){const c=structuredClone(context);mutation(c);await reject(c);}
for(const s of [{...settings,k:0},{...settings,k:10},{...settings,tolerance:-1},{...settings,tolerance:2},{...settings,exact_fixture_limit:13},{...settings,label:''},{...settings,chamber:'x'.repeat(201)},{...settings,created_at:''},{...settings,unexpected:1}])await reject(context,s);
await reject('x'.repeat(3*1024*1024+1));await reject('{');
// Valid whitespace-padded RCTX exercises the byte boundary without changing
// the native graph or formulation work bounds.
const paddedText=canonical(context)+' '.repeat(3*1024*1024);
const paddedOutput=await exact(paddedText);
assert.deepEqual(paddedOutput.report,weightedOut.report);
const paddedSource={name:'large-grid.rctx',base64:Buffer.from(paddedText).toString('base64')};
assert.equal(columnContextRequest(paddedSource,settings).context_json,paddedText);
const paddedProject=createProject({name:'Large native context',files:{exactSource:paddedSource,columnSettings:settings},operation:'solve-branch-price-context',constraints:[],result:paddedOutput,lastOperation:'solve-branch-price-context'});
assert.equal(parseProject(JSON.stringify(paddedProject)).files.exactSource.base64,paddedSource.base64);
await fs.writeFile(root+'/large-context.rctx',paddedText);
await fs.writeFile(root+'/large-browser-project.bisect',JSON.stringify(paddedProject));
await reject(canonical(context)+' '.repeat(8*1024*1024));
const browserRequest=columnContextRequest({name:'grid.rctx',base64:Buffer.from(canonical(context)).toString('base64')},settings);
const browserOutput=engine.execute(browserRequest);
const project=createProject({name:'Synthetic RCTX column benchmark',files:{exactSource:{name:'grid.rctx',base64:Buffer.from(browserRequest.context_json).toString('base64')},columnSettings:settings},operation:browserRequest.operation,constraints:[],result:browserOutput,lastOperation:browserRequest.operation});
assert.deepEqual(parseProject(JSON.stringify(project)),project);assert.equal(countResultTables(browserOutput)[0].rows.find(row=>row[0]==='Boundary objective')[1],'6');
await fs.writeFile(root+'/browser-project.bisect',JSON.stringify(project));await fs.writeFile(root+'/browser-result.json',JSON.stringify(browserOutput));await fs.writeFile(root+'/browser-plan.rplan',browserOutput.document_json);await fs.writeFile(root+'/browser-context.rctx',browserOutput.context_json);
console.log(`RCTX column WASM: ${parity} exact native/WASM results, ${rejections} matching rejections; lossless large counts, original context bytes, native plan validation, assignment/identity binding, weights/bridges and no-plan formulation checks passed.`);
