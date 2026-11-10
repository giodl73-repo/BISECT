import {commitFletchCacheBackup} from './fletch-cache-store.js';
export async function restoreFletchCacheBackup(indexFile,volumeFiles,options={}){
 const files=Array.from(volumeFiles||[]);
 if(!(indexFile instanceof Blob)||indexFile.size>40*1024*1024||!files.length||files.length>2000||files.some(f=>!(f instanceof Blob)||f.size>128*1024*1024))throw new Error('Select the backup index and all listed ZIP volumes.');
 const validated=await new Promise((resolve,reject)=>{
  if(options.signal?.aborted){reject(options.signal.reason);return;}
  const worker=new Worker(new URL('./fletch-cache-restore-worker.js',import.meta.url),{type:'module'});let done=false,timer;
  const finish=(error,result)=>{if(done)return;done=true;clearTimeout(timer);options.signal?.removeEventListener('abort',abort);worker.terminate();error?reject(error):resolve(result);};
  const abort=()=>finish(options.signal.reason||new Error('Cache restore cancelled.'));
  const reset=()=>{clearTimeout(timer);timer=setTimeout(()=>finish(new Error('Cache restore validation timed out.')),60000);};reset();options.signal?.addEventListener('abort',abort,{once:true});
  worker.onerror=worker.onmessageerror=()=>finish(new Error('Cache restore worker failed.'));
  worker.onmessage=({data})=>{if(data.type==='progress'){reset();try{options.onProgress?.(data);}catch(error){finish(error);}return;}data.ok===true?finish(null,data.result):finish(new Error(data.error||'Cache restore validation failed.'));};
  worker.postMessage({indexFile,volumeFiles:files});
 });
 options.signal?.throwIfAborted();options.assertCurrent?.();return commitFletchCacheBackup(validated,options);
}
