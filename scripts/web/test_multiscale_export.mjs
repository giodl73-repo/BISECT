import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
import {fineGraph,multiscaleRequest} from '../../web/lab/multiscale-input.js';
const root=process.argv[2]||'dist/wasm-multiscale',manifest=JSON.parse(await fs.readFile(path.join(root,'catalog.json'),'utf8'));
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const baseline=JSON.parse(await fs.readFile('target/multiscale-fixtures/request-0-0.json','utf8')).input.request.graph;
const originalProfile=JSON.parse(await fs.readFile('crates/rplan-audit/fixtures/grid3x3-valid-certificate.json','utf8')).legal_profile;
let exact=0,exports=0,misses=0,rejections=0,lastExportRequest;
async function parity(request){await fs.writeFile('target/multiscale-export-request.json',JSON.stringify(request));const native=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',['target/multiscale-export-request.json'],{encoding:'utf8',maxBuffer:32*1024*1024}));const result=engine.execute(request);assert.deepEqual(result,native);exact++;return result;}
async function reject(request){assert.throws(()=>engine.execute(request));await fs.writeFile('target/multiscale-export-request.json',JSON.stringify(request));assert.throws(()=>execFileSync('target/release/examples/execute_tool_request.exe',['target/multiscale-export-request.json'],{stdio:'pipe'}));rejections++;}
class Worker{postMessage(m){setImmediate(()=>{if(m.type==='initialize')this.onmessage({data:{type:'ready'}});else {assert.equal(m.request.operation,'export-multiscale-plan','Export must not generate a new plan.');lastExportRequest=m.request;try{this.onmessage({data:{type:'result',id:m.id,result:engine.execute(m.request)}});}catch(e){this.onmessage({data:{type:'error',id:m.id,error:e.message}});}}});}terminate(){}}
const names=(await fs.readdir('target')).filter(n=>/^multiscale-(state|national)-.*-(AL|AL-AK-RI|RI|IA|NC)\.bisect$/.test(n));
assert.equal(names.length,25,'Run the multiscale project gate first; use its 25 current fixtures.');
for(const name of names){const project=JSON.parse(await fs.readFile(path.join('target',name),'utf8')),c=project.experiment.config;
 for(const state of project.experiment.states){const code=state.code,synthetic=c.name.includes('SYNTHETIC multiscale'),g=synthetic?structuredClone(baseline):JSON.parse(await fs.readFile(path.join(root,manifest.graphs[`${code}:${c.year}`].graph_ref),'utf8'));
  if(synthetic){g.state=code;g.geoids=g.geoids.map(id=>({AL:'01',AK:'02',RI:'44'}[code])+id.slice(2));}
  const fine=fineGraph(g,c),input=multiscaleRequest(g,state.metrics.recorded_options,c).input;
  const request={operation:'export-multiscale-plan',input,fine_edges:c.multiscale.fine_level==='bg'?fine.edges:null,assignments:project.assignments[code],label:c.name,chamber:c.chamber,created_at:'2026-10-04T00:00:00Z'};
  const result=await parity(request);exports++;
  assert.equal(result.document.plan.units.unit_kind,c.multiscale.fine_level==='bg'?'block-group':'tract');
  assert.deepEqual(result.document.plan.units.unit_ids,fine.geoids);
  assert.deepEqual(result.document.plan.assignment,fine.geoids.map(id=>request.assignments[id]-1));
  assert.deepEqual(result.context.populations,fine.population);
  assert.equal(result.document.provenance.producer.multiscale_input.request.options.seed,c.seed);
  const edges=new Map(fine.edges.map(([i,j,w])=>[`${i}:${j}`,w]));
  for(let i=0;i<fine.geoids.length;i++){assert.deepEqual(result.context.graph.adjacency[i].map(e=>e.to),fine.adjacency[i]);for(const e of result.context.graph.adjacency[i]){assert.equal(e.kind,'custom');assert.equal(e.weight,edges.get(`${Math.min(i,e.to)}:${Math.max(i,e.to)}`));}}
  await parity({operation:'validate-rplan',document:result.document});
  const profile={...originalProfile,population_tolerance:{type:'exact-ppm',max_deviation_ppm:Math.round(c.balance_tolerance*10000)},contiguity_required:true,nesting_rule:{type:'not-evaluated'}};
  const certificate=await parity({operation:'audit-plan',plan:result.document.plan,context:result.context,profile,constraints:['plan-shape','population','contiguity'],generated_at_utc:request.created_at,lineage:null});
  assert.equal(certificate.result,state.metrics.balance_passed?'pass':'fail');if(!state.metrics.balance_passed)misses++;
  await parity({operation:'verify-certificate',certificate,plan:result.document.plan,context:result.context});
  if(!synthetic){const catalog=new WasmCatalog(manifest,async ref=>{const bytes=await fs.readFile(path.join(root,ref));return{ok:true,arrayBuffer:async()=>bytes.buffer.slice(bytes.byteOffset,bytes.byteOffset+bytes.length)};},()=>new Worker());const restored=await catalog.restoreProject(project);const exported=await catalog.exportPractitionerProject(restored.id,code);assert.deepEqual(exported.project.files.plan.plan.assignment,result.document.plan.assignment);assert.deepEqual(exported.project.files.context.populations,fine.population);assert.equal(exported.project.result,null);if(c.multiscale.fine_level==='bg'){await fs.writeFile('target/multiscale-browser-audit-project.bisect',JSON.stringify(exported.project));await fs.writeFile('target/multiscale-browser-audit-profile.json',JSON.stringify(profile));}catalog.destroy(new Error('Test complete.'));}
  for(const mutate of [r=>delete r.assignments[fine.geoids[0]],r=>r.assignments.extra=1,r=>r.assignments[fine.geoids[0]]=0,r=>r.label='',r=>r.input.request.options.seed='18446744073709551616',r=>{if(r.fine_edges)r.fine_edges.pop();else r.fine_edges=[];}]){const bad=structuredClone(request);mutate(bad);await reject(bad);}
  if(request.fine_edges)for(const mutate of[r=>r.fine_edges[0][2]=-1,r=>r.fine_edges.push(r.fine_edges[0]),r=>r.fine_edges[0][1]=100000,r=>r.input.block_groups.population[0]++,r=>r.input.block_groups.geoids[0]='990010000001']){const bad=structuredClone(request);mutate(bad);await reject(bad);}
 }
}
assert.ok(lastExportRequest,'Real-state UI export path exercised.');
await fs.writeFile('target/multiscale-export-verification.json',JSON.stringify({exports,exact_native_wasm_cases:exact,rejections,balance_misses_correctly_failed:misses,export_without_generation:true,scope:'25 project fixtures, 37 fine plans; real RI/IA/NC tracts and explicitly synthetic block groups'},null,2));
console.log(`Multiscale practitioner export: ${exports} plans, ${exact} exact native/WASM operations, ${rejections} matching rejections; ${misses} balance misses correctly fail native audit.`);
