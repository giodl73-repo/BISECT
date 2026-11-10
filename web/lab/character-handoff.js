import {createProject,validateProject,validateJsonTree} from './project.js';
import {acsHousingDownload,validateAcsHousingInputs,acsHousingRequest} from './acs-housing-files.js';
import {lodesDownload,validateLodesInputs,lodesRequest} from './lodes-files.js';
import {validateCharacterInput} from './character-input.js';
import {nativeCharacterDownload,validateNativeCharacterInputs,nativeCharacterRequest} from './native-character-files.js';
const decodeBytes=base64=>Uint8Array.from(atob(base64),c=>c.charCodeAt(0));

export function characterPreparationProject(result,files){
 const native=result?.schema_version==='bisect-native-character-source-v1',housing=result?.schema_version==='bisect-acs-housing-preparation-v1',key=native?'nativeCharacterInputs':housing?'acsHousingInputs':'lodesInputs',operation=native?'prepare-native-character-source':housing?'prepare-acs-housing-source':'prepare-lodes-source';
 (native?nativeCharacterDownload:housing?acsHousingDownload:lodesDownload)(result,'character');
 const project=createProject({name:'Prepared '+(native?result.kind:housing?'housing':'economic')+' character source',files:{[key]:structuredClone(files[key])},operation,constraints:[],result:structuredClone(result),lastOperation:operation});
 preparedCharacterInput(project);return project;
}
export function preparedCharacterInput(project){
 validateJsonTree(project);
 const native=project?.operation==='prepare-native-character-source',housing=project?.operation==='prepare-acs-housing-source',key=native?'nativeCharacterInputs':housing?'acsHousingInputs':'lodesInputs';
 if(!['prepare-native-character-source','prepare-acs-housing-source','prepare-lodes-source'].includes(project?.operation)||project.lastOperation!==project.operation||!project.files||Object.keys(project.files).length!==1||!Object.hasOwn(project.files,key))throw new Error('Select a character preparation project.');
 validateProject(project);
 const inputs=(native?validateNativeCharacterInputs:housing?validateAcsHousingInputs:validateLodesInputs)(project.files[key]);
 if(!inputs.source||!native&&!housing&&inputs.settings.kind!=='wac')throw new Error('Select a prepared housing response, LODES workplace source or compatible native tract CSV.');
 (native?nativeCharacterDownload:housing?acsHousingDownload:lodesDownload)(project.result,'character');
 if(!native)(housing?acsHousingDownload:lodesDownload)(project.result,'csv');
 const input=validateCharacterInput(project.result.character_input,null,native?inputs.settings.kind:housing?'housing':'economic'),s=inputs.settings;
 if(input.state!==s.state||input.year!==s.year||input.data_year!==s.data_year||input.source_label!==s.source_label)throw new Error('Prepared character scope disagrees with the retained source settings.');
 if(['state','year','data_year'].some(key=>project.result[key]!==input[key])||!native&&!housing&&project.result.kind!=='wac'||!native&&(project.result.written_rows!==Object.keys(input.data.characters).length||!Number.isSafeInteger(project.result.written_rows))||project.result.source_name!==inputs.source.name||!(native?['source_sha256']:['source_sha256','csv_sha256']).every(key=>/^[a-f0-9]{64}$/.test(project.result[key]??'')))throw new Error('Character preparation receipt scope, row count, source name or hashes disagree.');
 if(native){const fields=input.data.kind==='economic'?['commercial_intensity','industrial_fraction','jobs_per_resident']:['pct_single_family','pct_multifamily','pct_owner','housing_vintage'],records=project.result.records;
  if(project.result.native_status!=='loaded'||project.result.native_error!==null||project.result.kind!==input.data.kind||!Array.isArray(records)||records.length!==Object.keys(input.data.characters).length||new Set(records.map(r=>r.geoid)).size!==records.length||records.some(r=>r.finite!==true||!input.data.characters[r.geoid]||!same(r.values,fields.map(k=>input.data.characters[r.geoid][k]))))throw new Error('Native CSV observations disagree with the compatible character input.');
 }
 return structuredClone(input);
}
const same=(a,b)=>a===b||a&&b&&typeof a==='object'&&typeof b==='object'&&Array.isArray(a)===Array.isArray(b)&&Object.keys(a).length===Object.keys(b).length&&Object.keys(a).every(k=>Object.hasOwn(b,k)&&same(a[k],b[k]));
export function validateCharacterPreparation(project,input){
 const original=preparedCharacterInput(project);validateCharacterInput(input);
 for(const key of ['schema_version','state','year','data_year','source_label'])if(original[key]!==input[key])throw new Error('Character preparation scope differs from the selected observations.');
 if(!same(original.data,input.alignment?.source_data??input.data))throw new Error('Character preparation observations differ from the selected original source.');
 return project;
}
export async function verifyCharacterPreparation(project,input){
 validateCharacterPreparation(project,input);const native=project.operation==='prepare-native-character-source',housing=project.operation==='prepare-acs-housing-source',request=(native?nativeCharacterRequest:housing?acsHousingRequest:lodesRequest)(project.files[native?'nativeCharacterInputs':housing?'acsHousingInputs':'lodesInputs']).input;
 const bytes=request.source_base64?decodeBytes(request.source_base64):new TextEncoder().encode(request.body_json??request.csv_text),digest=async x=>[...new Uint8Array(await crypto.subtle.digest('SHA-256',x))].map(b=>b.toString(16).padStart(2,'0')).join('');
 if(project.result.source_name!==request.source_name||project.result.source_sha256!==await digest(bytes)||!native&&(typeof project.result.csv_text!=='string'||project.result.csv_sha256!==await digest(new TextEncoder().encode(project.result.csv_text))))throw new Error('Character preparation source or CSV bytes disagree with recorded hashes.');
 if(native&&project.result.source_byte_length!==bytes.length)throw new Error('Native CSV source length disagrees with retained bytes.');
 if(!native&&!housing&&(project.result.source_encoding!==(request.source_base64?'gzip':'csv')||project.result.source_byte_length!==bytes.length))throw new Error('LODES preparation encoding or source length disagrees with retained request bytes.');
 return project;
}
export function characterHandoffToken(hash){return /^#character=([a-f0-9]{32})$/.exec(hash||'')?.[1]??null;}
export function characterLaboratoryUrl(current,token){
 if(!/^[a-f0-9]{32}$/.test(token))throw new Error('Invalid character transfer token.');
 const base=new URL(current),toolkit=/\/toolkit\/(?:[^/]*)$/.test(base.pathname),url=new URL(toolkit?'../':'./',base);url.hash='character='+token;return url.href;
}
