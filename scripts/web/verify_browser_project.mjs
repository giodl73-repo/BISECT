import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {parseLabProject} from '../../web/lab/laboratory-project.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
const [root,source]=process.argv.slice(2);
assert.ok(root&&source,'Usage: node scripts/web/verify_browser_project.mjs <catalog directory> <downloaded project>');
const manifest=JSON.parse(await fs.readFile(path.join(root,'catalog.json'),'utf8'));
const moduleHash=createHash('sha256').update(await fs.readFile(path.join(root,'bisect_wasm.wasm'))).digest('hex');
assert.equal(moduleHash,manifest.wasm_sha256);
const project=parseLabProject(await fs.readFile(source,'utf8'));
const lab=new WasmCatalog(manifest,async ref=>{
  const bytes=await fs.readFile(path.join(root,ref));
  return {ok:true,arrayBuffer:async()=>bytes.buffer.slice(bytes.byteOffset,bytes.byteOffset+bytes.length)};
},()=>{throw new Error('Opening a downloaded project must not execute the engine.');});
const opened=await lab.restoreProject(project);
assert.equal(lab.worker,null);
const evidence=[];
for(const state of opened.states){
  if(!state.metrics){evidence.push({state:state.code,status:state.status});continue;}
  assert.equal(state.metrics.engine_provenance.wasm_sha256,moduleHash,'This gate requires the downloaded result to identify the tested module.');
  const map=await lab.api(`/api/runs/${opened.id}/${state.code}/map`);
  assert.equal(map.features.length,state.metrics.units);
  assert.ok(map.features.every(f=>f.properties.district===project.assignments[state.code][f.properties.geoid]));
  lab.assets.delete(manifest.geometries[`${state.code}:${opened.config.year}`]);
  evidence.push({state:state.code,status:state.status,units:state.metrics.units,contiguous:state.metrics.contiguous,balance_passed:state.metrics.balance_passed,maximum_deviation_percent:state.metrics.max_deviation_percent});
}
console.log(JSON.stringify({wasm_sha256:moduleHash,config:opened.config,evidence,engine_executed_on_open:false,provenance:'Recorded identity is checked for agreement, not attested execution or optimality.'}));
