import {instantiateEngine} from './wasm-engine.js';
self.onmessage=async({data})=>{try{
 const response=await fetch(new URL('./bisect_wasm.wasm',import.meta.url));if(!response.ok)throw new Error('Cache request engine is unavailable.');const engine=await instantiateEngine(await response.arrayBuffer());
 if(data.type==='key'){self.postMessage({ok:true,result:engine.execute({operation:'resolve-fletch-cache-request',input:{...data.input,force:false,offline:false,object_exists:false,observation:null,trusted_entries_json:'[]',expected_sha256:null,fetched_at_ms:null}})});return;}
 const input={...data.input};if(input.object_exists&&!input.force){if(!(data.blob instanceof Blob)||data.blob.size>64*1024*1024)throw new Error('Invalid cache request body.');input.observation={sha256:'sha256:'+await engine.hashFile(data.blob),bytes:String(data.blob.size),modified_at_ms:data.modified_at_ms};}
 self.postMessage({ok:true,result:engine.execute({operation:'resolve-fletch-cache-request',input})});
}catch(error){self.postMessage({ok:false,error:error.message||'Cache request failed.'});}};
