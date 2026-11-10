import {MAX_PROJECT_BYTES,validateJsonTree} from './project.js';
import {nativeCharacterRequest} from './native-character-files.js';
export const NATIVE_CHARACTER_ENGINE_DEFAULTS={label:'Native character engine run',chamber:'congressional',created_at:'',inspect_only:true};
export function validateNativeCharacterEngineInputs(input){
 validateJsonTree(input);
 if(!input||Object.keys(input).sort().join()!=='settings,source'||!input.settings||Object.keys(input.settings).sort().join()!==Object.keys(NATIVE_CHARACTER_ENGINE_DEFAULTS).sort().join())throw new Error('Invalid native-character engine selection.');
 const s=input.settings;if(typeof s.label!=='string'||!s.label.trim()||new TextEncoder().encode(s.label).length>200||!['congressional','house','senate'].includes(s.chamber)||typeof s.created_at!=='string'||s.created_at.length>200||typeof s.inspect_only!=='boolean')throw new Error('Choose export label, chamber, timestamp and explicit review/run mode.');
 if(input.source!==null){const f=input.source;if(!f||Object.keys(f).sort().join()!=='name,text'||typeof f.name!=='string'||!f.name||new TextEncoder().encode(f.name).length>200||typeof f.text!=='string'||!f.text.length||new TextEncoder().encode(f.text).length>MAX_PROJECT_BYTES)throw new Error('Select a UTF-8 engine request JSON of at most25MiB.');}
 return input;
}
export async function readNativeCharacterEngineSource(file){
 if(!file||!Number.isSafeInteger(file.size)||file.size<1||file.size>MAX_PROJECT_BYTES)throw new Error('Select a nonempty engine request JSON of at most25MiB.');const bytes=new Uint8Array(await file.arrayBuffer());if(bytes.length!==file.size)throw new Error('Engine request file changed while reading.');const source={name:file.name,text:new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(bytes)};validateNativeCharacterEngineInputs({source,settings:NATIVE_CHARACTER_ENGINE_DEFAULTS});return source;
}
export function nativeCharacterEngineRequest(source,inputs){
 validateNativeCharacterEngineInputs(inputs);if(!inputs.source)throw new Error('Select ordinary/build/multiscale engine request JSON first.');if(!inputs.settings.inspect_only&&!inputs.settings.created_at.trim())throw new Error('Record an export timestamp before executing.');
 return {operation:'run-native-character-engine',input:{source:nativeCharacterRequest(source).input,request_json:inputs.source.text,...inputs.settings}};
}
export function nativeCharacterEngineDownload(result,kind){
 if(!['bisect-native-character-engine-request-v1','bisect-native-character-engine-v1'].includes(result?.schema_version))throw new Error('Review or run a fresh native-character request first.');
 if(kind==='report')return {name:'native-character-engine.json',text:JSON.stringify(result,null,2)+'\n'};
 if(result.schema_version!=='bisect-native-character-engine-v1'||result.execution_performed!==true||!['plan','context'].includes(kind))throw new Error('Execute a fresh native-character run before exporting a plan.');
 const text=result.export?.[kind==='plan'?'document_json':'context_json'];if(typeof text!=='string')throw new Error('Native plan/context bytes are unavailable.');return {name:kind==='plan'?'native-character.rplan':'native-character.rctx',text};
}
