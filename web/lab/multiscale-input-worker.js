import {parseSafeJson} from './project.js';
import {validateMultiscaleBundle} from './multiscale-input.js';
self.onmessage=async({data:file})=>{try{if(!file||file.size>25*1024*1024)throw new Error('Fine input exceeds 25 MiB.');const bytes=await file.arrayBuffer();const result=validateMultiscaleBundle(parseSafeJson(new TextDecoder('utf-8',{fatal:true}).decode(bytes)));self.postMessage({ok:true,project:result});}catch(e){self.postMessage({ok:false,error:e.message});}};
