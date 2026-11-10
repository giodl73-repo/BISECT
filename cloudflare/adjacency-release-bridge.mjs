import {ADJACENCY_RELEASE_DEFAULTS,discoverAdjacencyRelease,retireAdjacencyRelease} from '../web/lab/adjacency-release.js';
export const RELEASE_BRIDGE_PATH='/api/adjacency-release';
const headers={'x-bisect-release-bridge':'v1','Cache-Control':'no-store','Content-Type':'application/octet-stream','X-Content-Type-Options':'nosniff'};
const refuse=(status,message)=>new Response(message,{status,headers:{...headers,'Content-Type':'text/plain; charset=utf-8'}});
async function boundedBody(body,limit,signal){if(!body)throw 0;const reader=body.getReader(),parts=[],abort=()=>reader.cancel().catch(()=>{});signal.addEventListener('abort',abort,{once:true});let count=0;try{signal.throwIfAborted();for(;;){const p=await reader.read();signal.throwIfAborted();if(p.done)break;if((count+=p.value.length)>limit)throw 0;parts.push(p.value);}const b=new Uint8Array(count);let i=0;for(const p of parts){b.set(p,i);i+=p.length;}return b;}finally{signal.removeEventListener('abort',abort);await reader.cancel().catch(()=>{});reader.releaseLock();}}
export async function bridgeAdjacencyRelease(request,{fetcher=fetch,timeoutMs=60000}={}){
 const url=new URL(request.url);
 if(request.method!=='POST')return refuse(405,'Use an explicit POST.');
 if(url.search||request.headers.get('Origin')!==url.origin||request.headers.get('Sec-Fetch-Site')&&!['same-origin','none'].includes(request.headers.get('Sec-Fetch-Site')))return refuse(403,'This bridge is same-origin only.');
 if(request.headers.get('x-bisect-release-bridge')!=='v1'||request.headers.get('content-type')!=='application/json')return refuse(400,'Use the explicit release bridge protocol.');
 const authorization=request.headers.get('Authorization')||'';
 if(authorization&&!/^Bearer [\x21-\x7e]{1,512}$/.test(authorization))return refuse(400,'Invalid credential format.');
 const controller=new AbortController(),abort=()=>controller.abort();request.signal.addEventListener('abort',abort,{once:true});if(request.signal.aborted)abort();const timer=setTimeout(abort,timeoutMs);
 let plan,reader,upstream;const close=()=>{clearTimeout(timer);request.signal.removeEventListener('abort',abort);retireAdjacencyRelease(plan);};
 const provider=(target,options)=>fetcher(target,{...options,headers:{...options.headers,'User-Agent':'BISECT-release-bridge'}});
 try{
  const body=JSON.parse(new TextDecoder('utf-8',{fatal:true}).decode(await boundedBody(request.body,2048,controller.signal)));
  if(!['asset_id,repository,state,tag,year','asset_id,purpose,repository,state,tag,year'].includes(Object.keys(body).sort().join())||body.purpose!==undefined&&!['native-inputs','native-fetch-bundle'].includes(body.purpose)||body.repository!==ADJACENCY_RELEASE_DEFAULTS.repository||body.tag!==ADJACENCY_RELEASE_DEFAULTS.tag||!Number.isSafeInteger(body.asset_id)||body.asset_id<1)return close(),refuse(400,'Select an asset in this site’s configured data release.');
  const {asset_id,purpose='native-inputs',...scope}=body;
  plan=await discoverAdjacencyRelease(scope,{purpose,token:authorization.slice(7),fetcher:provider,signal:controller.signal,timeoutMs});
  const asset=plan.assets.find(a=>a.id===asset_id);if(!asset)return close(),refuse(400,'Asset is not a matching native input in the selected release.');
  let target=asset.url;
  for(let hop=0;hop<4;hop++){
   const api=new URL(target).origin==='https://api.github.com';
   upstream=await provider(target,{method:'GET',redirect:'manual',cache:'no-store',credentials:'omit',referrerPolicy:'no-referrer',signal:controller.signal,headers:{Accept:'application/octet-stream','Accept-Encoding':'identity',...(api&&authorization?{Authorization:authorization}:{})}});
   if(upstream.status<300||upstream.status>=400)break;
   const location=upstream.headers.get('location');await upstream.body?.cancel();const next=new URL(location||'',target);
   if(!location||next.protocol!=='https:'||next.username||next.password||next.hash||!['release-assets.githubusercontent.com','objects.githubusercontent.com'].includes(next.hostname)||hop===3)throw 0;
   target=next.href;
  }
  if(!upstream.ok){const status=upstream.status;await upstream.body?.cancel();close();return refuse(status>=400&&status<600?status:502,'Release provider refused the asset.');}
  const length=upstream.headers.get('content-length'),encoding=upstream.headers.get('content-encoding');
  if(!upstream.body||encoding&&encoding!=='identity'||length!==null&&(!/^\d+$/.test(length)||Number(length)!==asset.size)||(new URL(target).origin==='https://api.github.com'||purpose==='native-inputs')&&/json/i.test(upstream.headers.get('content-type')||''))throw 0;
  reader=upstream.body.getReader();let count=0;
  const stream=new ReadableStream({async pull(output){try{const part=await reader.read();controller.signal.throwIfAborted();if(part.done){if(count!==asset.size)throw 0;reader.releaseLock();close();output.close();return;}count+=part.value.length;if(count>asset.size)throw 0;output.enqueue(part.value);}catch{controller.abort();await reader.cancel().catch(()=>{});close();output.error(new Error('Release bridge stream failed.'));}},async cancel(){controller.abort();await reader.cancel().catch(()=>{});close();}});
  return new Response(stream,{headers:{...headers,'Content-Length':String(asset.size)}});
 }catch{controller.abort();try{await upstream?.body?.cancel();}catch{}close();return refuse(502,'Release bridge could not validate or read the selected asset. Use native download and local import.');}
}
