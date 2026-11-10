import assert from 'node:assert/strict';import fs from 'node:fs/promises';import path from 'node:path';import {execFileSync} from 'node:child_process';import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {TUI_LIBRARY_DEFAULTS,tuiLibraryRequest,readTuiLibraryFolder,tuiLibraryDoctorInputs,validateTuiLibraryInputs} from '../../web/lab/tui-library-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const root='target/tui-library-wasm';await fs.mkdir(root,{recursive:true});const base=path.resolve(root+'/native'),engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const fixture={
 'plans/a/manifest.json':JSON.stringify({label:'Alpha',state_code:'RI',chamber:'house',year:'2020',num_districts:75}),
 'plans/a/analysis/splits.json':'{"split_count":2}',
 'plans/a/analysis/contiguity.json':'{"all_contiguous":false}',
 'plans/a/analysis/compactness.json':'{"mean_polsby_popper":0.3}',
 'plans/a/analysis/summary.json':'{"max_deviation_pct":1.5}',
 'plans/b/manifest.json':JSON.stringify({label:'Beta',state_code:'VT',chamber:'congressional',year:'2020',num_districts:1}),
 'plans/b/analysis/splits.json':'{"split_count":1}',
 'plans/b/analysis/compactness.json':'{"mean_polsby_popper":0.3}',
 'plans/b/analysis/summary.json':'{"max_deviation_pct":0.5}',
 'plans/c/manifest.json':'{"state_code":"IA","chamber":"senate","year":"2020","num_districts":18446744073709551615}',
 'plans/c/analysis/compactness.json':'broken',
 'plans/c/analysis/splits.json':'{"split":99}',
 'plans/c/analysis/summary.json':'\ufeff{"max_deviation_pct":0.2}',
 'plans/d/manifest.json':'{broken',
 'plans/e/analysis/summary.json':'{"max_deviation_pct":0.1}',
 'plans/a/audit.json':'not read',
 'plans/a/maps/districts.png':'not read',
 'plans/a/a.rplan':'not read',
};
for(const [name,text]of Object.entries(fixture)){const file=path.join(base,'v1','2020',name);await fs.mkdir(path.dirname(file),{recursive:true});await fs.writeFile(file,text);}
const selected=Object.entries(fixture).map(([name,text])=>({name:path.basename(name),webkitRelativePath:name,size:Buffer.byteLength(text),arrayBuffer:async()=>{assert(!['audit.json','districts.png','a.rplan'].includes(path.basename(name)),'presence-only files must not be read');return new TextEncoder().encode(text).buffer;}}));
const inputs=await readTuiLibraryFolder(selected);assert.equal(inputs.plans.length,5);assert.deepEqual(inputs.plans[0].present,['audit.json','maps/districts.png','a.rplan']);
let cases=0;
for(const column of ['label','splits','pp','deviation'])for(const direction of ['asc','desc'])for(const filter of ['','RI','house','beta','absent']){
 const source={...inputs,settings:{...inputs.settings,filter,sort_column:column,sort_direction:direction}},result=engine.execute(tuiLibraryRequest(source)),native=JSON.parse(execFileSync('target/debug/examples/plan_library_reference.exe',[base,filter,column,direction],{encoding:'utf8',maxBuffer:8*1024*1024}));assert.deepEqual(result.plans,native);assert.equal(result.total_plans,3);assert.equal(result.skipped.length,2);assert.equal(result.execution_performed,false);cases++;
}
const result=engine.execute(tuiLibraryRequest(inputs));assert.equal(result.plans.find(r=>r.plan_dir==='plans/c').num_districts,'18446744073709551615');assert.equal(result.plans.find(r=>r.plan_dir==='plans/c').max_deviation_pct,null);
for(const source of result.source_hashes){const text=fixture[source.directory+'/'+source.path];assert.equal(source.bytes,Buffer.byteLength(text));assert.equal(source.sha256,createHash('sha256').update(text).digest('hex'));}
assert.deepEqual(tuiLibraryDoctorInputs(inputs,'plans/a').plans[0].present,inputs.plans[0].present);
let reads=0;await assert.rejects(()=>readTuiLibraryFolder([{name:'manifest.json',webkitRelativePath:'plans/a/manifest.json',size:1024*1024+1,arrayBuffer:async()=>{reads++;}}]));assert.equal(reads,0);await assert.rejects(()=>readTuiLibraryFolder([selected[0],selected[0]]),/Duplicate/);await assert.rejects(()=>readTuiLibraryFolder([{name:'manifest.json',webkitRelativePath:'plans/a/manifest.json',size:1,arrayBuffer:async()=>new Uint8Array([255]).buffer}]));
for(const bad of [{...inputs,settings:{...TUI_LIBRARY_DEFAULTS,sort_column:'bad'}},{...inputs,plans:[...inputs.plans,inputs.plans[0]]},{...inputs,plans:[{...inputs.plans[0],directory:'../escape'}]}])assert.throws(()=>validateTuiLibraryInputs(bad));
for(const input of [{...tuiLibraryRequest(inputs).input,extra:true},{...tuiLibraryRequest(inputs).input,filter:'\n'},{...tuiLibraryRequest(inputs).input,plans:[{...tuiLibraryRequest(inputs).input.plans[0],directory:'../escape'}]},{...tuiLibraryRequest(inputs).input,plans:Array(101).fill(tuiLibraryRequest(inputs).input.plans[0])}])assert.throws(()=>engine.execute({operation:'inspect-tui-plan-library',input}));
const project=createProject({name:'Native plan library',files:{tuiLibraryInputs:inputs},operation:'inspect-tui-plan-library',constraints:[],result,lastOperation:'inspect-tui-plan-library'});assert.deepEqual(parseProject(JSON.stringify(project)),project);await fs.writeFile(root+'/library.bisect',JSON.stringify(project));await fs.writeFile(root+'/result.json',JSON.stringify(result));await fs.writeFile(root+'/sources.json',JSON.stringify(fixture));
console.log(`Native TUI library: ${cases} actual discovery/filter/sort comparisons, exact u64 counts, malformed/missing/unknown/BOM behavior, presence-only preflight, path/field/count guards, Doctor handoff and inert source-project retention passed.`);
