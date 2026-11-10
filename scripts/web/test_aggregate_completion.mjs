import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {readAggregateCompletion,readNativeAggregateFolder,nativeAggregateRequest} from '../../web/lab/native-aggregate-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const root=path.resolve('target/aggregate-completion');await fs.mkdir(root,{recursive:true});
const wasm=await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'),engine=await instantiateEngine(wasm);
const cli=path.resolve('target/release/bisect.exe'),sha=b=>createHash('sha256').update(b).digest('hex');
const summary='{"districts":[{"district":1,"population":100}]}',compactness='{"districts":[{"district":1,"polsby_popper":0.4}]}';
const settings={year:'2020',types:['summary','compactness'],format:'json',csv:true},records=[];
const source=(name,text)=>({name,base64:Buffer.from(text).toString('base64')});
const cases=[
 {name:'skip-summary',text:summary,existing:['us_summary.json'],force:false},
 {name:'force-summary',text:summary,existing:['us_summary.json'],force:true},
 {name:'skip-all',text:summary,existing:['us_summary.json','us_compactness.json'],force:false},
 {name:'skip-malformed',text:'{bad',existing:['us_summary.json'],force:false},
 {name:'force-malformed',text:'{bad',existing:['us_summary.json'],force:true},
 {name:'skip-unsafe-rows',text:'{"districts":[7]}',existing:['us_summary.json'],force:false},
 {name:'skip-oversized',text:' '.repeat(3*1024*1024+1),existing:['us_summary.json'],force:false},
 {name:'existing-directory',text:summary,existing:['us_summary.json'],force:false,directory:true},
 {name:'fresh',text:summary,existing:[],force:false},
 {name:'duplicate-summary',text:summary,existing:[],force:false,types:['summary','summary']},
 {name:'duplicate-summary-force',text:summary,existing:[],force:true,types:['summary','summary']}
];
for(const fixture of cases){
 const caseRoot=path.join(root,'run-'+Date.now()+'-'+fixture.name),national=path.join(caseRoot,'outputs/test/2020/national'),analysis=path.join(caseRoot,'outputs/test/2020/states/alabama/analysis');
 await fs.mkdir(analysis,{recursive:true});await fs.mkdir(national,{recursive:true});
 await fs.writeFile(path.join(analysis,'summary.json'),fixture.text);await fs.writeFile(path.join(analysis,'compactness.json'),compactness);
 for(const name of fixture.existing){
  if(fixture.directory){await fs.mkdir(path.join(national,name));await fs.writeFile(path.join(national,name,'marker'),'existing');}
  else await fs.writeFile(path.join(national,name),'EXISTING OUTPUT');
 }
 const request={operation:'aggregate-native-analysis',input:{...settings,types:fixture.types??settings.types,reports:{alabama:{summary:fixture.text,compactness}},completion:{year:'2020',existing_outputs:fixture.existing,force:fixture.force}}};
 const actual=engine.execute(request),native=spawnSync(cli,['aggregate','--version','test','--year','2020','--types',...(fixture.types??settings.types),'--format','json','--csv',...(fixture.force?['--force']:[])],{cwd:caseRoot,encoding:'utf8',maxBuffer:16*1024*1024});
 assert.equal(native.status,0,native.stderr);
 if(fixture.types){assert.deepEqual(actual.summaries.map(s=>s.status),fixture.force?['aggregated','aggregated']:['aggregated','skipped-existing']);assert.equal(native.stderr.includes('[skip] summary'),!fixture.force);}
 for(const [name,text]of Object.entries(actual.files))assert.equal(text,await fs.readFile(path.join(national,name),'utf8'));
 for(const name of fixture.existing)if(!fixture.force){
  assert(!Object.hasOwn(actual.files,name));assert(native.stderr.includes('[skip] '+name.slice(3,-5)));
  if(!fixture.directory)assert.equal(await fs.readFile(path.join(national,name),'utf8'),'EXISTING OUTPUT');
 }
 const skipped=fixture.existing.length&&!fixture.force;
 assert.equal(actual.skipped.some(s=>s.source==='alabama/analysis/summary.json'),!skipped&&fixture.text==='{bad');
 if(skipped)assert(!Object.hasOwn(actual.source_sha256,'alabama/analysis/summary.json'));
 if(fixture.name==='skip-all')assert.equal(Object.keys(actual.files).length,0);
 if(['skip-summary','force-summary'].includes(fixture.name)){
  const inputs={settings,reports:{alabama:{summary:source('summary.json',fixture.text),compactness:source('compactness.json',compactness)}},completion:request.input.completion};
  const packet=createProject({name:fixture.name,operation:request.operation,lastOperation:request.operation,constraints:[],files:{nativeAggregateInputs:inputs},result:actual});
  assert.deepEqual(parseProject(JSON.stringify(packet)),packet);await fs.writeFile(path.join(root,fixture.name+'.bisect'),JSON.stringify(packet));
 }
 records.push({fixture:fixture.name,native_exit:native.status,generated_files:Object.keys(actual.files),summaries:actual.summaries,skipped_source_parsing:Boolean(skipped),run_directory:caseRoot});
}
let bodyReads=0;
const observed=readAggregateCompletion([{webkitRelativePath:'national/us_summary.json',size:0,arrayBuffer:()=>{throw Error('Output bytes must not be read');}},{webkitRelativePath:'national/us_compactness.csv'},{webkitRelativePath:'national/us_summary_districts.json'}],'2020');
assert.deepEqual(observed.existing_outputs,['us_summary.json']);
const files=[{name:'summary.json',webkitRelativePath:'states/alabama/analysis/summary.json',size:4*1024*1024,arrayBuffer:()=>{bodyReads++;throw Error('Skipped oversized source must not be read');}},{name:'compactness.json',webkitRelativePath:'states/alabama/analysis/compactness.json',size:Buffer.byteLength(compactness),arrayBuffer:async()=>{bodyReads++;return Buffer.from(compactness);}}];
const selected=await readNativeAggregateFolder(files,settings,observed);assert.equal(bodyReads,1);assert.deepEqual(selected.deferred_types,['summary']);
assert.equal(engine.execute(nativeAggregateRequest(selected)).summaries[0].status,'skipped-existing');
assert.throws(()=>nativeAggregateRequest({...selected,completion:{...observed,force:true}}),/Reload or reselect/);
assert.throws(()=>nativeAggregateRequest({...selected,completion:{...observed,existing_outputs:[]}}),/Reload or reselect/);
for(const completion of [{...observed,year:'2010'},{...observed,existing_outputs:['../us_summary.json']},{...observed,existing_outputs:['us_summary.json','us_summary.json']},{...observed,existing_outputs:['us_summary.csv']}])assert.throws(()=>engine.execute({operation:'aggregate-native-analysis',input:{...settings,reports:{alabama:{}},completion}}));
const evidence={wasm_sha256:sha(wasm),native_cli_sha256:sha(await fs.readFile(cli)),native_cli_cases:records.length,source_skip_precedes_parsing_and_bounds:true,output_metadata_body_reads:0,skipped_oversized_source_body_reads:0,deferred_force_requires_explicit_reload:true,scope:'Actual native aggregate CLI output/skip behavior over selected temporary files. Filename observations only; no existing output bodies imported or writes to user folders. Browser FileList cannot discover completely empty directories. Saved observations do not restore filesystem authority.',records};
await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify(evidence,null,2)+'\n');
console.log('Aggregate completion: nine actual native CLI cases, source parsing/budget bypass on skip, directory existence, force, no output-body reads and deferred-source reload gate passed.');
