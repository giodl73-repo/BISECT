import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {countResultTables} from '../../web/lab/count-results.js';

const prepared=result=>({schema_version:'bisect-prepared-state-run-v1',prepared:{result}});
const row=(state,result,status='ok')=>({state_code:state,status,result});
const batch=states=>({schema_version:'bisect-state-run-batch-v1',status:'completed',states,summary:{total:states.length,completed:states.length,succeeded:states.filter(r=>r.status==='ok').length,failed:states.filter(r=>r.status==='failed').length}});
const native={within_requested_tolerance:false,max_deviation_percent:59.61723342619095,contiguous:false};
const result=batch([
 row('RI',prepared({metrics:native})),
 row('MA',{schema_version:'bisect-tui-run-v1',configured_run:{result:{metrics:{within_requested_tolerance:true,max_deviation_percent:0,contiguous:true}}}}),
 row('CT',prepared({within_requested_tolerance:true,max_deviation_percent:0.25,connected:[true,false]})),
 row('VT',prepared({connected:[]})),
 row('ME',prepared({metrics:native}),'failed'),
 row('NH',prepared({within_requested_tolerance:null,max_deviation_percent:NaN,connected:[true,'true']}))
]);
const before=structuredClone(result),tables=countResultTables(result),states=tables.find(t=>t.title==='State results');
assert.deepEqual(states.rows.map(r=>r.slice(1,5)),[
 ['Completed','No',String(native.max_deviation_percent),'No'],
 ['Completed','Yes','0','Yes'],['Completed','Yes','0.25','No'],
 ['Completed','Not recorded','Not recorded','Not recorded'],
 ['failed','Not recorded','Not recorded','Not recorded'],
 ['Completed','Not recorded','Not recorded','Not recorded']
]);
assert.deepEqual(result,before,'Presentation must not mutate saved native receipts');
assert.deepEqual(tables[0].rows.find(r=>r[0]==='Executed'),['Executed',5]);
assert.match(states.note,/does not establish a valid plan/);
const years=countResultTables({schema_version:'bisect-state-run-years-v1',years:[{year:2020,result},{year:2010,error:'No jobs'}]});
assert.deepEqual(years.find(t=>t.title==='2020 state results').rows,states.rows);
assert.equal(years.some(t=>t.title==='2010 state results'),false);
// Optional acceptance mode uses the retained whole native/WASM parity receipts.
// Default mode is self-contained and does not require ignored pipeline artifacts.
if(process.argv.includes('--retained-native')){
 const load=async name=>JSON.parse(await fs.readFile('target/context-geometry-attributes/'+name,'utf8'));
 const cvd=await load('tract-geographic-cvd-run.json'),configured=await load('tract-configured.json');
 assert.equal(cvd.result.metrics.within_requested_tolerance,false);
 assert.equal(cvd.result.metrics.contiguous,false);
 const actual=countResultTables(batch([row('RI',{schema_version:'bisect-prepared-state-run-v1',prepared:cvd}),row('MA',{schema_version:'bisect-tui-run-v1',configured_run:configured})]))[1];
 assert.deepEqual(actual.rows[0].slice(2,5),['No',String(cvd.result.metrics.max_deviation_percent),'No']);
 assert.deepEqual(actual.rows[1].slice(2,5),['Yes',String(configured.result.metrics.max_deviation_percent),'Yes']);
 console.log('PASS retained native CVD rejection and configured balanced candidate presentation');
}
console.log('PASS candidate checks: ordinary, configured, multiscale, failed, absent/malformed, multi-year; receipts unchanged');
