import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {laboratorySnapshot,laboratorySnapshotGeoJSON,verifyLaboratoryCapture} from '../../web/lab/laboratory-round-capture.js';
const root='target/laboratory-snapshot-maps';await fs.mkdir(root,{recursive:true});
const project=JSON.parse(await fs.readFile('target/laboratory-round-capture/browser.bisect','utf8')),manifest=JSON.parse(await fs.readFile('dist/wasm-lab-build/catalog.json','utf8'));
const graph=JSON.parse(await fs.readFile('dist/wasm-lab-build/'+manifest.graphs['RI:2020'].graph_ref,'utf8')),geometry=JSON.parse(await fs.readFile('dist/wasm-lab-build/'+manifest.geometries['RI:2020'],'utf8'));
const capture=project.experiment.states[0].metrics.round_capture,before=JSON.stringify(project),geometryBefore=JSON.stringify(geometry);
for(const event of capture.events){const snapshot=laboratorySnapshot(capture,event.sequence),mapped=laboratorySnapshotGeoJSON(geometry,snapshot),index=JSON.parse(capture.unit_index_json),values=JSON.parse(event.assignments_json);for(const [key,value]of Object.entries(values))assert.equal(mapped.features.find(f=>f.properties.geoid===index[key]).properties.district,value);assert.equal(mapped.bisect_snapshot.sequence,event.sequence);}
assert.equal(JSON.stringify(project),before);assert.equal(JSON.stringify(geometry),geometryBefore);
for(const value of [-1,999,.5,'0',null])assert.throws(()=>laboratorySnapshot(capture,value));
// A clearly identified synthetic imported partial trace exercises neutral units.
const partial=structuredClone(project),trace=partial.experiment.states[0].metrics.round_capture,event=trace.events[0],values=JSON.parse(event.assignments_json),keys=Object.keys(values);for(const key of keys.filter((_,i)=>i%2))delete values[key];event.assignments_json=JSON.stringify(values);event.assignments_sha256=createHash('sha256').update(event.assignments_json).digest('hex');event.units=Object.keys(values).length;event.regions=new Set(Object.values(values)).size;event.complete_universe=false;event.matches_returned_assignment=false;event.name='synthetic_partial_browser_fixture';trace.terminal_matches_returned_assignment=false;trace.events=[event];trace.scope='Synthetic partial snapshot fixture for UI coverage checks; not a native execution claim.';
await verifyLaboratoryCapture(graph,partial.experiment.states[0],partial.assignments.RI,partial.experiment.config);
const snapshot=laboratorySnapshot(trace,0),mapped=laboratorySnapshotGeoJSON(geometry,snapshot);assert.equal(mapped.features.filter(f=>f.properties.district===null).length,graph.geoids.length-event.units);assert.equal(mapped.bisect_snapshot.complete_universe,false);
await fs.writeFile(root+'/partial.bisect',JSON.stringify(partial));await fs.writeFile(root+'/evidence.json',JSON.stringify({native_snapshot_events:capture.events.length,exact_index_to_geoid_mapping:true,source_project_and_geometry_unchanged:true,invalid_selections_refused:5,synthetic_partial_unassigned_neutral:true},null,2));
console.log('Snapshot maps: exact native index mapping, unchanged source project/geometry, invalid selections and synthetic partial neutral coverage passed.');
