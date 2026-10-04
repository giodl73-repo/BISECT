import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {Worker as NodeWorker} from 'node:worker_threads';
import {readElectionFile} from '../../web/lab/election-input.js';
const root=process.argv[2]||'dist/wasm-election-csv',manifest=JSON.parse(await fs.readFile(path.join(root,'catalog.json'),'utf8'));
const moduleFile=path.resolve(root,'bisect_wasm.wasm'),workerModule=new URL('../../web/lab/election-csv-worker.js',import.meta.url).href;
const original=globalThis.Worker;let created=0,terminated=0;
try{
 globalThis.Worker=class{
  constructor(url){assert.equal(url.href,workerModule);created++;this.worker=new NodeWorker(`const {parentPort}=require('node:worker_threads');const fs=require('node:fs/promises');globalThis.crypto=require('node:crypto').webcrypto;globalThis.self={postMessage:data=>parentPort.postMessage(data)};globalThis.fetch=async()=>{const b=await fs.readFile(${JSON.stringify(moduleFile)});return{ok:true,arrayBuffer:async()=>b.buffer.slice(b.byteOffset,b.byteOffset+b.length)}};import(${JSON.stringify(workerModule)}).then(()=>parentPort.on('message',data=>self.onmessage({data})));`,{eval:true});this.worker.on('message',data=>this.onmessage({data}));this.worker.on('error',()=>this.onerror());}
  postMessage(data){this.worker.postMessage(data);}terminate(){terminated++;this.worker.terminate();}
 };
 const file=(bytes)=>Object.assign(new Blob([bytes]),{name:'Synthetic.csv'}),csvMetadata={state:'AL',year:'2020',election_year:'2016',source_label:'SYNTHETIC worker test'},options={csvMetadata,wasmSha256:manifest.wasm_sha256};
 const valid=file('geoid,dem_votes,rep_votes\n01001020100,800,500\n');
 const imported=await readElectionFile(valid,options);assert.deepEqual(imported.counts,{'01001020100':{democratic:800,two_party:1300}});
 await assert.rejects(readElectionFile(valid,{...options,wasmSha256:'0'.repeat(64)}),/integrity/);
 await assert.rejects(readElectionFile(file(Uint8Array.of(0xff)),options),/UTF-8/);
 await assert.rejects(readElectionFile(valid,{...options,csvMetadata:{...csvMetadata,state:'RI'}}),/selected state/);
 await assert.rejects(readElectionFile(file('geoid,dem_votes,rep_votes\n01001020100,800,500\n01001020100,800,500\n'),options),/duplicate/);
 assert.equal(created,5);assert.equal(terminated,5);
 const before=created;await assert.rejects(readElectionFile({name:'huge.csv',size:8*1024*1024+1},options),/8 MiB/);assert.equal(created,before);
 const controller=new AbortController();controller.abort();await assert.rejects(readElectionFile(valid,{...options,signal:controller.signal}),/cancelled/);assert.equal(created,before);
}finally{globalThis.Worker=original;}
console.log('Election CSV Workers: actual Rust/WASM conversion, count retention, integrity/UTF-8/scope/duplicate rejection, pre-read size/cancellation and termination passed.');
