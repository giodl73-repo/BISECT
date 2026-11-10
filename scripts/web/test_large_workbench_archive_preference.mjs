import assert from 'node:assert/strict';
import {createStoredWorkbenchArchive,parseWorkbenchArchiveBytes} from '../../web/lab/workbench-archive.js';
const original={name:'Archive compression control',operation:'inspect-civic-manifests',constraints:[],files:{},result:null,lastOperation:null};
assert.equal(typeof await createStoredWorkbenchArchive(original),'string');
const data={...original,lastOperation:original.operation,result:{payload:'0123456789é'.repeat(3*1024*1024)}};
const packed=await createStoredWorkbenchArchive(data);
assert(packed instanceof Uint8Array);assert.equal(packed[0],31);assert.equal(packed[1],139);
const restored=await parseWorkbenchArchiveBytes(packed);
assert.deepEqual(restored.files,data.files);assert.deepEqual(restored.result,data.result);
const C=globalThis.CompressionStream,D=globalThis.DecompressionStream;
try{
 globalThis.CompressionStream=undefined;
 let fallback=await createStoredWorkbenchArchive(data);
 assert.equal(typeof fallback,'string');assert.deepEqual(JSON.parse(fallback).result,data.result);
 globalThis.CompressionStream=C;globalThis.DecompressionStream=undefined;
 fallback=await createStoredWorkbenchArchive(data);
 assert.equal(typeof fallback,'string');assert.deepEqual(JSON.parse(fallback).result,data.result);
 globalThis.DecompressionStream=D;globalThis.CompressionStream=class {constructor(){throw Error('controlled compression failure');}};
 fallback=await createStoredWorkbenchArchive(data);
 assert.equal(typeof fallback,'string');assert.deepEqual(JSON.parse(fallback).result,data.result);
}finally{globalThis.CompressionStream=C;globalThis.DecompressionStream=D;}
console.log('Large compression preference, ordinary JSON, exact UTF-8 roundtrip and bounded fallback passed.');
