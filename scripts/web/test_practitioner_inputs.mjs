import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {practitionerInputChecklist as checklist} from '../../web/lab/practitioner-inputs.js';
const missing=(op,files)=>checklist(op,files).filter(row=>!row.present).map(row=>row.label);
assert.deepEqual(missing('run-tui-configured',{}),['Algorithm configuration','Prepared state graph']);
const selected={name:'selected source',base64:'AA=='};
assert.deepEqual(missing('run-tui-configured',{configSource:selected,configRunInputs:{graph:selected}}),[]);
const native={nativeStateSourceSettings:{enabled:true},nativeAdjacencyInputs:{adjacency:selected,geoids:selected},nativeAdjacencySettings:{state:'ZZ',year:'2024',unit_kind:'imported'}};
const prior=JSON.stringify(native);assert.deepEqual(missing('run-native-selected-state-source',native),['Aligned county IDs']);assert.equal(JSON.stringify(native),prior);
native.nativeAdjacencyInputs.county_ids=selected;assert.deepEqual(missing('run-native-selected-state-source',native),[]);
// Optional geometry mode must not acquire a new unconditional prerequisite.
native.nativeAdjacencySettings.require_geometry=false;assert.deepEqual(missing('run-native-selected-state-source',native),[]);
native.nativeStateSourceSettings.command_kind='states';assert.deepEqual(missing('run-native-selected-state-source',native),['Single-State source mode']);
const census={nativeStateSourceSettings:{enabled:true},nativeSelectedStateSourceSettings:{kind:'census'},tigerInputs:{shp:selected,dbf:selected},tractPreparationSettings:{projection:selected},pl94Inputs:{year:2000,geography:selected}};
assert.deepEqual(missing('run-native-selected-state-source',census),[]);census.pl94Inputs.year=2020;assert.deepEqual(missing('run-native-selected-state-source',census),['PL94 population']);census.pl94Inputs.population=selected;assert.deepEqual(missing('run-native-selected-state-source',census),[]);
assert.deepEqual(missing('run-state-batch',{}),['Prepared state jobs']);assert.deepEqual(missing('validate-rplan-text',{}),['RPLAN plan']);assert.deepEqual(checklist('plan-fetch-sources',{}),[]);assert.deepEqual(checklist('inspect-tui-plan-library',{}),[]);
// These are presence hints: invalid selected contents still go to native validation.
assert.deepEqual(missing('validate-rplan-text',{auditPlanText:{name:'unvalidated',base64:'invalid'}}),[]);
const html=await fs.readFile('web/lab/workbench.html','utf8'),ids=new Set([...html.matchAll(/\bid="([^"]+)"/g)].map(m=>m[1]));
for(const [op,files]of [['run-tui-configured',{}],['run-native-selected-state-source',native],['run-native-selected-state-source',census],['run-state-batch',{}],['validate-rplan-text',{}]])for(const row of checklist(op,files))assert(ids.has(row.target),row.target);
console.log('PASS practitioner input hints: configured/native/custom/Census/year-2000/batch/plan prerequisites, optional geometry, native validation boundary, no mutation and actual control targets.');
