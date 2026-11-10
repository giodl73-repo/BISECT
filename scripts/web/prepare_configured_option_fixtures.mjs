import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createProject} from '../../web/lab/project.js';
import {countResultTables} from '../../web/lab/count-results.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const root='target/configured-option-display';await fs.mkdir(root,{recursive:true});
const source=(name,text)=>({name,base64:Buffer.from(text).toString('base64')});
for(const [name,index]of [['moving-knife',75],['ratio-search',57]]){
 const request=JSON.parse(await fs.readFile(`target/config-run-wasm/case-${index}.json`,'utf8')),result=engine.execute(request),input=request.input;
 const table=countResultTables(result).find(t=>t.title==='Options sent to the engine');assert(table);assert.deepEqual(table.rows.map(r=>r[0]),Object.keys(result.engine_request.options));
 if(name==='moving-knife'){assert.equal(result.configuration.document.algorithm.mka_metric,'polsby-popper');assert.equal(result.engine_request.options.mka_metric,'reock');assert(table.rows.some(r=>r[0]==='mka_metric'&&r[1]==='reock'));}
 else {assert.equal(result.configuration.document.algorithm.search,'convergence');assert.equal(result.engine_request.options.search,'multi');assert(table.rows.some(r=>r[0]==='search'&&r[1]==='multi'));}
 const files={configSource:source(input.source_name,input.yaml_text),configSettings:{profile:input.profile},configRunInputs:{graph:source('graph.json',JSON.stringify(input.graph))},configRunSettings:{chamber:input.chamber,districts:input.districts}};
 const project=createProject({name:'Configured option display '+name,files,operation:'run-algorithm-config',constraints:[],result,lastOperation:'run-algorithm-config'});await fs.writeFile(`${root}/${name}.bisect`,JSON.stringify(project));
}
console.log('Configured option fixtures: actual native-dispatched moving-knife metric and ratio convergence-to-multi settings preserved.');
