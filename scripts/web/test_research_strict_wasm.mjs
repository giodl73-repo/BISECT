import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {researchRequest,validateResearchInputs} from '../../web/lab/research-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';

const root='target/research-strict';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const chains=Object.fromEntries(Array.from({length:4},(_,c)=>['chain-'+c+'.jsonl',Array.from({length:16},(_,i)=>JSON.stringify({step:i,metrics:{x:i%2?1:-1,trend:i/10}})).join('\n')+'\n']));
let cliMatches=0;
for(const [strict,ess_min,exit]of [[false,100,0],[true,100,1],[true,1,0]]){
 const request={operation:'validate-research-ensemble',chains_jsonl:chains,enacted_json:null,rhat_threshold:1.05,ess_min,strict};
 const actual=engine.execute(request);assert.equal(actual.exit_code,exit);assert.equal(actual.status,exit?'strict-failed':'completed');assert.equal(actual.strict,strict);
 const dir=path.join(root,'case-'+cliMatches);await fs.mkdir(dir,{recursive:true});for(const [name,text]of Object.entries(chains))await fs.writeFile(path.join(dir,name),text);
 const output=path.join(dir,'report.json'),native=spawnSync(path.resolve('target/debug/bisect.exe'),['research','validate-ensemble','--ensemble-dir',dir,'--output',output,'--rhat-threshold','1.05','--ess-min',String(ess_min),...(strict?['--strict']:[])],{encoding:'utf8'});
 assert.equal(native.status,exit,native.stderr);assert.equal(actual.report_json,await fs.readFile(output,'utf8'));if(exit)assert(native.stderr.includes(actual.strict_error));else assert.equal(actual.strict_error,null);
 await fs.writeFile(path.join(dir,'request.json'),JSON.stringify(request));const host=spawnSync(path.resolve('target/release/examples/execute_tool_request.exe'),[path.resolve(dir,'request.json')],{encoding:'utf8',maxBuffer:32*1024*1024});assert.equal(host.status,0,host.stderr);assert.deepEqual(JSON.parse(host.stdout),actual);cliMatches++;
 if(exit){const sources=Object.fromEntries(Object.entries(chains).map(([name,text])=>[name,{name,base64:Buffer.from(text).toString('base64')}])),inputs={chains:sources,enacted:null,rhat_threshold:1.05,ess_min,strict};
  assert.deepEqual(researchRequest(inputs),request);const project=createProject({name:'Strict research failure',files:{researchInputs:inputs},operation:request.operation,constraints:[],result:actual,lastOperation:request.operation});assert.deepEqual(parseProject(JSON.stringify(project)),project);await fs.writeFile(root+'/failed.bisect',JSON.stringify(project));await fs.writeFile(root+'/failed-report.json',actual.report_json);
  const legacy={...inputs};delete legacy.strict;assert.equal(researchRequest(legacy).strict,false);assert.throws(()=>validateResearchInputs({...inputs,strict:'true'}));assert.throws(()=>validateResearchInputs({...inputs,unexpected:true}));
 }
}
const omitted={operation:'validate-research-ensemble',chains_jsonl:chains,enacted_json:null,rhat_threshold:1.05,ess_min:100};assert.deepEqual(engine.execute(omitted),engine.execute({...omitted,strict:false}));
await fs.writeFile(root+'/evidence.json',JSON.stringify({native_cli_exit_and_report_matches:cliMatches,whole_host_wasm_matches:cliMatches,strict_failure_retains_report:true,legacy_omission_defaults_false:true,strict_project_roundtrip:true},null,2));
console.log('Research strict: three native CLI exit/report and host/WASM matches; strict failure retains report, legacy omission defaults false and projects retain settings.');
