import assert from 'node:assert/strict';
import {Worker as NodeWorker} from 'node:worker_threads';
import {verifyAssignments} from '../../web/lab/assignment-verification.js';

const graph={geoids:['44001000100','44001000200'],population:[10,10],adjacency:[[1],[0]],edges:[[0,1,5]]};
const assignments={'44001000100':1,'44001000200':2};
const state={metrics:{district_count:2,units:2,population:20,max_deviation_percent:0,graph_boundary_m:5,weighted_boundary:5,split_counties:1,contiguous:true,balance_passed:true,districts:[1,2].map(district=>({district,population:10,units:1,components:1,deviation_percent:0}))}};
const config={structure:'standard-bisect',search:'single',weights:'geographic',balance_tolerance:5};
const workerUrl=new URL('../../web/lab/assignment-verification-worker.js',import.meta.url).href;
let terminated=0;
function realWorker(){
  const native=new NodeWorker(`const {parentPort}=require('node:worker_threads');globalThis.self={postMessage:data=>parentPort.postMessage(data)};import(${JSON.stringify(workerUrl)}).then(()=>parentPort.on('message',data=>self.onmessage({data})));`,{eval:true});
  const adapter={postMessage:data=>native.postMessage(data),terminate(){terminated++;native.terminate();}};
  native.on('message',data=>adapter.onmessage({data}));native.on('error',()=>adapter.onerror());
  return adapter;
}
await verifyAssignments(graph,state,assignments,config,{workerFactory:realWorker});
assert.equal(terminated,1);
const bad=structuredClone(state);bad.metrics.population++;
await assert.rejects(verifyAssignments(graph,bad,assignments,config,{workerFactory:realWorker}),/disagree/);
assert.equal(terminated,2);

let worker,deadline;
const originalSetTimeout=globalThis.setTimeout,originalClearTimeout=globalThis.clearTimeout;
const factory=()=>worker={postMessage(){},terminate(){this.terminated=true;}};
try{
  globalThis.setTimeout=(callback,ms)=>{assert.equal(ms,30000);deadline=callback;return 1;};
  globalThis.clearTimeout=()=>{};
  const options={workerFactory:factory};
  const expired=verifyAssignments(graph,state,assignments,config,options);deadline();
  await assert.rejects(expired,/timed out.*kept/);assert.equal(worker.terminated,true);
  const controller=new AbortController();
  const cancelled=verifyAssignments(graph,state,assignments,config,{...options,signal:controller.signal});controller.abort();
  await assert.rejects(cancelled,/cancelled/);assert.equal(worker.terminated,true);
  worker=null;await assert.rejects(verifyAssignments(graph,state,assignments,config,{...options,signal:controller.signal}),/cancelled/);assert.equal(worker,null);
  for(const event of ['onerror','onmessageerror']){
    const failed=verifyAssignments(graph,state,assignments,config,options);worker[event]();
    await assert.rejects(failed,/failed.*kept/);assert.equal(worker.terminated,true);
  }
  const malformed=verifyAssignments(graph,state,assignments,config,options);worker.onmessage({data:null});
  await assert.rejects(malformed,/failed/);assert.equal(worker.terminated,true);
  const accepted=verifyAssignments(graph,state,assignments,config,options);worker.onmessage({data:{ok:true}});deadline();
  await accepted;assert.equal(worker.terminated,true);
}finally{globalThis.setTimeout=originalSetTimeout;globalThis.clearTimeout=originalClearTimeout;}
console.log('Project evidence workers: real assignment validation and tamper rejection; deadline, cancellation, crash, malformed reply and termination passed.');
