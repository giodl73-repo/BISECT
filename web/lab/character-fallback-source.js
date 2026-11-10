import {validateJsonTree,validateProject,createProject} from './project.js';
import {validateNativeCharacterInputs} from './native-character-files.js';
import {characterLaboratoryUrl} from './character-handoff.js';
export function isNativeCharacterFallback(project){validateJsonTree(project);return project?.operation==='prepare-native-character-source'&&['empty','error'].includes(project?.result?.native_status);}
export function nativeCharacterFallbackInfo(project){
 validateJsonTree(project);
 if(!isNativeCharacterFallback(project)||project.lastOperation!==project.operation||!project.files||Object.keys(project.files).join()!=='nativeCharacterInputs')throw new Error('Select a recorded native empty/error character source.');
 validateProject(project);const v=validateNativeCharacterInputs(project.files.nativeCharacterInputs),r=project.result,s=v.settings;
 if(!v.source||r.schema_version!=='bisect-native-character-source-v1'||r.execution_performed!==false||r.character_status!=='unavailable'||r.character_input!==null||!Array.isArray(r.records)||r.records.length||r.native_status==='empty'&&r.native_error!==null||r.native_status==='error'&&(typeof r.native_error!=='string'||!r.native_error.trim()||r.native_error.length>2000)||['state','year','data_year','kind'].some(k=>r[k]!==s[k])||r.source_name!==v.source.name||!Number.isSafeInteger(r.source_byte_length)||r.source_byte_length<0||!/^[a-f0-9]{64}$/.test(r.source_sha256??''))throw new Error('Native fallback source report disagrees with retained scope or outcome.');
 return {...s,source_name:v.source.name,native_status:r.native_status,native_error:r.native_error};
}
export function nativeCharacterFallbackProject(result,files){const p=createProject({name:'Recorded native character '+result?.native_status+' source',files:{nativeCharacterInputs:structuredClone(files.nativeCharacterInputs)},operation:'prepare-native-character-source',constraints:[],result:structuredClone(result),lastOperation:'prepare-native-character-source'});nativeCharacterFallbackInfo(p);return p;}
export async function verifyNativeCharacterFallback(project){
 nativeCharacterFallbackInfo(project);const bytes=Uint8Array.from(atob(project.files.nativeCharacterInputs.source.base64),c=>c.charCodeAt(0)),sha=[...new Uint8Array(await crypto.subtle.digest('SHA-256',bytes))].map(b=>b.toString(16).padStart(2,'0')).join('');if(bytes.length!==project.result.source_byte_length||sha!==project.result.source_sha256)throw new Error('Native fallback source bytes disagree with recorded hashes.');return project;
}
export function validateCharacterFallbackConfig(config){
 if(!Object.hasOwn(config,'character_fallback_sources'))return;
 const map=config.character_fallback_sources;if(config.character_absent_file_fallback!==true||!map||Array.isArray(map)||typeof map!=='object'||!Object.keys(map).length||Object.keys(map).some(code=>!config.states.includes(code)))throw new Error('Recorded fallback sources require explicit character fallback permission and selected states.');
 for(const [code,p]of Object.entries(map)){const s=nativeCharacterFallbackInfo(p);if(s.state!==code||s.year!==config.year||s.kind+'-character'!==config.weights||Object.hasOwn(config.characters,code)||config.character_preparations?.[code])throw new Error('Fallback source conflicts with selected observations or experiment scope.');}
}
export function characterFallbackToken(hash){return /^#character-fallback=([a-f0-9]{32})$/.exec(hash||'')?.[1]??null;}
export function characterFallbackUrl(current,token){const url=new URL(characterLaboratoryUrl(current,token));url.hash='character-fallback='+token;return url.href;}
