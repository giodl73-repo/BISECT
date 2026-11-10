// Fetch, verify, parse and encode selected graphs off the UI thread.
export function prepareBuildCatalogJobs(catalog,keys,chamber,inputs,{signal,timeoutMs=300000,WorkerClass=globalThis.Worker}={}){
 if(!Number.isSafeInteger(timeoutMs)||timeoutMs<1||timeoutMs>300000) return Promise.reject(new Error('Invalid preparation timeout.'));
 if(signal?.aborted)return Promise.reject(new Error('Build job preparation cancelled.'));
 return new Promise((resolve,reject)=>{
  let worker,timer,settled=false;
  const finish=(error,value)=>{if(settled)return;settled=true;clearTimeout(timer);signal?.removeEventListener('abort',abort);worker?.terminate();error?reject(error):resolve(value);};
  const abort=()=>finish(new Error('Build job preparation cancelled.'));
  try{
   worker=new WorkerClass(new URL('./config-catalog-worker.js?v=build-catalog-worker-v1',import.meta.url),{type:'module'});
   worker.onmessage=({data})=>{if(data?.ok===true)finish(null,data.value);else finish(new Error(data?.error||'Build job preparation failed.'));};
   worker.onerror=()=>finish(new Error('Build job preparation worker failed.'));
   signal?.addEventListener('abort',abort,{once:true});
   timer=setTimeout(()=>finish(new Error('Build job preparation timed out. Try fewer states.')),timeoutMs);
   worker.postMessage({type:'prepare-build-jobs',catalog,keys,chamber,inputs});
  }catch(error){finish(error);}
 });
}
