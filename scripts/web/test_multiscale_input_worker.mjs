import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {Worker as NodeWorker} from 'node:worker_threads';
import {readMultiscaleBundle} from '../../web/lab/multiscale-input.js';
const url=new URL('../../web/lab/multiscale-input-worker.js',import.meta.url).href,original=globalThis.Worker;
let created=0,terminated=0;
try{
 globalThis.Worker=class{constructor(workerUrl){assert.equal(workerUrl.href,url);created++;this.worker=new NodeWorker(`const {parentPort}=require('node:worker_threads');globalThis.self={postMessage:data=>parentPort.postMessage(data)};import(${JSON.stringify(url)}).then(()=>parentPort.on('message',data=>self.onmessage({data})));`,{eval:true});this.worker.on('message',data=>this.onmessage({data}));this.worker.on('error',()=>this.onerror());}postMessage(data){this.worker.postMessage(data);}terminate(){terminated++;this.worker.terminate();}};
 const source=await fs.readFile('target/AL-SYNTHETIC-multiscale.json'),bundle=JSON.parse(source),file=value=>Object.assign(new Blob([value]),{name:'SYNTHETIC fine bundle.json'});
 assert.deepEqual(await readMultiscaleBundle(file(source)),bundle);
 for(const mutate of[b=>b.extra=1,b=>b.graph.population[0]=-1,b=>b.graph.geoids[0]=b.graph.geoids[1],b=>b.boundary_edges[0][2]=-1,b=>b.geometry.features[0].properties.geoid='bad',b=>b.geometry.features[0].geometry.coordinates[0][0][0]=200,b=>b.geometry.features[0].geometry.coordinates[0].pop()]){const bad=structuredClone(bundle);mutate(bad);await assert.rejects(readMultiscaleBundle(file(JSON.stringify(bad))));}
 await assert.rejects(readMultiscaleBundle(file(Buffer.from([255]))));
 await assert.rejects(readMultiscaleBundle(file('{"__proto__":{}}')));
 let reads=0;await assert.rejects(readMultiscaleBundle({size:25*1024*1024+1,arrayBuffer(){reads++;throw new Error('Must not read oversized input.');}}));assert.equal(reads,0);
 const abort=new AbortController();abort.abort();const before=created;await assert.rejects(readMultiscaleBundle(file(source),{signal:abort.signal}));assert.equal(created,before);
 const active=new AbortController(),pending=readMultiscaleBundle(file(source),{signal:active.signal});active.abort();await assert.rejects(pending);assert.equal(created,terminated);
 console.log(`Fine graph/map Worker: ${created} workers created/terminated; valid import, malformed geometry/graph, UTF-8, unknown fields, pre-read size and cancellation passed.`);
}finally{globalThis.Worker=original;}
