import {validateFetchCachePolicy} from './fletch-cache-policy.js';
import {readFletchCacheRecord} from './fletch-cache-store.js';
function run(data,{signal}={}){return new Promise((resolve,reject)=>{
 if(signal?.aborted){reject(signal.reason);return;}const worker=new Worker(new URL('./fletch-cache-request-worker.js',import.meta.url),{type:'module'});let done=false;
 const finish=(error,value)=>{if(done)return;done=true;clearTimeout(timer);signal?.removeEventListener('abort',abort);worker.terminate();error?reject(error):resolve(value);};const abort=()=>finish(signal.reason||new Error('Cache request cancelled.')),timer=setTimeout(()=>finish(new Error('Cache request timed out.')),60000);signal?.addEventListener('abort',abort,{once:true});worker.onerror=worker.onmessageerror=()=>finish(new Error('Cache request worker failed.'));worker.onmessage=({data})=>data.ok===true?finish(null,data.result):finish(new Error(data.error||'Cache request failed.'));worker.postMessage(data);
});}
export async function readImmutableFetchCache(item,{signal,force=false,offline=false,freshness='immutable',maxAgeDays=null}={}){
 validateFetchCachePolicy({freshness,maxAgeDays});
 const identity={dataset_id:`bisect.${item.year}.${item.state_code.toLowerCase()}.${item.kind}`,version:item.year,source_url:item.url};
 const planned=await run({type:'key',input:identity},{signal}),record=await readFletchCacheRecord(planned.cache_key,{signal,validateMetadata:!force&&freshness!=='always-check'});signal?.throwIfAborted();
 // Indexed toolkit objects always carry the native timestamp. Unindexed orphan
 // bodies require explicit staging so a missing filesystem mtime is not invented.
 if(record.blob&&!record.row&&!force&&freshness==='immutable')throw new Error('Unindexed cache object needs explicit staging before acquisition reuse.');
 // Legacy objects have no observed storage time; maximum-age selection reacquires them.
 const bypass=force||freshness==='always-check'||freshness==='max-age-days'&&!!record.blob&&record.row?.object_modified_at_ms===undefined;
 const input={...identity,freshness,max_age_days:maxAgeDays,now_ms:String(Date.now()),object_modified_at_ms:record.row?.object_modified_at_ms??null,force:bypass,offline,object_exists:!!record.blob,observation:null,trusted_entries_json:bypass?'[]':record.row?.entry_json||'[]',expected_sha256:null,fetched_at_ms:null};
 const result=await run({type:'resolve',input,blob:record.blob,modified_at_ms:record.row?.fetched_at_ms||'0'},{signal});signal?.throwIfAborted();
 if(result.decision==='fetch')return null;if(result.decision!=='cache-hit'||result.cache_key!==planned.cache_key)throw new Error('Invalid native cache request result.');
 const bytes=new Uint8Array(await record.blob.arrayBuffer());signal?.throwIfAborted();if(String(bytes.length)!==result.byte_count_text)throw new Error('Cached body size changed after native check.');return {bytes,evidence:result};
}
