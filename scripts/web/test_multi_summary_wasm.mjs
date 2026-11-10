import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {multiSummaryRequest,readMultiSummaryFolder} from '../../web/lab/multi-summary-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const root=path.resolve('target/multi-summary');await fs.mkdir(root,{recursive:true});
const manifest=JSON.stringify({label:'alpha',state_code:'RI',year:'2020',chamber:'congressional',num_districts:2,population_source:'supplied',partition_mode:'standard',seed:42,binary_version:'test',binary_sha256:'',binary_download_url:'',adjacency_file:'',adjacency_sha256:'',adjacency_build_command:'',adjacency_build_version:'',tiger_source_url:'',tiger_sha256:null,created_at:'2026-10-13T00:00:00Z',balance_tolerance_pct:5,population_balance_valid:true});
const raw={alpha:{'summary.json':'{"max_deviation_pct":2.125,"population_max_deviation_pct":99}','compactness.json':'{"districts":[{"polsby_popper":0.3},{"polsby_popper":0.6},{"polsby_popper":"unavailable"}]}','splits.json':'{"counties":{"split":18446744073709551615}}','contiguity.json':'{"all_contiguous":true}','manifest.json':manifest},empty:{'compactness.json':'{"districts":[]}'},bad:{'summary.json':'{invalid','manifest.json':'{}'},a_very_long_plan_label_for_table_width:{'summary.json':'{"population_max_deviation_pct":1.25}','contiguity.json':'{"all_contiguous":false}'}};
const source=(name,text)=>({name,base64:Buffer.from(text).toString('base64')});
const plans=Object.fromEntries(Object.entries(raw).map(([label,files])=>[label,Object.fromEntries(Object.entries(files).map(([name,text])=>[name,source(name,text)]))]));
assert.deepEqual(Object.keys(multiSummaryRequest({labels:['alpha','alpha'],plans}).input.plans),['alpha']);
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));let cliMatches=0,hostMatches=0,last;
for(const labels of [['empty','alpha','bad','a_very_long_plan_label_for_table_width','missing'],['alpha','alpha'],['missing','bad']]){
 const inputs={labels,plans},request=multiSummaryRequest(inputs),actual=engine.execute(request);last={inputs,actual};
 const requestPath=path.join(root,'request.json');await fs.writeFile(requestPath,JSON.stringify(request));const host=spawnSync(path.resolve('target/release/examples/execute_tool_request.exe'),[requestPath],{encoding:'utf8',maxBuffer:32*1024*1024});assert.equal(host.status,0,host.stderr);assert.deepEqual(JSON.parse(host.stdout),actual);hostMatches++;
 const version='native-'+cliMatches,base=path.join(root,'native'),collection=path.join(base,version,'2020','plans');
 for(const [label,files]of Object.entries(raw))for(const [name,text]of Object.entries(files)){const file=path.join(collection,label,...(name==='manifest.json'?[]:['analysis']),name);await fs.mkdir(path.dirname(file),{recursive:true});await fs.writeFile(file,text);}
 for(const format of ['table','csv','json']){const cli=spawnSync(path.resolve('target/debug/bisect.exe'),['compare','--plan-a','unused','--labels',labels.join(','),'--year','2020','--version',version,'--output-base',base,'--format',format],{encoding:'utf8',maxBuffer:32*1024*1024});assert.equal(cli.status,0,cli.stderr);assert.equal(actual[format==='csv'?'csv':'table'],cli.stdout);cliMatches++;}
 if(labels[0]==='empty'){assert(actual.csv.includes('18446744073709551615'));assert(actual.csv.includes('empty,NaN,N/A,N/A,-,N/A'));assert(actual.csv.includes('alpha,0.450,2.1%,18446744073709551615,5.0%,Yes'));assert.equal(actual.availability.find(r=>r.label==='bad').manifest,false);
  const project=createProject({name:'Native multi-plan summary',files:{multiSummaryInputs:inputs},operation:'summarize-plan-labels',constraints:[],result:actual,lastOperation:'summarize-plan-labels'});assert.deepEqual(parseProject(JSON.stringify(project)),project);await fs.writeFile(path.join(root,'summary.bisect'),JSON.stringify(project));await fs.writeFile(path.join(root,'table.txt'),actual.table);await fs.writeFile(path.join(root,'summary.csv'),actual.csv);
 }
}
let reads=0;const selected=[];
for(const [label,files]of Object.entries(raw))for(const [name,text]of Object.entries(files)){const bytes=Buffer.from(text),relative=label+'/'+(name==='manifest.json'?'':'analysis/')+name,file=path.join(root,'plans',relative);await fs.mkdir(path.dirname(file),{recursive:true});await fs.writeFile(file,text);selected.push({name,webkitRelativePath:'plans/'+relative,size:bytes.length,arrayBuffer:async()=>{reads++;return bytes.buffer.slice(bytes.byteOffset,bytes.byteOffset+bytes.byteLength);}});}
const ignored={name:'summary.json',webkitRelativePath:'plans/empty/backup/analysis/summary.json',size:100*1024*1024,arrayBuffer:async()=>{throw new Error('Backup must not be read');}};selected.push(ignored);const folder=await readMultiSummaryFolder(selected);assert.equal(reads,10);assert.deepEqual(JSON.parse(JSON.stringify(folder.plans)),plans);assert.deepEqual(folder.labels,Object.keys(raw).sort());
let largeRead=false;await assert.rejects(()=>readMultiSummaryFolder([{...ignored,webkitRelativePath:'plans/alpha/analysis/summary.json',arrayBuffer:async()=>{largeRead=true;}}]),/3 MiB/);assert.equal(largeRead,false);
assert.throws(()=>engine.execute({operation:'summarize-plan-labels',input:{labels:['a'],plans:{}}}));assert.throws(()=>multiSummaryRequest({labels:['unsafe/label','a'],plans:{}}));
await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({whole_host_wasm_matches:hostMatches,actual_cli_byte_matches:cliMatches,raw_u64_retained:true,native_nan_and_missing_placeholders:true,label_order_and_duplicates:true,typed_manifest_tolerance:true,exact_project_roundtrip:true,pre_read_bounds:true,nested_backup_ignored:true},null,2));
console.log(`Multi-plan summary: ${hostMatches} host/WASM and ${cliMatches} actual CLI format byte matches; requested order, duplicate/unavailable labels, native NaN/u64/tolerance formatting and source/project bounds passed.`);
