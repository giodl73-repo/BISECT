import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {Worker as NodeWorker} from 'node:worker_threads';
import {readCharacterFile} from '../../web/lab/character-input.js';
const root=process.argv[2]||'dist/wasm-character',manifest=JSON.parse(await fs.readFile(path.join(root,'catalog.json'),'utf8'));
const moduleFile=path.resolve(root,'bisect_wasm.wasm'),workerModule=new URL('../../web/lab/character-csv-worker.js',import.meta.url).href;
const original=globalThis.Worker;let created=0,terminated=0;
try{
 globalThis.Worker=class{
  constructor(url){assert.equal(url.href,workerModule);created++;this.worker=new NodeWorker(`const {parentPort}=require('node:worker_threads');const fs=require('node:fs/promises');globalThis.crypto=require('node:crypto').webcrypto;globalThis.self={postMessage:data=>parentPort.postMessage(data)};globalThis.fetch=async()=>{const b=await fs.readFile(${JSON.stringify(moduleFile)});return{ok:true,arrayBuffer:async()=>b.buffer.slice(b.byteOffset,b.byteOffset+b.length)}};import(${JSON.stringify(workerModule)}).then(()=>parentPort.on('message',data=>self.onmessage({data})));`,{eval:true});this.worker.on('message',data=>this.onmessage({data}));this.worker.on('error',()=>this.onerror());}
  postMessage(data){this.worker.postMessage(data);}terminate(){terminated++;this.worker.terminate();}
 };
 const file=bytes=>Object.assign(new Blob([bytes]),{name:'Synthetic.csv'}),csvMetadata={state:'AL',year:'2020',data_year:'2022',kind:'housing',source_label:'SYNTHETIC worker test'},options={csvMetadata,wasmSha256:manifest.wasm_sha256};
 const source='geoid,pct_single_family,pct_multifamily,pct_owner,housing_vintage\n01001020100,.5,.4,.75,.5\n',valid=file(source);
 const imported=await readCharacterFile(valid,options);assert.equal(imported.data.characters['01001020100'].pct_owner,.75);assert.equal(imported.data_year,'2022');
 const economic=await readCharacterFile(file('geoid,c000,cns01,cns02,cns05,cns07,cns08,cns09,cns10,cns11\n01001020100,1000,10,20,30,400,40,100,50,50'),{...options,csvMetadata:{...csvMetadata,kind:'economic'}});assert.equal(economic.data.raw_counts['01001020100'].c000,1000);assert.equal(economic.data.characters['01001020100'].commercial_intensity,.6);
 await assert.rejects(readCharacterFile(valid,{...options,wasmSha256:'0'.repeat(64)}),/integrity/);
 await assert.rejects(readCharacterFile(file(Uint8Array.of(0xff)),options),/UTF-8/);
 await assert.rejects(readCharacterFile(valid,{...options,csvMetadata:{...csvMetadata,state:'RI'}}),/selected state/);
 await assert.rejects(readCharacterFile(file(source+'01001020100,.5,.4,.75,.5'),options),/duplicate/);
 await assert.rejects(readCharacterFile(valid,{...options,csvMetadata:{...csvMetadata,data_year:'oops'}}),/four digits/);
 assert.equal(created,7);assert.equal(terminated,7);
 const before=created;await assert.rejects(readCharacterFile({name:'huge.csv',size:8*1024*1024+1},options),/8 MiB/);assert.equal(created,before);
 const controller=new AbortController();controller.abort();await assert.rejects(readCharacterFile(valid,{...options,signal:controller.signal}),/cancelled/);assert.equal(created,before);
}finally{globalThis.Worker=original;}
console.log('Character CSV Workers: actual Rust/WASM conversion, raw retention, integrity/UTF-8/scope/duplicate/year rejection, pre-read size/cancellation and termination passed.');
