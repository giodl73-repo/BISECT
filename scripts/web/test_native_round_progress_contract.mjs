// Portable build gate: no retained target fixtures or native executable required.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {validateNativeRoundProgress} from '../../web/lab/native-round-progress.js';
const engine=await instantiateEngine(await fs.readFile(process.argv[2]||'target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const graph={schema_version:1,state:'RI',year:'2020',geoids:Array.from({length:6},(_,i)=>'44001'+String(i).padStart(6,'0')),population:Array(6).fill(100),adjacency:Array.from({length:6},()=>[]),edges:[],areas:[],exterior_perimeters:[],centroids:[]};
for(let u=0;u<6;u++)for(const v of [u%3<2?u+1:-1,u<3?u+3:-1])if(v>=0){graph.adjacency[u].push(v);graph.adjacency[v].push(u);graph.edges.push([u,v,100]);}
const options={structure:'bfs-growth',weights:'geographic',search:'single',districts:2,seed:'18446744073709551615',seeds:1,steps:1,percentile:0,alpha_county:0,balance_tolerance:50,area_swing:1,iterations:10,capture_rounds:true},events=[];let returned=false;
const result=engine.execute({graph,options},{onProgress:p=>{assert.equal(returned,false);events.push(structuredClone(validateNativeRoundProgress(p)));}});returned=true;assert(events.length>0);assert.equal(events.length,result.round_capture.events.length);
for(const [i,p]of events.entries()){const frame=result.round_capture.events[i];assert.equal(p.index,i+1);assert.equal(p.name,frame.name);assert.equal(p.assigned_units,frame.units);assert.equal(p.units,6);assert.equal(p.complete_universe,frame.complete_universe);}
const silent=[];const plain=engine.execute({graph,options:{...options,capture_rounds:undefined}},{onProgress:p=>silent.push(p)});assert.deepEqual(silent.filter(p=>p.schema_version==='bisect-native-round-progress-v1'),[]);assert.deepEqual(plain.assignments,result.assignments);assert.deepEqual(plain.metrics,result.metrics);
assert.throws(()=>engine.execute({graph,options},{onProgress(){throw Error('progress host failure');}}),/progress host failure/);assert.deepEqual(engine.execute({graph,options}),result);
for(const change of [{index:0},{index:129},{units:0},{assigned_units:7},{complete_universe:!events[0].complete_universe},{name:'<html>'},{extra:1}])assert.throws(()=>validateNativeRoundProgress({...events[0],...change}));
console.log('PASS native round progress contract: real pre-return events, exact retained snapshot summaries, unchanged output, capture-off trace-event silence, host refusal cleanup and invalid-event guards');
