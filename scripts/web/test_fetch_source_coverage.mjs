// Source reconciliation evidence only: these comparisons perform no acquisition.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {publicFetchUrl} from '../../web/lab/fetch-acquisition.js';
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const coverage=JSON.parse(await fs.readFile('docs/specs/wasm-fetch-source-coverage.json','utf8'));
assert.equal(coverage.schema_version,'bisect-fetch-source-coverage-v1');
for(const pin of coverage.authority){const bytes=await fs.readFile(pin.path);assert.equal(sha(bytes),pin.sha256,pin.path+' source drift');assert.equal(bytes.length,pin.byte_length);}
const names=['tiger','redistricting','adjacency','enacted','geography','elections','lodes','school-districts','eia-861','lodes-od','acs-housing','all'];
assert.deepEqual(coverage.rows.map(row=>row.data_type),names);
for(const row of coverage.rows){assert(row.assignee&&row.consumer&&row.next_test&&row.classification);assert.equal(row.expected_outcome.acquisition_claim,false);assert.deepEqual(row.expected_outcome.planned_kinds,row.planned_kinds);for(const key of ['native_authority','browser_sources','consumer_sources']){assert(row[key]?.length);for(const source of row[key])assert(coverage.authority.some(pin=>pin.path===source),source+' unpinned');}}
const options=process.argv.slice(2);assert(options.every(arg=>arg.startsWith('--wasm='))&&options.length<=1);
const wasmFile=options[0]?.slice(7)||process.env.BISECT_TEST_WASM||'target/wasm32-unknown-unknown/release/bisect_wasm.wasm';
const bytes=await fs.readFile(wasmFile),engine=await instantiateEngine(bytes);
const root=path.resolve('target/fetch-source-coverage',sha(bytes).slice(0,16)+'-'+Date.now());await fs.mkdir(root,{recursive:true});
const planner=path.resolve('target/debug/examples/fetch_plan_reference.exe'),handoff=path.resolve('target/debug/examples/fletch_sources_reference.exe');
const builtin=JSON.parse(await fs.readFile('data/manifest.json','utf8'));
const sort=rows=>rows.sort((a,b)=>JSON.stringify([a.state_code,a.year,a.kind,a.local_path]).localeCompare(JSON.stringify([b.state_code,b.year,b.kind,b.local_path])));
const run=(file,args)=>execFileSync(file,args,{encoding:'utf8',maxBuffer:16*1024*1024,stdio:['ignore','pipe','pipe'],windowsHide:true});
const cases=[];
const selections=coverage.rows.map(row=>[row.data_type,[row.data_type],row.planned_kinds]);
selections.push(['empty',[],['adjacency','pl94171','tiger']],['all-and-extras',['all','elections','school-districts','eia-861'],['adjacency','pl94171','tiger','school-districts','eia-861']]);
for(const year of ['2000','2010','2020'])for(const [name,data_types,expectedKinds] of selections){
 const id=year+'-'+name,caseRoot=path.join(root,id);await fs.mkdir(caseRoot);
 const manifest={...builtin,local_data_dir:path.relative(process.cwd(),path.join(caseRoot,'data')).replaceAll('\\','/'),local_outputs_dir:path.relative(process.cwd(),path.join(caseRoot,'outputs')).replaceAll('\\','/')};
 const input={manifest_json:JSON.stringify(manifest),states:['RI','NC','RI'],year,data_types,selected_paths:[]};
 const request={operation:'plan-fetch-sources',input},actual=engine.execute(request);
 assert.equal(actual.execution_performed,false);assert.equal(actual.acquisition_performed,false);
 assert.deepEqual([...new Set(actual.items.map(item=>item.kind))].sort(),[...expectedKinds].sort(),id);
 const requestFile=path.join(caseRoot,'selection.json');await fs.writeFile(requestFile,JSON.stringify(input));
 const nativeText=run(planner,[requestFile]),native=JSON.parse(nativeText);await fs.writeFile(path.join(caseRoot,'native-plan.json'),nativeText);await fs.writeFile(path.join(caseRoot,'wasm-plan.json'),JSON.stringify(actual));
 assert.deepEqual(sort(actual.items.map(({check_status,...item})=>item)),sort(native),id+' native filesystem planner');
 if(expectedKinds.includes('eia-861'))assert.equal(actual.items.filter(item=>item.kind==='eia-861'&&item.state_code==='US').length,1,id+' national item');
 assert.equal(actual.warnings.length,name==='elections'?1:0,id+' warning semantics');
 const report=engine.execute({operation:'inspect-fletch-sources',input});
 const itemsFile=path.join(caseRoot,'native-items.json');await fs.writeFile(itemsFile,JSON.stringify(native));
 const nativeReportText=run(handoff,[itemsFile,path.join(caseRoot,'native-handoff.csv')]),nativeReport=JSON.parse(nativeReportText);
 assert.deepEqual(report.report,nativeReport.report,id+' FLETCH report');assert.deepEqual(report.gate_failures,nativeReport.gate_failures);
 assert.equal(report.execution_performed,false);await fs.writeFile(path.join(caseRoot,'native-handoff.json'),nativeReportText);await fs.writeFile(path.join(caseRoot,'wasm-handoff.json'),JSON.stringify(report));
 const eligibility=actual.items.map(item=>{try{publicFetchUrl(item);return {kind:item.kind,state:item.state_code,eligible:true};}catch{return {kind:item.kind,state:item.state_code,eligible:false};}});
 for(const row of eligibility)assert.equal(row.eligible,['tiger','pl94171','lodes-wac','lodes-od','acs-housing','school-districts','eia-861'].includes(row.kind),id+' current browser eligibility');
 cases.push({id,year,data_types,items:actual.items.length,warnings:actual.warnings.length,planned_kinds:[...new Set(actual.items.map(item=>item.kind))].sort(),eligibility,native_plan_equal:true,native_handoff_equal:true,acquisition_performed:false});
}
const evidence={schema_version:'bisect-fetch-source-coverage-evidence-v1',coverage_sha256:sha(await fs.readFile('docs/specs/wasm-fetch-source-coverage.json')),wasm:{path:wasmFile,sha256:sha(bytes),byte_length:bytes.length},native_planner:{path:planner,sha256:sha(await fs.readFile(planner))},native_handoff:{path:handoff,sha256:sha(await fs.readFile(handoff))},source_pins:coverage.authority,cases,scope:'Exact native planner/FLETCH projection comparisons and public browser eligibility only. No HTTP, extraction, filesystem acquisition, partition or complete lifecycle acceptance.'};
await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify(evidence,null,2)+'\n');
console.log(JSON.stringify({cases:cases.length,native_plan_comparisons:cases.length,native_handoff_comparisons:cases.length,evidence:path.join(root,'evidence.json'),acquisition_performed:false}));
