import {countResultTables} from '../../web/lab/count-results.js';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {FETCH_PLAN_DEFAULTS,readFetchPlanSource} from '../../web/lab/fetch-plan-files.js';
import {fletchSourcesRequest,fletchSourcesReportBytes} from '../../web/lab/fletch-source-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const root='target/fletch-sources-wasm';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const builtin=JSON.parse(await fs.readFile('data/manifest.json','utf8'));
const key=r=>JSON.stringify([r.state_code,r.year,r.kind,r.url,r.local_path,r.done_marker]);
for(const [name,changes,settings={year:'selection'}] of [
 ['default',{}],['older',{year:'2010'}],['oldest',{year:'2000'}],['all-vintages',{}, {year:'all'}],
 ['all-states-vintages',{states:[]},{year:'all'}],['optional',{states:['NC'],data_types:['lodes','lodes-od','acs-housing','school-districts','eia-861']}],
 ['duplicate-states',{states:['RI','RI']}],['empty',{states:['XX']}],['unsupported',{data_types:['elections']}],
 ['quoted-url',{quoted:true}]
]){
 const manifest=structuredClone(builtin);
 if(changes.quoted)manifest.states.RI.tiger['2020']='https://example.test/a,"b"\nc/tl_2020_44_tract.zip';
 const {quoted,...selectionChanges}=changes;
 const selection={...FETCH_PLAN_DEFAULTS,...selectionChanges,manifest_json:JSON.stringify(manifest)};
 const text=JSON.stringify(selection),bytes=new TextEncoder().encode(text);
 const source=await readFetchPlanSource({name:name+'.json',size:bytes.length,arrayBuffer:async()=>bytes.buffer});
 const request=fletchSourcesRequest({selection:{source},settings}),result=engine.execute(request);
 await fs.writeFile(root+'/planner-request.json',JSON.stringify(request.input));
 const planned=JSON.parse(execFileSync('target/debug/examples/fetch_plan_reference.exe',[root+'/planner-request.json'],{encoding:'utf8'}));
 assert.deepEqual(result.source_plan.items.map(key).sort(),planned.map(key).sort(),name+' native planner');
 await fs.writeFile(root+'/oracle.json',JSON.stringify(planned));
 const native=JSON.parse(execFileSync('target/debug/examples/fletch_sources_reference.exe',[root+'/oracle.json',root+'/native.csv'],{encoding:'utf8'}));
 assert.deepEqual(result.report,native.report,name+' upstream handoff');assert.deepEqual(result.gate_failures,native.gate_failures);
 const csv=await fs.readFile(root+'/native.csv');assert.deepEqual(Buffer.from(fletchSourcesReportBytes(result)),csv,name+' CSV');
 assert.equal(result.execution_performed,false);assert.equal(result.gate_passed,true);
 if(name==='quoted-url')assert.ok(result.report.rows.some(r=>r.source_url.includes('\n')&&r.source_url.includes('"b"')));
 if(name==='default'){assert.equal(result.report.adapter_source_count,1);assert.ok(result.report.rows.some(r=>r.handoff_status==='adapter-required'));}
 if(name==='all-states-vintages'){assert.equal(new Set(result.report.rows.map(r=>r.state_code)).size,50);assert.deepEqual([...new Set(result.report.rows.map(r=>r.year))].sort(),['2000','2010','2020']);}
 if(name==='duplicate-states')assert.equal(result.report.fletch_count,3);
 if(name==='empty'||name==='unsupported')assert.equal(result.report_csv,'');
 if(name==='empty')assert.ok(countResultTables(result)[0].rows.some(r=>r[0]==='Ignored states'&&r[1]==='XX'));
 if(name==='unsupported')assert.ok(countResultTables(result).some(t=>t.title==='Native Fetch warnings'&&t.rows[0][0].includes('not implemented')));
 const project=createProject({name:'Source handoff '+name,files:{fetchPlanInputs:{source},fletchSourceSettings:settings},operation:'inspect-fletch-sources',constraints:[],result,lastOperation:'inspect-fletch-sources'});
 assert.deepEqual(parseProject(JSON.stringify(project)),project);
 await fs.writeFile(root+'/'+name+'.bisect',JSON.stringify(project));await fs.writeFile(root+'/'+name+'-result.json',JSON.stringify(result));
}
assert.throws(()=>fletchSourcesRequest({selection:{source:null},settings:{year:'bad'}}));
console.log('FLETCH sources: ten actual WASM/native planner/upstream handoff/CSV comparisons, all states/vintages, adapters, deduplication, empty/unsupported sources, CSV quoting and inert project/source preservation passed.');
