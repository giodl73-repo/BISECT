import {instantiateEngine} from './wasm-engine.js';
let engine;
self.onmessage=async ({data})=>{
  try {
    if (data.type==='initialize') {
      const response=await fetch(data.url);
      if (!response.ok) throw new Error('Browser engine download failed.');
      const bytes=await response.arrayBuffer();
      if(data.sha256){const hash=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>b.toString(16).padStart(2,'0')).join('');if(hash!==data.sha256)throw new Error('Browser engine failed its SHA-256 integrity check.');}
      engine=await instantiateEngine(bytes);
      self.postMessage({type:'ready'});
    } else if (data.type==='run') {
      if (!engine) throw new Error('Browser engine is not initialized.');
      self.postMessage({type:'started',id:data.id});
      const result=engine.execute(data.request);
      self.postMessage({type:'result',id:data.id,result,memoryBytes:engine.memoryBytes()});
    } else throw new Error('Unknown worker request.');
  } catch(error) { self.postMessage({type:'error',id:data.id,error:error.message}); }
};
