import {offlineSelection,offlineDigest,validateOfflineManifest,offlineManifestText} from './offline-package.js';
const base=new URL('./',self.location.href),prefix='bisect-offline-v1-'+encodeURIComponent(base.pathname)+'-',pointerName=prefix+'pointer',pointerUrl=new URL('__offline_active__',base).href;
let installing=false;
self.addEventListener('install',event=>event.waitUntil(self.skipWaiting()));
self.addEventListener('activate',event=>event.waitUntil(self.clients.claim()));
async function active(){if(!await caches.has(pointerName))return null;const cache=await caches.open(pointerName),r=await cache.match(pointerUrl);return r?await r.json():null;}
async function clientAllowed(event){const client=await self.clients.get(event.source?.id);if(!client)return false;const url=new URL(client.url);return url.origin===base.origin&&url.pathname.startsWith(base.pathname);}
async function install(states,port){
 const manifestAbort=new AbortController(),manifestDeadline=setTimeout(()=>manifestAbort.abort(),60000);let text;
 try{const response=await fetch(new URL('offline-manifest.json',base),{cache:'no-store',redirect:'error',signal:manifestAbort.signal});if(!response.ok)throw new Error('Offline manifest unavailable.');text=await offlineManifestText(response);}finally{clearTimeout(manifestDeadline);}
 const manifest=validateOfflineManifest(JSON.parse(text)),selection=offlineSelection(manifest,states),name=prefix+manifest.version+'-'+crypto.randomUUID(),cache=await caches.open(name);
 let committed=false,received=0;const abort=new AbortController(),deadline=setTimeout(()=>abort.abort(),10*60*1000);
 try{
  for(const [i,path]of selection.paths.entries()){
   const expected=manifest.assets[path],url=new URL(path,base),r=await fetch(url,{cache:'no-store',redirect:'error',credentials:'same-origin',signal:abort.signal});
   if(!r.ok||r.type==='opaque'||r.headers.get('Content-Length')&&Number(r.headers.get('Content-Length'))>expected.bytes)throw new Error('Offline download failed: '+path);
   const reader=r.body.getReader(),chunks=[];let length=0;
   try{while(true){const {value,done}=await reader.read();if(done)break;length+=value.length;if(length>expected.bytes)throw new Error('Offline asset exceeds recorded size: '+path);chunks.push(value);}}catch(error){await reader.cancel();throw error;}
   if(length!==expected.bytes)throw new Error('Offline asset size mismatch: '+path);
   const bytes=new Uint8Array(length);let offset=0;for(const chunk of chunks){bytes.set(chunk,offset);offset+=chunk.length;}
   if(await offlineDigest(bytes)!==expected.sha256)throw new Error('Offline asset hash mismatch: '+path);
   const headers=new Headers(r.headers);headers.delete('content-encoding');headers.delete('content-length');headers.delete('set-cookie');
   await cache.put(url,new Response(bytes,{status:200,headers}));received+=length;
   port.postMessage({type:'progress',completed:i+1,total:selection.paths.length,bytes:received});
  }
  await cache.put(new URL('offline-manifest.json',base),new Response(text,{headers:{'Content-Type':'application/json'}}));
  const pointer=await caches.open(pointerName),record={version:manifest.version,cache_name:name,states:selection.states,bytes:selection.bytes+new TextEncoder().encode(text).length,paths:[...selection.paths,'offline-manifest.json']};
  // This single write publishes a fully verified package. Failure keeps the old one.
  await pointer.put(pointerUrl,new Response(JSON.stringify(record),{headers:{'Content-Type':'application/json'}}));committed=true;
  for(const old of await caches.keys())if(old.startsWith(prefix)&&old!==pointerName&&old!==name)await caches.delete(old).catch(()=>{});
  return record;
 }finally{clearTimeout(deadline);if(!committed)await caches.delete(name);}
}
self.addEventListener('message',event=>{
 const port=event.ports[0];if(!port)return;
 event.waitUntil((async()=>{
  try{
   if(!await clientAllowed(event))throw new Error('Offline request has no matching application client.');
   const type=event.data?.type;
   if(type==='status'){port.postMessage({type:'complete',installed:await active()});return;}
   if(!['install','remove'].includes(type)||installing)throw new Error('Finish the current offline operation first.');
   installing=true;try{
    if(type==='install')port.postMessage({type:'complete',installed:await install(event.data.states,port)});
    else{for(const name of await caches.keys())if(name.startsWith(prefix))await caches.delete(name);port.postMessage({type:'complete',installed:null});}
   }finally{installing=false;}
  }catch(error){port.postMessage({type:'error',error:String(error.message)});}
 })());
});
self.addEventListener('fetch',event=>{
 const url=new URL(event.request.url);if(event.request.method!=='GET'||url.origin!==base.origin||!url.pathname.startsWith(base.pathname))return;
 event.respondWith((async()=>{
  const record=await active();let path=url.pathname.slice(base.pathname.length);if(!path||path==='toolkit/')path+='index.html';
  if(record?.paths?.includes(path)&&record.cache_name.startsWith(prefix)){
   const cache=await caches.open(record.cache_name),cached=await cache.match(new URL(path,base));if(cached)return cached;
   return new Response('Installed offline asset is missing. Reinstall the offline package when online.',{status:503,headers:{'Content-Type':'text/plain'}});
  }
  // Never cache user projects, provider responses, arbitrary queries or writes.
  return fetch(event.request);
 })());
});
