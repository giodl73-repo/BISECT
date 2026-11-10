import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
import {createLabProject} from '../../web/lab/laboratory-project.js';

const root = 'target/partisan-prime-reference', source = 'dist/wasm-lab-build';
const native = JSON.parse(await fs.readFile(root + '/evidence.json', 'utf8'));
const engine = await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const manifest = JSON.parse(await fs.readFile(source + '/catalog.json', 'utf8'));
manifest.wasm_sha256 = engine.artifactIdentity.sha256;
const fetcher = async ref => {const b = await fs.readFile(source + '/' + ref);return {ok: true,arrayBuffer: async () => b.buffer.slice(b.byteOffset,b.byteOffset+b.length)};};
class Worker {
  postMessage(m) {queueMicrotask(() => {try {this.onmessage({data:m.type==='initialize'?{type:'ready'}:{type:'result',id:m.id,result:engine.execute(m.request)}});}catch(e){this.onmessage({data:{type:'error',id:m.id,error:e.message}});}});}
  terminate() {}
}
const base = JSON.parse(await fs.readFile('target/county-state-oracle/browser.bisect','utf8')).experiment.config;
let projects = 0, refusals = 0;
for (const code of ['RI','IA']) {
  const candidates = native.records.filter(r=>r.label===code&&r.districts===2&&r.seed==='18446744073709551615'&&r.status==='native-completed');
  const selected = candidates.find(r=>r.profile==='stripe')??candidates[0];assert(selected);
  const request = JSON.parse(await fs.readFile(root+'/engine-'+selected.case+'.json','utf8'));
  const config = {...base,...request.options,states:[code],partisans:{[code]:request.partisan}};
  const lab = new WasmCatalog(manifest,fetcher,()=>new Worker()), job=lab.submit(config);
  while(lab.active)await new Promise(r=>setTimeout(r,5));
  assert.equal(lab.jobs.get(job.id).status,'completed',lab.jobs.get(job.id).logs.join('\n'));
  const project=await createLabProject(lab,job.id), restore=new WasmCatalog(manifest,fetcher,()=>{throw new Error('Open must not execute');});
  await restore.restoreProject(project);
  for(const mutate of [
    p=>{p.experiment.config.partisans[code].dem_shares[request.graph.geoids[0]]=.6;},
    p=>{p.experiment.states[0].metrics.structure_evidence.research_balance_limit_percent=5;},
    p=>{p.experiment.states[0].metrics.structure_evidence.factor_sequence=[2,2];},
  ]){const changed=structuredClone(project);mutate(changed);await assert.rejects(()=>restore.restoreProject(changed));refusals++;}
  const exported=await lab.exportPractitionerProject(job.id,code);
  assert.deepEqual(exported.exported.document.provenance.producer.partisan_input,request.partisan);
  if(code==='RI')await fs.writeFile(root+'/browser.bisect',JSON.stringify(project));
  projects++;
}
await fs.writeFile(root+'/project-evidence.json',JSON.stringify({real_state_projects_and_exports:projects,source_or_schedule_tamper_refusals:refusals,inert_open:true,wasm_sha256:engine.artifactIdentity.sha256},null,2));
console.log(`Partisan prime projects: ${projects} actual RI/IA plans, exact source-preserving exports and inert Open; ${refusals} source/schedule/limit changes refused.`);
