import fs from 'node:fs/promises';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createProject} from '../../web/lab/project.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const root='target/configured-option-display';await fs.mkdir(root,{recursive:true});
const source=(name,text)=>({name,base64:Buffer.from(text).toString('base64')});
for(const [name,index]of [['inactive-county',16],['active-county',32]]){
 const request=JSON.parse(await fs.readFile(`target/config-county-oracle/case-${index}.json.tool.json`,'utf8')),result=engine.execute(request),input=request.input;
 const files={configSource:source(input.source_name,input.yaml_text),configSettings:{profile:input.profile},configRunInputs:{graph:source('graph.json',JSON.stringify(input.graph))},configRunSettings:{chamber:input.chamber,districts:input.districts}};
 await fs.writeFile(`${root}/${name}.bisect`,JSON.stringify(createProject({name:'Configured county '+name,files,operation:'run-algorithm-config',constraints:[],result,lastOperation:'run-algorithm-config'})));
}
