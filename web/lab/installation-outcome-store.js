// Bounded passive local history. Stored data can never select or grant a folder.
import {inspectJsonText} from './project.js';
export const INSTALL_OUTCOME_DB='bisect-installation-outcome-v1';
const MAX_BYTES=1024*1024;
export function parseStoredInstallationOutcome(text){
 if(typeof text!=='string'||text.length>MAX_BYTES||new TextEncoder().encode(text).length>MAX_BYTES)throw new Error('Stored installation outcome exceeds 1 MiB.');
 inspectJsonText(text,50000);const record=JSON.parse(text),o=record?.outcome;
 if(!record||Object.keys(record).sort().join()!=='id,outcome,schema_version'||record.schema_version!=='bisect-local-installation-outcome-v1'||typeof record.id!=='string'||!/^[a-f0-9-]{36}$/.test(record.id)||!o||Array.isArray(o)||Object.keys(o).some(k=>!['schema_version','status','package_sha256','target_namespace','recovery','receipt','error','scope'].includes(k))||o.schema_version!=='bisect-browser-installation-outcome-v1'||!['in-progress','completed','cancelled','interrupted'].includes(o.status)||typeof o.package_sha256!=='string'||!/^[a-f0-9]{64}$/.test(o.package_sha256)||typeof o.target_namespace!=='string'||o.target_namespace.length>512||typeof o.recovery!=='boolean'||typeof o.scope!=='string'||o.scope.length>2000||o.error!==undefined&&(typeof o.error!=='string'||o.error.length>2000)||!Object.hasOwn(o,'receipt')||o.receipt!==null&&(!o.receipt||!['bisect-browser-native-install-v1','bisect-native-batch-install-v1'].includes(o.receipt.schema_version)))throw new Error('Invalid stored installation outcome.');
 // A progress marker carries no claim about completed effects.
 if(o.status==='in-progress'&&o.receipt!==null)throw new Error('Invalid in-progress installation outcome.');
 return record;
}
function connect(idb,dbName){return new Promise((resolve,reject)=>{
 if(!idb){reject(new Error('Browser outcome storage is unavailable.'));return;}
 let done=false;const timer=setTimeout(()=>finish(new Error('Browser outcome storage timed out.')),5000);let request;
 function finish(error,db){if(done){db?.close();return;}done=true;clearTimeout(timer);error?reject(error):resolve(db);}
 try{request=idb.open(dbName,1);}catch(error){finish(error);return;}
 request.onupgradeneeded=()=>request.result.createObjectStore('outcomes');
 request.onblocked=()=>finish(new Error('Browser outcome storage is blocked.'));
 request.onerror=()=>finish(request.error||new Error('Browser outcome storage failed.'));
 request.onsuccess=()=>{request.result.onversionchange=()=>request.result.close();finish(null,request.result);};
});}
async function transaction(mode,action,{idb=globalThis.indexedDB,dbName=INSTALL_OUTCOME_DB}={}){
 const db=await connect(idb,dbName);return new Promise((resolve,reject)=>{
  let tx,output,error,done=false;
  const finish=failure=>{if(done)return;done=true;clearTimeout(timer);db.close();failure?reject(failure):resolve(output);};
  const timer=setTimeout(()=>{error=new Error('Browser outcome transaction timed out.');try{tx?.abort();}catch{}finish(error);},5000);
  try{tx=db.transaction('outcomes',mode);tx.oncomplete=()=>finish(error);tx.onabort=()=>finish(error||tx.error||new Error('Browser outcome transaction aborted.'));tx.onerror=e=>{error=e.target?.error||tx.error;};
   action(tx.objectStore('outcomes'),value=>{output=value;},failure=>{error=failure;try{tx.abort();}catch{finish(error);}});
  }catch(failure){finish(failure);}
 });
}
// Shared bounded transaction mechanism; callers own their schema and database.
export {transaction as installationOutcomeTransaction};
export async function readInstallationOutcome(options){return transaction('readonly',(store,set,fail)=>{const r=store.get('latest');r.onsuccess=()=>{try{set(r.result===undefined?null:parseStoredInstallationOutcome(r.result));}catch(e){fail(e);}};},{...options,dbName:INSTALL_OUTCOME_DB});}
export async function saveInstallationOutcome(record,{expectedId,...options}={}){
 const text=JSON.stringify(record);parseStoredInstallationOutcome(text);
 return transaction('readwrite',(store,set,fail)=>{const r=store.get('latest');r.onsuccess=()=>{try{
  if(expectedId!==undefined&&(r.result===undefined||parseStoredInstallationOutcome(r.result).id!==expectedId)){set(false);return;}
  store.put(text,'latest');set(true);
 }catch(e){fail(e);}};},{...options,dbName:INSTALL_OUTCOME_DB});
}
export async function forgetInstallationOutcome(expectedId,options){return transaction('readwrite',(store,set,fail)=>{const r=store.get('latest');r.onsuccess=()=>{try{
 // A corrupt record may be explicitly forgotten; a valid newer tab's record stays.
 let prior;try{prior=r.result===undefined?null:parseStoredInstallationOutcome(r.result);}catch{prior=null;}
 if(prior&&prior.id!==expectedId){set(false);return;}store.delete('latest');set(true);
 }catch(e){fail(e);}};},{...options,dbName:INSTALL_OUTCOME_DB});}
