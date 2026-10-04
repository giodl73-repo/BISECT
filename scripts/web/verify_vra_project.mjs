import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
import {parseLabProject} from '../../web/lab/laboratory-project.js';
const [root,file,reportFile]=process.argv.slice(2);assert.ok(root&&file,'Usage: verify_vra_project.mjs <catalog> <downloaded .bisect> [report.json]');
const manifest=JSON.parse(await fs.readFile(path.join(root,'catalog.json'),'utf8')),project=parseLabProject(await fs.readFile(file,'utf8'));
assert.ok(project.experiment.config.search==='vra-recom'||project.experiment.config.structure==='ratio-optimal-vra');
const engine=await instantiateEngine(await fs.readFile(path.join(root,'bisect_wasm.wasm'))),fetcher=async ref=>{const bytes=await fs.readFile(path.join(root,ref));return{ok:true,arrayBuffer:async()=>bytes.buffer.slice(bytes.byteOffset,bytes.byteOffset+bytes.length)};};
const restorer=new WasmCatalog(manifest,fetcher,()=>{throw new Error('Open must not execute an engine.');});
const restored=await restorer.restoreProject(project),results=[];
for(const state of project.experiment.states){
 assert.equal(state.status,'completed');const code=state.code,graph=JSON.parse(await fs.readFile(path.join(root,manifest.graphs[`${code}:${project.experiment.config.year}`].graph_ref),'utf8'));
 const request={graph,options:state.metrics.requested_options,demographics:project.experiment.config.demographics[code]},referenceFile=`target/vra-browser-replay-${code}.json`;await fs.writeFile(referenceFile,JSON.stringify(request));
 const actual=engine.execute(request),native=spawnSync('target/release/examples/execute_request.exe',[referenceFile],{encoding:'utf8',maxBuffer:32*1024*1024});assert.equal(native.status,0,native.stderr);const expected=JSON.parse(native.stdout);
 assert.deepEqual(actual,expected);assert.deepEqual(expected.assignments,project.assignments[code]);
 for(const [key,value]of Object.entries(expected.metrics))assert.deepEqual(state.metrics[key==='districts'?'district_count':key],value);
 const map=await restorer.api(`/api/runs/${restored.id}/${code}/map`);assert.equal(map.features.length,graph.geoids.length);
 const e=state.metrics.structure_evidence;
 results.push({state:code,units:graph.geoids.length,districts:state.metrics.district_count,max_deviation_percent:state.metrics.max_deviation_percent,contiguous:state.metrics.contiguous,...(e.method==='vra-section'?{root_ratio:state.metrics.root_split.left_districts,alignment:e.alignment,selection_score:e.selection_score,w_vra:e.w_vra}:{protected_districts:e.run.protected_districts,minority_rejections:e.run.minority_rejections})});
}
await fs.writeFile(reportFile||(project.experiment.config.structure==='ratio-optimal-vra'?'target/wasm-vra-section-browser-verification.json':'target/wasm-vra-browser-verification.json'),JSON.stringify({downloaded_project:file,config:project.experiment.config.search,structure:project.experiment.config.structure,exact_native_wasm_replay:true,open_executed_engine:false,results},null,2));
console.log(JSON.stringify({exact_native_wasm_replay:true,open_executed_engine:false,results}));
