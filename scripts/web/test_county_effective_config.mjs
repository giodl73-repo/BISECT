import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {effectiveConfig} from '../../web/lab/static.js';
import {validateLabProject,verifyLabAssignments} from '../../web/lab/laboratory-project.js';
const project=JSON.parse(await fs.readFile('target/county-state-oracle/browser.bisect','utf8'));
const config=project.experiment.config;
assert.equal(config.alpha_county,.125);
assert.equal(effectiveConfig(config,2).alpha_county,1);
assert.equal(effectiveConfig(config,2,'wasm').alpha_county,.125);
validateLabProject(project);
const metrics=project.experiment.states[0].metrics;
assert.equal(metrics.effective_config.alpha_county,.125);
const legacy=structuredClone(project);
legacy.experiment.states[0].metrics.effective_config.alpha_county=1;
validateLabProject(legacy);
for(const alpha of [0,.5,2]){
 const altered=structuredClone(project);
 altered.experiment.states[0].metrics.effective_config.alpha_county=alpha;
 assert.throws(()=>validateLabProject(altered),/Recorded options/);
}
const altered=structuredClone(legacy);
altered.experiment.states[0].metrics.weighting_evidence.alpha=1;
const root='dist/wasm-lab-build';
const manifest=JSON.parse(await fs.readFile(root+'/catalog.json','utf8'));
const graph=JSON.parse(await fs.readFile(root+'/'+manifest.graphs['RI:2020'].graph_ref,'utf8'));
assert.throws(()=>verifyLabAssignments(graph,altered.experiment.states[0],altered.assignments.RI,config),/county weighting evidence/);
console.log('County metadata: exact WASM alpha, native override clamp, legacy import and inconsistent summaries/evidence checks passed.');
