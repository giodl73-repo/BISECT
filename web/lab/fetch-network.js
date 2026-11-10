export function validateFetchRequestOptions({retryAttempts=0,timeoutMs=30000}={}){
 if(!Number.isInteger(retryAttempts)||retryAttempts<0||retryAttempts>10||!Number.isSafeInteger(timeoutMs)||timeoutMs<1||timeoutMs>300000)throw new Error('Use 0–10 retries and a request timeout of 1–300000 ms.');return {retryAttempts,timeoutMs};
}
export function validateFetchAttemptStatus(value){
 if(!value||Object.keys(value).sort().join()!=='attempts,last_retryable_error,retries'||!Number.isInteger(value.attempts)||value.attempts<1||value.attempts>11||value.retries!==value.attempts-1||(value.retries>0&&typeof value.last_retryable_error!=='string')||(value.retries===0?value.last_retryable_error!==null:!['HTTP transport failed','HTTP response body read failed','HTTP request timed out'].includes(value.last_retryable_error)&&!/^HTTP status [1-5][0-9]{2}$/.test(value.last_retryable_error)))throw new Error('Invalid acquisition attempt status.');return value;
}
class RequestFailure extends Error{constructor(message,{retryable=false,status=null,category=null}={}){super(message);this.retryable=retryable;this.status=status;this.category=category;}}
export class SourceTransportError extends Error {}
export async function downloadPublicSource(requestUrl,{fetcher=fetch,signal,maxBytes,largeArchive=false,retryAttempts=0,timeoutMs=30000,onAttempt=()=>{}}={}){
 validateFetchRequestOptions({retryAttempts,timeoutMs});if(typeof largeArchive!=='boolean'||!Number.isSafeInteger(maxBytes)||maxBytes<1||maxBytes>(largeArchive?128:64)*1024*1024)throw new Error('Invalid source byte limit.');signal?.throwIfAborted();let retries=0,last=null;
 for(let attempts=1;attempts<=retryAttempts+1;attempts++){
  signal?.throwIfAborted();onAttempt({attempts,retries,max_attempts:retryAttempts+1});const controller=new AbortController();let timer,reader,response,abortListener;
  const stop=()=>controller.abort(signal.reason);signal?.addEventListener('abort',stop,{once:true});
  const aborted=new Promise((_,reject)=>{abortListener=()=>reject(controller.signal.reason);controller.signal.addEventListener('abort',abortListener,{once:true});});
  timer=setTimeout(()=>controller.abort(new RequestFailure('Source request timed out.',{retryable:true,category:'HTTP request timed out'})),timeoutMs);
  const cancelBody=()=>{try{const cancelled=reader?reader.cancel():response?.body?.cancel();cancelled?.catch(()=>{});}catch{}};
  const attempt=(async()=>{
   try{response=await fetcher(requestUrl,{mode:'cors',credentials:'omit',redirect:'error',referrerPolicy:'no-referrer',signal:controller.signal});}catch(error){controller.signal.throwIfAborted();if(error instanceof SourceTransportError)throw new RequestFailure(error.message);throw new RequestFailure('Source request failed. Check network access, provider browser access and the Census key for ACS. Local file import remains available.',{retryable:true,category:'HTTP transport failed'});}
   if(controller.signal.aborted){cancelBody();controller.signal.throwIfAborted();}
   if(response.redirected||response.url&&response.url!==requestUrl)throw new RequestFailure('Source response changed its destination.');
   if(!response.ok){cancelBody();throw new RequestFailure(`Source request returned HTTP ${response.status}.`,{retryable:true,status:response.status,category:'HTTP status '+response.status});}
   const limitError=`Source exceeds the ${maxBytes/1024/1024} MiB acquisition limit.`;if(Number(response.headers.get('content-length'))>maxBytes)throw new RequestFailure(limitError);
   if(!response.body)throw new RequestFailure('Source response has no readable body.');reader=response.body.getReader();const chunks=[];let total=0;
   for(;;){controller.signal.throwIfAborted();let part;try{part=await reader.read();}catch{controller.signal.throwIfAborted();throw new RequestFailure('Source response body could not be read.',{retryable:true,category:'HTTP response body read failed'});}controller.signal.throwIfAborted();if(part.done)break;total+=part.value.byteLength;if(total>maxBytes)throw new RequestFailure(limitError);chunks.push(part.value);}
   const bytes=new Uint8Array(total);let offset=0;for(const chunk of chunks){bytes.set(chunk,offset);offset+=chunk.length;}return {status:response.status,bytes};
  })();
  try{const downloaded=await Promise.race([attempt,aborted]);return {...downloaded,attempt_status:validateFetchAttemptStatus({attempts,retries,last_retryable_error:last})};}
  catch(error){signal?.throwIfAborted();if(error instanceof RequestFailure&&error.retryable&&attempts<=retryAttempts){retries++;last=error.category;continue;}
   if(error instanceof RequestFailure&&error.status===404)return {status:404,bytes:null,attempt_status:validateFetchAttemptStatus({attempts,retries,last_retryable_error:last})};throw error;
  }finally{clearTimeout(timer);signal?.removeEventListener('abort',stop);controller.signal.removeEventListener('abort',abortListener);cancelBody();try{reader?.releaseLock();}catch{}}
 }
}
