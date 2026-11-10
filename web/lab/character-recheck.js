import {preparedCharacterInput,verifyCharacterPreparation} from './character-handoff.js';
import {acsHousingRequest} from './acs-housing-files.js';
import {lodesRequest} from './lodes-files.js';
import {nativeCharacterRequest} from './native-character-files.js';
import {isNativeCharacterFallback,nativeCharacterFallbackInfo,verifyNativeCharacterFallback} from './character-fallback-source.js';
import {validateJsonTree,readValidatedFile,MAX_PROJECT_BYTES} from './project.js';
const hash=x=>typeof x==='string'&&/^[a-f0-9]{64}$/.test(x);
const same=(a,b)=>a===b||a&&b&&typeof a==='object'&&typeof b==='object'&&Array.isArray(a)===Array.isArray(b)&&Object.keys(a).length===Object.keys(b).length&&Object.keys(a).every(k=>Object.hasOwn(b,k)&&same(a[k],b[k]));
const digest=async bytes=>[...new Uint8Array(await crypto.subtle.digest('SHA-256',bytes))].map(b=>b.toString(16).padStart(2,'0')).join('');
export function characterPreparationReplayRequest(project){
 if(isNativeCharacterFallback(project))nativeCharacterFallbackInfo(project);else preparedCharacterInput(project);return project.operation==='prepare-native-character-source'?nativeCharacterRequest(project.files.nativeCharacterInputs):project.operation==='prepare-acs-housing-source'?acsHousingRequest(project.files.acsHousingInputs):lodesRequest(project.files.lodesInputs);
}
export function validateCharacterRecheck(r,project){
 validateJsonTree(r);const keys=['schema_version','status','preparation_sha256','source_sha256','expected_csv_sha256','recomputed_csv_sha256','wasm_sha256','differing_fields','scope'];
 if(!r||Object.keys(r).length!==keys.length||keys.some(k=>!Object.hasOwn(r,k))||r.schema_version!=='bisect-character-preparation-recheck-v1'||!['matched','mismatch'].includes(r.status)||!['preparation_sha256','source_sha256','expected_csv_sha256','recomputed_csv_sha256','wasm_sha256'].every(k=>hash(r[k]))||!Array.isArray(r.differing_fields)||r.differing_fields.length>100||r.differing_fields.some(k=>typeof k!=='string'||!/^[a-z][a-z0-9_]{0,100}$/.test(k))||new Set(r.differing_fields).size!==r.differing_fields.length||r.differing_fields.join()!==[...r.differing_fields].sort().join()||(r.status==='matched')!==(r.differing_fields.length===0)||r.scope!=='current-wasm-preparation-output-equality; no publisher authentication or plan certification')throw new Error('Invalid character preparation recheck receipt.');
 if(project){if(isNativeCharacterFallback(project))nativeCharacterFallbackInfo(project);else preparedCharacterInput(project);if(r.source_sha256!==project.result.source_sha256||r.expected_csv_sha256!==(project.operation==='prepare-native-character-source'?project.result.source_sha256:project.result.csv_sha256))throw new Error('Recheck receipt differs from retained preparation.');}
 return r;
}
export async function recheckCharacterPreparation(engine,project){
 if(isNativeCharacterFallback(project))await verifyNativeCharacterFallback(project);else await verifyCharacterPreparation(project,preparedCharacterInput(project));const request=characterPreparationReplayRequest(project),actual=engine.execute(request);
 const fields=[...new Set([...Object.keys(project.result),...Object.keys(actual)])].sort().filter(k=>!same(project.result[k],actual[k]));
 const native=project.operation==='prepare-native-character-source';return validateCharacterRecheck({schema_version:'bisect-character-preparation-recheck-v1',status:fields.length?'mismatch':'matched',preparation_sha256:await digest(new TextEncoder().encode(JSON.stringify(project))),source_sha256:project.result.source_sha256,expected_csv_sha256:native?project.result.source_sha256:project.result.csv_sha256,recomputed_csv_sha256:native?actual.source_sha256:actual.csv_sha256,wasm_sha256:engine.artifactIdentity.sha256,differing_fields:fields,scope:'current-wasm-preparation-output-equality; no publisher authentication or plan certification'},project);
}
export async function verifyCharacterRecheckBinding(receipt,project){
 validateCharacterRecheck(receipt,project);if(receipt.preparation_sha256!==await digest(new TextEncoder().encode(JSON.stringify(project))))throw new Error('Preparation changed after the recorded recheck.');return receipt;
}
export function runCharacterRecheck(project,sha256,{signal}={}){
 if(isNativeCharacterFallback(project))nativeCharacterFallbackInfo(project);else preparedCharacterInput(project);if(!hash(sha256))throw new Error('Load a verified browser engine before rechecking preparation.');const file=new Blob([JSON.stringify(project)],{type:'application/json'});
 return readValidatedFile({size:file.size,file,sha256},new URL('./character-recheck-worker.js',import.meta.url),{signal,maxBytes:MAX_PROJECT_BYTES,timeoutMs:60000});
}
