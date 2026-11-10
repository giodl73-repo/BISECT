import {parseSafeJson} from './project.js';
import {readNativeAdjacencyBundle} from './native-adjacency-bundle.js';
import {characterStateFips} from './character-input.js';
export const ADJACENCY_RELEASE_DEFAULTS=Object.freeze({repository:'giodl73-repo/REDIST',tag:'data-inputs-v1'});
const sessions=new WeakMap(),api='https://api.github.com',hash=async b=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',b)),v=>v.toString(16).padStart(2,'0')).join('');
export function validateAdjacencyReleaseScope(value){
 if(!value||Object.keys(value).sort().join()!=='repository,state,tag,year'||typeof value.repository!=='string'||!/^[A-Za-z0-9][A-Za-z0-9_.-]{0,99}\/[A-Za-z0-9][A-Za-z0-9_.-]{0,99}$/.test(value.repository)||value.repository.split('/').some(p=>['.','..'].includes(p))||typeof value.tag!=='string'||!value.tag||value.tag.length>200||/[\x00-\x20\x7f]/.test(value.tag)||!characterStateFips(value.state)||!['2000','2010','2020'].includes(value.year))throw new Error('Choose a GitHub owner/repository, release tag, Census state and year.');return structuredClone(value);
}
function credential(token){if(typeof token!=='string'||token.length>512||/[^\x21-\x7e]/.test(token))throw new Error('Use an optional GitHub token without whitespace.');return token;}
async function requestBytes(url,{token='',binary=false,maxBytes,expectedSize,fetcher=fetch,signal,assertCurrent=()=>{},timeoutMs=30000,expectedResponseUrl=null,allowJsonAsset=false}){
 if(!Number.isSafeInteger(timeoutMs)||timeoutMs<1||timeoutMs>300000)throw new Error('Choose a bounded release request timeout.');
 signal?.throwIfAborted();assertCurrent();const controller=new AbortController(),abort=()=>controller.abort(signal.reason),timer=setTimeout(()=>controller.abort(new Error('GitHub release request timed out.')),timeoutMs);signal?.addEventListener('abort',abort,{once:true});let response,reader;
 const failure=new Promise((_,reject)=>controller.signal.addEventListener('abort',()=>reject(controller.signal.reason),{once:true}));
 try{return await Promise.race([failure,(async()=>{
  // API version headers survive cross-origin redirects and trigger an unsupported
  // asset-host preflight. Binary redirects need only the safelisted Accept header
  // after the browser strips Authorization at the origin boundary.
  const headers={Accept:binary?'application/octet-stream':'application/vnd.github+json',...(binary?{}:{'X-GitHub-Api-Version':'2022-11-28'}),...(token?{Authorization:'Bearer '+token}:{})};
  try{response=await fetcher(url,{method:'GET',headers,credentials:'omit',cache:'no-store',referrerPolicy:'no-referrer',redirect:binary?'follow':'error',mode:'cors',signal:controller.signal});}catch{controller.signal.throwIfAborted();throw new Error('GitHub release transport failed. Check browser access and use local file import if needed.');}
  controller.signal.throwIfAborted();assertCurrent();
  if(!response.ok)throw new Error('GitHub release request returned HTTP '+response.status+(response.status===404?' (release absent or inaccessible with this credential).':'.'));
  const final=new URL(response.url||url);
  if(expectedResponseUrl&&(final.href!==expectedResponseUrl||response.headers.get('x-bisect-release-bridge')!=='v1'))throw new Error('This site’s release bridge is unavailable. Use native download and local import.');
  if(!expectedResponseUrl&&(final.username||final.password||final.protocol!=='https:'||(!binary?(response.redirected||final.href!==url):!(final.href===url||['release-assets.githubusercontent.com','objects.githubusercontent.com','github.com'].includes(final.hostname)))))throw new Error('GitHub release response used an unexpected destination.');
  if(binary&&(!allowJsonAsset||final.origin===api)&&/json/i.test(response.headers.get('content-type')||''))throw new Error('GitHub returned asset metadata instead of a binary download.');
  const length=response.headers.get('content-length');if(length!==null&&(!/^\d+$/.test(length)||Number(length)>maxBytes))throw new Error('GitHub release response exceeds its byte limit.');
  if(!response.body)throw new Error('GitHub release response has no readable body.');reader=response.body.getReader();const chunks=[];let total=0;
  for(;;){const part=await reader.read();controller.signal.throwIfAborted();assertCurrent();if(part.done)break;total+=part.value.byteLength;if(total>maxBytes)throw new Error('GitHub release response exceeds its byte limit.');chunks.push(part.value);}
  if(length!==null&&Number(length)!==total||expectedSize!==undefined&&total!==expectedSize)throw new Error('GitHub release asset length differs from its published metadata.');
  const bytes=new Uint8Array(total);let offset=0;for(const chunk of chunks){bytes.set(chunk,offset);offset+=chunk.byteLength;}return bytes;
 })()]);}finally{clearTimeout(timer);signal?.removeEventListener('abort',abort);try{await reader?.cancel();reader?.releaseLock();}catch{}try{if(!reader)await response?.body?.cancel();}catch{}}
}
const json=bytes=>parseSafeJson(new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(bytes));
function assetEndpointMatches(value,base,id){try{const url=new URL(value);return url.origin===api&&!url.username&&!url.password&&!url.search&&!url.hash&&url.pathname.toLowerCase()===new URL(base+'/releases/assets/'+id).pathname.toLowerCase();}catch{return false;}}
export function retireAdjacencyRelease(plan){const session=sessions.get(plan);if(session)session.token='';sessions.delete(plan);}
export async function discoverAdjacencyRelease(scope,{token='',bridgeOrigin=null,purpose='native-inputs',...options}={}){
 if(!['native-inputs','native-fetch-bundle'].includes(purpose))throw new Error('Choose native inputs or the complete native Fetch bundle explicitly.');
 if(bridgeOrigin!==null){const origin=new URL(bridgeOrigin);if(origin.origin!==bridgeOrigin||origin.protocol!=='https:'&&!(origin.protocol==='http:'&&['127.0.0.1','localhost'].includes(origin.hostname))||globalThis.location&&bridgeOrigin!==globalThis.location.origin)throw new Error('Release bridge must use this site’s origin.');}
 const selected=validateAdjacencyReleaseScope(scope),secret=credential(token),base=api+'/repos/'+selected.repository;
 const release=json(await requestBytes(base+'/releases/tags/'+encodeURIComponent(selected.tag),{...options,token:secret,maxBytes:2*1024*1024}));
 if(!Number.isSafeInteger(release?.id)||release.id<1||release.tag_name!==selected.tag||typeof release.draft!=='boolean'||typeof release.prerelease!=='boolean')throw new Error('GitHub release identity differs from the selected tag.');
 const stem=selected.state.toLowerCase()+'_adjacency_'+selected.year,names=new Set([stem+'.adj.bin',stem+'_geoids.json',stem+'_centroids.json']),assets=[],ids=new Set(),seen=new Set();let bytes=0,finished=false;
 for(let page=1;page<=20;page++){
  const rows=json(await requestBytes(base+'/releases/'+release.id+'/assets?per_page=100&page='+page,{...options,token:secret,maxBytes:2*1024*1024}));
  if(!Array.isArray(rows)||rows.length>100)throw new Error('Invalid GitHub release asset page.');
  for(const asset of rows){
   if(!Number.isSafeInteger(asset?.id)||asset.id<1||ids.has(asset.id))throw new Error('Release asset inventory changed or contains duplicate IDs.');ids.add(asset.id);
   if(purpose==='native-inputs'?!names.has(asset.name):typeof asset.name!=='string'||!asset.name.startsWith(stem))continue;
   if(purpose==='native-fetch-bundle'&&(!/^[A-Za-z0-9_.-]{1,180}$/.test(asset.name)||/[.]$/.test(asset.name)||asset.name.toLowerCase()===stem+'.done'||assets.length>=256))throw new Error('Matching release names are unsafe, collide with the completion marker or exceed 256 files.');
   if(seen.has(asset.name.toLowerCase())||asset.state!=='uploaded'||!assetEndpointMatches(asset.url,base,asset.id)||!Number.isSafeInteger(asset.size)||asset.size<(purpose==='native-inputs'?1:0)||asset.size>(purpose==='native-fetch-bundle'||asset.name.endsWith('.adj.bin')?64:16)*1024*1024||(bytes+=asset.size)>64*1024*1024||asset.digest!=null&&!/^sha256:[a-f0-9]{64}$/.test(asset.digest))throw new Error('Invalid, duplicate or oversized matching release asset.');
   seen.add(asset.name.toLowerCase());assets.push({id:asset.id,name:asset.name,size:asset.size,digest:asset.digest??null,url:base+'/releases/assets/'+asset.id});
  }
  if(rows.length<100){finished=true;break;}
 }
 if(!finished)throw new Error('Release discovery exceeds 2,000 assets; use explicit local import.');
 if(purpose==='native-fetch-bundle'&&!assets.length)throw new Error('This release has no files matching the native Fetch wildcard.');
 if(purpose==='native-inputs'&&!seen.has(stem+'.adj.bin'))throw new Error('This release has no matching native .adj.bin. Pickle files cannot be executed.');
 assets.sort((a,b)=>a.name.localeCompare(b.name));options.signal?.throwIfAborted();options.assertCurrent?.();
 const plan=Object.freeze({schema_version:'bisect-adjacency-release-selection-v1',purpose,...selected,release_id:release.id,draft:release.draft,prerelease:release.prerelease,assets:Object.freeze(assets.map(a=>Object.freeze({...a}))),byte_length:bytes,published_digests:assets.filter(a=>a.digest).length});
 sessions.set(plan,{scope:selected,token:secret,assets:structuredClone(assets),release_id:release.id,bridgeOrigin,purpose});return plan;
}
export async function acquireAdjacencyRelease(plan,options={}){
 const session=sessions.get(plan);if(!session||session.active)throw new Error('Discover this release again. Saved, active or copied metadata grants no download authority.');session.active=true;
 const ensure=()=>{if(sessions.get(plan)!==session)throw new Error('Release selection was retired. Discover again.');options.assertCurrent?.();};
 const files=[],receipts=[];try{
  for(const asset of session.assets){
   options.onProgress?.({name:asset.name});
   ensure();const bridgeUrl=session.bridgeOrigin?session.bridgeOrigin+'/api/adjacency-release':null,upstreamFetch=options.fetcher||fetch;
   const fetcher=bridgeUrl?(_url,init)=>upstreamFetch(bridgeUrl,{...init,method:'POST',redirect:'error',headers:{Accept:'application/octet-stream','Content-Type':'application/json','x-bisect-release-bridge':'v1',...(session.token?{Authorization:'Bearer '+session.token}:{})},body:JSON.stringify({...session.scope,asset_id:asset.id,...(session.purpose==='native-fetch-bundle'?{purpose:session.purpose}:{})})}):upstreamFetch;
   const bytes=await requestBytes(asset.url,{...options,fetcher,expectedResponseUrl:bridgeUrl,allowJsonAsset:session.purpose==='native-fetch-bundle',assertCurrent:ensure,token:session.token,binary:true,maxBytes:asset.size,expectedSize:asset.size}),sha256=await hash(bytes);options.signal?.throwIfAborted();ensure();
   if(asset.digest&&asset.digest!=='sha256:'+sha256)throw new Error('Release asset SHA-256 differs from its published digest.');
   files.push(session.purpose==='native-fetch-bundle'?{name:asset.name,bytes}:new File([bytes],asset.name));receipts.push({asset_id:asset.id,name:asset.name,byte_length:bytes.length,sha256,published_digest:asset.digest,published_digest_verified:asset.digest!==null});
  }
  const imported=session.purpose==='native-fetch-bundle'?{opaque_files:files}:await readNativeAdjacencyBundle(files,{state:session.scope.state,year:session.scope.year},{signal:options.signal,assertCurrent:ensure});
  return {...imported,release_receipt:{schema_version:'bisect-adjacency-release-acquisition-v1',purpose:session.purpose,...session.scope,release_id:session.release_id,files:receipts,authentication:'not-recorded',transport:session.bridgeOrigin?'site-bridge':'direct',execution_performed:false,source_authenticity_verified:false,scope:session.purpose==='native-fetch-bundle'?'Complete bounded native wildcard bytes only, including opaque files. No file is decoded or executed. A separate portable ZIP can reproduce native layout/marker bytes; it grants no installation or authenticity authority.':'GitHub asset metadata and observed bytes; supplied SHA-256 digests checked when present. No token, signed redirect URL, disk installation, native completion marker or execution grant is retained.'}};
 }finally{retireAdjacencyRelease(plan);}
}
