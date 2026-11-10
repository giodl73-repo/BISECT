import {parseProject,MAX_PROJECT_BYTES} from './project.js';
import {instantiateEngine} from './wasm-engine.js';
import {recheckCharacterPreparation} from './character-recheck.js';
self.onmessage=async({data})=>{
 try{
  if(!data.file||!Number.isSafeInteger(data.size)||data.size<1||data.size>MAX_PROJECT_BYTES||data.file.size!==data.size||!/^[a-f0-9]{64}$/.test(data.sha256??''))throw new Error('Invalid bounded preparation recheck.');
  const bytes=new Uint8Array(await data.file.arrayBuffer());if(bytes.length!==data.size)throw new Error('Preparation packet changed while reading.');const project=parseProject(new TextDecoder('utf-8',{fatal:true}).decode(bytes));
  const response=await fetch(new URL('./bisect_wasm.wasm',import.meta.url));if(!response.ok)throw new Error('Browser preparation engine unavailable.');const wasm=await response.arrayBuffer(),sha=[...new Uint8Array(await crypto.subtle.digest('SHA-256',wasm))].map(b=>b.toString(16).padStart(2,'0')).join('');if(sha!==data.sha256)throw new Error('Preparation recheck engine integrity mismatch.');
  const engine=await instantiateEngine(wasm),receipt=await recheckCharacterPreparation(engine,project);self.postMessage({ok:true,project:receipt});
 }catch(e){self.postMessage({ok:false,error:e.message||'Preparation recheck failed.'});}
};
