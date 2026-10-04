import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {Worker as NodeWorker} from 'node:worker_threads';
import {readPartisanFile} from '../../web/lab/partisan-input.js';
const root=process.argv[2]||'dist/wasm-partisan-tsv',manifest=JSON.parse(await fs.readFile(path.join(root,'catalog.json'),'utf8'));
const moduleFile=path.resolve(root,'bisect_wasm.wasm'),workerModule=new URL('../../web/lab/partisan-tsv-worker.js',import.meta.url).href;
const original=globalThis.Worker;let created=0,terminated=0;
try{
 globalThis.Worker=class{
  constructor(url){assert.equal(url.href,workerModule);created++;this.worker=new NodeWorker(`const {parentPort}=require('node:worker_threads');const fs=require('node:fs/promises');globalThis.crypto=require('node:crypto').webcrypto;globalThis.self={postMessage:data=>parentPort.postMessage(data)};globalThis.fetch=async()=>{const b=await fs.readFile(${JSON.stringify(moduleFile)});return{ok:true,arrayBuffer:async()=>b.buffer.slice(b.byteOffset,b.byteOffset+b.length)}};import(${JSON.stringify(workerModule)}).then(()=>parentPort.on('message',data=>self.onmessage({data})));`,{eval:true});this.worker.on('message',data=>this.onmessage({data}));this.worker.on('error',()=>this.onerror());}
  postMessage(data){this.worker.postMessage(data);}terminate(){terminated++;this.worker.terminate();}
 };
 const file=(bytes)=>Object.assign(new Blob([bytes]),{name:'Synthetic.tsv'}),tsvMetadata={state:'AL',year:'2020',source_label:'SYNTHETIC worker test'},options={tsvMetadata,wasmSha256:manifest.wasm_sha256};
 const valid=file('geoid\tdem_share\n01001020100\t0.5\n');
 const imported=await readPartisanFile(valid,options);assert.deepEqual(imported.dem_shares,{'01001020100':0.5});
 await assert.rejects(readPartisanFile(valid,{...options,wasmSha256:'0'.repeat(64)}),/integrity/);
 await assert.rejects(readPartisanFile(file(Uint8Array.of(0xff)),options),/UTF-8/);
 await assert.rejects(readPartisanFile(valid,{...options,tsvMetadata:{...tsvMetadata,state:'RI'}}),/selected state/);
 await assert.rejects(readPartisanFile(file('geoid\tdem_share\n01001020100\t0.5\n01001020100\t0.5\n'),options),/duplicate/);
 assert.equal(created,5);assert.equal(terminated,5);
 const before=created;await assert.rejects(readPartisanFile({name:'huge.tsv',size:8*1024*1024+1},options),/8 MiB/);assert.equal(created,before);
 const controller=new AbortController();controller.abort();await assert.rejects(readPartisanFile(valid,{...options,signal:controller.signal}),/cancelled/);assert.equal(created,before);
}finally{globalThis.Worker=original;}
console.log('Partisan TSV Workers: actual Rust/WASM conversion, share retention, integrity/UTF-8/scope/duplicate rejection, pre-read size/cancellation and termination passed.');
