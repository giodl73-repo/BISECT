import {encodeBytes,isBase64Encoding} from './package-files.js';
export const CIVIC_SNAPSHOT_LIMIT=5*1024*1024;
const utf8=s=>new TextEncoder().encode(s).length;
export function validateCivicSnapshots(observations){
 if(!Array.isArray(observations)||observations.length>20)throw new Error('Record up to 20 unique civic source URLs.');let total=0;const seen=new Set();
 for(const o of observations){
  if(!o||Object.keys(o).sort().join()!=='body_base64,content_type,error,fetched_at,headers_text,http_status,snapshot_format,url'||typeof o.url!=='string'||!o.url||utf8(o.url)>8192||seen.has(o.url)||typeof o.fetched_at!=='string'||!o.fetched_at||utf8(o.fetched_at)>100||typeof o.content_type!=='string'||utf8(o.content_type)>1000||!['headers-body','browser-fetch-observed-headers-body'].includes(o.snapshot_format))throw new Error('Invalid civic snapshot observations.');seen.add(o.url);
  if(o.error!==null){if(typeof o.error!=='string'||utf8(o.error)>1000||o.http_status!==0||o.body_base64!==null||o.headers_text!==null||o.content_type!=='')throw new Error('Failed observations require status zero and no response bytes.');continue;}
  if(!Number.isInteger(o.http_status)||o.http_status<100||o.http_status>599||typeof o.headers_text!=='string'||utf8(o.headers_text)>65536||typeof o.body_base64!=='string'||o.body_base64.length>6990508||!isBase64Encoding(o.body_base64))throw new Error('Invalid snapshot response bytes or metadata.');
  const bytes=Uint8Array.from(atob(o.body_base64),c=>c.charCodeAt(0));if(bytes.length>CIVIC_SNAPSHOT_LIMIT+1||encodeBytes(bytes)!==o.body_base64)throw new Error('Civic snapshot exceeds 5 MiB plus one truncation witness byte.');total+=bytes.length+utf8(o.headers_text);if(total>16*1024*1024)throw new Error('Civic snapshots exceed 16 MiB total.');
 }
 return observations;
}
export async function captureCivicSnapshots(descriptors,{fetcher=fetch,signal,isCurrent=()=>true,onProgress=()=>{},now=()=>new Date().toISOString(),timeoutMs=10000}={}){
 if(!Array.isArray(descriptors)||descriptors.length>20||!Number.isInteger(timeoutMs)||timeoutMs<1||timeoutMs>10000)throw new Error('Select up to 20 source URLs and a bounded timeout.');
 if(descriptors.some(d=>!d||typeof d.url!=='string'||!d.url||utf8(d.url)>8192||typeof d.native_valid!=='boolean')||new Set(descriptors.map(d=>d.url)).size!==descriptors.length)throw new Error('Source URLs must be unique and at most 8192 UTF-8 bytes.');
 const check=()=>{signal?.throwIfAborted();if(!isCurrent())throw new Error('Civic project changed during acquisition. Run again.');};const out=[];let total=0;
 for(const d of descriptors){
  check();const observation={url:d.url,fetched_at:now(),http_status:0,content_type:'',body_base64:null,headers_text:null,snapshot_format:'browser-fetch-observed-headers-body',error:null};const controller=new AbortController();let reader,timer;
  const stop=()=>controller.abort(signal.reason);signal?.addEventListener('abort',stop,{once:true});
  try{
   if(d.native_valid!==true)throw new Error('Source URL failed the native structural URL check.');
   const url=new URL(d.url);if(url.protocol!=='https:'||url.username||url.password)throw new Error('Browser snapshots require public HTTPS URLs without embedded credentials.');url.hash='';
   const abort=new Promise((_,reject)=>{controller.signal.addEventListener('abort',()=>reject(controller.signal.reason),{once:true});});timer=setTimeout(()=>controller.abort(new Error('Snapshot request timed out.')),timeoutMs);
   const acquisition=(async()=>{
    const response=await fetcher(url.href,{mode:'cors',credentials:'omit',redirect:'error',referrerPolicy:'no-referrer',signal:controller.signal});check();controller.signal.throwIfAborted();
    if(response.redirected||response.url&&response.url!==url.href||response.type==='opaque'||response.status<100||response.status>599)throw new Error('Response destination or metadata is unavailable.');
    const headers=[...response.headers.entries()].sort(([a],[b])=>a<b?-1:a>b?1:0).map(([k,v])=>k+': '+v+'\r\n').join('');const text='BROWSER-FETCH '+response.status+' '+response.statusText+'\r\n'+headers;if(utf8(text)>65536)throw new Error('Observed headers exceed 64 KiB.');
    let length=0;const chunks=[];
    if(response.body){reader=response.body.getReader();while(length<=CIVIC_SNAPSHOT_LIMIT){check();controller.signal.throwIfAborted();const part=await reader.read();check();controller.signal.throwIfAborted();if(part.done)break;const slice=part.value.subarray(0,CIVIC_SNAPSHOT_LIMIT+1-length);chunks.push(slice.slice());length+=slice.length;if(length>CIVIC_SNAPSHOT_LIMIT){try{await reader.cancel();}catch{}break;}}}
    const body=new Uint8Array(length);let offset=0;for(const chunk of chunks){body.set(chunk,offset);offset+=chunk.length;}return {http_status:response.status,content_type:response.headers.get('content-type')||'',headers_text:text,body_base64:encodeBytes(body)};
   })();
   Object.assign(observation,await Promise.race([acquisition,abort]));check();total+=atob(observation.body_base64).length+utf8(observation.headers_text);if(total>16*1024*1024)throw new Error('Civic snapshots exceed 16 MiB total.');
  }catch(error){check();if(error?.message?.includes('16 MiB'))throw error;observation.http_status=0;observation.content_type='';observation.body_base64=null;observation.headers_text=null;observation.error=error?.message?.includes('16 MiB')?error.message:'Browser snapshot unavailable (URL policy, redirect, CORS, network, timeout or body read failure).';}
  finally{clearTimeout(timer);signal?.removeEventListener('abort',stop);try{reader?.cancel().catch(()=>{});reader?.releaseLock();}catch{}}
  out.push(observation);onProgress(out.length+' / '+descriptors.length+' source URLs observed.');
 }
 check();return validateCivicSnapshots(out);
}
