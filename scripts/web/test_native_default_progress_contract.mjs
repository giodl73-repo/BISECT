// Portable default-progress gate; neither native binaries nor target fixtures are needed.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {validateNativeRoundMonitor,nativeRoundProgressText} from '../../web/lab/native-round-progress.js';
const engine=await instantiateEngine(await fs.readFile(process.argv[2]||'target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const graph={schema_version:1,state:'RI',year:'2020',geoids:Array.from({length:6},(_,i)=>'44001'+String(i).padStart(6,'0')),population:Array(6).fill(100),adjacency:Array.from({length:6},()=>[]),edges:[],areas:[],exterior_perimeters:[],centroids:[]};
for(let u=0;u<6;u++)for(const v of [u%3<2?u+1:-1,u<3?u+3:-1])if(v>=0){graph.adjacency[u].push(v);graph.adjacency[v].push(u);graph.edges.push([u,v,100]);}
const options={structure:'bfs-growth',weights:'geographic',search:'single',districts:2,seed:'18446744073709551615',seeds:1,steps:1,percentile:0,alpha_county:0,balance_tolerance:50,area_swing:1,iterations:10},events=[];let returned=false;
const result=engine.execute({graph,options},{onProgress:p=>{assert.equal(returned,false);events.push(structuredClone(validateNativeRoundMonitor(p)));}});returned=true;
assert.equal(result.round_capture,undefined);assert.equal(result.options.capture_rounds,undefined);assert.deepEqual(events.map(p=>p.phase),['entered','snapshot','closed']);assert.equal(events[1].assigned_units,6);assert.equal(events[2].index,'1');assert.equal(events[2].omitted,'0');assert.match(nativeRoundProgressText(events[2]),/Checks or retries may continue/);
const trace=engine.execute({graph,options:{...options,capture_rounds:true}});assert.deepEqual(trace.metrics,result.metrics);assert.deepEqual(trace.assignments,result.assignments);
const empty=[];const one=engine.execute({graph,options:{...options,districts:1}},{onProgress:p=>empty.push(p)});assert.equal(one.round_capture,undefined);assert.deepEqual(empty.map(p=>p.phase),['entered','closed']);assert.equal(empty.at(-1).index,'0');
assert.throws(()=>engine.execute({graph,options},{onProgress(){throw Error('default observer refused');}}),/default observer refused/);assert.deepEqual(engine.execute({graph,options}),result);
for(const change of [{index:'01'},{index:'18446744073709551616'},{notified:'0'},{omitted:'1'},{units:1000001},{name:'<html>'},{phase:'certified'},{complete_universe:false},{extra:1}])assert.throws(()=>validateNativeRoundMonitor({...events[1],...change}));
// Sparse summaries and u64 counts are checked exactly, without numeric rounding.
const sparse={...events[1],index:'32',notified:'17',omitted:'15'};validateNativeRoundMonitor(sparse);assert.throws(()=>validateNativeRoundMonitor({...sparse,index:'33'}));
console.log('PASS default progress contract: real pre-return observations without traces, exact counters/summary, no-event disclosure, unchanged assignments/metrics, host failure cleanup and closed-schema/u64 guards');
