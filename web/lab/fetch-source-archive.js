import {decodeNativeWorkspaceArchive} from './native-workspace-source.js';
export async function decodeFetchSourceArchive(bytes,options={}){return decodeNativeWorkspaceArchive(bytes,{signal:options.signal,layout:'fetch-source'});}
export function readFetchSourceArchive(file,{signal}={}){
 if(!file||!Number.isSafeInteger(file.size)||file.size<22||file.size>64*1024*1024)return Promise.reject(new Error('Select a source ZIP up to 64 MiB.'));
 return new Promise((resolve,reject)=>{
  if(signal?.aborted){reject(signal.reason||new Error('Source archive cancelled.'));return;}
  const worker=new Worker(new URL('./fetch-source-archive-worker.js',import.meta.url),{type:'module'});let settled=false;
  const finish=(error,result)=>{if(settled)return;settled=true;clearTimeout(timer);signal?.removeEventListener('abort',abort);worker.terminate();error?reject(error):resolve(result);},abort=()=>finish(signal.reason||new Error('Source archive cancelled.')),timer=setTimeout(()=>finish(new Error('Source archive inspection timed out.')),60000);
  signal?.addEventListener('abort',abort,{once:true});worker.onerror=worker.onmessageerror=()=>finish(new Error('Source archive inspection failed.'));worker.onmessage=({data})=>data?.ok?finish(null,data.result):finish(new Error(data?.error||'Source archive inspection failed.'));worker.postMessage({file});
 });
}
