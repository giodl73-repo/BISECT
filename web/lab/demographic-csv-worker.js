import {instantiateEngine} from './wasm-engine.js';
import {encodeBytes} from './package-files.js';
import {validateDemographicInput} from './demographic-input.js';

// A dedicated, bounded worker keeps file parsing separate from active searches.
self.onmessage=async({data})=>{
  try{
    const {file,metadata,sha256}=data;
    if(!file||!Number.isSafeInteger(file.size)||file.size<1||file.size>8*1024*1024||! /^[a-f0-9]{64}$/.test(sha256||''))throw new Error('Invalid demographic CSV import.');
    const source=new Uint8Array(await file.arrayBuffer());
    if(!source.length||source.length>8*1024*1024)throw new Error('Demographic CSV exceeds 8 MiB.');
    const response=await fetch(new URL('./bisect_wasm.wasm',import.meta.url));
    if(!response.ok)throw new Error('Browser demographic importer unavailable.');
    const bytes=await response.arrayBuffer(),actual=[...new Uint8Array(await crypto.subtle.digest('SHA-256',bytes))].map(b=>b.toString(16).padStart(2,'0')).join('');
    if(actual!==sha256)throw new Error('Browser demographic importer failed its integrity check.');
    const engine=await instantiateEngine(bytes);
    const project=validateDemographicInput(engine.execute({operation:'import-demographic-csv',source_base64:encodeBytes(source),state:metadata.state,year:metadata.year,basis:metadata.basis,source_label:metadata.source_label}));
    self.postMessage({ok:true,project});
  }catch(error){self.postMessage({ok:false,error:error.message||'Invalid demographic CSV.'});}
};
