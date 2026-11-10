import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {labelBuildPlanChoices,labelBuildPlanSources} from '../../web/lab/label-build-files.js';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const archive=JSON.parse(await fs.readFile('target/label-build-wasm/browser-archive.bisect-build','utf8'));
const result=archive.result;
const choices=labelBuildPlanChoices(result);assert.equal(choices.length,3);
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
for(const path of choices){
 const sources=labelBuildPlanSources(result,path);
 const decode=s=>Buffer.from(s.base64,'base64').toString('utf8');
 assert.equal(decode(sources.auditPlanText),result.files[path]);
 assert.equal(decode(sources.auditContextText),result.files[path.replace(/plan\.rplan$/,'context.rctx')]);
 const response=engine.execute({operation:'validate-rplan-text',document_json:decode(sources.auditPlanText)});
 assert.equal(response.schema_version,'bisect-text-plan-validation-v1');assert.ok(response.units>0);assert.ok(response.districts>0);
 const saved=createProject({name:'Built plan handoff',files:sources,operation:'validate-rplan-text',constraints:[],result:null,lastOperation:null});
 assert.deepEqual(parseProject(JSON.stringify(saved)).files,sources);
}
assert.throws(()=>labelBuildPlanSources(result,'runs/../plan.rplan'));
const changed=structuredClone(result);delete changed.files[choices[0].replace(/plan\.rplan$/,'context.rctx')];assert.ok(!labelBuildPlanChoices(changed).includes(choices[0]));
changed.files[choices[1]]=' '.repeat(3*1024*1024+1);assert.throws(()=>labelBuildPlanSources(changed,choices[1]),/3 MiB/);
const large=structuredClone(result);large.files[choices[0]]=result.files[choices[0]].replace(/"seed"\s*:\s*\d+/, '"seed":9007199254740993');assert.equal(Buffer.from(labelBuildPlanSources(large,choices[0]).auditPlanText.base64,'base64').toString(),large.files[choices[0]]);
console.log('Three native build plan/context pairs: exact-byte handoff, real WASM validation, project roundtrip; missing pair/path/size rejection and exact integer retention passed.');
