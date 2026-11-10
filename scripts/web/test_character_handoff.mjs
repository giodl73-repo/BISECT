import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {characterPreparationProject,preparedCharacterInput,characterHandoffToken,characterLaboratoryUrl} from '../../web/lab/character-handoff.js';
import {acsHousingRequest} from '../../web/lab/acs-housing-files.js';
import {lodesRequest} from '../../web/lab/lodes-files.js';
import {parseProject} from '../../web/lab/project.js';
const out='target/character-handoff';await fs.mkdir(out,{recursive:true});const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));let cases=0,refusals=0;
for(const [prefix,file,key,requestFor] of [['acs-housing','target/acs-housing-wasm/valid.bisect','acsHousingInputs',acsHousingRequest],['lodes','target/lodes-source-wasm/workplace.bisect','lodesInputs',lodesRequest]]){
 const saved=JSON.parse(await fs.readFile(file,'utf8')),request=requestFor(saved.files[key]),result=engine.execute(request);await fs.writeFile(out+'/request.json',JSON.stringify(request));assert.deepEqual(result,JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[out+'/request.json'],{encoding:'utf8',maxBuffer:64*1024*1024})));
 const project=characterPreparationProject(result,saved.files);assert.deepEqual(project.files[key],saved.files[key]);const input=preparedCharacterInput(parseProject(JSON.stringify(project)));assert.deepEqual(input,result.character_input);input.source_label='edited copy';assert.notEqual(input.source_label,project.result.character_input.source_label);await fs.writeFile(out+'/'+prefix+'.bisect',JSON.stringify(project));cases++;
 for(const mutate of [p=>p.lastOperation='run-algorithm-config',p=>p.result.execution_performed=true,p=>p.result.character_status='unavailable',p=>p.files[key].settings.data_year='1999',p=>p.result.character_input.state='XX',p=>p.files[key].source=null]){const bad=structuredClone(project);mutate(bad);assert.throws(()=>preparedCharacterInput(bad));refusals++;}
}
const commuting=JSON.parse(await fs.readFile('target/lodes-source-wasm/commuting-result.json','utf8'));assert.throws(()=>characterPreparationProject(commuting,{}));refusals++;
const token='a'.repeat(32);for(const [from,to]of [['https://example.test/lab/toolkit/','https://example.test/lab/'],['https://example.test/lab/toolkit/index.html','https://example.test/lab/'],['https://example.test/lab/workbench.html','https://example.test/lab/']])assert.equal(characterLaboratoryUrl(from,token),to+'#character='+token);
assert.equal(characterHandoffToken('#character='+token),token);for(const h of ['#character=x','#project='+token,'#character='+token+'&run=true'])assert.equal(characterHandoffToken(h),null);
await fs.writeFile(out+'/evidence.json',JSON.stringify({cases,refusals,exact_native_preparation:true,original_source_project_preserved:true,path_prefixes_checked:3,wasm_sha256:engine.artifactIdentity.sha256},null,2));console.log(`Character handoff: ${cases} native-matching preparation projects, original sources and detached input copies, ${refusals} unavailable/scope/type refusals and same-origin path prefixes passed.`);
