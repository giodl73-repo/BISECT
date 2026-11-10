import {validateContextEngineSource} from './context-engine-files.js';
import {validateNativeCharacterEngineInputs,readNativeCharacterEngineSource} from './native-character-engine-files.js';
export const PREPARED_ENGINE_DEFAULTS={label:'Prepared engine run',chamber:'congressional',created_at:'',inspect_only:true};
export function validatePreparedEngineInputs(input){if(!input||typeof input!=='object')return validateNativeCharacterEngineInputs(input);const {context_source,...base}=input;validateNativeCharacterEngineInputs(base);if(Object.hasOwn(input,'context_source'))validateContextEngineSource(context_source);return input;}
export const readPreparedEngineSource=readNativeCharacterEngineSource;
export function preparedEngineRequest(inputs){
 validatePreparedEngineInputs(inputs);
 if(!inputs.source)throw new Error('Select ordinary/build/multiscale engine request JSON first.');
 if(!inputs.settings.inspect_only&&!inputs.settings.created_at.trim())throw new Error('Record an export timestamp before executing.');
 return {operation:'run-prepared-engine',input:{request_json:inputs.source.text,...inputs.settings,...(inputs.context_source?{context_source:inputs.context_source}:{})}};
}
export function preparedEngineDownload(result,kind){
 if(!['bisect-prepared-engine-request-v1','bisect-prepared-engine-v1'].includes(result?.schema_version))throw new Error('Review or run a fresh prepared request first.');
 if(kind==='report')return {name:'prepared-engine.json',text:JSON.stringify(result,null,2)+'\n'};
 if(result.schema_version!=='bisect-prepared-engine-v1'||result.execution_performed!==true||!['plan','context'].includes(kind))throw new Error('Execute a fresh prepared request before exporting a plan.');
 const text=result.export?.[kind==='plan'?'document_json':'context_json'];
 if(typeof text!=='string')throw new Error('Native plan/context bytes are unavailable.');
 return {name:kind==='plan'?'prepared-engine.rplan':'prepared-engine.rctx',text};
}
