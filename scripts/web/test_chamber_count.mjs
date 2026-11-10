import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {CHAMBER_COUNT_DEFAULTS,chamberCountRequest,graphWithChamberCount} from '../../web/lab/chamber-count-files.js';
import {preparedGraphRequest} from '../../web/lab/prepared-graph-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const root='target/chamber-count';await fs.mkdir(root,{recursive:true});const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const manifest=JSON.parse(await fs.readFile('data/manifest.json','utf8'));const cli=path.resolve('target/release/examples/chamber_count_reference.exe'),host=path.resolve('target/release/examples/execute_tool_request.exe');
let matches=0,cliCases=0,absent=0,refusals=0;
async function parity(request){const actual=engine.execute(request);await fs.writeFile(root+'/request.json',JSON.stringify(request));assert.deepEqual(actual,JSON.parse(execFileSync(host,[path.resolve(root+'/request.json')],{encoding:'utf8',maxBuffer:64*1024*1024,stdio:['ignore','pipe','pipe']})));matches++;return actual;}
for(const year of ['2000','2010','2020'])for(const state of Object.keys(manifest.states).filter(state=>manifest.states[state].districts[year]>0).sort())for(const chamber of ['congressional','house','senate']){
 const inputs={...CHAMBER_COUNT_DEFAULTS,state,year,chamber},fallback=manifest.states[state].districts[year];let expected,error;
 try{expected=Number(execFileSync(cli,[state,chamber,String(fallback)],{encoding:'utf8',stdio:['ignore','pipe','pipe']}));}catch(e){error=e;}
 if(error){assert.throws(()=>engine.execute(chamberCountRequest(inputs)),/No .* chamber/);absent++;}else{const result=await parity(chamberCountRequest(inputs));assert.equal(result.districts,expected);assert.equal(result.execution_performed,false);}
 cliCases++;
}
const override={RI:{house_districts:2,senate_districts:1}},policy={name:'TEST-ONLY-policy.json',base64:Buffer.from(JSON.stringify(override)).toString('base64')};await fs.writeFile(root+'/policy.json',JSON.stringify(override));
for(const chamber of ['house','senate','congressional']){const inputs={...CHAMBER_COUNT_DEFAULTS,chamber,policy},result=await parity(chamberCountRequest(inputs));assert.equal(result.districts,Number(execFileSync(cli,['RI',chamber,'2',path.resolve(root+'/policy.json')],{encoding:'utf8'})));cliCases++;}
for(const value of [{},{RI:{}},{RI:{house_districts:'2'}},{RI:{house_districts:-1}},{RI:{house_districts:2.5}}]){const text=JSON.stringify(value);await fs.writeFile(root+'/fallback-policy.json',text);const input={...CHAMBER_COUNT_DEFAULTS,policy:{name:'fallback-policy.json',base64:Buffer.from(text).toString('base64')}},result=await parity(chamberCountRequest(input));assert.equal(result.allocation_source,'congressional-fallback');assert.equal(result.districts,Number(execFileSync(cli,['RI','house','2',path.resolve(root+'/fallback-policy.json')],{encoding:'utf8'})));cliCases++;}
const bomPolicy={name:'bom-policy.json',base64:Buffer.from('\ufeff{}').toString('base64')},bomRequest=chamberCountRequest({...CHAMBER_COUNT_DEFAULTS,policy:bomPolicy});assert.equal(bomRequest.input.policy_json.charCodeAt(0),0xfeff);assert.throws(()=>engine.execute(bomRequest),/Invalid location policy JSON/);refusals++;
const defaults=await parity(chamberCountRequest(CHAMBER_COUNT_DEFAULTS));assert.equal(defaults.districts,75);
const original=JSON.parse(await fs.readFile('target/prepared-graph-files/real-binding.bisect','utf8')).files.preparedGraphInputs,next=graphWithChamberCount(original,CHAMBER_COUNT_DEFAULTS,defaults);
assert.equal(next.districts,75);assert.equal(next.settings.chamber,'house');assert.equal(next.template.text,original.template.text);assert.equal(next.graph.text,original.graph.text);const bound=await parity(preparedGraphRequest(next));assert.equal(bound.review.engine_request.options.districts,75);assert.equal(bound.execution_performed,false);
for(const mutate of [r=>r.state='WA',r=>r.year='2010',r=>r.chamber='senate',r=>r.execution_performed=true,r=>r.districts=0,r=>r.districts=501]){const bad=structuredClone(defaults);mutate(bad);assert.throws(()=>graphWithChamberCount(original,CHAMBER_COUNT_DEFAULTS,bad));refusals++;}
const wrongGraph=structuredClone(original);wrongGraph.graph.text=wrongGraph.graph.text.replace('"state":"RI"','"state":"WA"');assert.throws(()=>graphWithChamberCount(wrongGraph,CHAMBER_COUNT_DEFAULTS,defaults));refusals++;
const sourceFiles={...JSON.parse(await fs.readFile('target/prepared-graph-files/real-binding.bisect','utf8')).files,chamberCountInputs:CHAMBER_COUNT_DEFAULTS};const project=createProject({name:'Native chamber count',files:sourceFiles,operation:'resolve-native-chamber-count',constraints:[],result:defaults,lastOperation:'resolve-native-chamber-count'});assert.deepEqual(parseProject(JSON.stringify(project)),project);await fs.writeFile(root+'/browser.bisect',JSON.stringify(project));await fs.writeFile(root+'/expected-binding.json',JSON.stringify(bound));
await fs.writeFile(root+'/evidence.json',JSON.stringify({wasm_sha256:engine.artifactIdentity.sha256,native_host_matches:matches,actual_cli_count_cases:cliCases,absent_chambers:absent,refusals,all_50_states_three_vintages_three_chambers:true,explicit_override_matches_cli:true,original_ri_graph_binds_75_house_districts_without_partition:true,template_graph_and_tolerance_unchanged:true,inert_project_roundtrip:true},null,2));
console.log(`Native chamber counts: ${cliCases} actual CLI cases, ${matches} Native/WASM matches, ${absent} absent chambers, ${refusals} handoff refusals; original RI 75-seat binding without partition, exact sources/template and inert Open passed.`);
