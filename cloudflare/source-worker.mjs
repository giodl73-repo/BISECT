import {SOURCE_RELAY_PATH,SOURCE_RELAY_HEADER,SOURCE_RELAY_VERSION,sourceRelayLimit} from '../web/lab/source-relay-policy.js';
import {RELEASE_BRIDGE_PATH,bridgeAdjacencyRelease} from './adjacency-release-bridge.mjs';
const reply=(message,status)=>new Response(message,{status,headers:{[SOURCE_RELAY_HEADER]:SOURCE_RELAY_VERSION,'Cache-Control':'no-store','Content-Type':'text/plain; charset=utf-8','X-Content-Type-Options':'nosniff'}});
export async function relaySource(request,{fetcher=fetch,timeoutMs=300000}={}){
 const url=new URL(request.url);
 if(!['GET','HEAD'].includes(request.method))return reply('Use GET or HEAD.',405);
 const origin=request.headers.get('Origin'),site=request.headers.get('Sec-Fetch-Site');
 if(origin&&origin!==url.origin||site&&!['same-origin','none'].includes(site))return reply('Source relay is same-origin only.',403);
 if([...url.searchParams.keys()].join()!=='url'||url.search.length>2048)return reply('Select exactly one pinned source URL.',400);
 const target=url.searchParams.get('url');let limit;try{limit=sourceRelayLimit(target);}catch{return reply('Source is outside the public allowlist.',400);}
 const controller=new AbortController(),abort=()=>controller.abort();request.signal.addEventListener('abort',abort,{once:true});if(request.signal.aborted)abort();
 const timer=setTimeout(abort,timeoutMs);let upstream,reader;
 const close=()=>{clearTimeout(timer);request.signal.removeEventListener('abort',abort);};
 try{
  upstream=await fetcher(target,{method:request.method,redirect:'manual',signal:controller.signal,headers:{'Accept':'application/octet-stream','Accept-Encoding':'identity'}});
  if(upstream.status>=300&&upstream.status<400||upstream.url&&upstream.url!==target){await upstream.body?.cancel();close();return reply('Provider redirected the pinned source.',502);}
  if(!upstream.ok){const status=upstream.status;await upstream.body?.cancel();close();return reply('Provider returned HTTP '+status+'.',status>=400&&status<=599?status:502);}
  const encoding=upstream.headers.get('content-encoding');if(encoding&&encoding!=='identity'){await upstream.body?.cancel();close();return reply('Provider applied an unsupported transfer encoding.',502);}
  const length=upstream.headers.get('content-length');if(length!==null&&(!/^\d+$/.test(length)||Number(length)>limit)){await upstream.body?.cancel();close();return reply('Source exceeds its relay byte limit.',413);}
  const headers=new Headers({[SOURCE_RELAY_HEADER]:SOURCE_RELAY_VERSION,'Cache-Control':'no-store','Content-Type':'application/octet-stream','X-Content-Type-Options':'nosniff'});if(length!==null)headers.set('Content-Length',length);
  if(request.method==='HEAD'){await upstream.body?.cancel();close();return new Response(null,{status:upstream.status,headers});}
  if(!upstream.body){close();return reply('Provider returned no source body.',502);}reader=upstream.body.getReader();let total=0;
  const stream=new ReadableStream({
   async pull(output){try{const part=await reader.read();controller.signal.throwIfAborted();if(part.done){if(length!==null&&total!==Number(length))throw new Error('Provider source length changed.');close();output.close();reader.releaseLock();return;}total+=part.value.byteLength;if(total>limit)throw new Error('Source exceeds its relay byte limit.');output.enqueue(part.value);}catch(error){close();controller.abort();try{await reader.cancel();}catch{}output.error(new Error('Source relay stream failed.'));}},
   async cancel(){close();controller.abort();try{await reader.cancel();}catch{}}
  });
  return new Response(stream,{status:upstream.status,headers});
 }catch{close();controller.abort();try{await upstream?.body?.cancel();}catch{}return reply('Provider source transport failed.',502);}
}
export default {fetch(request,env){const pathname=new URL(request.url).pathname;return pathname===SOURCE_RELAY_PATH?relaySource(request):pathname===RELEASE_BRIDGE_PATH?bridgeAdjacencyRelease(request):env.ASSETS.fetch(request);}};
