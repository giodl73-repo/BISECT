import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {planImportRequest} from '../../web/lab/package-files.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const root='target/polygon-import-wasm';await fs.mkdir(root,{recursive:true});let matched=0,rejected=0;
const ring=(x1,x2)=>[[x1,47],[x2,47],[x2,48],[x1,48],[x1,47]];
const polygon=(id,rings)=>({type:'Feature',properties:{district_id:id},geometry:{type:'Polygon',coordinates:rings}});
const geometry={type:'FeatureCollection',features:[polygon(1,[ring(-123,-122)]),polygon(2,[ring(-122,-121)])]};
const centroids={'53001000100':[-122.75,47.5],'53001000200':[-121.75,47.5],'53001000300':[-122,47.5],'53001000400':[-120,47.5]};
const settings={format:'geojson-polygons',state:'WA',year:2020,label:'Native polygon import fixture',created_at:'2026-10-06T00:00:00Z',source_name:'districts.geojson',as_civic_counter_proposal:false,submitted_by:null,submitted_at:null};
const base={operation:'import-plan-interchange',source_text:JSON.stringify(geometry),centroids_json:JSON.stringify(centroids),context_json:null,settings};
async function native(req){await fs.writeFile(root+'/request.json',JSON.stringify(req));return JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[root+'/request.json'],{encoding:'utf8',maxBuffer:32*1024*1024,stdio:['ignore','pipe','pipe']}));}
async function exact(req){const out=engine.execute(req);assert.deepEqual(out,await native(req));matched++;return out;}
async function reject(req){assert.throws(()=>engine.execute(req));await assert.rejects(()=>native(req));rejected++;}
let out=await exact(base),plan=JSON.parse(out.document_json).plan;assert.deepEqual(plan.assignment,[0,1,0,1]);assert.deepEqual(out.polygon_diagnostic.nearest_centroid_units,['53001000300','53001000400']);assert.equal(out.polygon_diagnostic.point_in_polygon_count,2);assert.equal(out.polygon_diagnostic.centroids_sha256,createHash('sha256').update(base.centroids_json).digest('hex'));
const hole=structuredClone(geometry);hole.features[0].geometry.coordinates.push([[-122.6,47.4],[-122.4,47.4],[-122.4,47.6],[-122.6,47.6],[-122.6,47.4]]);
const holeOut=await exact({...base,source_text:JSON.stringify(hole),centroids_json:JSON.stringify({...centroids,'53001000500':[-122.5,47.5]})});assert.ok(holeOut.polygon_diagnostic.nearest_centroid_units.includes('53001000500'));assert.equal(JSON.parse(holeOut.document_json).plan.assignment.at(-1),0);
const multi=structuredClone(geometry);multi.features[1].geometry={type:'MultiPolygon',coordinates:[[ring(-122,-121)],[ring(-120,-119)]]};const multiOut=await exact({...base,source_text:JSON.stringify(multi),centroids_json:JSON.stringify({...centroids,'53001000500':[-119.5,47.5]})});assert.equal(JSON.parse(multiOut.document_json).plan.assignment.at(-1),1);assert.ok(!multiOut.polygon_diagnostic.nearest_centroid_units.includes('53001000500'));
const overlap=structuredClone(geometry);overlap.features[0].geometry.coordinates=[ring(-123,-121.5)];assert.equal(JSON.parse((await exact({...base,source_text:JSON.stringify(overlap)})).document_json).plan.assignment[1],0);overlap.features.reverse();assert.equal(JSON.parse((await exact({...base,source_text:JSON.stringify(overlap)})).document_json).plan.assignment[1],1);
for(const centroids_json of [null,'{}','{"bad":[-122,47]}','{"53001000100":[181,47]}','{"53001000100":[-122,91]}','{"53001000100":[-122,47],"53001000100":[-121,47]}','x'.repeat(3*1024*1024+1)])await reject({...base,centroids_json});
for(const mutate of [g=>g.features[0].geometry.coordinates[0].pop(),g=>g.features[0].properties.district_id=0,g=>g.features[0].properties.district_id=501,g=>g.features[0].geometry.type='Point',g=>g.features[0].geometry.coordinates[0][0]=[-190,47]]){const bad=structuredClone(geometry);mutate(bad);await reject({...base,source_text:JSON.stringify(bad)});}
const many=Object.fromEntries(Array.from({length:10000},(_,i)=>['53000'+String(i).padStart(6,'0'),[-122.5,47.5]]));const expensive=structuredClone(geometry);expensive.features[0].geometry.coordinates=[Array(5002).fill([-123,47])];await reject({...base,source_text:JSON.stringify(expensive),centroids_json:JSON.stringify(many)});
const files={planImportSource:{name:'districts.geojson',base64:Buffer.from(base.source_text).toString('base64')},planImportCentroids:{name:'centroids.json',base64:Buffer.from(base.centroids_json).toString('base64')},planImportSettings:settings};assert.deepEqual(planImportRequest(files,settings),base);const project=createProject({name:settings.label,files,operation:base.operation,constraints:[],result:out,lastOperation:base.operation});assert.equal(parseProject(JSON.stringify(project)).files.planImportCentroids.base64,files.planImportCentroids.base64);await fs.writeFile(root+'/browser.bisect',JSON.stringify(project));await fs.writeFile(root+'/fixture.json',JSON.stringify({base,output:out}));await fs.writeFile(root+'/imported.rplan',out.document_json);
console.log(`District polygons: ${matched} native/WASM matches, ${rejected} rejected inputs; independent inside/border/outside/hole/island/overlap assignments, reported fallbacks, centroid hashes and exact project inputs.`);
