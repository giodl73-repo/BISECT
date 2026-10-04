import {instantiateEngine} from './wasm-engine.js';
import {encodeBytes} from './package-files.js';
import {validatePartisanInput} from './partisan-input.js';

self.onmessage=async({data})=>{
  try{
    const {file,metadata,sha256}=data;
    if(!file||!Number.isSafeInteger(file.size)||file.size<1||file.size>8*1024*1024||! /^[a-f0-9]{64}$/.test(sha256||''))throw new Error('Invalid partisan TSV import.');
    const source=new Uint8Array(await file.arrayBuffer());
    if(!source.length||source.length>8*1024*1024)throw new Error('Partisan TSV exceeds 8 MiB.');
    const response=await fetch(new URL('./bisect_wasm.wasm',import.meta.url));
    if(!response.ok)throw new Error('Browser partisan importer unavailable.');
    const bytes=await response.arrayBuffer(),actual=[...new Uint8Array(await crypto.subtle.digest('SHA-256',bytes))].map(b=>b.toString(16).padStart(2,'0')).join('');
    if(actual!==sha256)throw new Error('Browser partisan importer failed its integrity check.');
    const engine=await instantiateEngine(bytes);
    const project=validatePartisanInput(engine.execute({operation:'import-partisan-shares-tsv',source_base64:encodeBytes(source),state:metadata.state,year:metadata.year,source_label:metadata.source_label}));
    self.postMessage({ok:true,project});
  }catch(error){self.postMessage({ok:false,error:error.message||'Invalid partisan TSV.'});}
};
