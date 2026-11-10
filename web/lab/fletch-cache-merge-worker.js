export function mergeFletchCacheInWorker(base,entries,options={}){return executeFletchCacheToolInWorker({operation:'merge-fletch-cache-manifest',cache_manifest_json:base,entries_json:entries},options);}
export function executeFletchCacheToolInWorker(request,{signal}={}){
 return new Promise((resolve,reject)=>{
  if(signal?.aborted){reject(signal.reason||new Error('Cache action cancelled.'));return;}
  const worker=new Worker(new URL('./wasm-worker.js',import.meta.url),{type:'module'});let settled=false;
  const finish=(error,result)=>{if(settled)return;settled=true;clearTimeout(timer);signal?.removeEventListener('abort',abort);worker.terminate();error?reject(error):resolve(result);};
  const abort=()=>finish(signal.reason||new Error('Cache action cancelled.')),timer=setTimeout(()=>finish(new Error('Native cache operation timed out.')),60000);
  signal?.addEventListener('abort',abort,{once:true});worker.onerror=worker.onmessageerror=()=>finish(new Error('Native cache worker failed.'));
  worker.onmessage=({data})=>{if(data.type==='ready')worker.postMessage({type:'run',id:1,request});else if(data.type==='result'&&data.id===1)finish(null,data.result);else if(data.type==='error')finish(new Error(data.error||'Native cache operation failed.'));};
  worker.postMessage({type:'initialize',url:new URL('./bisect_wasm.wasm',import.meta.url).href});
 });
}
