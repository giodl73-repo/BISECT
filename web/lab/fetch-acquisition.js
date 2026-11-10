import {validateFetchCachePolicy} from './fletch-cache-policy.js';
import {fletchHistoryFromEntry} from './fletch-history.js';
import {downloadPublicSource,validateFetchRequestOptions,validateFetchAttemptStatus,SourceTransportError} from './fetch-network.js';
import {characterStateFips} from './character-input.js?v=fetch-acquire-v1';
import {readAcsHousingSource} from './acs-housing-files.js';
import {readLodesSource} from './lodes-files.js?v=lodes-gzip-v1';
import {CENSUS_ARCHIVE_ENDPOINTS} from './census-endpoints.js';
import {CENSUS_RECORDED_ENDPOINTS} from './census-recorded-endpoints.js';
import {readCensusArchive,MAX_CENSUS_ZIP_BYTES} from './census-archive.js';
import {readFetchSourceArchive} from './fetch-source-archive.js';
import {encodeBytes,validateSourceFile} from './package-files.js';
import {sourceRelayUrl,SOURCE_RELAY_HEADER,SOURCE_RELAY_VERSION} from './source-relay-policy.js';
const MAX_BYTES=16*1024*1024;
const archiveKind=kind=>['tiger','pl94171'].includes(kind);
export const fetchZipKind=kind=>archiveKind(kind)||['school-districts','eia-861'].includes(kind);
// Historical labels can travel with retained bytes. Active requests still use
// publicFetchUrl, whose current endpoint allowlist remains unchanged.
function validateRecordedFetchUrl(item){
 try{return publicFetchUrl(item);}catch(error){
  if(item?.year==='2010'&&item.kind==='pl94171'&&typeof item.state_code==='string'){
   const recorded=CENSUS_RECORDED_ENDPOINTS[`${item.state_code}:2010:pl94171`];
   if(typeof recorded==='string'&&item.url===recorded)return recorded;
  }
  throw error;
 }
}
export function validateFetchArchiveSource(v){
 if(!v||Object.keys(v).sort().join()!=='kind,source,state,url,year'||!fetchZipKind(v.kind))throw new Error('Invalid retained raw public-source archive.');
 validateRecordedFetchUrl({state_code:v.state,year:v.year,kind:v.kind,url:v.url});
 validateSourceFile(v.source,{maxBytes:MAX_CENSUS_ZIP_BYTES});return v;
}
const byteLimit=(kind,profile)=>{if(profile!==undefined&&(profile!=='large-state'||!archiveKind(kind)))throw new Error('Invalid acquisition archive profile.');return kind==='pl94171'&&profile==='large-state'?128*1024*1024:fetchZipKind(kind)?MAX_CENSUS_ZIP_BYTES:MAX_BYTES;};
export function publicFetchUrl(item){
 const state=item?.state_code,year=item?.year,fips=characterStateFips(state);if((item?.kind==='eia-861'?state!=='US':!fips)||!['2000','2010','2020'].includes(year))throw new Error('Select a supported source jurisdiction and vintage.');
 let expected;if(item.kind==='acs-housing')expected=`https://api.census.gov/data/${year}/acs/acs5?get=B25024_001E,B25024_002E,B25024_003E,B25024_007E,B25024_008E,B25024_009E,B25003_001E,B25003_002E,B25035_001E,NAME&for=tract:*&in=state:${fips}`;
 else if(['lodes-wac','lodes-od'].includes(item.kind)){const abbr=state.toLowerCase(),kind=item.kind==='lodes-wac'?'wac':'od';expected=`https://lehd.ces.census.gov/data/lodes/LODES8/${abbr}/${kind}/${abbr}_${kind}_${kind==='wac'?'S000_JT00':'main_JT00'}_${year}.csv.gz`;}
 else if(archiveKind(item.kind)){expected=CENSUS_ARCHIVE_ENDPOINTS[`${state}:${year}:${item.kind}`];if(!expected)throw new Error('No pinned Census archive endpoint for this state/year.');}
 else if(item.kind==='school-districts')expected=`https://www2.census.gov/geo/tiger/TIGER${year}/UNSD/tl_${year}_${fips}_unsd.zip`;
 else if(item.kind==='eia-861')expected=`https://www.eia.gov/electricity/data/eia861/zip/f861${year}.zip`;
 else throw new Error('No public browser acquisition adapter for this source type. Use its explicit local import or release adapter.');
 if(item.url!==expected)throw new Error('Source URL does not match the native public endpoint for this state/year.');return expected;
}
export function fetchAcquisitionAvailability(result){if(result?.schema_version!=='bisect-fetch-source-plan-v1'||result.status!=='planned'||result.acquisition_performed!==false)return [];return (result.items||[]).map((item,index)=>{try{publicFetchUrl(item);return {item,index,eligible:true,reason:null};}catch(error){return {item,index,eligible:false,reason:error.message};}});}
export function fetchAcquisitionChoices(result){return fetchAcquisitionAvailability(result).filter(row=>row.eligible).map(({item,index})=>({item,index}));}
export function validateFetchAcquisition(v){
 if(Object.hasOwn(v||{},'archive_profile')&&v.archive_profile!=='large-state')throw new Error('Invalid acquisition archive profile.');
 const cached=v?.transport==='browser-cache',local=v?.transport==='local-file',relayed=v?.transport==='site-relay',keys=cached?'availability,byte_length,cache_key,http_status,kind,schema_version,source_sha256,state,transport,url,year':relayed||local?'availability,byte_length,http_status,kind,schema_version,source_sha256,state,transport,url,year':'availability,byte_length,http_status,kind,schema_version,source_sha256,state,url,year';
 if(v?.attempt_status!==undefined){validateFetchAttemptStatus(v.attempt_status);if(cached||local)throw new Error('Cache hits and local imports do not record new network attempts.');}
 if(!v||Object.keys(v).filter(k=>!['attempt_status','cache_entry_json','archive_profile'].includes(k)).sort().join()!==keys||v.schema_version!=='bisect-fetch-acquisition-v1'||!Number.isSafeInteger(v.byte_length)||v.byte_length<0||v.byte_length>byteLimit(v.kind,v.archive_profile)||(cached?(v.http_status!==null||!/^sha256:[0-9a-f]{64}$/.test(v.cache_key)):local?(v.http_status!==null||!fetchZipKind(v.kind)):!Number.isInteger(v.http_status))||!['acquired','unavailable'].includes(v.availability))throw new Error('Invalid Fetch acquisition receipt.');
 if(Object.hasOwn(v,'cache_entry_json')){
  if(!cached)throw new Error('Native cache history requires a cache receipt.');
  fletchHistoryFromEntry(v.cache_entry_json);const e=JSON.parse(v.cache_entry_json)[0];
  if(e.cache_key!==v.cache_key||e.dataset_id!==`bisect.${v.year}.${v.state.toLowerCase()}.${v.kind}`||e.version!==v.year||e.source_url!==v.url||e.sha256!=='sha256:'+v.source_sha256||e.bytes!==v.byte_length)throw new Error('Native cache history differs from receipt identity.');
 }
 validateRecordedFetchUrl({state_code:v.state,year:v.year,kind:v.kind,url:v.url});
 if(v.availability==='acquired'?(!cached&&!local&&(v.http_status<200||v.http_status>=300)||v.byte_length<1||!/^[0-9a-f]{64}$/.test(v.source_sha256)):(cached||local||v.http_status!==404||v.byte_length!==0||v.source_sha256!==null))throw new Error('Invalid Fetch availability or byte identity.');return v;
}
export async function acquireFetchSource(result,index,{fetcher=fetch,signal,localArchive=null,censusApiKey='',sourceTransport='direct',relayOrigin=globalThis.location?.origin,archiveReader=readCensusArchive,sourceArchiveReader=readFetchSourceArchive,archiveProfile,retainRawArchive=true,onArchive,cacheMode='network',forceCacheRefresh=false,freshness='immutable',maxAgeDays=null,retryAttempts=0,timeoutMs=30000,onAttempt,cacheReader=async(item,options)=>(await import('./fletch-cache-request.js')).readImmutableFetchCache(item,options)}={}){
 const chosen=fetchAcquisitionChoices(result).find(v=>v.index===index);if(!chosen)throw new Error('Select an eligible source from a fresh Fetch plan.');const {item}=chosen,url=publicFetchUrl(item);byteLimit(item.kind,archiveProfile);if(typeof retainRawArchive!=='boolean'||!retainRawArchive&&typeof onArchive!=='function')throw new Error('External archive retention needs a local archive writer.');signal?.throwIfAborted();
 if(!['network','prefer-cache','offline'].includes(cacheMode)||typeof forceCacheRefresh!=='boolean')throw new Error('Invalid acquisition cache mode.');
 if(!['direct','site-relay'].includes(sourceTransport))throw new Error('Choose direct provider access or this site’s source relay.');
 validateFetchRequestOptions({retryAttempts,timeoutMs});validateFetchCachePolicy({freshness,maxAgeDays});
 if(localArchive&&(!fetchZipKind(item.kind)||cacheMode!=='network'||sourceTransport!=='direct'||forceCacheRefresh||censusApiKey||retryAttempts!==0))throw new Error('Local ZIP import uses no provider, key, cache lookup, refresh or retries.');
 const cached=cacheMode==='network'?null:await cacheReader(item,{signal,offline:cacheMode==='offline',force:forceCacheRefresh,freshness,maxAgeDays});signal?.throwIfAborted();
 if(cacheMode==='offline'&&!cached)throw new Error('Offline source is not available in this browser cache.');
 let bytes,receipt;const profile=archiveProfile?{archive_profile:archiveProfile}:{};
 if(cached){if(!(cached.bytes instanceof Uint8Array)||cached.evidence?.decision!=='cache-hit'||cached.bytes.length>byteLimit(item.kind,archiveProfile))throw new Error('Invalid checked cache acquisition body.');bytes=cached.bytes;receipt={...profile,schema_version:'bisect-fetch-acquisition-v1',state:item.state_code,year:item.year,kind:item.kind,url,http_status:null,byte_length:bytes.length,source_sha256:cached.evidence.source_sha256,availability:'acquired',transport:'browser-cache',cache_key:cached.evidence.cache_key,...(cached.evidence.entries_json?{cache_entry_json:cached.evidence.entries_json}:{})};}
 else if(localArchive){
 if(!Number.isSafeInteger(localArchive.size)||localArchive.size<22||localArchive.size>byteLimit(item.kind,archiveProfile))throw new Error('Select a local source ZIP within the source size limit.');
 bytes=new Uint8Array(await localArchive.arrayBuffer());signal?.throwIfAborted();if(bytes.length!==localArchive.size)throw new Error('Local source ZIP changed while reading.');
 receipt={...profile,schema_version:'bisect-fetch-acquisition-v1',state:item.state_code,year:item.year,kind:item.kind,url,http_status:null,byte_length:bytes.length,source_sha256:null,availability:'acquired',transport:'local-file'};
 }
 else{
 let requestUrl=url;
 if(item.kind==='acs-housing'){if(typeof censusApiKey!=='string'||!/^[-_a-zA-Z0-9]{1,128}$/.test(censusApiKey))throw new Error('Enter your Census API key for this ACS request. It is not saved.');requestUrl+='&key='+encodeURIComponent(censusApiKey);}
 const relayed=sourceTransport==='site-relay'&&item.kind!=='acs-housing';if(relayed)requestUrl=sourceRelayUrl(url,relayOrigin);
 const requestFetcher=relayed?async(...args)=>{const response=await fetcher(...args);if(response.headers.get(SOURCE_RELAY_HEADER)!==SOURCE_RELAY_VERSION){await response.body?.cancel();throw new SourceTransportError('This site’s source relay is unavailable. Choose Direct provider or deploy the relay.');}return response;}:fetcher;
 const downloaded=await downloadPublicSource(requestUrl,{fetcher:requestFetcher,signal,maxBytes:byteLimit(item.kind,archiveProfile),largeArchive:archiveProfile==='large-state'&&item.kind==='pl94171',retryAttempts,timeoutMs,onAttempt});signal?.throwIfAborted();
 receipt={...profile,...(relayed?{transport:'site-relay'}:{}),schema_version:'bisect-fetch-acquisition-v1',state:item.state_code,year:item.year,kind:item.kind,url,http_status:downloaded.status,byte_length:0,source_sha256:null,availability:'unavailable',attempt_status:downloaded.attempt_status};
 if(downloaded.status===404)return {receipt:validateFetchAcquisition(receipt),inputs:null};bytes=downloaded.bytes;
 }
 const total=bytes.length;
 const housing=item.kind==='acs-housing',file={name:`${item.state_code.toLowerCase()}-${item.year}-${item.kind}.${housing?'json':'csv.gz'}`,size:bytes.length,arrayBuffer:async()=>bytes.buffer};
 const genericArchive=['school-districts','eia-861'].includes(item.kind)?await sourceArchiveReader(new File([bytes],url.split('/').at(-1)),{signal}):null;
 const archive=archiveKind(item.kind)?await archiveReader(new File([bytes],url.split('/').at(-1)),{kind:item.kind==='tiger'?'tiger':'pl94',year:Number(item.year),...(archiveProfile?{archive_profile:archiveProfile}:{})},{signal}):null;
 const source=archive||genericArchive?null:housing?await readAcsHousingSource(file):await readLodesSource(file);signal?.throwIfAborted();
 receipt.availability='acquired';receipt.byte_length=total;const observedSha=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>b.toString(16).padStart(2,'0')).join('');if(cached&&observedSha!==cached.evidence.source_sha256)throw new Error('Cache bytes changed after native identity check.');receipt.source_sha256=observedSha;
 if(archive&&archive.archive_sha256!==receipt.source_sha256)throw new Error('Archive identity changed during extraction.');
 if(genericArchive&&genericArchive.archive_sha256!==receipt.source_sha256)throw new Error('Archive identity changed during inspection.');
 if((archive||genericArchive)&&onArchive)await onArchive({state:item.state_code,year:item.year,kind:item.kind,bytes,receipt:validateFetchAcquisition(receipt)});
 const raw_archive_source=(archive||genericArchive)&&retainRawArchive?validateFetchArchiveSource({source:{name:localArchive?.name||url.split('/').at(-1),base64:encodeBytes(bytes)},state:item.state_code,year:item.year,kind:item.kind,url}):null;
 return {receipt:validateFetchAcquisition(receipt),...(cached?{cache_evidence:cached.evidence}:{}),...(raw_archive_source?{raw_archive_source}:{}),inputs:genericArchive?{source_archive:genericArchive}:archive?{archive,state:item.state_code}: {source,settings:{...(housing?{}:{kind:item.kind==='lodes-wac'?'wac':'od'}),state:item.state_code,year:item.year,data_year:item.year,source_label:'Census public '+item.kind+' response'}}};
}
